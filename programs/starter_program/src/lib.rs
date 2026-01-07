pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("gARh1g6reuvsAHB7DXqiuYzzyiJeoiJmtmCpV8Y5uWC");

#[program]
pub mod starter_program {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        initialize::handler(ctx)
    }

    pub fn initialize_config(ctx: Context<InitializeConfig>, fee_destination: Pubkey) -> Result<()> {
        initialize_config_handler(ctx, fee_destination)
    }

    pub fn update_config(
        ctx: Context<UpdateConfig>,
        new_admin: Pubkey,
        new_fee_destination: Pubkey,
        new_fee: u64
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

    pub fn transfer_tokens_with_pda(ctx: Context<TransferTokensWithPda>, amount: u64) -> Result<()> {
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
        payment: u64
    ) -> Result<()> {
        increment_with_payment_from_pda_handler(ctx, payment)
    }
}

