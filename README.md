# Solana Starter Program

A production-ready Anchor framework starter template with advanced features including RBAC, treasury management, comprehensive events, and emergency controls. Demonstrates all essential Solana program patterns: configuration management, PDAs, SPL tokens, cross-program invocations, and inter-program communication.

## 🚀 Features

### Core Components

- ✅ **Program Configuration** - Admin-controlled config with pause functionality
- ✅ **PDA Patterns** - User accounts with seeds-based derivation
- ✅ **SPL Token Operations** - Mint, transfer, and burn tokens
- ✅ **Cross-Program Invocation** - CPI examples with and without PDA signers
- ✅ **Inter-Program Communication** - Counter program with bidirectional CPI patterns
- ✅ **Error Handling** - Custom error codes with descriptive messages
- ✅ **Security Best Practices** - Account validation, authority checks, rent exemption

### Phase 1 Advanced Features

- ✅ **Role-Based Access Control (RBAC)** - Permission-based authorization with Admin, Moderator, and User roles
- ✅ **Advanced Token Operations** - Delegate approval/revocation, account freeze/thaw, and account closing
- ✅ **Treasury Management** - Centralized fund management with emergency controls and circuit breaker
- ✅ **Comprehensive Events System** - 13 event types for monitoring all program activities
- ✅ **Emergency Controls** - Circuit breaker for pausing deposits and emergency withdrawal functionality

### Phase 2 Advanced Features

- ✅ **NFT Support** - Collection-based NFT minting with metadata, marketplace listing, and offer system
- ✅ **Program Upgradability** - Proposal-based upgrade system with voting and time-delayed execution

## 📁 Project Structure

```
starter_program/
├── programs/
│   ├── starter_program/
│   │   └── src/
│   │       ├── lib.rs              # Program entry point (32 instructions)
│   │       ├── constants.rs        # PDA seeds and constants
│   │       ├── error.rs            # Custom error definitions (15 errors)
│   │       ├── events.rs           # Event definitions (13 events)
│   │       ├── state/              # Account structures
│   │       │   ├── mod.rs
│   │       │   ├── config.rs       # Program configuration
│   │       │   ├── user.rs         # User account state
│   │       │   ├── role.rs         # RBAC role definitions
│   │       │   └── treasury.rs     # Treasury state
│   │       └── instructions/       # Instruction handlers
│   │           ├── mod.rs
│   │           ├── initialize.rs   # Initialize program
│   │           ├── config.rs       # Config management
│   │           ├── user.rs         # User account operations
│   │           ├── token.rs        # SPL token operations (basic + advanced)
│   │           ├── rbac.rs         # Role-based access control
│   │           ├── treasury.rs     # Treasury management
│   │           ├── cpi.rs          # Cross-program invocations
│   │           └── cross_program.rs # CPI with counter program
│   └── counter_program/
│       └── src/
│           └── lib.rs              # Counter program (5 instructions)
├── tests/
│   ├── starter_program.ts          # Starter program tests (25+ tests)
│   ├── cross_program.ts            # Cross-program interaction tests (14 tests)
│   ├── rbac.ts                     # RBAC tests (25+ tests)
│   ├── advanced_token.ts           # Advanced token tests (14+ tests)
│   └── treasury.ts                 # Treasury & emergency tests (18+ tests)
├── CROSS_PROGRAM.md                # Cross-program interaction guide
└── README.md
```

## 🛠️ Setup

### Prerequisites

- Rust 1.70+
- Solana CLI 1.18+
- Anchor CLI 0.31.1
- Node.js 18+

### Installation

```bash
# Install Anchor
cargo install --git https://github.com/coral-xyz/anchor avm --locked --force
avm install 0.31.1
avm use 0.31.1

# Clone and install dependencies
cd starter_program
yarn install
```

### Build

```bash
anchor build
```

### Test

```bash
anchor test
```

## 📖 Program Instructions

### 1. Initialize

Basic program initialization.

```typescript
await program.methods.initialize().rpc();
```

### 2. Program Configuration

#### Initialize Config

Create program configuration with admin authority and fee settings.

```typescript
const [configPda] = PublicKey.findProgramAddressSync(
  [Buffer.from("program_config")],
  program.programId
);

await program.methods
  .initializeConfig(feeDestination)
  .accounts({
    programConfig: configPda,
    authority: admin.publicKey,
    systemProgram: SystemProgram.programId,
  })
  .rpc();
```

#### Update Config

Update admin, fee destination, and fee basis points.

```typescript
await program.methods
  .updateConfig(newAdmin, newFeeDestination, new BN(200))
  .accounts({
    programConfig: configPda,
    admin: currentAdmin.publicKey,
  })
  .rpc();
```

#### Toggle Pause

Enable or disable program operations.

```typescript
await program.methods
  .togglePause()
  .accounts({
    programConfig: configPda,
    admin: admin.publicKey,
  })
  .rpc();
```

### 3. User Account (PDA Pattern)

#### Create User Account

Create a PDA-based user account.

```typescript
const [userPda] = PublicKey.findProgramAddressSync(
  [Buffer.from("user_account"), user.publicKey.toBuffer()],
  program.programId
);

await program.methods
  .createUserAccount()
  .accounts({
    userAccount: userPda,
    authority: user.publicKey,
    systemProgram: SystemProgram.programId,
  })
  .rpc();
```

#### Update User Account

Update user points.

```typescript
await program.methods
  .updateUserAccount(new BN(100))
  .accounts({
    userAccount: userPda,
    authority: user.publicKey,
  })
  .rpc();
```

#### Close User Account

Close account and reclaim rent.

```typescript
await program.methods
  .closeUserAccount()
  .accounts({
    userAccount: userPda,
    authority: user.publicKey,
  })
  .rpc();
```

### 4. SPL Token Operations

#### Create Mint

Create a new token mint with PDA authority.

```typescript
const [mintPda] = PublicKey.findProgramAddressSync(
  [Buffer.from("mint")],
  program.programId
);

const [mintAuthority] = PublicKey.findProgramAddressSync(
  [Buffer.from("mint_authority")],
  program.programId
);

await program.methods
  .createMint()
  .accounts({
    signer: payer.publicKey,
    mint: mintPda,
    mintAuthority: mintAuthority,
    tokenProgram: TOKEN_PROGRAM_ID,
    systemProgram: SystemProgram.programId,
  })
  .rpc();
```

#### Mint Tokens

Mint tokens to a user's associated token account.

```typescript
const userTokenAccount = await getAssociatedTokenAddress(
  mintPda,
  user.publicKey
);

await program.methods
  .mintTokens(new BN(1000000))
  .accounts({
    signer: user.publicKey,
    tokenAccount: userTokenAccount,
    mint: mintPda,
    mintAuthority: mintAuthority,
    tokenProgram: TOKEN_PROGRAM_ID,
    associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
    systemProgram: SystemProgram.programId,
  })
  .rpc();
```

#### Transfer Tokens

Transfer tokens between accounts.

```typescript
await program.methods
  .transferTokens(new BN(500000))
  .accounts({
    fromAccount: fromTokenAccount,
    toAccount: toTokenAccount,
    mint: mintPda,
    authority: user.publicKey,
    tokenProgram: TOKEN_PROGRAM_ID,
  })
  .rpc();
```

#### Burn Tokens

Burn tokens from an account.

```typescript
await program.methods
  .burnTokens(new BN(100000))
  .accounts({
    tokenAccount: userTokenAccount,
    mint: mintPda,
    authority: user.publicKey,
    tokenProgram: TOKEN_PROGRAM_ID,
  })
  .rpc();
```

### 5. Cross-Program Invocation (CPI)

#### Transfer SOL

Transfer SOL using System Program CPI.

```typescript
await program.methods
  .transferSol(new BN(1000000))
  .accounts({
    from: sender.publicKey,
    to: recipient.publicKey,
    systemProgram: SystemProgram.programId,
  })
  .rpc();
```

#### Transfer SOL with PDA

Transfer SOL using a PDA as the authority.

```typescript
const [vaultPda] = PublicKey.findProgramAddressSync(
  [Buffer.from("token_vault")],
  program.programId
);

await program.methods
  .transferSolWithPda(new BN(500000))
  .accounts({
    vault: vaultPda,
    recipient: recipient.publicKey,
    systemProgram: SystemProgram.programId,
  })
  .rpc();
```

#### Transfer Tokens with PDA

Transfer tokens using a PDA as the authority.

```typescript
const [vaultAuthority] = PublicKey.findProgramAddressSync(
  [Buffer.from("token_vault")],
  program.programId
);

await program.methods
  .transferTokensWithPda(new BN(1000000))
  .accounts({
    vaultAuthority: vaultAuthority,
    from: fromTokenAccount,
    to: toTokenAccount,
    mint: mintPda,
    tokenProgram: TOKEN_PROGRAM_ID,
  })
  .rpc();
```

### 6. Cross-Program Interaction

The project includes a **counter_program** to demonstrate inter-program communication patterns via CPI.

#### Counter Program Overview

```typescript
// Counter Program ID
CounzVsCGF4VzNkAwePKC9mXr6YWiFYF4kLW6YdV8Cc

// Counter Account Structure
{
  authority: Pubkey,  // Counter owner
  count: u64,         // Current count
  bump: u8            // PDA bump seed
}
```

#### Initialize Counter via CPI

```typescript
const counterProgram = anchor.workspace.CounterProgram;

const [counterPda] = anchor.web3.PublicKey.findProgramAddressSync(
  [Buffer.from("counter"), user.publicKey.toBuffer()],
  counterProgram.programId
);

// Initialize counter through starter_program CPI
await program.methods
  .initializeCounter()
  .accounts({
    counter: counterPda,
    authority: user.publicKey,
    counterProgram: counterProgram.programId,
    systemProgram: SystemProgram.programId,
  })
  .rpc();
```

#### Increment Counter via CPI

```typescript
// Increment counter through starter_program
await program.methods
  .incrementCounter()
  .accounts({
    counter: counterPda,
    authority: user.publicKey,
    counterProgram: counterProgram.programId,
  })
  .rpc();

// Check counter value
const counter = await counterProgram.account.counter.fetch(counterPda);
console.log(`Count: ${counter.count.toString()}`);
```

#### Add Value via CPI

```typescript
// Add arbitrary value through starter_program
await program.methods
  .addToCounter(new BN(10))
  .accounts({
    counter: counterPda,
    authority: user.publicKey,
    counterProgram: counterProgram.programId,
  })
  .rpc();
```

#### Multiple CPI Calls

```typescript
// Increment counter multiple times in one transaction
await program.methods
  .incrementMultiple(5) // Increment 5 times
  .accounts({
    counter: counterPda,
    authority: user.publicKey,
    counterProgram: counterProgram.programId,
  })
  .rpc();
```

#### Payment Function with PDA Signer

Increment counter with payment, where PDA acts as payer:

```typescript
const [pdaVault] = anchor.web3.PublicKey.findProgramAddressSync(
  [Buffer.from("token_vault")],
  program.programId
);

// Increment with payment from PDA
await program.methods
  .incrementWithPaymentFromPda(new BN(100_000))
  .accounts({
    counter: counterPda,
    pdaVault: pdaVault,
    feeCollector: feeCollectorPubkey,
    counterProgram: counterProgram.programId,
    systemProgram: SystemProgram.programId,
  })
  .rpc();
```

#### Direct Counter Operations

You can also call counter_program directly without going through starter_program:

```typescript
// Direct increment
await counterProgram.methods
  .increment()
  .accounts({ counter: counterPda })
  .rpc();

// Direct add
await counterProgram.methods
  .add(new BN(5))
  .accounts({ counter: counterPda })
  .rpc();

// Reset counter (requires authority)
await counterProgram.methods
  .reset()
  .accounts({
    counter: counterPda,
    authority: user.publicKey,
  })
  .rpc();
```

**📚 See [CROSS_PROGRAM.md](./CROSS_PROGRAM.md) for comprehensive guide on CPI patterns, best practices, and security considerations.**

## 🎯 Phase 1 Advanced Features

### 7. Role-Based Access Control (RBAC)

The RBAC system provides fine-grained permission management with three role types and eight permission flags.

#### Role Types

- **Admin** - Full system access with all permissions
- **Moderator** - Limited administrative capabilities
- **User** - Basic user permissions

#### Permission Flags (Bitmask)

```typescript
const PERMISSIONS = {
  MANAGE_CONFIG: 1 << 0, // 0x01
  MANAGE_USERS: 1 << 1, // 0x02
  MANAGE_TOKENS: 1 << 2, // 0x04
  PAUSE_PROGRAM: 1 << 3, // 0x08
  EMERGENCY_ACTIONS: 1 << 4, // 0x10
  MANAGE_TREASURY: 1 << 5, // 0x20
  MANAGE_ROLES: 1 << 6, // 0x40
  BATCH_OPERATIONS: 1 << 7, // 0x80
};
```

#### Assign Role

```typescript
const [rolePda] = PublicKey.findProgramAddressSync(
  [Buffer.from("role"), user.publicKey.toBuffer()],
  program.programId
);

await program.methods
  .assignRole({ admin: {} }) // or { moderator: {} }, { user: {} }
  .accounts({
    role: rolePda,
    programConfig: configPda,
    admin: admin.publicKey,
    targetAuthority: user.publicKey,
    systemProgram: SystemProgram.programId,
  })
  .rpc();
```

#### Update Permissions

```typescript
const addPermissions = PERMISSIONS.MANAGE_TOKENS | PERMISSIONS.MANAGE_USERS;
const removePermissions = 0;

await program.methods
  .updateRolePermissions(addPermissions, removePermissions)
  .accounts({
    role: rolePda,
    programConfig: configPda,
    admin: admin.publicKey,
    targetAuthority: user.publicKey,
  })
  .rpc();
```

#### Check Permission

```typescript
const hasPermission = await program.methods
  .checkPermission(PERMISSIONS.MANAGE_TOKENS)
  .accounts({
    role: rolePda,
    authority: user.publicKey,
  })
  .view();
```

#### Revoke Role

```typescript
await program.methods
  .revokeRole()
  .accounts({
    role: rolePda,
    programConfig: configPda,
    admin: admin.publicKey,
    targetAuthority: user.publicKey,
    recipient: admin.publicKey,
  })
  .rpc();
```

### 8. Advanced Token Operations

Extended SPL token functionality including delegation, freezing, and account management.

#### Approve Delegate

Allow another account to spend tokens on your behalf.

```typescript
await program.methods
  .approveDelegate(new BN(1000000))
  .accounts({
    tokenAccount: userTokenAccount,
    delegate: delegatePublicKey,
    authority: owner.publicKey,
    tokenProgram: TOKEN_PROGRAM_ID,
  })
  .rpc();
```

#### Revoke Delegate

Revoke previously approved delegate.

```typescript
await program.methods
  .revokeDelegate()
  .accounts({
    tokenAccount: userTokenAccount,
    authority: owner.publicKey,
    tokenProgram: TOKEN_PROGRAM_ID,
  })
  .rpc();
```

#### Freeze Token Account

Freeze an account to prevent all token operations. Requires freeze authority.

```typescript
const [freezeAuthority] = PublicKey.findProgramAddressSync(
  [Buffer.from("mint_authority")],
  program.programId
);

await program.methods
  .freezeTokenAccount()
  .accounts({
    tokenAccount: userTokenAccount,
    mint: mintPda,
    freezeAuthority: freezeAuthority,
    tokenProgram: TOKEN_PROGRAM_ID,
  })
  .rpc();
```

#### Thaw Token Account

Unfreeze a previously frozen account.

```typescript
await program.methods
  .thawTokenAccount()
  .accounts({
    tokenAccount: userTokenAccount,
    mint: mintPda,
    freezeAuthority: freezeAuthority,
    tokenProgram: TOKEN_PROGRAM_ID,
  })
  .rpc();
```

#### Close Token Account

Close an empty token account and reclaim rent.

```typescript
await program.methods
  .closeTokenAccount()
  .accounts({
    tokenAccount: userTokenAccount,
    destination: owner.publicKey,
    authority: owner.publicKey,
    tokenProgram: TOKEN_PROGRAM_ID,
  })
  .rpc();
```

### 9. Treasury Management

Centralized treasury for managing program funds with emergency controls.

#### Initialize Treasury

```typescript
const [treasuryPda] = PublicKey.findProgramAddressSync(
  [Buffer.from("treasury")],
  program.programId
);

await program.methods
  .initializeTreasury()
  .accounts({
    treasury: treasuryPda,
    authority: admin.publicKey,
    systemProgram: SystemProgram.programId,
  })
  .rpc();
```

#### Deposit to Treasury

Anyone can deposit SOL to the treasury.

```typescript
await program.methods
  .depositToTreasury(new BN(LAMPORTS_PER_SOL))
  .accounts({
    treasury: treasuryPda,
    depositor: user.publicKey,
    systemProgram: SystemProgram.programId,
  })
  .rpc();
```

#### Withdraw from Treasury

Only admin can withdraw funds (requires admin role).

```typescript
await program.methods
  .withdrawFromTreasury(new BN(0.5 * LAMPORTS_PER_SOL))
  .accounts({
    treasury: treasuryPda,
    programConfig: configPda,
    admin: admin.publicKey,
    recipient: recipient.publicKey,
    systemProgram: SystemProgram.programId,
  })
  .rpc();
```

#### Emergency Withdraw

Withdraw all funds and activate emergency mode. Sets `emergency_mode = true` flag.

```typescript
await program.methods
  .emergencyWithdraw()
  .accounts({
    treasury: treasuryPda,
    programConfig: configPda,
    admin: admin.publicKey,
    recipient: admin.publicKey,
    systemProgram: SystemProgram.programId,
  })
  .rpc();
```

#### Toggle Circuit Breaker

Pause/unpause treasury deposits for security purposes.

```typescript
await program.methods
  .toggleCircuitBreaker()
  .accounts({
    treasury: treasuryPda,
    programConfig: configPda,
    admin: admin.publicKey,
  })
  .rpc();
```

### 10. Events System

All major operations emit events for monitoring and indexing.

#### Event Types

**Token Events:**

- `TokensMintedEvent` - Fired when tokens are minted
- `TokensTransferredEvent` - Fired on token transfers
- `TokensBurnedEvent` - Fired when tokens are burned
- `DelegateApprovedEvent` - Delegate approval granted
- `DelegateRevokedEvent` - Delegate approval revoked
- `TokenAccountClosedEvent` - Token account closed
- `TokenAccountFrozenEvent` - Account frozen
- `TokenAccountThawedEvent` - Account unfrozen

**User Events:**

- `UserAccountCreatedEvent` - New user account created
- `UserAccountUpdatedEvent` - User account modified
- `UserAccountClosedEvent` - User account closed

**Config Events:**

- `ConfigUpdatedEvent` - Program config updated
- `ProgramPausedEvent` - Program pause toggled

**RBAC Events:**

- `RoleAssignedEvent` - Role assigned to user
- `RoleUpdatedEvent` - Role permissions modified
- `RoleRevokedEvent` - Role revoked

**Treasury Events:**

- `TreasuryInitializedEvent` - Treasury created
- `TreasuryDepositEvent` - Funds deposited
- `TreasuryWithdrawEvent` - Funds withdrawn
- `EmergencyWithdrawEvent` - Emergency withdrawal executed
- `CircuitBreakerToggledEvent` - Circuit breaker state changed

#### Listening to Events

```typescript
const listener = program.addEventListener(
  "TokensMintedEvent",
  (event, slot) => {
    console.log("Tokens minted:", {
      mint: event.mint.toString(),
      recipient: event.recipient.toString(),
      amount: event.amount.toString(),
      timestamp: new Date(event.timestamp * 1000),
    });
  }
);

program.removeEventListener(listener);
```

## 🔐 Security Features

### Account Validation

- `has_one` constraints for authority validation
- Seeds validation for PDAs
- Bump seed storage and verification
- Account type checks

### Error Handling

Custom error codes with descriptive messages:

**Core Errors:**

- `Unauthorized` - Unauthorized access attempt
- `InvalidAmount` - Invalid amount provided
- `ArithmeticOverflow` - Arithmetic overflow occurred
- `NotRentExempt` - Account is not rent exempt
- `InvalidStateTransition` - Invalid state transition
- `CpiFailed` - CPI call failed
- `InvalidMint` - Invalid mint address
- `InvalidTokenAccount` - Invalid token account
- `InsufficientBalance` - Insufficient balance
- `ProgramPaused` - Program is paused

**RBAC Errors:**

- `InsufficientPermissions` - User lacks required permissions
- `InvalidRoleType` - Invalid role type provided
- `RoleAlreadyExists` - Role already assigned to user
- `RoleNotFound` - Role does not exist for user
- `CannotModifySuperAdmin` - Cannot modify super admin permissions

### Rent Exemption

All accounts are created with rent exemption by default using proper space calculation.

## 🧪 Testing

### Run All Tests

```bash
anchor test
```

### Run Specific Test Files

```bash
# Run starter program tests only
anchor test tests/starter_program.ts

# Run cross-program interaction tests only
anchor test tests/cross_program.ts

# Run RBAC tests only
anchor test tests/rbac.ts

# Run advanced token tests only
anchor test tests/advanced_token.ts

# Run treasury tests only
anchor test tests/treasury.ts
```

### Test Coverage

**Starter Program** (25+ tests):

- Program initialization
- Configuration management (create, update, pause)
- User account operations (create, update, close)
- Token operations (mint, transfer, burn)
- CPI examples (SOL and token transfers)

**Cross-Program Interaction** (14 tests):

- Direct counter operations (initialize, increment, add, reset)
- CPI from starter_program to counter_program
- Multiple CPI calls in one transaction
- Payment functions with SOL transfers
- PDA signer for CPI operations
- Arithmetic overflow handling
- State consistency verification

**RBAC System** (25+ tests):

- Role assignment (Admin, Moderator, User)
- Permission management (add, remove, check)
- Role revocation
- Admin-only operations
- Permission validation
- Multiple role scenarios
- Edge cases and error handling

**Advanced Token Operations** (14+ tests):

- Delegate approval and revocation
- Token account freezing and thawing
- Account closing with rent reclaim
- Frozen account transfer blocking
- Delegate transfer functionality
- Comprehensive error scenarios

**Treasury Management** (18+ tests):

- Treasury initialization
- Deposit operations (multiple users)
- Admin-only withdrawals
- Emergency withdrawal with mode flag
- Circuit breaker toggle (pause/unpause)
- Permission validation
- State consistency checks
- Balance verification

**Total: 96+ integration tests**

### Test Results

```
Cross-Program Interaction
  Direct Counter Program Operations
    ✓ Should initialize counter directly
    ✓ Should increment counter directly
    ✓ Should add value to counter directly
    ✓ Should reset counter
  Cross-Program Invocation via Starter Program
    ✓ Should increment counter via CPI
    ✓ Should add value to counter via CPI
    ✓ Should increment multiple times via CPI
    ✓ Should handle arithmetic overflow
  Complex Cross-Program Scenarios
    ✓ Should chain multiple CPI operations
    ✓ Should read counter state after CPI
    ✓ Should verify counter state consistency
  Payment Function with PDA Signer
    ✓ Should increment with payment directly
    ✓ Should increment with payment from PDA via CPI
    ✓ Should fail with insufficient PDA balance

14 passing (19s)
```

## 📚 Learning Resources

### Anchor Framework

- [Anchor Documentation](https://www.anchor-lang.com/)
- [Anchor Book](https://book.anchor-lang.com/)

### Solana Development

- [Solana Cookbook](https://solanacookbook.com/)
- [Solana Developer Docs](https://solana.com/docs)

### Best Practices

**Core Principles:**

- Always validate account ownership
- Use PDAs for program-owned accounts
- Store bump seeds in account data
- Implement proper error handling
- Ensure rent exemption for all accounts
- Use `has_one` for authority checks
- Add descriptive error messages

**RBAC & Permissions:**

- Use role-based access control for administrative functions
- Check permissions before executing privileged operations
- Implement principle of least privilege
- Use bitmask for efficient permission storage
- Emit events for all role changes

**Token Management:**

- Always validate mint and token account ownership
- Use PDA as freeze authority for program-controlled freezing
- Verify token account is empty before closing
- Emit events for all token operations
- Handle frozen accounts gracefully

**Treasury & Emergency Controls:**

- Implement circuit breaker for emergency situations
- Use emergency mode flag to prevent operations after emergency
- Separate deposit and withdrawal permissions
- Track total deposits and withdrawals
- Maintain rent-exempt balance in treasury

**Events & Monitoring:**

- Emit events for all state-changing operations
- Include timestamps in events for tracking
- Use descriptive event names
- Index events for off-chain querying
- Monitor events for suspicious activity

## 🤝 Contributing

This is a starter template. Feel free to use it as a foundation for your Solana programs!

## 📄 License

MIT

## 🎯 Next Steps

### Phase 1 Complete ✅

This starter program now includes:

- ✅ Role-Based Access Control (RBAC)
- ✅ Advanced Token Operations
- ✅ Treasury Management
- ✅ Comprehensive Events System
- ✅ Emergency Controls

### Recommended Extensions

1. **Governance System** - Add proposal creation, voting, and execution
2. **Staking Mechanism** - Implement token staking with rewards
3. **NFT Support** - Add NFT minting and marketplace features
4. **Oracle Integration** - Connect with price feeds (Pyth, Chainlink)
5. **Multi-Signature** - Add multi-sig approval for critical operations
6. **Time-Locks** - Implement time-delayed operations for security
7. **Account Freezing** - Add program-wide account freeze capabilities
8. **Batch Operations** - Support batch token transfers and operations
9. **Fee Collection** - Implement protocol fees with distribution logic
10. **Upgradability** - Add program upgrade mechanism with governance

### Deployment Checklist

- [ ] Run all tests and ensure 100% pass rate
- [ ] Review and audit all error handling paths
- [ ] Verify all PDAs use correct seeds
- [ ] Confirm rent-exempt minimum for all accounts
- [ ] Test on localnet with realistic scenarios
- [ ] Deploy to devnet and perform integration testing
- [ ] Conduct security audit (recommended for mainnet)
- [ ] Set up monitoring for events and errors
- [ ] Document all admin operations
- [ ] Prepare incident response plan for emergency controls

### Integration Guide

1. **Customize** - Modify the program logic for your specific use case
2. **Extend** - Add more instructions and account types based on recommended extensions
3. **Test** - Write comprehensive tests for all custom logic
4. **Deploy** - Deploy to devnet/mainnet after thorough testing
5. **Build UI** - Create a frontend using the generated IDL
6. **Monitor** - Set up event listeners for real-time monitoring

---

Built with ❤️ using [Anchor Framework](https://www.anchor-lang.com/)
