pub mod constants;
pub mod error;
pub mod events;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use error::*;
pub use events::*;
pub use instructions::*;
pub use state::*;

declare_id!("gARh1g6reuvsAHB7DXqiuYzzyiJeoiJmtmCpV8Y5uWC");

#[program]
pub mod starter_program {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        initialize::handler(ctx)
    }

    pub fn initialize_config(
        ctx: Context<InitializeConfig>,
        fee_destination: Pubkey,
    ) -> Result<()> {
        initialize_config_handler(ctx, fee_destination)
    }

    pub fn update_config(
        ctx: Context<UpdateConfig>,
        new_admin: Pubkey,
        new_fee_destination: Pubkey,
        new_fee: u64,
    ) -> Result<()> {
        update_config_handler(ctx, new_admin, new_fee_destination, new_fee)
    }

    pub fn toggle_pause(ctx: Context<TogglePause>) -> Result<()> {
        toggle_pause_handler(ctx)
    }

    pub fn create_user_account(ctx: Context<CreateUserAccount>) -> Result<()> {
        create_user_account_handler(ctx)
    }

    pub fn update_user_account(ctx: Context<UpdateUserAccount>, new_points: u64) -> Result<()> {
        update_user_account_handler(ctx, new_points)
    }

    pub fn close_user_account(ctx: Context<CloseUserAccount>) -> Result<()> {
        close_user_account_handler(ctx)
    }

    pub fn create_mint(ctx: Context<CreateMint>) -> Result<()> {
        create_mint_handler(ctx)
    }

    pub fn mint_tokens(ctx: Context<MintTokens>, amount: u64) -> Result<()> {
        mint_tokens_handler(ctx, amount)
    }

    pub fn transfer_tokens(ctx: Context<TransferTokens>, amount: u64) -> Result<()> {
        transfer_tokens_handler(ctx, amount)
    }

    pub fn burn_tokens(ctx: Context<BurnTokens>, amount: u64) -> Result<()> {
        burn_tokens_handler(ctx, amount)
    }

    pub fn transfer_sol(ctx: Context<TransferSol>, amount: u64) -> Result<()> {
        transfer_sol_handler(ctx, amount)
    }

    pub fn transfer_sol_with_pda(ctx: Context<TransferSolWithPda>, amount: u64) -> Result<()> {
        transfer_sol_with_pda_handler(ctx, amount)
    }

    pub fn transfer_tokens_with_pda(
        ctx: Context<TransferTokensWithPda>,
        amount: u64,
    ) -> Result<()> {
        transfer_tokens_with_pda_handler(ctx, amount)
    }

    pub fn initialize_counter(ctx: Context<InitializeCounter>) -> Result<()> {
        initialize_counter_handler(ctx)
    }

    pub fn increment_counter(ctx: Context<IncrementCounter>) -> Result<()> {
        increment_counter_handler(ctx)
    }

    pub fn add_to_counter(ctx: Context<AddToCounter>, value: u64) -> Result<()> {
        add_to_counter_handler(ctx, value)
    }

    pub fn increment_multiple(ctx: Context<IncrementMultiple>, times: u8) -> Result<()> {
        increment_multiple_handler(ctx, times)
    }

    pub fn increment_with_payment_from_pda(
        ctx: Context<IncrementWithPaymentFromPda>,
        payment: u64,
    ) -> Result<()> {
        increment_with_payment_from_pda_handler(ctx, payment)
    }

    pub fn assign_role(ctx: Context<AssignRole>, role_type: RoleType) -> Result<()> {
        assign_role_handler(ctx, role_type)
    }

    pub fn update_role_permissions(
        ctx: Context<UpdateRole>,
        add_permissions: u8,
        remove_permissions: u8,
    ) -> Result<()> {
        update_role_permissions_handler(ctx, add_permissions, remove_permissions)
    }

    pub fn revoke_role(ctx: Context<RevokeRole>) -> Result<()> {
        revoke_role_handler(ctx)
    }

    pub fn check_permission(
        ctx: Context<CheckPermission>,
        required_permission: u8,
    ) -> Result<bool> {
        check_permission_handler(ctx, required_permission)
    }

    pub fn approve_delegate(ctx: Context<ApproveDelegate>, amount: u64) -> Result<()> {
        approve_delegate_handler(ctx, amount)
    }

    pub fn revoke_delegate(ctx: Context<RevokeDelegate>) -> Result<()> {
        revoke_delegate_handler(ctx)
    }

    pub fn close_token_account(ctx: Context<CloseTokenAccount>) -> Result<()> {
        close_token_account_handler(ctx)
    }

    pub fn freeze_token_account(ctx: Context<FreezeTokenAccount>) -> Result<()> {
        freeze_token_account_handler(ctx)
    }

    pub fn thaw_token_account(ctx: Context<ThawTokenAccount>) -> Result<()> {
        thaw_token_account_handler(ctx)
    }

    pub fn initialize_treasury(ctx: Context<InitializeTreasury>) -> Result<()> {
        initialize_treasury_handler(ctx)
    }

    pub fn deposit_to_treasury(ctx: Context<DepositToTreasury>, amount: u64) -> Result<()> {
        deposit_to_treasury_handler(ctx, amount)
    }

    pub fn withdraw_from_treasury(ctx: Context<WithdrawFromTreasury>, amount: u64) -> Result<()> {
        withdraw_from_treasury_handler(ctx, amount)
    }

    pub fn emergency_withdraw(ctx: Context<EmergencyWithdraw>) -> Result<()> {
        emergency_withdraw_handler(ctx)
    }

    pub fn toggle_circuit_breaker(ctx: Context<ToggleCircuitBreaker>) -> Result<()> {
        toggle_circuit_breaker_handler(ctx)
    }

    pub fn create_collection(
        ctx: Context<CreateCollection>,
        name: String,
        symbol: String,
        uri: String,
        seller_fee_basis_points: u16,
        total_supply: u64,
        is_mutable: bool,
    ) -> Result<()> {
        create_collection_handler(
            ctx,
            name,
            symbol,
            uri,
            seller_fee_basis_points,
            total_supply,
            is_mutable,
        )
    }

    pub fn mint_nft(
        ctx: Context<MintNft>,
        name: String,
        uri: String,
        creators: Vec<crate::state::nft::Creator>,
    ) -> Result<()> {
        mint_nft_handler(ctx, name, uri, creators)
    }

    pub fn update_nft_metadata(
        ctx: Context<UpdateMetadata>,
        name: Option<String>,
        uri: Option<String>,
    ) -> Result<()> {
        update_metadata_handler(ctx, name, uri)
    }

    pub fn list_nft(
        ctx: Context<ListNft>,
        price: u64,
        currency_mint: Option<Pubkey>,
        expires_at: Option<i64>,
    ) -> Result<()> {
        list_nft_handler(ctx, price, currency_mint, expires_at)
    }

    pub fn buy_nft(ctx: Context<BuyNft>) -> Result<()> {
        buy_nft_handler(ctx)
    }

    pub fn cancel_nft_listing(ctx: Context<CancelListing>) -> Result<()> {
        cancel_listing_handler(ctx)
    }

    pub fn create_nft_offer(
        ctx: Context<CreateOffer>,
        offer_amount: u64,
        currency_mint: Option<Pubkey>,
        expires_at: i64,
    ) -> Result<()> {
        create_offer_handler(ctx, offer_amount, currency_mint, expires_at)
    }

    pub fn accept_nft_offer(ctx: Context<AcceptOffer>) -> Result<()> {
        accept_offer_handler(ctx)
    }

    pub fn initialize_upgrade_authority(
        ctx: Context<InitializeUpgradeAuthority>,
        voting_threshold: u8,
        voting_period_seconds: i64,
        execution_delay_seconds: i64,
    ) -> Result<()> {
        initialize_upgrade_authority_handler(
            ctx,
            voting_threshold,
            voting_period_seconds,
            execution_delay_seconds,
        )
    }

    pub fn transfer_upgrade_authority(ctx: Context<TransferUpgradeAuthority>) -> Result<()> {
        transfer_upgrade_authority_handler(ctx)
    }

    pub fn accept_upgrade_authority(ctx: Context<AcceptUpgradeAuthority>) -> Result<()> {
        accept_upgrade_authority_handler(ctx)
    }

    pub fn create_upgrade_proposal(
        ctx: Context<CreateUpgradeProposal>,
        proposal_id: u64,
        description: String,
    ) -> Result<()> {
        create_upgrade_proposal_handler(ctx, proposal_id, description)
    }

    pub fn cast_vote(
        ctx: Context<CastVote>,
        proposal_id: u64,
        in_favor: bool,
        voting_power: u64,
    ) -> Result<()> {
        cast_vote_handler(ctx, proposal_id, in_favor, voting_power)
    }

    pub fn execute_proposal(
        ctx: Context<ExecuteProposal>,
        proposal_id: u64,
        old_version: String,
        new_version: String,
    ) -> Result<()> {
        execute_proposal_handler(ctx, proposal_id, old_version, new_version)
    }

    pub fn cancel_proposal(ctx: Context<CancelProposal>, proposal_id: u64) -> Result<()> {
        cancel_proposal_handler(ctx, proposal_id)
    }
}
