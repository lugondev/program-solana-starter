use anchor_lang::prelude::*;

#[account]
pub struct UpgradeAuthority {
    pub authority: Pubkey,
    pub pending_authority: Option<Pubkey>,
    pub voting_threshold: u8,
    pub proposal_count: u64,
    pub voting_period_seconds: i64,
    pub execution_delay_seconds: i64,
    pub is_locked: bool,
    pub bump: u8,
}

impl UpgradeAuthority {
    pub const LEN: usize = 8 + // discriminator
        32 +  // authority
        1 + 32 + // pending_authority (Option<Pubkey>)
        1 +   // voting_threshold
        8 +   // proposal_count
        8 +   // voting_period_seconds
        8 +   // execution_delay_seconds
        1 +   // is_locked
        1; // bump
}

#[account]
pub struct UpgradeProposal {
    pub proposal_id: u64,
    pub proposer: Pubkey,
    pub new_program_data: Pubkey,
    pub description: String,
    pub status: ProposalStatus,
    pub votes_for: u64,
    pub votes_against: u64,
    pub created_at: i64,
    pub voting_ends_at: i64,
    pub executed_at: Option<i64>,
    pub bump: u8,
}

impl UpgradeProposal {
    pub const MAX_DESCRIPTION_LENGTH: usize = 200;

    pub const LEN: usize = 8 + // discriminator
        8 +   // proposal_id
        32 +  // proposer
        32 +  // new_program_data
        4 + Self::MAX_DESCRIPTION_LENGTH + // description
        1 +   // status (enum, 1 byte for small enums)
        8 +   // votes_for
        8 +   // votes_against
        8 +   // created_at
        8 +   // voting_ends_at
        1 + 8 + // executed_at (Option<i64>)
        1; // bump

    pub fn is_voting_active(&self, current_timestamp: i64) -> bool {
        matches!(self.status, ProposalStatus::Pending) && current_timestamp < self.voting_ends_at
    }

    pub fn has_passed(&self, voting_threshold: u8) -> bool {
        let total_votes = self.votes_for.checked_add(self.votes_against).unwrap_or(0);
        if total_votes == 0 {
            return false;
        }
        let percentage = (self.votes_for * 100) / total_votes;
        percentage >= voting_threshold as u64 && self.votes_for > self.votes_against
    }

    pub fn can_execute(&self) -> bool {
        matches!(self.status, ProposalStatus::Pending)
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum ProposalStatus {
    Pending,
    Approved,
    Rejected,
    Executed,
    Cancelled,
}

#[account]
pub struct Vote {
    pub proposal_id: u64,
    pub voter: Pubkey,
    pub in_favor: bool,
    pub voting_power: u64,
    pub timestamp: i64,
    pub bump: u8,
}

impl Vote {
    pub const LEN: usize = 8 + // discriminator
        8 +   // proposal_id
        32 +  // voter
        1 +   // in_favor
        8 +   // voting_power
        8 +   // timestamp
        1; // bump
}

#[account]
pub struct ProgramVersion {
    pub version_number: u64,
    pub version_string: String,
    pub program_data: Pubkey,
    pub upgraded_at: i64,
    pub upgraded_by: Pubkey,
    pub proposal_id: u64,
    pub bump: u8,
}

impl ProgramVersion {
    pub const MAX_VERSION_LENGTH: usize = 20;

    pub const LEN: usize = 8 + // discriminator
        8 +   // version_number
        4 + Self::MAX_VERSION_LENGTH + // version_string
        32 +  // program_data
        8 +   // upgraded_at
        32 +  // upgraded_by
        8 +   // proposal_id
        1; // bump
}
