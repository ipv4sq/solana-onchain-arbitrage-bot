pub const MEV_BOT_ONCHAIN_PROGRAM_ID: &str = "MEViEnscUm6tsQRoGd9h6nLQaQspKj7DB2M5FwM3Xvz";

// 这个地址大概率是通过 seeds + program_id 生成的, 然后创建一堆token account, owner设置成这个地址.
pub const FLASHLOAN_ACCOUNT_ID: &str = "5LFpzqgsxrSfhKwbaFiAEJ2kbc9QyimjKueswsyU4T3o";

pub struct MevBotFeeCollector;
impl MevBotFeeCollector {
    pub const FLASHLOAN_FEE_ID: &'static str = "6AGB9kqgSp2mQXwYpdrV4QVV8urvCaDS35U1wsLssy6H";

    pub const NON_FLASHLOAN_FEE_ID_1: &'static str = "GPpkDpzCDmYJY5qNhYmM14c7rct1zmkjWc2CjR5g7RZ1";
    pub const NON_FLASHLOAN_FEE_ID_2: &'static str = "J6c7noBHvWju4mMA3wXt3igbBSp2m9ATbA6cjMtAUged";
    pub const NON_FLASHLOAN_FEE_ID_3: &'static str = "BjsfwxDu7GX7RRW6oSRTpMkASdXAgCcHnXEcatqSfuuY";
}
