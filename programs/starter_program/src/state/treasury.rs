use anchor_lang::prelude::*;

#[account]
pub struct Treasury {
    pub authority: Pubkey,
    pub total_deposited: u64,
    pub total_withdrawn: u64,
    pub emergency_mode: bool,
    pub circuit_breaker_active: bool,
    pub created_at: i64,
    pub bump: u8,
}

impl Treasury {
    pub const LEN: usize = 8 + 32 + 8 + 8 + 1 + 1 + 8 + 1;
}
