use anchor_lang::prelude::*;

#[account]
pub struct NftCollection {
    pub authority: Pubkey,
    pub collection_mint: Pubkey,
    pub name: String,
    pub symbol: String,
    pub uri: String,
    pub seller_fee_basis_points: u16,
    pub total_supply: u64,
    pub minted_count: u64,
    pub is_mutable: bool,
    pub created_at: i64,
    pub bump: u8,
}

impl NftCollection {
    pub const MAX_NAME_LENGTH: usize = 32;
    pub const MAX_SYMBOL_LENGTH: usize = 10;
    pub const MAX_URI_LENGTH: usize = 200;

    pub const LEN: usize = 8 + // discriminator
        32 +  // authority
        32 +  // collection_mint
        4 + Self::MAX_NAME_LENGTH +  // name (String with length prefix)
        4 + Self::MAX_SYMBOL_LENGTH + // symbol
        4 + Self::MAX_URI_LENGTH +    // uri
        2 +   // seller_fee_basis_points
        8 +   // total_supply
        8 +   // minted_count
        1 +   // is_mutable
        8 +   // created_at
        1; // bump
}

#[account]
pub struct NftMetadata {
    pub mint: Pubkey,
    pub collection: Pubkey,
    pub owner: Pubkey,
    pub name: String,
    pub symbol: String,
    pub uri: String,
    pub seller_fee_basis_points: u16,
    pub creators: Vec<Creator>,
    pub is_mutable: bool,
    pub minted_at: i64,
    pub bump: u8,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct Creator {
    pub address: Pubkey,
    pub verified: bool,
    pub share: u8,
}

impl NftMetadata {
    pub const MAX_NAME_LENGTH: usize = 32;
    pub const MAX_SYMBOL_LENGTH: usize = 10;
    pub const MAX_URI_LENGTH: usize = 200;
    pub const MAX_CREATORS: usize = 5;

    pub const LEN: usize = 8 + // discriminator
        32 +  // mint
        32 +  // collection
        32 +  // owner
        4 + Self::MAX_NAME_LENGTH +  // name
        4 + Self::MAX_SYMBOL_LENGTH + // symbol
        4 + Self::MAX_URI_LENGTH +    // uri
        2 +   // seller_fee_basis_points
        4 + (Self::MAX_CREATORS * (32 + 1 + 1)) + // creators (Vec with length prefix)
        1 +   // is_mutable
        8 +   // minted_at
        1; // bump
}

#[account]
pub struct NftListing {
    pub seller: Pubkey,
    pub nft_mint: Pubkey,
    pub nft_token_account: Pubkey,
    pub price: u64,
    pub currency_mint: Option<Pubkey>,
    pub listed_at: i64,
    pub expires_at: Option<i64>,
    pub bump: u8,
}

impl NftListing {
    pub const LEN: usize = 8 + // discriminator
        32 +  // seller
        32 +  // nft_mint
        32 +  // nft_token_account
        8 +   // price
        1 + 32 + // currency_mint (Option<Pubkey>)
        8 +   // listed_at
        1 + 8 + // expires_at (Option<i64>)
        1; // bump

    pub fn is_expired(&self, current_timestamp: i64) -> bool {
        if let Some(expires_at) = self.expires_at {
            current_timestamp > expires_at
        } else {
            false
        }
    }
}

#[account]
pub struct NftOffer {
    pub buyer: Pubkey,
    pub nft_mint: Pubkey,
    pub offer_amount: u64,
    pub currency_mint: Option<Pubkey>,
    pub escrow_account: Pubkey,
    pub created_at: i64,
    pub expires_at: i64,
    pub bump: u8,
}

impl NftOffer {
    pub const LEN: usize = 8 + // discriminator
        32 +  // buyer
        32 +  // nft_mint
        8 +   // offer_amount
        1 + 32 + // currency_mint
        32 +  // escrow_account
        8 +   // created_at
        8 +   // expires_at
        1; // bump

    pub fn is_expired(&self, current_timestamp: i64) -> bool {
        current_timestamp > self.expires_at
    }
}
