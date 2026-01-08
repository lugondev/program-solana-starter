use anchor_lang::prelude::*;
use anchor_lang::solana_program::bpf_loader_upgradeable;

use crate::constants::*;
use crate::error::ErrorCode;
use crate::events::*;
use crate::state::*;

#[derive(Accounts)]
pub struct InitializeUpgradeAuthority<'info> {
    #[account(
        init,
        payer = admin,
        space = 8 + UpgradeAuthority::LEN,
        seeds = [SEED_UPGRADE_AUTHORITY],
        bump
    )]
    pub upgrade_authority: Account<'info, UpgradeAuthority>,

    #[account(mut)]
    pub admin: Signer<'info>,

    pub system_program: Program<'info, System>,
}

pub fn initialize_upgrade_authority_handler(
    ctx: Context<InitializeUpgradeAuthority>,
    voting_threshold: u8,
    voting_period_seconds: i64,
    execution_delay_seconds: i64,
) -> Result<()> {
    let upgrade_authority = &mut ctx.accounts.upgrade_authority;
    let clock = Clock::get()?;

    require!(
        voting_threshold > 0 && voting_threshold <= 100,
        ErrorCode::InvalidAmount
    );

    require!(voting_period_seconds > 0, ErrorCode::InvalidAmount);

    require!(execution_delay_seconds > 0, ErrorCode::InvalidAmount);

    upgrade_authority.authority = ctx.accounts.admin.key();
    upgrade_authority.pending_authority = None;
    upgrade_authority.voting_threshold = voting_threshold;
    upgrade_authority.proposal_count = 0;
    upgrade_authority.voting_period_seconds = voting_period_seconds;
    upgrade_authority.execution_delay_seconds = execution_delay_seconds;
    upgrade_authority.is_locked = false;
    upgrade_authority.bump = ctx.bumps.upgrade_authority;

    emit!(UpgradeAuthorityInitializedEvent {
        authority: upgrade_authority.authority,
        admin: ctx.accounts.admin.key(),
        voting_threshold,
        timestamp: clock.unix_timestamp,
    });

    Ok(())
}

#[derive(Accounts)]
pub struct TransferUpgradeAuthority<'info> {
    #[account(
        mut,
        seeds = [SEED_UPGRADE_AUTHORITY],
        bump = upgrade_authority.bump,
        has_one = authority @ ErrorCode::Unauthorized
    )]
    pub upgrade_authority: Account<'info, UpgradeAuthority>,

    pub authority: Signer<'info>,

    /// CHECK: New authority to transfer to
    pub new_authority: AccountInfo<'info>,
}

pub fn transfer_upgrade_authority_handler(ctx: Context<TransferUpgradeAuthority>) -> Result<()> {
    let upgrade_authority = &mut ctx.accounts.upgrade_authority;

    require!(
        !upgrade_authority.is_locked,
        ErrorCode::InvalidStateTransition
    );

    upgrade_authority.pending_authority = Some(ctx.accounts.new_authority.key());

    Ok(())
}

#[derive(Accounts)]
pub struct AcceptUpgradeAuthority<'info> {
    #[account(
        mut,
        seeds = [SEED_UPGRADE_AUTHORITY],
        bump = upgrade_authority.bump
    )]
    pub upgrade_authority: Account<'info, UpgradeAuthority>,

    pub new_authority: Signer<'info>,
}

pub fn accept_upgrade_authority_handler(ctx: Context<AcceptUpgradeAuthority>) -> Result<()> {
    let upgrade_authority = &mut ctx.accounts.upgrade_authority;

    require!(
        upgrade_authority.pending_authority == Some(ctx.accounts.new_authority.key()),
        ErrorCode::Unauthorized
    );

    upgrade_authority.authority = ctx.accounts.new_authority.key();
    upgrade_authority.pending_authority = None;

    Ok(())
}

#[derive(Accounts)]
#[instruction(proposal_id: u64)]
pub struct CreateUpgradeProposal<'info> {
    #[account(
        mut,
        seeds = [SEED_UPGRADE_AUTHORITY],
        bump = upgrade_authority.bump
    )]
    pub upgrade_authority: Account<'info, UpgradeAuthority>,

    #[account(
        init,
        payer = proposer,
        space = 8 + UpgradeProposal::LEN,
        seeds = [SEED_UPGRADE_PROPOSAL, proposal_id.to_le_bytes().as_ref()],
        bump
    )]
    pub proposal: Account<'info, UpgradeProposal>,

    #[account(mut)]
    pub proposer: Signer<'info>,

    /// CHECK: New program data account
    pub new_program_data: AccountInfo<'info>,

    pub system_program: Program<'info, System>,
}

pub fn create_upgrade_proposal_handler(
    ctx: Context<CreateUpgradeProposal>,
    proposal_id: u64,
    description: String,
) -> Result<()> {
    let upgrade_authority = &mut ctx.accounts.upgrade_authority;
    let proposal = &mut ctx.accounts.proposal;
    let clock = Clock::get()?;

    require!(
        !upgrade_authority.is_locked,
        ErrorCode::InvalidStateTransition
    );

    require!(description.len() <= 200, ErrorCode::InvalidAmount);

    require!(
        proposal_id == upgrade_authority.proposal_count,
        ErrorCode::InvalidAmount
    );

    let voting_end = clock
        .unix_timestamp
        .checked_add(upgrade_authority.voting_period_seconds)
        .ok_or(ErrorCode::ArithmeticOverflow)?;

    proposal.proposal_id = proposal_id;
    proposal.proposer = ctx.accounts.proposer.key();
    proposal.new_program_data = ctx.accounts.new_program_data.key();
    proposal.description = description.clone();
    proposal.status = ProposalStatus::Pending;
    proposal.votes_for = 0;
    proposal.votes_against = 0;
    proposal.created_at = clock.unix_timestamp;
    proposal.voting_ends_at = voting_end;
    proposal.executed_at = None;
    proposal.bump = ctx.bumps.proposal;

    upgrade_authority.proposal_count = upgrade_authority
        .proposal_count
        .checked_add(1)
        .ok_or(ErrorCode::ArithmeticOverflow)?;

    emit!(UpgradeProposalCreatedEvent {
        proposal_id,
        proposer: ctx.accounts.proposer.key(),
        new_program_data: ctx.accounts.new_program_data.key(),
        description,
        timestamp: clock.unix_timestamp,
    });

    Ok(())
}

#[derive(Accounts)]
#[instruction(proposal_id: u64)]
pub struct CastVote<'info> {
    #[account(
        seeds = [SEED_UPGRADE_AUTHORITY],
        bump = upgrade_authority.bump
    )]
    pub upgrade_authority: Account<'info, UpgradeAuthority>,

    #[account(
        mut,
        seeds = [SEED_UPGRADE_PROPOSAL, proposal_id.to_le_bytes().as_ref()],
        bump = proposal.bump
    )]
    pub proposal: Account<'info, UpgradeProposal>,

    #[account(
        init,
        payer = voter,
        space = 8 + Vote::LEN,
        seeds = [SEED_VOTE, proposal_id.to_le_bytes().as_ref(), voter.key().as_ref()],
        bump
    )]
    pub vote: Account<'info, Vote>,

    #[account(mut)]
    pub voter: Signer<'info>,

    pub system_program: Program<'info, System>,
}

pub fn cast_vote_handler(
    ctx: Context<CastVote>,
    _proposal_id: u64,
    in_favor: bool,
    voting_power: u64,
) -> Result<()> {
    let proposal = &mut ctx.accounts.proposal;
    let vote = &mut ctx.accounts.vote;
    let clock = Clock::get()?;

    require!(
        proposal.is_voting_active(clock.unix_timestamp),
        ErrorCode::InvalidStateTransition
    );

    require!(
        proposal.status == ProposalStatus::Pending,
        ErrorCode::InvalidStateTransition
    );

    require!(voting_power > 0, ErrorCode::InvalidAmount);

    vote.proposal_id = proposal.proposal_id;
    vote.voter = ctx.accounts.voter.key();
    vote.in_favor = in_favor;
    vote.voting_power = voting_power;
    vote.timestamp = clock.unix_timestamp;
    vote.bump = ctx.bumps.vote;

    if in_favor {
        proposal.votes_for = proposal
            .votes_for
            .checked_add(voting_power)
            .ok_or(ErrorCode::ArithmeticOverflow)?;
    } else {
        proposal.votes_against = proposal
            .votes_against
            .checked_add(voting_power)
            .ok_or(ErrorCode::ArithmeticOverflow)?;
    }

    emit!(VoteCastEvent {
        proposal_id: proposal.proposal_id,
        voter: ctx.accounts.voter.key(),
        in_favor,
        timestamp: clock.unix_timestamp,
    });

    Ok(())
}

#[derive(Accounts)]
#[instruction(proposal_id: u64)]
pub struct ExecuteProposal<'info> {
    #[account(
        mut,
        seeds = [SEED_UPGRADE_AUTHORITY],
        bump = upgrade_authority.bump
    )]
    pub upgrade_authority: Account<'info, UpgradeAuthority>,

    #[account(
        mut,
        seeds = [SEED_UPGRADE_PROPOSAL, proposal_id.to_le_bytes().as_ref()],
        bump = proposal.bump
    )]
    pub proposal: Account<'info, UpgradeProposal>,

    #[account(
        init,
        payer = executor,
        space = 8 + ProgramVersion::LEN,
        seeds = [
            SEED_PROGRAM_VERSION,
            upgrade_authority.proposal_count.to_le_bytes().as_ref()
        ],
        bump
    )]
    pub program_version: Account<'info, ProgramVersion>,

    #[account(mut)]
    pub executor: Signer<'info>,

    /// CHECK: Program data account for BPF Loader Upgradeable
    #[account(mut)]
    pub program_data: AccountInfo<'info>,

    /// CHECK: New program data with upgraded code
    pub new_program_data: AccountInfo<'info>,

    /// CHECK: BPF Loader Upgradeable Program
    #[account(address = bpf_loader_upgradeable::ID)]
    pub bpf_loader_upgradeable_program: AccountInfo<'info>,

    pub system_program: Program<'info, System>,
}

pub fn execute_proposal_handler(
    ctx: Context<ExecuteProposal>,
    _proposal_id: u64,
    old_version: String,
    new_version: String,
) -> Result<()> {
    let upgrade_authority = &mut ctx.accounts.upgrade_authority;
    let proposal = &mut ctx.accounts.proposal;
    let program_version = &mut ctx.accounts.program_version;
    let clock = Clock::get()?;

    require!(
        old_version.len() <= 20 && new_version.len() <= 20,
        ErrorCode::InvalidAmount
    );

    require!(
        !proposal.is_voting_active(clock.unix_timestamp),
        ErrorCode::InvalidStateTransition
    );

    require!(
        proposal.has_passed(upgrade_authority.voting_threshold),
        ErrorCode::Unauthorized
    );

    let execution_allowed_at = proposal
        .voting_ends_at
        .checked_add(upgrade_authority.execution_delay_seconds)
        .ok_or(ErrorCode::ArithmeticOverflow)?;

    require!(
        clock.unix_timestamp >= execution_allowed_at,
        ErrorCode::InvalidStateTransition
    );

    require!(proposal.can_execute(), ErrorCode::InvalidStateTransition);

    require!(
        !upgrade_authority.is_locked,
        ErrorCode::InvalidStateTransition
    );

    upgrade_authority.is_locked = true;

    proposal.status = ProposalStatus::Executed;
    proposal.executed_at = Some(clock.unix_timestamp);

    program_version.version_number = upgrade_authority.proposal_count;
    program_version.version_string = new_version.clone();
    program_version.program_data = ctx.accounts.new_program_data.key();
    program_version.upgraded_at = clock.unix_timestamp;
    program_version.upgraded_by = ctx.accounts.executor.key();
    program_version.proposal_id = proposal.proposal_id;
    program_version.bump = ctx.bumps.program_version;

    emit!(ProposalExecutedEvent {
        proposal_id: proposal.proposal_id,
        executor: ctx.accounts.executor.key(),
        new_program_data: ctx.accounts.new_program_data.key(),
        timestamp: clock.unix_timestamp,
    });

    emit!(UpgradeCompletedEvent {
        old_version,
        new_version,
        program_data: ctx.accounts.new_program_data.key(),
        timestamp: clock.unix_timestamp,
    });

    upgrade_authority.is_locked = false;

    Ok(())
}

#[derive(Accounts)]
#[instruction(proposal_id: u64)]
pub struct CancelProposal<'info> {
    #[account(
        seeds = [SEED_UPGRADE_AUTHORITY],
        bump = upgrade_authority.bump,
        has_one = authority @ ErrorCode::Unauthorized
    )]
    pub upgrade_authority: Account<'info, UpgradeAuthority>,

    #[account(
        mut,
        seeds = [SEED_UPGRADE_PROPOSAL, proposal_id.to_le_bytes().as_ref()],
        bump = proposal.bump
    )]
    pub proposal: Account<'info, UpgradeProposal>,

    pub authority: Signer<'info>,
}

pub fn cancel_proposal_handler(ctx: Context<CancelProposal>, _proposal_id: u64) -> Result<()> {
    let proposal = &mut ctx.accounts.proposal;

    require!(
        proposal.status == ProposalStatus::Pending,
        ErrorCode::InvalidStateTransition
    );

    proposal.status = ProposalStatus::Cancelled;

    Ok(())
}
