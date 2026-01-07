use crate::error::ErrorCode;
use crate::events::*;
use crate::{ProgramConfig, Treasury, SEED_PROGRAM_CONFIG, SEED_TREASURY};
use anchor_lang::prelude::*;
use anchor_lang::system_program::{transfer, Transfer};

#[derive(Accounts)]
pub struct InitializeTreasury<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + Treasury::LEN,
        seeds = [SEED_TREASURY],
        bump
    )]
    pub treasury: Account<'info, Treasury>,

    #[account(
        seeds = [SEED_PROGRAM_CONFIG],
        bump = program_config.bump,
    )]
    pub program_config: Account<'info, ProgramConfig>,

    #[account(
        mut,
        constraint = authority.key() == program_config.admin @ ErrorCode::Unauthorized
    )]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct DepositToTreasury<'info> {
    #[account(
        mut,
        seeds = [SEED_TREASURY],
        bump = treasury.bump,
    )]
    pub treasury: Account<'info, Treasury>,

    #[account(mut)]
    pub depositor: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct WithdrawFromTreasury<'info> {
    #[account(
        mut,
        seeds = [SEED_TREASURY],
        bump = treasury.bump,
        constraint = !treasury.circuit_breaker_active @ ErrorCode::ProgramPaused
    )]
    pub treasury: Account<'info, Treasury>,

    #[account(
        seeds = [SEED_PROGRAM_CONFIG],
        bump = program_config.bump,
    )]
    pub program_config: Account<'info, ProgramConfig>,

    #[account(
        mut,
        constraint = authority.key() == program_config.admin @ ErrorCode::Unauthorized
    )]
    pub authority: Signer<'info>,

    #[account(mut)]
    /// CHECK: Destination for withdrawn funds
    pub destination: AccountInfo<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct EmergencyWithdraw<'info> {
    #[account(
        mut,
        seeds = [SEED_TREASURY],
        bump = treasury.bump,
    )]
    pub treasury: Account<'info, Treasury>,

    #[account(
        seeds = [SEED_PROGRAM_CONFIG],
        bump = program_config.bump,
    )]
    pub program_config: Account<'info, ProgramConfig>,

    #[account(
        mut,
        constraint = authority.key() == program_config.admin @ ErrorCode::Unauthorized
    )]
    pub authority: Signer<'info>,

    #[account(mut)]
    /// CHECK: Destination for emergency withdrawn funds
    pub destination: AccountInfo<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ToggleCircuitBreaker<'info> {
    #[account(
        mut,
        seeds = [SEED_TREASURY],
        bump = treasury.bump,
    )]
    pub treasury: Account<'info, Treasury>,

    #[account(
        seeds = [SEED_PROGRAM_CONFIG],
        bump = program_config.bump,
    )]
    pub program_config: Account<'info, ProgramConfig>,

    #[account(
        constraint = authority.key() == program_config.admin @ ErrorCode::Unauthorized
    )]
    pub authority: Signer<'info>,
}

pub fn initialize_treasury_handler(ctx: Context<InitializeTreasury>) -> Result<()> {
    let treasury = &mut ctx.accounts.treasury;
    let clock = Clock::get()?;

    treasury.authority = ctx.accounts.authority.key();
    treasury.total_deposited = 0;
    treasury.total_withdrawn = 0;
    treasury.emergency_mode = false;
    treasury.circuit_breaker_active = false;
    treasury.created_at = clock.unix_timestamp;
    treasury.bump = ctx.bumps.treasury;

    emit!(TreasuryInitializedEvent {
        treasury: treasury.key(),
        authority: treasury.authority,
        timestamp: clock.unix_timestamp,
    });

    msg!("Treasury initialized");
    Ok(())
}

pub fn deposit_to_treasury_handler(ctx: Context<DepositToTreasury>, amount: u64) -> Result<()> {
    require!(amount > 0, ErrorCode::InvalidAmount);
    require!(
        !ctx.accounts.treasury.circuit_breaker_active,
        ErrorCode::ProgramPaused
    );

    let treasury = &mut ctx.accounts.treasury;

    transfer(
        CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            Transfer {
                from: ctx.accounts.depositor.to_account_info(),
                to: treasury.to_account_info(),
            },
        ),
        amount,
    )?;

    treasury.total_deposited = treasury
        .total_deposited
        .checked_add(amount)
        .ok_or(ErrorCode::ArithmeticOverflow)?;

    emit!(TreasuryDepositEvent {
        treasury: treasury.key(),
        depositor: ctx.accounts.depositor.key(),
        amount,
        total_deposited: treasury.total_deposited,
        timestamp: Clock::get()?.unix_timestamp,
    });

    msg!("Deposited {} lamports to treasury", amount);
    Ok(())
}

pub fn withdraw_from_treasury_handler(
    ctx: Context<WithdrawFromTreasury>,
    amount: u64,
) -> Result<()> {
    require!(amount > 0, ErrorCode::InvalidAmount);

    let treasury = &mut ctx.accounts.treasury;
    let treasury_balance = treasury.to_account_info().lamports();

    require!(treasury_balance >= amount, ErrorCode::InsufficientBalance);

    **treasury.to_account_info().try_borrow_mut_lamports()? -= amount;
    **ctx.accounts.destination.try_borrow_mut_lamports()? += amount;

    treasury.total_withdrawn = treasury
        .total_withdrawn
        .checked_add(amount)
        .ok_or(ErrorCode::ArithmeticOverflow)?;

    emit!(TreasuryWithdrawEvent {
        treasury: treasury.key(),
        destination: ctx.accounts.destination.key(),
        amount,
        total_withdrawn: treasury.total_withdrawn,
        timestamp: Clock::get()?.unix_timestamp,
    });

    msg!("Withdrew {} lamports from treasury", amount);
    Ok(())
}

pub fn emergency_withdraw_handler(ctx: Context<EmergencyWithdraw>) -> Result<()> {
    let treasury = &mut ctx.accounts.treasury;
    let treasury_balance = treasury.to_account_info().lamports();

    require!(treasury_balance > 0, ErrorCode::InsufficientBalance);

    let amount = treasury_balance;

    **treasury.to_account_info().try_borrow_mut_lamports()? = 0;
    **ctx.accounts.destination.try_borrow_mut_lamports()? += amount;

    treasury.emergency_mode = true;
    treasury.total_withdrawn = treasury
        .total_withdrawn
        .checked_add(amount)
        .ok_or(ErrorCode::ArithmeticOverflow)?;

    emit!(EmergencyWithdrawEvent {
        treasury: treasury.key(),
        destination: ctx.accounts.destination.key(),
        amount,
        timestamp: Clock::get()?.unix_timestamp,
    });

    msg!("Emergency withdrawal of {} lamports", amount);
    Ok(())
}

pub fn toggle_circuit_breaker_handler(ctx: Context<ToggleCircuitBreaker>) -> Result<()> {
    let treasury = &mut ctx.accounts.treasury;
    treasury.circuit_breaker_active = !treasury.circuit_breaker_active;

    emit!(CircuitBreakerToggledEvent {
        treasury: treasury.key(),
        active: treasury.circuit_breaker_active,
        toggled_by: ctx.accounts.authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
    });

    msg!(
        "Circuit breaker {}",
        if treasury.circuit_breaker_active {
            "activated"
        } else {
            "deactivated"
        }
    );
    Ok(())
}

#[event]
pub struct TreasuryInitializedEvent {
    pub treasury: Pubkey,
    pub authority: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct TreasuryDepositEvent {
    pub treasury: Pubkey,
    pub depositor: Pubkey,
    pub amount: u64,
    pub total_deposited: u64,
    pub timestamp: i64,
}

#[event]
pub struct TreasuryWithdrawEvent {
    pub treasury: Pubkey,
    pub destination: Pubkey,
    pub amount: u64,
    pub total_withdrawn: u64,
    pub timestamp: i64,
}

#[event]
pub struct EmergencyWithdrawEvent {
    pub treasury: Pubkey,
    pub destination: Pubkey,
    pub amount: u64,
    pub timestamp: i64,
}

#[event]
pub struct CircuitBreakerToggledEvent {
    pub treasury: Pubkey,
    pub active: bool,
    pub toggled_by: Pubkey,
    pub timestamp: i64,
}
