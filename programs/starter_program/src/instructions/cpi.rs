use anchor_lang::prelude::*;
use anchor_lang::system_program::{transfer, Transfer as SystemTransfer};
use anchor_spl::token_interface::{
    self, TokenAccount, TokenInterface, TransferChecked,
};
use crate::constants::SEED_TOKEN_VAULT;

#[derive(Accounts)]
pub struct TransferSol<'info> {
    #[account(mut)]
    pub from: Signer<'info>,

    #[account(mut)]
    /// CHECK: Recipient can be any account
    pub to: AccountInfo<'info>,

    pub system_program: Program<'info, System>,
}

pub fn transfer_sol_handler(ctx: Context<TransferSol>, amount: u64) -> Result<()> {
    let cpi_accounts = SystemTransfer {
        from: ctx.accounts.from.to_account_info(),
        to: ctx.accounts.to.to_account_info(),
    };

    let cpi_program = ctx.accounts.system_program.to_account_info();
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

    transfer(cpi_ctx, amount)?;

    msg!("Transferred {} lamports", amount);
    Ok(())
}

#[derive(Accounts)]
pub struct TransferSolWithPda<'info> {
    #[account(
        mut,
        seeds = [SEED_TOKEN_VAULT],
        bump
    )]
    pub vault: SystemAccount<'info>,

    #[account(mut)]
    /// CHECK: Recipient can be any account
    pub recipient: AccountInfo<'info>,

    pub system_program: Program<'info, System>,
}

pub fn transfer_sol_with_pda_handler(
    ctx: Context<TransferSolWithPda>,
    amount: u64,
) -> Result<()> {
    let seeds = &[
        SEED_TOKEN_VAULT,
        &[ctx.bumps.vault],
    ];
    let signer = &[&seeds[..]];

    let cpi_accounts = SystemTransfer {
        from: ctx.accounts.vault.to_account_info(),
        to: ctx.accounts.recipient.to_account_info(),
    };

    let cpi_program = ctx.accounts.system_program.to_account_info();
    let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);

    transfer(cpi_ctx, amount)?;

    msg!("Transferred {} lamports from PDA vault", amount);
    Ok(())
}

#[derive(Accounts)]
pub struct TransferTokensWithPda<'info> {
    #[account(
        seeds = [SEED_TOKEN_VAULT],
        bump
    )]
    /// CHECK: PDA used as token authority
    pub vault_authority: UncheckedAccount<'info>,

    #[account(mut)]
    pub from: InterfaceAccount<'info, TokenAccount>,

    #[account(mut)]
    pub to: InterfaceAccount<'info, TokenAccount>,

    pub mint: InterfaceAccount<'info, anchor_spl::token_interface::Mint>,

    pub token_program: Interface<'info, TokenInterface>,
}

pub fn transfer_tokens_with_pda_handler(
    ctx: Context<TransferTokensWithPda>,
    amount: u64,
) -> Result<()> {
    let seeds = &[
        SEED_TOKEN_VAULT,
        &[ctx.bumps.vault_authority],
    ];
    let signer = &[&seeds[..]];

    let cpi_accounts = TransferChecked {
        from: ctx.accounts.from.to_account_info(),
        to: ctx.accounts.to.to_account_info(),
        authority: ctx.accounts.vault_authority.to_account_info(),
        mint: ctx.accounts.mint.to_account_info(),
    };

    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);

    token_interface::transfer_checked(cpi_ctx, amount, ctx.accounts.mint.decimals)?;

    msg!("Transferred {} tokens with PDA authority", amount);
    Ok(())
}
