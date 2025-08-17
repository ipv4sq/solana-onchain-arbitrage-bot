use crate::constants::helpers::ToPubkey;
use lazy_static::lazy_static;
use solana_sdk::pubkey::Pubkey;

pub const EMV_BOT_PROGRAM_ID: &str = "MEViEnscUm6tsQRoGd9h6nLQaQspKj7DB2M5FwM3Xvz";
pub const FLASHLOAN_ACCOUNT_ID: &str = "5LFpzqgsxrSfhKwbaFiAEJ2kbc9QyimjKueswsyU4T3o";

pub struct SmbFeeCollector;

impl SmbFeeCollector {
    pub const FLASHLOAN_FEE_ID: &'static str = "6AGB9kqgSp2mQXwYpdrV4QVV8urvCaDS35U1wsLssy6H";
    pub const NON_FLASHLOAN_FEE_ID_1: &'static str = "GPpkDpzCDmYJY5qNhYmM14c7rct1zmkjWc2CjR5g7RZ1";
    pub const NON_FLASHLOAN_FEE_ID_2: &'static str = "J6c7noBHvWju4mMA3wXt3igbBSp2m9ATbA6cjMtAUged";
    pub const NON_FLASHLOAN_FEE_ID_3: &'static str = "BjsfwxDu7GX7RRW6oSRTpMkASdXAgCcHnXEcatqSfuuY";
}

lazy_static! {
    // Direct Pubkey constants that can be used without .to_pubkey()
    pub static ref SMB_ONCHAIN_PROGRAM: Pubkey = EMV_BOT_PROGRAM_ID.to_pubkey();
    pub static ref FLASHLOAN_ACCOUNT: Pubkey = FLASHLOAN_ACCOUNT_ID.to_pubkey();
}

lazy_static! {
    pub static ref FLASHLOAN_FEE: Pubkey = SmbFeeCollector::FLASHLOAN_FEE_ID.to_pubkey();
    pub static ref NON_FLASHLOAN_FEE_1: Pubkey =
        SmbFeeCollector::NON_FLASHLOAN_FEE_ID_1.to_pubkey();
    pub static ref NON_FLASHLOAN_FEE_2: Pubkey =
        SmbFeeCollector::NON_FLASHLOAN_FEE_ID_2.to_pubkey();
    pub static ref NON_FLASHLOAN_FEE_3: Pubkey =
        SmbFeeCollector::NON_FLASHLOAN_FEE_ID_3.to_pubkey();
}
