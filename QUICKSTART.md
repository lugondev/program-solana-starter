# 🚀 Quick Start Guide

## Build & Deploy in 5 Minutes

### 1. Build the Program
```bash
cd starter_program
anchor build
```

### 2. Get Program ID
```bash
anchor keys list
```

### 3. Update Program ID
Replace the program ID in:
- `programs/starter_program/src/lib.rs` (line 12)
- `Anchor.toml` (line 9)

### 4. Rebuild
```bash
anchor build
```

### 5. Deploy to Devnet
```bash
solana config set --url devnet
anchor deploy
```

### 6. Run Tests
```bash
anchor test
```

## 🎯 Common Use Cases

### Use Case 1: Token with Admin Control
```typescript
// 1. Initialize config
const [config] = PublicKey.findProgramAddressSync(
  [Buffer.from('program_config')],
  program.programId
);
await program.methods.initializeConfig(feeDestination).rpc();

// 2. Create mint
const [mint] = PublicKey.findProgramAddressSync(
  [Buffer.from('mint')],
  program.programId
);
await program.methods.createMint().rpc();

// 3. Mint tokens
await program.methods.mintTokens(new BN(1000000)).rpc();
```

### Use Case 2: User Points System
```typescript
// 1. Create user account
const [userPda] = PublicKey.findProgramAddressSync(
  [Buffer.from('user_account'), user.publicKey.toBuffer()],
  program.programId
);
await program.methods.createUserAccount().rpc();

// 2. Update points
await program.methods.updateUserAccount(new BN(100)).rpc();

// 3. Read points
const account = await program.account.userAccount.fetch(userPda);
console.log('Points:', account.points.toString());
```

### Use Case 3: Cross-Program Interaction
```typescript
// 1. Initialize counter via CPI
const counterProgram = anchor.workspace.CounterProgram;
const [counterPda] = PublicKey.findProgramAddressSync(
  [Buffer.from('counter'), user.publicKey.toBuffer()],
  counterProgram.programId
);

await program.methods
  .initializeCounter()
  .accounts({
    counter: counterPda,
    authority: user.publicKey,
    counterProgram: counterProgram.programId,
    systemProgram: SystemProgram.programId,
  })
  .rpc();

// 2. Increment counter via CPI
await program.methods
  .incrementCounter()
  .accounts({
    counter: counterPda,
    authority: user.publicKey,
    counterProgram: counterProgram.programId,
  })
  .rpc();

// 3. Add value via CPI
await program.methods
  .addToCounter(new BN(10))
  .accounts({
    counter: counterPda,
    authority: user.publicKey,
    counterProgram: counterProgram.programId,
  })
  .rpc();

// 4. Check counter value
const counter = await counterProgram.account.counter.fetch(counterPda);
console.log('Count:', counter.count.toString());
```

### Use Case 4: Treasury Management
```typescript
// 1. Fund PDA vault
const [vault] = PublicKey.findProgramAddressSync(
  [Buffer.from('token_vault')],
  program.programId
);
const airdrop = await connection.requestAirdrop(vault, 10000000);

// 2. Transfer from vault
await program.methods
  .transferSolWithPda(new BN(500000))
  .accounts({vault, recipient: recipient.publicKey})
  .rpc();
```

## 📦 What's Included

### Two Programs
1. **Starter Program** - Main program with 17 instructions
2. **Counter Program** - Example program for CPI with 5 instructions

### Source Files (Starter Program)
- `lib.rs` - Program entry (17 instructions)
- `constants.rs` - PDA seeds
- `error.rs` - 10 error types
- `state/config.rs` - Program config
- `state/user.rs` - User accounts
- `instructions/initialize.rs` - Init handler
- `instructions/config.rs` - Config management (3 instructions)
- `instructions/user.rs` - User operations (3 instructions)
- `instructions/token.rs` - Token operations (4 instructions)
- `instructions/cpi.rs` - CPI examples (3 instructions)
- `instructions/cross_program.rs` - Counter CPI (4 instructions)

### Source Files (Counter Program)
- `lib.rs` - Counter program (5 instructions: init, increment, decrement, add, reset)

### Test Suite
- 25+ starter program tests
- 11 cross-program interaction tests
- **Total: 36+ integration tests**
- All instruction coverage
- Success and failure cases

### Documentation
- Complete README with examples
- CROSS_PROGRAM.md - CPI patterns guide
- Security best practices
- Learning resources

## 🔧 Customization Guide

### Add New Instruction

1. **Create handler** in `instructions/`
```rust
// instructions/my_feature.rs
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct MyFeature<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
}

pub fn my_feature_handler(ctx: Context<MyFeature>) -> Result<()> {
    msg!("My feature executed");
    Ok(())
}
```

2. **Export in** `instructions/mod.rs`
```rust
pub mod my_feature;
pub use my_feature::*;
```

3. **Add to program** in `lib.rs`
```rust
pub fn my_feature(ctx: Context<MyFeature>) -> Result<()> {
    my_feature_handler(ctx)
}
```

4. **Rebuild**
```bash
anchor build
```

### Add New Account Type

1. **Create state** in `state/`
```rust
// state/my_account.rs
#[account]
pub struct MyAccount {
    pub owner: Pubkey,
    pub data: u64,
    pub bump: u8,
}

impl MyAccount {
    pub const LEN: usize = 8 + 32 + 8 + 1;
}
```

2. **Export** in `state/mod.rs`
```rust
pub mod my_account;
pub use my_account::*;
```

## 🎓 Learning Path

1. **Day 1**: Understand PDAs
   - Run user account tests
   - Modify `user.rs` instructions
   
2. **Day 2**: Master Token Operations
   - Run token tests
   - Create custom token features

3. **Day 3**: Learn CPIs
   - Study `cpi.rs` examples
   - Add custom CPI calls

4. **Day 4**: Cross-Program Interaction
   - Study counter_program implementation
   - Learn CPI patterns from `cross_program.rs`
   - Read CROSS_PROGRAM.md guide

5. **Day 5**: Build Real Project
   - Combine all patterns
   - Add frontend integration

## 🐛 Troubleshooting

### Build Fails
```bash
# Clean and rebuild
anchor clean
anchor build
```

### Tests Fail
```bash
# Check Solana test validator
solana-test-validator

# In another terminal
anchor test --skip-local-validator
```

### Deploy Fails
```bash
# Check balance
solana balance

# Request airdrop (devnet)
solana airdrop 2

# Check cluster
solana config get
```

## 📞 Support

- [Anchor Discord](https://discord.gg/anchor)
- [Solana Stack Exchange](https://solana.stackexchange.com/)
- [GitHub Issues](https://github.com/coral-xyz/anchor/issues)

---

**Next Steps**: Read the full [README.md](README.md) for detailed API documentation.
