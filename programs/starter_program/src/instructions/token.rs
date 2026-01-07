use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{
        self, Mint, TokenAccount, TokenInterface, MintTo, Burn, TransferChecked,
    },
};
use crate::constants::SEED_MINT_AUTHORITY;

#[derive(Accounts)]
pub struct CreateMint<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        init,
        payer = signer,
        mint::decimals = 6,
        mint::authority = mint_authority,
        mint::token_program = token_program,
        seeds = [b"mint"],
        bump
    )]
    pub mint: InterfaceAccount<'info, Mint>,

    #[account(
        seeds = [SEED_MINT_AUTHORITY],
        bump
    )]
    /// CHECK: PDA used as mint authority
    pub mint_authority: UncheckedAccount<'info>,

    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

pub fn create_mint_handler(ctx: Context<CreateMint>) -> Result<()> {
    msg!("Created Mint: {:?}", ctx.accounts.mint.key());
    Ok(())
}

#[derive(Accounts)]
pub struct MintTokens<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        init_if_needed,
        payer = signer,
        associated_token::mint = mint,
        associated_token::authority = signer,
        associated_token::token_program = token_program,
    )]
    pub token_account: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [b"mint"],
        bump
    )]
    pub mint: InterfaceAccount<'info, Mint>,

    #[account(
        seeds = [SEED_MINT_AUTHORITY],
        bump
    )]
    /// CHECK: PDA used as mint authority
    pub mint_authority: UncheckedAccount<'info>,

    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

pub fn mint_tokens_handler(ctx: Context<MintTokens>, amount: u64) -> Result<()> {
    let signer_seeds: &[&[&[u8]]] = &[&[SEED_MINT_AUTHORITY, &[ctx.bumps.mint_authority]]];

    let cpi_accounts = MintTo {
        mint: ctx.accounts.mint.to_account_info(),
        to: ctx.accounts.token_account.to_account_info(),
        authority: ctx.accounts.mint_authority.to_account_info(),
    };

    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_context = CpiContext::new(cpi_program, cpi_accounts).with_signer(signer_seeds);

    token_interface::mint_to(cpi_context, amount)?;

    msg!("Minted {} tokens to {}", amount, ctx.accounts.token_account.key());
    Ok(())
}

#[derive(Accounts)]
pub struct TransferTokens<'info> {
    #[account(mut)]
    pub from_account: InterfaceAccount<'info, TokenAccount>,

    #[account(mut)]
    pub to_account: InterfaceAccount<'info, TokenAccount>,

    pub mint: InterfaceAccount<'info, Mint>,

    pub authority: Signer<'info>,

    pub token_program: Interface<'info, TokenInterface>,
}

pub fn transfer_tokens_handler(ctx: Context<TransferTokens>, amount: u64) -> Result<()> {
    let cpi_accounts = TransferChecked {
        from: ctx.accounts.from_account.to_account_info(),
        to: ctx.accounts.to_account.to_account_info(),
        authority: ctx.accounts.authority.to_account_info(),
        mint: ctx.accounts.mint.to_account_info(),
    };

    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

    token_interface::transfer_checked(cpi_ctx, amount, ctx.accounts.mint.decimals)?;

    msg!("Transferred {} tokens", amount);
    Ok(())
}

#[derive(Accounts)]
pub struct BurnTokens<'info> {
    #[account(mut)]
    pub token_account: InterfaceAccount<'info, TokenAccount>,

    #[account(mut)]
    pub mint: InterfaceAccount<'info, Mint>,

    pub authority: Signer<'info>,

    pub token_program: Interface<'info, TokenInterface>,
}

pub fn burn_tokens_handler(ctx: Context<BurnTokens>, amount: u64) -> Result<()> {
    let cpi_accounts = Burn {
        mint: ctx.accounts.mint.to_account_info(),
        from: ctx.accounts.token_account.to_account_info(),
        authority: ctx.accounts.authority.to_account_info(),
    };

    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

    token_interface::burn(cpi_ctx, amount)?;

    msg!("Burned {} tokens", amount);
    Ok(())
}
