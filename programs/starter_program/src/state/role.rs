use anchor_lang::prelude::*;

/// Role types for access control
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq)]
pub enum RoleType {
    /// Full access to all functions
    Admin,
    /// Limited administrative access
    Moderator,
    /// Standard user access
    User,
}

/// Permissions bitmap flags
pub mod permissions {
    /// Can manage program configuration
    pub const MANAGE_CONFIG: u8 = 1 << 0;
    /// Can manage user accounts
    pub const MANAGE_USERS: u8 = 1 << 1;
    /// Can manage tokens
    pub const MANAGE_TOKENS: u8 = 1 << 2;
    /// Can pause/unpause program
    pub const PAUSE_PROGRAM: u8 = 1 << 3;
    /// Can perform emergency actions
    pub const EMERGENCY_ACTIONS: u8 = 1 << 4;
    /// Can manage treasury
    pub const MANAGE_TREASURY: u8 = 1 << 5;
    /// Can manage roles
    pub const MANAGE_ROLES: u8 = 1 << 6;
    /// Can perform batch operations
    pub const BATCH_OPERATIONS: u8 = 1 << 7;
}

impl RoleType {
    /// Get default permissions for each role type
    pub fn default_permissions(&self) -> u8 {
        match self {
            RoleType::Admin => {
                // Admin has all permissions
                permissions::MANAGE_CONFIG
                    | permissions::MANAGE_USERS
                    | permissions::MANAGE_TOKENS
                    | permissions::PAUSE_PROGRAM
                    | permissions::EMERGENCY_ACTIONS
                    | permissions::MANAGE_TREASURY
                    | permissions::MANAGE_ROLES
                    | permissions::BATCH_OPERATIONS
            }
            RoleType::Moderator => {
                // Moderator has limited permissions
                permissions::MANAGE_USERS | permissions::MANAGE_TOKENS
            }
            RoleType::User => {
                // User has no special permissions
                0
            }
        }
    }
}

/// Role account structure
#[account]
pub struct Role {
    /// The user who holds this role
    pub authority: Pubkey,
    /// Type of role
    pub role_type: RoleType,
    /// Permission bitmap
    pub permissions: u8,
    /// Who assigned this role
    pub assigned_by: Pubkey,
    /// When the role was assigned
    pub assigned_at: i64,
    /// When the role was last updated
    pub updated_at: i64,
    /// PDA bump seed
    pub bump: u8,
}

impl Role {
    pub const LEN: usize = 8 + // discriminator
        32 + // authority
        1 +  // role_type (enum)
        1 +  // permissions
        32 + // assigned_by
        8 +  // assigned_at
        8 +  // updated_at
        1; // bump

    /// Check if role has specific permission
    pub fn has_permission(&self, permission: u8) -> bool {
        (self.permissions & permission) != 0
    }

    /// Add permission to role
    pub fn add_permission(&mut self, permission: u8) {
        self.permissions |= permission;
        self.updated_at = Clock::get().unwrap().unix_timestamp;
    }

    /// Remove permission from role
    pub fn remove_permission(&mut self, permission: u8) {
        self.permissions &= !permission;
        self.updated_at = Clock::get().unwrap().unix_timestamp;
    }
}
