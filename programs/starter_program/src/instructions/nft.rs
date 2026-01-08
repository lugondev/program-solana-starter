use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{self, Mint, MintTo, Token, TokenAccount, Transfer};

use crate::constants::*;
use crate::error::ErrorCode;
use crate::events::*;
use crate::state::nft::*;

#[derive(Accounts)]
#[instruction(name: String, symbol: String, uri: String)]
pub struct CreateCollection<'info> {
    #[account(
        init,
        payer = authority,
        space = NftCollection::LEN,
        seeds = [SEED_NFT_COLLECTION, collection_mint.key().as_ref()],
        bump
    )]
    pub collection: Account<'info, NftCollection>,

    #[account(mut)]
    pub collection_mint: Account<'info, Mint>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
}

pub fn create_collection_handler(
    ctx: Context<CreateCollection>,
    name: String,
    symbol: String,
    uri: String,
    seller_fee_basis_points: u16,
    total_supply: u64,
    is_mutable: bool,
) -> Result<()> {
    require!(
        name.len() <= NftCollection::MAX_NAME_LENGTH,
        ErrorCode::InvalidAmount
    );
    require!(
        symbol.len() <= NftCollection::MAX_SYMBOL_LENGTH,
        ErrorCode::InvalidAmount
    );
    require!(
        uri.len() <= NftCollection::MAX_URI_LENGTH,
        ErrorCode::InvalidAmount
    );
    require!(seller_fee_basis_points <= 10000, ErrorCode::InvalidAmount);

    let collection = &mut ctx.accounts.collection;
    let clock = Clock::get()?;

    collection.authority = ctx.accounts.authority.key();
    collection.collection_mint = ctx.accounts.collection_mint.key();
    collection.name = name.clone();
    collection.symbol = symbol.clone();
    collection.uri = uri.clone();
    collection.seller_fee_basis_points = seller_fee_basis_points;
    collection.total_supply = total_supply;
    collection.minted_count = 0;
    collection.is_mutable = is_mutable;
    collection.created_at = clock.unix_timestamp;
    collection.bump = ctx.bumps.collection;

    emit!(NftCollectionCreatedEvent {
        collection: collection.key(),
        authority: ctx.accounts.authority.key(),
        name,
        symbol,
        timestamp: clock.unix_timestamp,
    });

    Ok(())
}

#[derive(Accounts)]
#[instruction(name: String, uri: String)]
pub struct MintNft<'info> {
    #[account(
        mut,
        seeds = [SEED_NFT_COLLECTION, collection.collection_mint.as_ref()],
        bump = collection.bump,
        has_one = authority @ ErrorCode::Unauthorized
    )]
    pub collection: Account<'info, NftCollection>,

    #[account(
        init,
        payer = authority,
        space = NftMetadata::LEN,
        seeds = [SEED_NFT_METADATA, nft_mint.key().as_ref()],
        bump
    )]
    pub nft_metadata: Account<'info, NftMetadata>,

    #[account(mut)]
    pub nft_mint: Signer<'info>,

    #[account(
        init_if_needed,
        payer = authority,
        associated_token::mint = nft_mint,
        associated_token::authority = recipient
    )]
    pub recipient_token_account: Account<'info, TokenAccount>,

    /// CHECK: Recipient can be any account
    pub recipient: AccountInfo<'info>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn mint_nft_handler(
    ctx: Context<MintNft>,
    name: String,
    uri: String,
    creators: Vec<Creator>,
) -> Result<()> {
    let collection = &mut ctx.accounts.collection;

    require!(
        collection.total_supply == 0 || collection.minted_count < collection.total_supply,
        ErrorCode::InvalidAmount
    );
    require!(
        name.len() <= NftMetadata::MAX_NAME_LENGTH,
        ErrorCode::InvalidAmount
    );
    require!(
        uri.len() <= NftMetadata::MAX_URI_LENGTH,
        ErrorCode::InvalidAmount
    );
    require!(
        creators.len() <= NftMetadata::MAX_CREATORS,
        ErrorCode::InvalidAmount
    );

    let total_share: u16 = creators.iter().map(|c| c.share as u16).sum();
    require!(total_share == 100, ErrorCode::InvalidAmount);

    let clock = Clock::get()?;
    let nft_metadata = &mut ctx.accounts.nft_metadata;

    nft_metadata.mint = ctx.accounts.nft_mint.key();
    nft_metadata.collection = collection.key();
    nft_metadata.owner = ctx.accounts.recipient.key();
    nft_metadata.name = name.clone();
    nft_metadata.symbol = collection.symbol.clone();
    nft_metadata.uri = uri.clone();
    nft_metadata.seller_fee_basis_points = collection.seller_fee_basis_points;
    nft_metadata.creators = creators;
    nft_metadata.is_mutable = collection.is_mutable;
    nft_metadata.minted_at = clock.unix_timestamp;
    nft_metadata.bump = ctx.bumps.nft_metadata;

    collection.minted_count = collection
        .minted_count
        .checked_add(1)
        .ok_or(ErrorCode::ArithmeticOverflow)?;

    token::mint_to(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            MintTo {
                mint: ctx.accounts.nft_mint.to_account_info(),
                to: ctx.accounts.recipient_token_account.to_account_info(),
                authority: ctx.accounts.authority.to_account_info(),
            },
        ),
        1,
    )?;

    emit!(NftMintedEvent {
        nft_mint: ctx.accounts.nft_mint.key(),
        collection: collection.key(),
        owner: ctx.accounts.recipient.key(),
        name,
        uri,
        timestamp: clock.unix_timestamp,
    });

    Ok(())
}

#[derive(Accounts)]
pub struct UpdateMetadata<'info> {
    #[account(
        seeds = [SEED_NFT_COLLECTION, collection.collection_mint.as_ref()],
        bump = collection.bump
    )]
    pub collection: Account<'info, NftCollection>,

    #[account(
        mut,
        seeds = [SEED_NFT_METADATA, nft_metadata.mint.as_ref()],
        bump = nft_metadata.bump,
        constraint = nft_metadata.collection == collection.key() @ ErrorCode::InvalidMint,
        constraint = nft_metadata.is_mutable @ ErrorCode::InvalidStateTransition
    )]
    pub nft_metadata: Account<'info, NftMetadata>,

    #[account(
        constraint = authority.key() == collection.authority || authority.key() == nft_metadata.owner @ ErrorCode::Unauthorized
    )]
    pub authority: Signer<'info>,
}

pub fn update_metadata_handler(
    ctx: Context<UpdateMetadata>,
    name: Option<String>,
    uri: Option<String>,
) -> Result<()> {
    let nft_metadata = &mut ctx.accounts.nft_metadata;

    if let Some(new_name) = name {
        require!(
            new_name.len() <= NftMetadata::MAX_NAME_LENGTH,
            ErrorCode::InvalidAmount
        );
        nft_metadata.name = new_name;
    }

    if let Some(new_uri) = uri {
        require!(
            new_uri.len() <= NftMetadata::MAX_URI_LENGTH,
            ErrorCode::InvalidAmount
        );
        nft_metadata.uri = new_uri;
    }

    Ok(())
}

#[derive(Accounts)]
pub struct ListNft<'info> {
    #[account(
        init,
        payer = seller,
        space = NftListing::LEN,
        seeds = [SEED_NFT_LISTING, nft_mint.key().as_ref()],
        bump
    )]
    pub listing: Account<'info, NftListing>,

    #[account(
        seeds = [SEED_NFT_METADATA, nft_mint.key().as_ref()],
        bump = nft_metadata.bump,
        constraint = nft_metadata.owner == seller.key() @ ErrorCode::Unauthorized
    )]
    pub nft_metadata: Account<'info, NftMetadata>,

    pub nft_mint: Account<'info, Mint>,

    #[account(
        mut,
        constraint = nft_token_account.mint == nft_mint.key() @ ErrorCode::InvalidMint,
        constraint = nft_token_account.owner == seller.key() @ ErrorCode::Unauthorized,
        constraint = nft_token_account.amount == 1 @ ErrorCode::InsufficientBalance
    )]
    pub nft_token_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub seller: Signer<'info>,

    pub system_program: Program<'info, System>,
}

pub fn list_nft_handler(
    ctx: Context<ListNft>,
    price: u64,
    currency_mint: Option<Pubkey>,
    expires_at: Option<i64>,
) -> Result<()> {
    require!(price > 0, ErrorCode::InvalidAmount);

    let clock = Clock::get()?;
    let listing = &mut ctx.accounts.listing;

    listing.seller = ctx.accounts.seller.key();
    listing.nft_mint = ctx.accounts.nft_mint.key();
    listing.nft_token_account = ctx.accounts.nft_token_account.key();
    listing.price = price;
    listing.currency_mint = currency_mint;
    listing.listed_at = clock.unix_timestamp;
    listing.expires_at = expires_at;
    listing.bump = ctx.bumps.listing;

    emit!(NftListedEvent {
        nft_mint: ctx.accounts.nft_mint.key(),
        seller: ctx.accounts.seller.key(),
        price,
        timestamp: clock.unix_timestamp,
    });

    Ok(())
}

#[derive(Accounts)]
pub struct BuyNft<'info> {
    #[account(
        mut,
        seeds = [SEED_NFT_LISTING, nft_mint.key().as_ref()],
        bump = listing.bump,
        close = seller
    )]
    pub listing: Account<'info, NftListing>,

    #[account(
        mut,
        seeds = [SEED_NFT_METADATA, nft_mint.key().as_ref()],
        bump = nft_metadata.bump
    )]
    pub nft_metadata: Account<'info, NftMetadata>,

    pub nft_mint: Account<'info, Mint>,

    #[account(
        mut,
        constraint = seller_nft_account.key() == listing.nft_token_account @ ErrorCode::InvalidTokenAccount,
        constraint = seller_nft_account.amount == 1 @ ErrorCode::InsufficientBalance
    )]
    pub seller_nft_account: Account<'info, TokenAccount>,

    #[account(
        init_if_needed,
        payer = buyer,
        associated_token::mint = nft_mint,
        associated_token::authority = buyer
    )]
    pub buyer_nft_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub buyer: Signer<'info>,

    #[account(
        mut,
        constraint = seller.key() == listing.seller @ ErrorCode::Unauthorized
    )]
    /// CHECK: Seller receives payment
    pub seller: AccountInfo<'info>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

pub fn buy_nft_handler(ctx: Context<BuyNft>) -> Result<()> {
    let listing = &ctx.accounts.listing;
    let clock = Clock::get()?;

    require!(
        !listing.is_expired(clock.unix_timestamp),
        ErrorCode::InvalidStateTransition
    );

    require!(listing.currency_mint.is_none(), ErrorCode::InvalidMint);

    let transfer_instruction = anchor_lang::solana_program::system_instruction::transfer(
        &ctx.accounts.buyer.key(),
        &ctx.accounts.seller.key(),
        listing.price,
    );

    anchor_lang::solana_program::program::invoke(
        &transfer_instruction,
        &[
            ctx.accounts.buyer.to_account_info(),
            ctx.accounts.seller.to_account_info(),
        ],
    )?;

    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.seller_nft_account.to_account_info(),
                to: ctx.accounts.buyer_nft_account.to_account_info(),
                authority: ctx.accounts.seller.to_account_info(),
            },
        ),
        1,
    )?;

    let nft_metadata = &mut ctx.accounts.nft_metadata;
    nft_metadata.owner = ctx.accounts.buyer.key();

    emit!(NftSoldEvent {
        nft_mint: ctx.accounts.nft_mint.key(),
        seller: ctx.accounts.seller.key(),
        buyer: ctx.accounts.buyer.key(),
        price: listing.price,
        timestamp: clock.unix_timestamp,
    });

    Ok(())
}

#[derive(Accounts)]
pub struct CancelListing<'info> {
    #[account(
        mut,
        seeds = [SEED_NFT_LISTING, nft_mint.key().as_ref()],
        bump = listing.bump,
        has_one = seller @ ErrorCode::Unauthorized,
        close = seller
    )]
    pub listing: Account<'info, NftListing>,

    pub nft_mint: Account<'info, Mint>,

    #[account(mut)]
    pub seller: Signer<'info>,
}

pub fn cancel_listing_handler(ctx: Context<CancelListing>) -> Result<()> {
    let clock = Clock::get()?;

    emit!(NftListingCancelledEvent {
        nft_mint: ctx.accounts.nft_mint.key(),
        seller: ctx.accounts.seller.key(),
        timestamp: clock.unix_timestamp,
    });

    Ok(())
}

#[derive(Accounts)]
pub struct CreateOffer<'info> {
    #[account(
        init,
        payer = buyer,
        space = NftOffer::LEN,
        seeds = [SEED_NFT_OFFER, nft_mint.key().as_ref(), buyer.key().as_ref()],
        bump
    )]
    pub offer: Account<'info, NftOffer>,

    pub nft_mint: Account<'info, Mint>,

    #[account(
        seeds = [SEED_NFT_METADATA, nft_mint.key().as_ref()],
        bump = nft_metadata.bump
    )]
    pub nft_metadata: Account<'info, NftMetadata>,

    /// CHECK: Escrow account for holding offer funds
    #[account(mut)]
    pub escrow_account: AccountInfo<'info>,

    #[account(mut)]
    pub buyer: Signer<'info>,

    pub system_program: Program<'info, System>,
}

pub fn create_offer_handler(
    ctx: Context<CreateOffer>,
    offer_amount: u64,
    currency_mint: Option<Pubkey>,
    expires_at: i64,
) -> Result<()> {
    require!(offer_amount > 0, ErrorCode::InvalidAmount);

    let clock = Clock::get()?;
    require!(expires_at > clock.unix_timestamp, ErrorCode::InvalidAmount);

    let transfer_instruction = anchor_lang::solana_program::system_instruction::transfer(
        &ctx.accounts.buyer.key(),
        &ctx.accounts.escrow_account.key(),
        offer_amount,
    );

    anchor_lang::solana_program::program::invoke(
        &transfer_instruction,
        &[
            ctx.accounts.buyer.to_account_info(),
            ctx.accounts.escrow_account.to_account_info(),
        ],
    )?;

    let offer = &mut ctx.accounts.offer;
    offer.buyer = ctx.accounts.buyer.key();
    offer.nft_mint = ctx.accounts.nft_mint.key();
    offer.offer_amount = offer_amount;
    offer.currency_mint = currency_mint;
    offer.escrow_account = ctx.accounts.escrow_account.key();
    offer.created_at = clock.unix_timestamp;
    offer.expires_at = expires_at;
    offer.bump = ctx.bumps.offer;

    emit!(NftOfferCreatedEvent {
        nft_mint: ctx.accounts.nft_mint.key(),
        buyer: ctx.accounts.buyer.key(),
        offer_amount,
        timestamp: clock.unix_timestamp,
    });

    Ok(())
}

#[derive(Accounts)]
pub struct AcceptOffer<'info> {
    #[account(
        mut,
        seeds = [SEED_NFT_OFFER, nft_mint.key().as_ref(), buyer.key().as_ref()],
        bump = offer.bump,
        close = buyer
    )]
    pub offer: Account<'info, NftOffer>,

    #[account(
        mut,
        seeds = [SEED_NFT_METADATA, nft_mint.key().as_ref()],
        bump = nft_metadata.bump,
        has_one = owner @ ErrorCode::Unauthorized
    )]
    pub nft_metadata: Account<'info, NftMetadata>,

    pub nft_mint: Account<'info, Mint>,

    #[account(
        mut,
        constraint = seller_nft_account.mint == nft_mint.key() @ ErrorCode::InvalidMint,
        constraint = seller_nft_account.owner == owner.key() @ ErrorCode::Unauthorized,
        constraint = seller_nft_account.amount == 1 @ ErrorCode::InsufficientBalance
    )]
    pub seller_nft_account: Account<'info, TokenAccount>,

    #[account(
        init_if_needed,
        payer = owner,
        associated_token::mint = nft_mint,
        associated_token::authority = buyer
    )]
    pub buyer_nft_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = escrow_account.key() == offer.escrow_account @ ErrorCode::InvalidTokenAccount
    )]
    /// CHECK: Escrow account holding offer funds
    pub escrow_account: AccountInfo<'info>,

    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(mut)]
    /// CHECK: Buyer receives NFT
    pub buyer: AccountInfo<'info>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

pub fn accept_offer_handler(ctx: Context<AcceptOffer>) -> Result<()> {
    let offer = &ctx.accounts.offer;
    let clock = Clock::get()?;

    require!(
        !offer.is_expired(clock.unix_timestamp),
        ErrorCode::InvalidStateTransition
    );

    require!(offer.currency_mint.is_none(), ErrorCode::InvalidMint);

    let escrow_lamports = ctx.accounts.escrow_account.lamports();
    require!(
        escrow_lamports >= offer.offer_amount,
        ErrorCode::InsufficientBalance
    );

    **ctx.accounts.escrow_account.try_borrow_mut_lamports()? -= offer.offer_amount;
    **ctx.accounts.owner.try_borrow_mut_lamports()? += offer.offer_amount;

    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.seller_nft_account.to_account_info(),
                to: ctx.accounts.buyer_nft_account.to_account_info(),
                authority: ctx.accounts.owner.to_account_info(),
            },
        ),
        1,
    )?;

    let nft_metadata = &mut ctx.accounts.nft_metadata;
    nft_metadata.owner = ctx.accounts.buyer.key();

    emit!(NftOfferAcceptedEvent {
        nft_mint: ctx.accounts.nft_mint.key(),
        seller: ctx.accounts.owner.key(),
        buyer: ctx.accounts.buyer.key(),
        amount: offer.offer_amount,
        timestamp: clock.unix_timestamp,
    });

    Ok(())
}
