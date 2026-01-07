use anchor_lang::prelude::*;
use crate::{state::ProgramConfig, constants::*, error::ErrorCode};

#[derive(Accounts)]
pub struct InitializeConfig<'info> {
    #[account(
        init,
        payer = authority,
        space = ProgramConfig::LEN,
        seeds = [SEED_PROGRAM_CONFIG],
        bump
    )]
    pub program_config: Account<'info, ProgramConfig>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
}

pub fn initialize_config_handler(ctx: Context<InitializeConfig>, fee_destination: Pubkey) -> Result<()> {
    let config = &mut ctx.accounts.program_config;
    config.admin = ctx.accounts.authority.key();
    config.fee_destination = fee_destination;
    config.fee_basis_points = 100;
    config.paused = false;
    config.bump = ctx.bumps.program_config;

    msg!("Program config initialized by: {}", config.admin);
    Ok(())
}

#[derive(Accounts)]
pub struct UpdateConfig<'info> {
    #[account(
        mut,
        seeds = [SEED_PROGRAM_CONFIG],
        bump = program_config.bump,
        has_one = admin @ ErrorCode::Unauthorized
    )]
    pub program_config: Account<'info, ProgramConfig>,

    pub admin: Signer<'info>,
}

pub fn update_config_handler(
    ctx: Context<UpdateConfig>,
    new_admin: Pubkey,
    new_fee_destination: Pubkey,
    new_fee: u64,
) -> Result<()> {
    let config = &mut ctx.accounts.program_config;
    config.admin = new_admin;
    config.fee_destination = new_fee_destination;
    config.fee_basis_points = new_fee;

    msg!("Config updated by: {}", ctx.accounts.admin.key());
    Ok(())
}

#[derive(Accounts)]
pub struct TogglePause<'info> {
    #[account(
        mut,
        seeds = [SEED_PROGRAM_CONFIG],
        bump = program_config.bump,
        has_one = admin @ ErrorCode::Unauthorized
    )]
    pub program_config: Account<'info, ProgramConfig>,

    pub admin: Signer<'info>,
}

pub fn toggle_pause_handler(ctx: Context<TogglePause>) -> Result<()> {
    let config = &mut ctx.accounts.program_config;
    config.paused = !config.paused;

    msg!("Program paused status: {}", config.paused);
    Ok(())
}
