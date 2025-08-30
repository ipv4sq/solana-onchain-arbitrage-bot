use serde::{Deserialize, Serialize};
use strum_macros::{AsRefStr, Display};

#[derive(Debug, Clone, Serialize, Deserialize, AsRefStr, Display)]
pub enum StepType {
    AccountUpdateReceived,
    AccountUpdateDebouncing,
    AccountUpdateDebounced,
    DeterminePoolExists,
    ReceivePoolUpdate,
    IsAccountPoolData,
    TradeStrategyStarted,
    DetermineOpportunityStarted,
    DetermineOpportunityLoadingRelatedMints,
    DetermineOpportunityLoadedRelatedMints,
    DetermineOpportunityFinished,
    MevTxFired,
    MevTxTryToFile,
    MevTxReadyToBuild,
    MevIxBuilding,
    MevIxBuilt,
    MevSimulationTxRpcCall,
    MevSimulationTxRpcReturned,
    MevRealTxBuilding,
    MevRealTxRpcCall,
    MevRealTxRpcReturned,
    #[strum(default)]
    Custom(String),
}
