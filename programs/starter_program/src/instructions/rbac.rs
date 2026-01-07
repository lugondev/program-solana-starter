use crate::error::ErrorCode;
use crate::{ProgramConfig, Role, RoleType, SEED_PROGRAM_CONFIG, SEED_ROLE};
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct AssignRole<'info> {
    #[account(
        init,
        payer = admin,
        space = 8 + Role::LEN,
        seeds = [SEED_ROLE, target_authority.key().as_ref()],
        bump
    )]
    pub role: Account<'info, Role>,

    #[account(
        mut,
        seeds = [SEED_PROGRAM_CONFIG],
        bump = program_config.bump,
    )]
    pub program_config: Account<'info, ProgramConfig>,

    #[account(
        mut,
        constraint = admin.key() == program_config.admin @ ErrorCode::Unauthorized
    )]
    pub admin: Signer<'info>,

    /// CHECK: The user receiving the role
    pub target_authority: AccountInfo<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdateRole<'info> {
    #[account(
        mut,
        seeds = [SEED_ROLE, role.authority.as_ref()],
        bump = role.bump,
    )]
    pub role: Account<'info, Role>,

    #[account(
        seeds = [SEED_PROGRAM_CONFIG],
        bump = program_config.bump,
    )]
    pub program_config: Account<'info, ProgramConfig>,

    #[account(
        constraint = admin.key() == program_config.admin @ ErrorCode::Unauthorized
    )]
    pub admin: Signer<'info>,
}

#[derive(Accounts)]
pub struct RevokeRole<'info> {
    #[account(
        mut,
        close = admin,
        seeds = [SEED_ROLE, role.authority.as_ref()],
        bump = role.bump,
    )]
    pub role: Account<'info, Role>,

    #[account(
        seeds = [SEED_PROGRAM_CONFIG],
        bump = program_config.bump,
    )]
    pub program_config: Account<'info, ProgramConfig>,

    #[account(
        mut,
        constraint = admin.key() == program_config.admin @ ErrorCode::Unauthorized
    )]
    pub admin: Signer<'info>,
}

#[derive(Accounts)]
pub struct CheckPermission<'info> {
    #[account(
        seeds = [SEED_ROLE, authority.key().as_ref()],
        bump = role.bump,
    )]
    pub role: Account<'info, Role>,

    pub authority: Signer<'info>,
}

pub fn assign_role_handler(ctx: Context<AssignRole>, role_type: RoleType) -> Result<()> {
    let role = &mut ctx.accounts.role;
    let clock = Clock::get()?;

    role.authority = ctx.accounts.target_authority.key();
    role.role_type = role_type;
    role.permissions = role_type.default_permissions();
    role.assigned_by = ctx.accounts.admin.key();
    role.assigned_at = clock.unix_timestamp;
    role.updated_at = clock.unix_timestamp;
    role.bump = ctx.bumps.role;

    emit!(RoleAssignedEvent {
        authority: role.authority,
        role_type,
        assigned_by: role.assigned_by,
        timestamp: clock.unix_timestamp,
    });

    Ok(())
}

pub fn update_role_permissions_handler(
    ctx: Context<UpdateRole>,
    add_permissions: u8,
    remove_permissions: u8,
) -> Result<()> {
    let role = &mut ctx.accounts.role;

    if add_permissions > 0 {
        role.add_permission(add_permissions);
    }

    if remove_permissions > 0 {
        role.remove_permission(remove_permissions);
    }

    emit!(RoleUpdatedEvent {
        authority: role.authority,
        permissions: role.permissions,
        updated_by: ctx.accounts.admin.key(),
        timestamp: Clock::get()?.unix_timestamp,
    });

    Ok(())
}

pub fn revoke_role_handler(ctx: Context<RevokeRole>) -> Result<()> {
    let role = &ctx.accounts.role;

    emit!(RoleRevokedEvent {
        authority: role.authority,
        role_type: role.role_type,
        revoked_by: ctx.accounts.admin.key(),
        timestamp: Clock::get()?.unix_timestamp,
    });

    Ok(())
}

pub fn check_permission_handler(
    ctx: Context<CheckPermission>,
    required_permission: u8,
) -> Result<bool> {
    let role = &ctx.accounts.role;

    require!(
        role.authority == ctx.accounts.authority.key(),
        ErrorCode::Unauthorized
    );

    Ok(role.has_permission(required_permission))
}

#[event]
pub struct RoleAssignedEvent {
    pub authority: Pubkey,
    pub role_type: RoleType,
    pub assigned_by: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct RoleUpdatedEvent {
    pub authority: Pubkey,
    pub permissions: u8,
    pub updated_by: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct RoleRevokedEvent {
    pub authority: Pubkey,
    pub role_type: RoleType,
    pub revoked_by: Pubkey,
    pub timestamp: i64,
}
