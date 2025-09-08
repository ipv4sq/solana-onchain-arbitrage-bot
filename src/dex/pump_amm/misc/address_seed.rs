use crate::global::constant::pool_program::PoolProgram;
use solana_program::pubkey::Pubkey;

pub fn get_coin_creator_vault_authority(coin_creator: &Pubkey) -> Pubkey {
    let (addr, _) = Pubkey::find_program_address(
        &[b"creator_vault", coin_creator.as_ref()],
        &PoolProgram::PUMP_AMM,
    );
    addr
}

pub fn get_global_volume_accumulator() -> Pubkey {
    let (addr, _) =
        Pubkey::find_program_address(&[b"global_volume_accumulator"], &PoolProgram::PUMP_AMM);
    addr
}

pub fn get_user_volume_accumulator(user: &Pubkey) -> Pubkey {
    let (addr, _) = Pubkey::find_program_address(
        &[b"user_volume_accumulator", user.as_ref()],
        &PoolProgram::PUMP_AMM,
    );
    addr
}
