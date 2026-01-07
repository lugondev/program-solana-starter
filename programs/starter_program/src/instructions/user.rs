use anchor_lang::prelude::*;
use crate::{state::UserAccount, constants::SEED_USER_ACCOUNT, error::ErrorCode};

#[derive(Accounts)]
pub struct CreateUserAccount<'info> {
    #[account(
        init,
        payer = authority,
        space = UserAccount::LEN,
        seeds = [SEED_USER_ACCOUNT, authority.key().as_ref()],
        bump
    )]
    pub user_account: Account<'info, UserAccount>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
}

pub fn create_user_account_handler(ctx: Context<CreateUserAccount>) -> Result<()> {
    let user = &mut ctx.accounts.user_account;
    user.authority = ctx.accounts.authority.key();
    user.points = 0;
    user.created_at = Clock::get()?.unix_timestamp;
    user.updated_at = Clock::get()?.unix_timestamp;
    user.bump = ctx.bumps.user_account;

    msg!("User account created: {}", user.authority);
    Ok(())
}

#[derive(Accounts)]
pub struct UpdateUserAccount<'info> {
    #[account(
        mut,
        seeds = [SEED_USER_ACCOUNT, authority.key().as_ref()],
        bump = user_account.bump,
        has_one = authority @ ErrorCode::Unauthorized
    )]
    pub user_account: Account<'info, UserAccount>,

    pub authority: Signer<'info>,
}

pub fn update_user_account_handler(
    ctx: Context<UpdateUserAccount>,
    new_points: u64,
) -> Result<()> {
    let user = &mut ctx.accounts.user_account;
    user.points = new_points;
    user.updated_at = Clock::get()?.unix_timestamp;

    msg!("User account updated: {} points", new_points);
    Ok(())
}

#[derive(Accounts)]
pub struct CloseUserAccount<'info> {
    #[account(
        mut,
        close = authority,
        seeds = [SEED_USER_ACCOUNT, authority.key().as_ref()],
        bump = user_account.bump,
        has_one = authority @ ErrorCode::Unauthorized
    )]
    pub user_account: Account<'info, UserAccount>,

    #[account(mut)]
    pub authority: Signer<'info>,
}

pub fn close_user_account_handler(ctx: Context<CloseUserAccount>) -> Result<()> {
    msg!("User account closed: {}", ctx.accounts.authority.key());
    Ok(())
}
