use anchor_lang::prelude::*;

#[event]
pub struct TokensMintedEvent {
    pub mint: Pubkey,
    pub recipient: Pubkey,
    pub amount: u64,
    pub timestamp: i64,
}

#[event]
pub struct TokensTransferredEvent {
    pub mint: Pubkey,
    pub from: Pubkey,
    pub to: Pubkey,
    pub amount: u64,
    pub timestamp: i64,
}

#[event]
pub struct TokensBurnedEvent {
    pub mint: Pubkey,
    pub owner: Pubkey,
    pub amount: u64,
    pub timestamp: i64,
}

#[event]
pub struct DelegateApprovedEvent {
    pub token_account: Pubkey,
    pub delegate: Pubkey,
    pub amount: u64,
    pub timestamp: i64,
}

#[event]
pub struct DelegateRevokedEvent {
    pub token_account: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct TokenAccountClosedEvent {
    pub token_account: Pubkey,
    pub destination: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct TokenAccountFrozenEvent {
    pub token_account: Pubkey,
    pub mint: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct TokenAccountThawedEvent {
    pub token_account: Pubkey,
    pub mint: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct UserAccountCreatedEvent {
    pub user: Pubkey,
    pub authority: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct UserAccountUpdatedEvent {
    pub user: Pubkey,
    pub old_points: u64,
    pub new_points: u64,
    pub timestamp: i64,
}

#[event]
pub struct UserAccountClosedEvent {
    pub user: Pubkey,
    pub authority: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct ConfigUpdatedEvent {
    pub admin: Pubkey,
    pub old_fee: u64,
    pub new_fee: u64,
    pub timestamp: i64,
}

#[event]
pub struct ProgramPausedEvent {
    pub admin: Pubkey,
    pub paused: bool,
    pub timestamp: i64,
}
