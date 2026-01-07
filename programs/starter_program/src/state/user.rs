use anchor_lang::prelude::*;

#[account]
pub struct UserAccount {
    pub authority: Pubkey,
    pub points: u64,
    pub created_at: i64,
    pub updated_at: i64,
    pub bump: u8,
}

impl UserAccount {
    pub const LEN: usize = 8 + 32 + 8 + 8 + 8 + 1;
}
