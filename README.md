# Solana Starter Program

A comprehensive Anchor framework starter template demonstrating all essential Solana program patterns including configuration management, PDAs, SPL tokens, cross-program invocations, and inter-program communication.

## 🚀 Features

### Core Components
- ✅ **Program Configuration** - Admin-controlled config with pause functionality
- ✅ **PDA Patterns** - User accounts with seeds-based derivation
- ✅ **SPL Token Operations** - Mint, transfer, and burn tokens
- ✅ **Cross-Program Invocation** - CPI examples with and without PDA signers
- ✅ **Inter-Program Communication** - Counter program with bidirectional CPI patterns
- ✅ **Error Handling** - Custom error codes with descriptive messages
- ✅ **Security Best Practices** - Account validation, authority checks, rent exemption

## 📁 Project Structure

```
starter_program/
├── programs/
│   ├── starter_program/
│   │   └── src/
│   │       ├── lib.rs              # Program entry point (17 instructions)
│   │       ├── constants.rs        # PDA seeds and constants
│   │       ├── error.rs            # Custom error definitions
│   │       ├── state/              # Account structures
│   │       │   ├── mod.rs
│   │       │   ├── config.rs       # Program configuration
│   │       │   └── user.rs         # User account state
│   │       └── instructions/       # Instruction handlers
│   │           ├── mod.rs
│   │           ├── initialize.rs   # Initialize program
│   │           ├── config.rs       # Config management
│   │           ├── user.rs         # User account operations
│   │           ├── token.rs        # SPL token operations
│   │           ├── cpi.rs          # Cross-program invocations
│   │           └── cross_program.rs # CPI with counter program
│   └── counter_program/
│       └── src/
│           └── lib.rs              # Counter program (5 instructions)
├── tests/
│   ├── starter_program.ts          # Starter program tests (25+ tests)
│   └── cross_program.ts            # Cross-program interaction tests (11 tests)
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
  [Buffer.from('program_config')],
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
  [Buffer.from('user_account'), user.publicKey.toBuffer()],
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
  [Buffer.from('mint')],
  program.programId
);

const [mintAuthority] = PublicKey.findProgramAddressSync(
  [Buffer.from('mint_authority')],
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
  [Buffer.from('token_vault')],
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
  [Buffer.from('token_vault')],
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
  [Buffer.from('counter'), user.publicKey.toBuffer()],
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
  .incrementMultiple(5)  // Increment 5 times
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
  [Buffer.from('token_vault')],
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

## 🔐 Security Features

### Account Validation
- `has_one` constraints for authority validation
- Seeds validation for PDAs
- Bump seed storage and verification
- Account type checks

### Error Handling
Custom error codes with descriptive messages:
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

**Total: 39+ integration tests**

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
- Always validate account ownership
- Use PDAs for program-owned accounts
- Store bump seeds in account data
- Implement proper error handling
- Ensure rent exemption for all accounts
- Use `has_one` for authority checks
- Add descriptive error messages

## 🤝 Contributing

This is a starter template. Feel free to use it as a foundation for your Solana programs!

## 📄 License

MIT

## 🎯 Next Steps

1. **Customize** - Modify the program logic for your specific use case
2. **Extend** - Add more instructions and account types
3. **Deploy** - Deploy to devnet/mainnet after thorough testing
4. **Build UI** - Create a frontend using the generated IDL

---

Built with ❤️ using [Anchor Framework](https://www.anchor-lang.com/)
