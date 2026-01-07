use anchor_lang::prelude::*;
use counter_program::{
    cpi::accounts::{Add, Increment, IncrementWithPayment, Initialize},
    program::CounterProgram,
    Counter,
};

use crate::constants::SEED_TOKEN_VAULT;

#[derive(Accounts)]
pub struct InitializeCounter<'info> {
    #[account(mut)]
    pub counter: Signer<'info>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub counter_program: Program<'info, CounterProgram>,
    pub system_program: Program<'info, System>,
}

pub fn initialize_counter_handler(ctx: Context<InitializeCounter>) -> Result<()> {
    let cpi_accounts = Initialize {
        counter: ctx.accounts.counter.to_account_info(),
        authority: ctx.accounts.authority.to_account_info(),
        system_program: ctx.accounts.system_program.to_account_info(),
    };

    let cpi_program = ctx.accounts.counter_program.to_account_info();
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

    counter_program::cpi::initialize(cpi_ctx)?;

    msg!("Counter initialized via CPI");
    Ok(())
}

#[derive(Accounts)]
pub struct IncrementCounter<'info> {
    #[account(mut)]
    pub counter: Account<'info, Counter>,

    pub authority: Signer<'info>,

    pub counter_program: Program<'info, CounterProgram>,
}

pub fn increment_counter_handler(ctx: Context<IncrementCounter>) -> Result<()> {
    let cpi_accounts = Increment {
        counter: ctx.accounts.counter.to_account_info(),
    };

    let cpi_program = ctx.accounts.counter_program.to_account_info();
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

    counter_program::cpi::increment(cpi_ctx)?;

    msg!("Counter incremented via CPI");
    Ok(())
}

#[derive(Accounts)]
pub struct AddToCounter<'info> {
    #[account(mut)]
    pub counter: Account<'info, Counter>,

    pub authority: Signer<'info>,

    pub counter_program: Program<'info, CounterProgram>,
}

pub fn add_to_counter_handler(ctx: Context<AddToCounter>, value: u64) -> Result<()> {
    let cpi_accounts = Add {
        counter: ctx.accounts.counter.to_account_info(),
    };

    let cpi_program = ctx.accounts.counter_program.to_account_info();
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

    counter_program::cpi::add(cpi_ctx, value)?;

    msg!("Added {} to counter via CPI", value);
    Ok(())
}

#[derive(Accounts)]
pub struct IncrementMultiple<'info> {
    #[account(mut)]
    pub counter: Account<'info, Counter>,

    pub authority: Signer<'info>,

    pub counter_program: Program<'info, CounterProgram>,
}

pub fn increment_multiple_handler(ctx: Context<IncrementMultiple>, times: u8) -> Result<()> {
    for i in 0..times {
        let cpi_accounts = Increment {
            counter: ctx.accounts.counter.to_account_info(),
        };

        let cpi_program = ctx.accounts.counter_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

        counter_program::cpi::increment(cpi_ctx)?;
        msg!("Increment #{}", i + 1);
    }

    msg!("Counter incremented {} times via CPI", times);
    Ok(())
}

#[derive(Accounts)]
pub struct IncrementWithPaymentFromPda<'info> {
    #[account(mut)]
    pub counter: Account<'info, Counter>,

    #[account(
        mut,
        seeds = [SEED_TOKEN_VAULT],
        bump
    )]
    pub pda_vault: SystemAccount<'info>,

    #[account(mut)]
    /// CHECK: Fee collector can be any account
    pub fee_collector: AccountInfo<'info>,

    pub counter_program: Program<'info, CounterProgram>,
    pub system_program: Program<'info, System>,
}

pub fn increment_with_payment_from_pda_handler(
    ctx: Context<IncrementWithPaymentFromPda>,
    payment: u64,
) -> Result<()> {
    let seeds = &[SEED_TOKEN_VAULT, &[ctx.bumps.pda_vault]];
    let signer = &[&seeds[..]];

    let cpi_accounts = IncrementWithPayment {
        counter: ctx.accounts.counter.to_account_info(),
        payer: ctx.accounts.pda_vault.to_account_info(),
        fee_collector: ctx.accounts.fee_collector.to_account_info(),
        system_program: ctx.accounts.system_program.to_account_info(),
    };

    let cpi_program = ctx.accounts.counter_program.to_account_info();
    let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);

    counter_program::cpi::increment_with_payment(cpi_ctx, payment)?;

    msg!(
        "Counter incremented with payment of {} lamports from PDA",
        payment
    );
    Ok(())
}
