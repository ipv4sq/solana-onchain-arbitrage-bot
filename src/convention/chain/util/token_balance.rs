use crate::convention::chain::Transaction;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct TokenBalanceChange {
    pub mint: String,
    pub pre_balance: u64,
    pub post_balance: u64,
    pub change: i128,
    pub decimals: u8,
}

impl Transaction {
    pub fn token_balance_changes(&self) -> HashMap<String, HashMap<String, TokenBalanceChange>> {
        let meta = match &self.meta {
            Some(meta) => meta,
            None => return HashMap::new(),
        };

        let mut balance_map: HashMap<(String, String), (Option<u64>, Option<u64>, u8)> =
            HashMap::new();

        meta.pre_token_balances.iter().for_each(|pre| {
            if let Some(owner) = &pre.owner {
                let key = (pre.mint.clone(), owner.clone());
                let amount = pre.ui_token_amount.amount.parse::<u64>().unwrap_or(0);
                balance_map
                    .entry(key)
                    .or_insert((None, None, pre.ui_token_amount.decimals))
                    .0 = Some(amount);
            }
        });

        meta.post_token_balances.iter().for_each(|post| {
            if let Some(owner) = &post.owner {
                let key = (post.mint.clone(), owner.clone());
                let amount = post.ui_token_amount.amount.parse::<u64>().unwrap_or(0);
                let entry =
                    balance_map
                        .entry(key)
                        .or_insert((None, None, post.ui_token_amount.decimals));
                entry.1 = Some(amount);
                entry.2 = post.ui_token_amount.decimals;
            }
        });

        let mut result: HashMap<String, HashMap<String, TokenBalanceChange>> = HashMap::new();

        balance_map
            .into_iter()
            .for_each(|((mint, owner), (pre, post, decimals))| {
                let pre_balance = pre.unwrap_or(0);
                let post_balance = post.unwrap_or(0);
                let change = post_balance as i128 - pre_balance as i128;

                if change != 0 {
                    result
                        .entry(mint.clone())
                        .or_insert_with(HashMap::new)
                        .insert(
                            owner,
                            TokenBalanceChange {
                                mint,
                                pre_balance,
                                post_balance,
                                change,
                                decimals,
                            },
                        );
                }
            });

        result
    }
}
