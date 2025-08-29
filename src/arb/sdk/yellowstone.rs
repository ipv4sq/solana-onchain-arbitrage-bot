use anyhow::{Context, Result};
use solana_sdk::pubkey::Pubkey;
use std::collections::HashMap;
use std::sync::Arc;
use tokio_stream::StreamExt;
use tracing::{error, info, warn};
use yellowstone_grpc_client::GeyserGrpcClient;
use yellowstone_grpc_proto::prelude::*;

pub struct SolanaGrpcClient {
    endpoint: String,
    token: String,
    client: Option<GeyserGrpcClient<yellowstone_grpc_client::InterceptorXToken>>,
}

impl SolanaGrpcClient {
    pub fn new(endpoint: String, token: String) -> Self {
        Self {
            endpoint,
            token,
            client: None,
        }
    }

    pub fn from_env() -> Result<Self> {
        // Load .env file if it exists
        dotenv::dotenv().ok();

        let endpoint = std::env::var("GRPC_URL").context("GRPC_URL not found in environment")?;
        let token = std::env::var("GRPC_TOKEN").context("GRPC_TOKEN not found in environment")?;

        Ok(Self::new(endpoint, token))
    }

    pub async fn subscribe_transactions<F, Fut>(
        mut self,
        filter: TransactionFilter,
        callback: F,
        auto_retry: bool,
    ) -> Result<()>
    where
        F: Fn(GrpcTransactionUpdate) -> Fut + Send + Sync + Clone + 'static,
        Fut: std::future::Future<Output = Result<()>> + Send,
    {
        if auto_retry {
            self.subscribe_with_retry_internal(filter, callback).await
        } else {
            self.connect_if_needed().await?;
            self.subscribe_once(filter, callback).await
        }
    }

    pub async fn subscribe_accounts<F, Fut>(
        mut self,
        filter: AccountFilter,
        callback: F,
        auto_retry: bool,
    ) -> Result<()>
    where
        F: Fn(GrpcAccountUpdate) -> Fut + Send + Sync + Clone + 'static,
        Fut: std::future::Future<Output = Result<()>> + Send,
    {
        if auto_retry {
            self.subscribe_accounts_with_retry_internal(filter, callback)
                .await
        } else {
            self.connect_if_needed().await?;
            self.subscribe_accounts_once(filter, callback).await
        }
    }

    async fn connect_if_needed(&mut self) -> Result<()> {
        if self.client.is_none() {
            self.connect().await?;
        }
        Ok(())
    }

    async fn connect(&mut self) -> Result<()> {
        let endpoint = tonic::transport::Endpoint::from_shared(self.endpoint.clone())
            .context("Failed to create endpoint")?
            .tls_config(tonic::transport::ClientTlsConfig::new())
            .context("Failed to configure TLS")?;

        let channel = endpoint
            .connect()
            .await
            .context("Failed to connect to gRPC endpoint")?;

        use tonic::metadata::AsciiMetadataValue;
        use tonic_health::pb::health_client::HealthClient;
        use yellowstone_grpc_proto::geyser::geyser_client::GeyserClient;

        let x_token = AsciiMetadataValue::try_from(self.token.clone())
            .context("Failed to create metadata value")?;

        let interceptor = yellowstone_grpc_client::InterceptorXToken {
            x_token: Some(x_token.clone()),
            x_request_snapshot: false,
        };

        let interceptor2 = yellowstone_grpc_client::InterceptorXToken {
            x_token: Some(x_token),
            x_request_snapshot: false,
        };

        let health_service =
            tonic::service::interceptor::InterceptedService::new(channel.clone(), interceptor);
        let geyser_service =
            tonic::service::interceptor::InterceptedService::new(channel, interceptor2);

        let health_client = HealthClient::new(health_service);
        let geyser_client = GeyserClient::new(geyser_service);

        self.client = Some(GeyserGrpcClient::new(health_client, geyser_client));

        info!("Connected to gRPC endpoint: {}", self.endpoint);
        Ok(())
    }

    async fn subscribe_once<F, Fut>(&mut self, filter: TransactionFilter, callback: F) -> Result<()>
    where
        F: Fn(GrpcTransactionUpdate) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = Result<()>> + Send,
    {
        let client = self.client.as_mut().context("Client not connected")?;

        let filter_name = filter.name.clone();
        let mut transactions = HashMap::new();
        transactions.insert(filter_name.clone(), filter.into_request_filter());

        let subscribe_request = SubscribeRequest {
            accounts: HashMap::new(),
            slots: HashMap::new(),
            transactions,
            transactions_status: HashMap::new(),
            blocks: HashMap::new(),
            blocks_meta: HashMap::new(),
            entry: HashMap::new(),
            commitment: Some(CommitmentLevel::Confirmed as i32),
            accounts_data_slice: vec![],
            ping: None,
            from_slot: None,
        };

        let (_subscribe_tx, response) = client
            .subscribe_with_request(Some(subscribe_request))
            .await
            .context("Failed to subscribe")?;

        info!("Subscription established for filter: {}", filter_name);

        let callback = Arc::new(callback);
        let mut stream = response;

        while let Some(message) = stream.next().await {
            match message {
                Ok(update) => {
                    if let Some(update) = update.update_oneof {
                        match update {
                            subscribe_update::UpdateOneof::Transaction(tx) => {
                                let transaction_update = GrpcTransactionUpdate::from_grpc(tx);
                                if let Err(e) = callback(transaction_update).await {
                                    error!("Callback error: {}", e);
                                }
                            }
                            subscribe_update::UpdateOneof::Ping(_) => {
                                info!("Received ping from gRPC");
                            }
                            _ => {}
                        }
                    }
                }
                Err(e) => {
                    error!("Stream error: {}", e);
                    return Err(anyhow::anyhow!("Stream error: {}", e));
                }
            }
        }

        Ok(())
    }

    async fn subscribe_with_retry_internal<F, Fut>(
        mut self,
        filter: TransactionFilter,
        callback: F,
    ) -> Result<()>
    where
        F: Fn(GrpcTransactionUpdate) -> Fut + Send + Sync + Clone + 'static,
        Fut: std::future::Future<Output = Result<()>> + Send,
    {
        loop {
            info!("Starting gRPC subscription...");

            if self.client.is_none() {
                if let Err(e) = self.connect().await {
                    error!("Failed to connect: {}, retrying in 5 seconds...", e);
                    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                    continue;
                }
            }

            match self.subscribe_once(filter.clone(), callback.clone()).await {
                Ok(_) => {
                    warn!("Subscription ended, reconnecting in 5 seconds...");
                }
                Err(e) => {
                    error!("Subscription error: {}, reconnecting in 5 seconds...", e);
                }
            }

            self.client = None;
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
        }
    }

    async fn subscribe_accounts_once<F, Fut>(
        &mut self,
        filter: AccountFilter,
        callback: F,
    ) -> Result<()>
    where
        F: Fn(GrpcAccountUpdate) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = Result<()>> + Send,
    {
        let client = self.client.as_mut().context("Client not connected")?;

        let filter_name = filter.name.clone();
        let mut accounts = HashMap::new();
        accounts.insert(filter_name.clone(), filter.into_request_filter());

        let subscribe_request = SubscribeRequest {
            accounts,
            slots: HashMap::new(),
            transactions: HashMap::new(),
            transactions_status: HashMap::new(),
            blocks: HashMap::new(),
            blocks_meta: HashMap::new(),
            entry: HashMap::new(),
            commitment: Some(CommitmentLevel::Confirmed as i32),
            accounts_data_slice: vec![],
            ping: None,
            from_slot: None,
        };

        let (_subscribe_tx, response) = client
            .subscribe_with_request(Some(subscribe_request))
            .await
            .context("Failed to subscribe")?;

        info!(
            "Account subscription established for filter: {}",
            filter_name
        );

        let callback = Arc::new(callback);
        let mut stream = response;

        while let Some(message) = stream.next().await {
            match message {
                Ok(update) => {
                    if let Some(update) = update.update_oneof {
                        match update {
                            subscribe_update::UpdateOneof::Account(account) => {
                                let account_update = GrpcAccountUpdate::from_grpc(account);
                                if let Err(e) = callback(account_update).await {
                                    error!("Callback error: {}", e);
                                }
                            }
                            subscribe_update::UpdateOneof::Ping(_) => {
                                info!("Received ping from gRPC");
                            }
                            _ => {}
                        }
                    }
                }
                Err(e) => {
                    error!("Stream error: {}", e);
                    return Err(anyhow::anyhow!("Stream error: {}", e));
                }
            }
        }

        Ok(())
    }

    async fn subscribe_accounts_with_retry_internal<F, Fut>(
        mut self,
        filter: AccountFilter,
        callback: F,
    ) -> Result<()>
    where
        F: Fn(GrpcAccountUpdate) -> Fut + Send + Sync + Clone + 'static,
        Fut: std::future::Future<Output = Result<()>> + Send,
    {
        loop {
            info!("Starting gRPC account subscription...");

            if self.client.is_none() {
                if let Err(e) = self.connect().await {
                    error!("Failed to connect: {}, retrying in 5 seconds...", e);
                    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                    continue;
                }
            }

            match self
                .subscribe_accounts_once(filter.clone(), callback.clone())
                .await
            {
                Ok(_) => {
                    warn!("Account subscription ended, reconnecting in 5 seconds...");
                }
                Err(e) => {
                    error!(
                        "Account subscription error: {}, reconnecting in 5 seconds...",
                        e
                    );
                }
            }

            self.client = None;
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
        }
    }
}

#[derive(Clone)]
pub struct TransactionFilter {
    pub name: String,
    pub vote: Option<bool>,
    pub failed: Option<bool>,
    pub account_include: Vec<String>,
    pub account_exclude: Vec<String>,
    pub account_required: Vec<String>,
}

impl TransactionFilter {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            vote: Some(false),
            failed: Some(false),
            account_include: Vec::new(),
            account_exclude: Vec::new(),
            account_required: Vec::new(),
        }
    }

    pub fn with_program(mut self, program_id: &Pubkey) -> Self {
        self.account_include.push(program_id.to_string());
        self
    }

    pub fn with_programs(mut self, program_ids: &[Pubkey]) -> Self {
        for id in program_ids {
            self.account_include.push(id.to_string());
        }
        self
    }

    pub fn exclude_account(mut self, account: &Pubkey) -> Self {
        self.account_exclude.push(account.to_string());
        self
    }

    pub fn require_account(mut self, account: &Pubkey) -> Self {
        self.account_required.push(account.to_string());
        self
    }

    pub fn include_votes(mut self, include: bool) -> Self {
        self.vote = Some(include);
        self
    }

    pub fn include_failed(mut self, include: bool) -> Self {
        self.failed = Some(include);
        self
    }

    fn into_request_filter(self) -> SubscribeRequestFilterTransactions {
        SubscribeRequestFilterTransactions {
            vote: self.vote,
            failed: self.failed,
            signature: None,
            account_include: self.account_include,
            account_exclude: self.account_exclude,
            account_required: self.account_required,
        }
    }
}

#[derive(Clone)]
pub struct AccountFilter {
    pub name: String,
    pub account: Vec<String>,
    pub owner: Vec<String>,
    pub filters: Vec<SubscribeRequestFilterAccountsFilter>,
}

impl AccountFilter {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            account: Vec::new(),
            owner: Vec::new(),
            filters: Vec::new(),
        }
    }

    pub fn with_account(mut self, account: &Pubkey) -> Self {
        self.account.push(account.to_string());
        self
    }

    pub fn with_accounts(mut self, accounts: &[Pubkey]) -> Self {
        for account in accounts {
            self.account.push(account.to_string());
        }
        self
    }

    pub fn with_owner(mut self, owner: &Pubkey) -> Self {
        self.owner.push(owner.to_string());
        self
    }

    pub fn with_owners(mut self, owners: &[Pubkey]) -> Self {
        for owner in owners {
            self.owner.push(owner.to_string());
        }
        self
    }

    fn into_request_filter(self) -> SubscribeRequestFilterAccounts {
        SubscribeRequestFilterAccounts {
            account: self.account,
            owner: self.owner,
            filters: self.filters,
            nonempty_txn_signature: None,
        }
    }
}

#[derive(Clone)]
pub struct GrpcAccountUpdate {
    pub account: Pubkey,
    pub slot: u64,
    pub data: Vec<u8>,
    pub owner: Pubkey,
    pub lamports: u64,
    pub executable: bool,
    pub rent_epoch: u64,
}

impl GrpcAccountUpdate {
    fn from_grpc(update: SubscribeUpdateAccount) -> Self {
        let account = update.account.as_ref();
        let pubkey = account
            .map(|a| Pubkey::try_from(a.pubkey.as_slice()).unwrap_or_default())
            .unwrap_or_default();

        let owner = account
            .map(|a| Pubkey::try_from(a.owner.as_slice()).unwrap_or_default())
            .unwrap_or_default();

        let data = account.map(|a| a.data.clone()).unwrap_or_default();
        let lamports = account.map(|a| a.lamports).unwrap_or_default();
        let executable = account.map(|a| a.executable).unwrap_or_default();
        let rent_epoch = account.map(|a| a.rent_epoch).unwrap_or_default();

        Self {
            account: pubkey,
            slot: update.slot,
            data,
            owner,
            lamports,
            executable,
            rent_epoch,
        }
    }
}

#[derive(Clone)]
pub struct GrpcTransactionUpdate {
    pub signature: String,
    pub slot: u64,
    pub is_vote: bool,
    pub transaction: Option<Transaction>,
    pub meta: Option<TransactionStatusMeta>,
}

impl GrpcTransactionUpdate {
    fn from_grpc(update: SubscribeUpdateTransaction) -> Self {
        let tx = update.transaction.as_ref();
        let signature = tx
            .map(|t| bs58::encode(&t.signature).into_string())
            .unwrap_or_default();

        let transaction = tx.and_then(|t| t.transaction.clone());
        let meta = tx.and_then(|t| t.meta.clone());

        Self {
            signature,
            slot: update.slot,
            is_vote: tx.map(|t| t.is_vote).unwrap_or(false),
            transaction,
            meta,
        }
    }
}
