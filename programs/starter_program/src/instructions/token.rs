use crate::constants::SEED_MINT_AUTHORITY;
use crate::events::*;
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{
        self, Approve, Burn, CloseAccount, FreezeAccount, Mint, MintTo, Revoke, ThawAccount,
        TokenAccount, TokenInterface, TransferChecked,
    },
};

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

    emit!(TokensMintedEvent {
        mint: ctx.accounts.mint.key(),
        recipient: ctx.accounts.token_account.key(),
        amount,
        timestamp: Clock::get()?.unix_timestamp,
    });

    msg!(
        "Minted {} tokens to {}",
        amount,
        ctx.accounts.token_account.key()
    );
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

    emit!(TokensTransferredEvent {
        mint: ctx.accounts.mint.key(),
        from: ctx.accounts.from_account.key(),
        to: ctx.accounts.to_account.key(),
        amount,
        timestamp: Clock::get()?.unix_timestamp,
    });

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

    emit!(TokensBurnedEvent {
        mint: ctx.accounts.mint.key(),
        owner: ctx.accounts.token_account.key(),
        amount,
        timestamp: Clock::get()?.unix_timestamp,
    });

    msg!("Burned {} tokens", amount);
    Ok(())
}

#[derive(Accounts)]
pub struct ApproveDelegate<'info> {
    #[account(mut)]
    pub token_account: InterfaceAccount<'info, TokenAccount>,

    /// CHECK: The delegate being approved
    pub delegate: AccountInfo<'info>,

    pub authority: Signer<'info>,

    pub token_program: Interface<'info, TokenInterface>,
}

pub fn approve_delegate_handler(ctx: Context<ApproveDelegate>, amount: u64) -> Result<()> {
    let cpi_accounts = Approve {
        to: ctx.accounts.token_account.to_account_info(),
        delegate: ctx.accounts.delegate.to_account_info(),
        authority: ctx.accounts.authority.to_account_info(),
    };

    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

    token_interface::approve(cpi_ctx, amount)?;

    emit!(DelegateApprovedEvent {
        token_account: ctx.accounts.token_account.key(),
        delegate: ctx.accounts.delegate.key(),
        amount,
        timestamp: Clock::get()?.unix_timestamp,
    });

    msg!(
        "Approved {} tokens for delegate {}",
        amount,
        ctx.accounts.delegate.key()
    );
    Ok(())
}

#[derive(Accounts)]
pub struct RevokeDelegate<'info> {
    #[account(mut)]
    pub token_account: InterfaceAccount<'info, TokenAccount>,

    pub authority: Signer<'info>,

    pub token_program: Interface<'info, TokenInterface>,
}

pub fn revoke_delegate_handler(ctx: Context<RevokeDelegate>) -> Result<()> {
    let cpi_accounts = Revoke {
        source: ctx.accounts.token_account.to_account_info(),
        authority: ctx.accounts.authority.to_account_info(),
    };

    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

    token_interface::revoke(cpi_ctx)?;

    emit!(DelegateRevokedEvent {
        token_account: ctx.accounts.token_account.key(),
        timestamp: Clock::get()?.unix_timestamp,
    });

    msg!("Revoked delegate for token account");
    Ok(())
}

#[derive(Accounts)]
pub struct CloseTokenAccount<'info> {
    #[account(mut)]
    pub token_account: InterfaceAccount<'info, TokenAccount>,

    #[account(mut)]
    /// CHECK: Destination for rent refund
    pub destination: AccountInfo<'info>,

    pub authority: Signer<'info>,

    pub token_program: Interface<'info, TokenInterface>,
}

pub fn close_token_account_handler(ctx: Context<CloseTokenAccount>) -> Result<()> {
    let cpi_accounts = CloseAccount {
        account: ctx.accounts.token_account.to_account_info(),
        destination: ctx.accounts.destination.to_account_info(),
        authority: ctx.accounts.authority.to_account_info(),
    };

    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

    token_interface::close_account(cpi_ctx)?;

    emit!(TokenAccountClosedEvent {
        token_account: ctx.accounts.token_account.key(),
        destination: ctx.accounts.destination.key(),
        timestamp: Clock::get()?.unix_timestamp,
    });

    msg!("Closed token account");
    Ok(())
}

#[derive(Accounts)]
pub struct FreezeTokenAccount<'info> {
    #[account(mut)]
    pub token_account: InterfaceAccount<'info, TokenAccount>,

    pub mint: InterfaceAccount<'info, Mint>,

    #[account(
        seeds = [SEED_MINT_AUTHORITY],
        bump
    )]
    /// CHECK: PDA used as freeze authority
    pub freeze_authority: UncheckedAccount<'info>,

    pub token_program: Interface<'info, TokenInterface>,
}

pub fn freeze_token_account_handler(ctx: Context<FreezeTokenAccount>) -> Result<()> {
    let signer_seeds: &[&[&[u8]]] = &[&[SEED_MINT_AUTHORITY, &[ctx.bumps.freeze_authority]]];

    let cpi_accounts = FreezeAccount {
        account: ctx.accounts.token_account.to_account_info(),
        mint: ctx.accounts.mint.to_account_info(),
        authority: ctx.accounts.freeze_authority.to_account_info(),
    };

    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts).with_signer(signer_seeds);

    token_interface::freeze_account(cpi_ctx)?;

    emit!(TokenAccountFrozenEvent {
        token_account: ctx.accounts.token_account.key(),
        mint: ctx.accounts.mint.key(),
        timestamp: Clock::get()?.unix_timestamp,
    });

    msg!("Frozen token account {}", ctx.accounts.token_account.key());
    Ok(())
}

#[derive(Accounts)]
pub struct ThawTokenAccount<'info> {
    #[account(mut)]
    pub token_account: InterfaceAccount<'info, TokenAccount>,

    pub mint: InterfaceAccount<'info, Mint>,

    #[account(
        seeds = [SEED_MINT_AUTHORITY],
        bump
    )]
    /// CHECK: PDA used as freeze authority
    pub freeze_authority: UncheckedAccount<'info>,

    pub token_program: Interface<'info, TokenInterface>,
}

pub fn thaw_token_account_handler(ctx: Context<ThawTokenAccount>) -> Result<()> {
    let signer_seeds: &[&[&[u8]]] = &[&[SEED_MINT_AUTHORITY, &[ctx.bumps.freeze_authority]]];

    let cpi_accounts = ThawAccount {
        account: ctx.accounts.token_account.to_account_info(),
        mint: ctx.accounts.mint.to_account_info(),
        authority: ctx.accounts.freeze_authority.to_account_info(),
    };

    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts).with_signer(signer_seeds);

    token_interface::thaw_account(cpi_ctx)?;

    emit!(TokenAccountThawedEvent {
        token_account: ctx.accounts.token_account.key(),
        mint: ctx.accounts.mint.key(),
        timestamp: Clock::get()?.unix_timestamp,
    });

    msg!("Thawed token account {}", ctx.accounts.token_account.key());
    Ok(())
}
