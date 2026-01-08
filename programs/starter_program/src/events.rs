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

// NFT Events
#[event]
pub struct NftCollectionCreatedEvent {
    pub collection: Pubkey,
    pub authority: Pubkey,
    pub name: String,
    pub symbol: String,
    pub timestamp: i64,
}

#[event]
pub struct NftMintedEvent {
    pub nft_mint: Pubkey,
    pub collection: Pubkey,
    pub owner: Pubkey,
    pub name: String,
    pub uri: String,
    pub timestamp: i64,
}

#[event]
pub struct NftListedEvent {
    pub nft_mint: Pubkey,
    pub seller: Pubkey,
    pub price: u64,
    pub timestamp: i64,
}

#[event]
pub struct NftSoldEvent {
    pub nft_mint: Pubkey,
    pub seller: Pubkey,
    pub buyer: Pubkey,
    pub price: u64,
    pub timestamp: i64,
}

#[event]
pub struct NftListingCancelledEvent {
    pub nft_mint: Pubkey,
    pub seller: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct NftOfferCreatedEvent {
    pub nft_mint: Pubkey,
    pub buyer: Pubkey,
    pub offer_amount: u64,
    pub timestamp: i64,
}

#[event]
pub struct NftOfferAcceptedEvent {
    pub nft_mint: Pubkey,
    pub seller: Pubkey,
    pub buyer: Pubkey,
    pub amount: u64,
    pub timestamp: i64,
}

// Upgrade Events
#[event]
pub struct UpgradeAuthorityInitializedEvent {
    pub authority: Pubkey,
    pub admin: Pubkey,
    pub voting_threshold: u8,
    pub timestamp: i64,
}

#[event]
pub struct UpgradeProposalCreatedEvent {
    pub proposal_id: u64,
    pub proposer: Pubkey,
    pub new_program_data: Pubkey,
    pub description: String,
    pub timestamp: i64,
}

#[event]
pub struct VoteCastEvent {
    pub proposal_id: u64,
    pub voter: Pubkey,
    pub in_favor: bool,
    pub timestamp: i64,
}

#[event]
pub struct ProposalExecutedEvent {
    pub proposal_id: u64,
    pub executor: Pubkey,
    pub new_program_data: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct UpgradeCompletedEvent {
    pub old_version: String,
    pub new_version: String,
    pub program_data: Pubkey,
    pub timestamp: i64,
}
