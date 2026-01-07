# Cross-Program Interaction Guide

## Overview

This guide demonstrates how to implement **Cross-Program Invocation (CPI)** patterns in Solana using the Anchor framework. The example includes two programs:

1. **Counter Program** - A simple counter with increment/decrement operations
2. **Starter Program** - Calls counter program via CPI to demonstrate inter-program communication

## Architecture

```
┌─────────────────────┐         CPI          ┌──────────────────────┐
│  Starter Program    │ ──────────────────► │  Counter Program     │
│                     │                      │                      │
│  - initialize       │                      │  - initialize        │
│  - increment        │                      │  - increment         │
│  - add_to_counter   │                      │  - add               │
│  - increment_multi  │                      │  - decrement         │
└─────────────────────┘                      │  - reset             │
                                             └──────────────────────┘
```

## Counter Program

### Program ID
```
CounzVsCGF4VzNkAwePKC9mXr6YWiFYF4kLW6YdV8Cc
```

### Account Structure

```rust
#[account]
pub struct Counter {
    pub authority: Pubkey,  // Owner of the counter
    pub count: u64,         // Current count value
    pub bump: u8,           // PDA bump seed
}

// Space: 8 (discriminator) + 32 (Pubkey) + 8 (u64) + 1 (u8) = 49 bytes
```

### PDA Seeds
```rust
seeds = [b"counter", authority.key().as_ref()]
```

### Instructions

#### 1. Initialize
Creates a new counter PDA for the authority.

```rust
pub fn initialize(ctx: Context<Initialize>) -> Result<()>
```

**Accounts:**
- `counter` (init, mut) - Counter PDA to create
- `authority` (signer, mut) - Owner and payer
- `system_program` - System Program

**Usage:**
```typescript
const [counterPda] = anchor.web3.PublicKey.findProgramAddressSync(
  [Buffer.from('counter'), user.publicKey.toBuffer()],
  counterProgram.programId
)

await counterProgram.methods
  .initialize()
  .accounts({
    counter: counterPda,
    authority: user.publicKey,
    systemProgram: anchor.web3.SystemProgram.programId,
  })
  .rpc()
```

#### 2. Increment
Increases counter by 1.

```rust
pub fn increment(ctx: Context<Update>) -> Result<()>
```

**Accounts:**
- `counter` (mut) - Counter PDA

**Usage:**
```typescript
await counterProgram.methods
  .increment()
  .accounts({ counter: counterPda })
  .rpc()
```

#### 3. Decrement
Decreases counter by 1.

```rust
pub fn decrement(ctx: Context<Update>) -> Result<()>
```

#### 4. Add
Adds arbitrary value to counter.

```rust
pub fn add(ctx: Context<Update>, value: u64) -> Result<()>
```

**Usage:**
```typescript
await counterProgram.methods
  .add(new anchor.BN(5))
  .accounts({ counter: counterPda })
  .rpc()
```

#### 5. Reset
Resets counter to 0 (requires authority).

```rust
pub fn reset(ctx: Context<Reset>) -> Result<()>
```

**Accounts:**
- `counter` (mut, has_one = authority) - Counter PDA
- `authority` (signer) - Counter owner

#### 6. Increment with Payment
Increments counter and requires SOL payment to fee collector.

```rust
pub fn increment_with_payment(ctx: Context<IncrementWithPayment>, payment: u64) -> Result<()>
```

**Accounts:**
- `counter` (mut) - Counter PDA
- `payer` (signer, mut) - Pays the fee
- `fee_collector` (mut) - Receives the payment
- `system_program` - System Program for transfer

**Usage:**
```typescript
await counterProgram.methods
  .incrementWithPayment(new anchor.BN(100_000))
  .accounts({
    counter: counterPda,
    payer: user.publicKey,
    feeCollector: feeCollectorPubkey,
    systemProgram: anchor.web3.SystemProgram.programId,
  })
  .rpc()
```

**Key Points:**
- Payment is transferred BEFORE incrementing (security pattern)
- Fee collector must be rent-exempt or have sufficient balance
- Both payment transfer and increment happen atomically
- If either operation fails, entire transaction reverts

---

## Cross-Program Invocation (CPI)

### Enabling CPI in Counter Program

#### 1. Add CPI Feature to Cargo.toml

```toml
[features]
cpi = ["no-entrypoint"]
default = []
```

#### 2. Add Dependency in Caller Program

```toml
[dependencies]
counter_program = { path = "../counter_program", features = ["cpi"] }
```

This enables:
- `counter_program::cpi::accounts::{Initialize, Increment, Add}`
- `counter_program::cpi::{initialize, increment, add}`
- `counter_program::Counter` account type

### CPI Instructions in Starter Program

#### 1. Initialize Counter via CPI

```rust
#[derive(Accounts)]
pub struct InitializeCounter<'info> {
    #[account(mut)]
    pub counter: Signer<'info>,
    
    #[account(mut)]
    pub authority: Signer<'info>,
    
    pub counter_program: Program<'info, CounterProgram>,
    pub system_program: Program<'info, System>,
}

pub fn initialize_counter_handler(ctx: Context<InitializeCounter>) -> Result<()> {
    let cpi_accounts = Initialize {
        counter: ctx.accounts.counter.to_account_info(),
        authority: ctx.accounts.authority.to_account_info(),
        system_program: ctx.accounts.system_program.to_account_info(),
    };
    
    let cpi_program = ctx.accounts.counter_program.to_account_info();
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
    
    counter_program::cpi::initialize(cpi_ctx)?;
    
    msg!("Counter initialized via CPI");
    Ok(())
}
```

**Key Points:**
- Counter account passed as `Signer` to allow counter_program to init it
- No seeds validation needed - counter_program handles that
- System program required for account creation

#### 2. Increment Counter via CPI

```rust
#[derive(Accounts)]
pub struct IncrementCounter<'info> {
    #[account(mut)]
    pub counter: Account<'info, Counter>,
    
    pub authority: Signer<'info>,
    
    pub counter_program: Program<'info, CounterProgram>,
}

pub fn increment_counter_handler(ctx: Context<IncrementCounter>) -> Result<()> {
    let cpi_accounts = Increment {
        counter: ctx.accounts.counter.to_account_info(),
    };
    
    let cpi_program = ctx.accounts.counter_program.to_account_info();
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
    
    counter_program::cpi::increment(cpi_ctx)?;
    
    msg!("Counter incremented via CPI");
    Ok(())
}
```

**Usage:**
```typescript
await starterProgram.methods
  .incrementCounter()
  .accounts({
    counter: counterPda,
    authority: user.publicKey,
    counterProgram: counterProgram.programId,
  })
  .rpc()
```

#### 3. Add to Counter via CPI

```rust
pub fn add_to_counter_handler(ctx: Context<AddToCounter>, value: u64) -> Result<()> {
    let cpi_accounts = Add {
        counter: ctx.accounts.counter.to_account_info(),
    };
    
    let cpi_program = ctx.accounts.counter_program.to_account_info();
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
    
    counter_program::cpi::add(cpi_ctx, value)?;
    
    msg!("Added {} to counter via CPI", value);
    Ok(())
}
```

#### 4. Multiple CPI Calls in One Transaction

```rust
pub fn increment_multiple_handler(ctx: Context<IncrementMultiple>, times: u8) -> Result<()> {
    for i in 0..times {
        let cpi_accounts = Increment {
            counter: ctx.accounts.counter.to_account_info(),
        };
        
        let cpi_program = ctx.accounts.counter_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        
        counter_program::cpi::increment(cpi_ctx)?;
        msg!("Increment #{}", i + 1);
    }
    
    msg!("Counter incremented {} times via CPI", times);
    Ok(())
}
```

**Usage:**
```typescript
await starterProgram.methods
  .incrementMultiple(3)  // Increment 3 times
  .accounts({
    counter: counterPda,
    authority: user.publicKey,
    counterProgram: counterProgram.programId,
  })
  .rpc()
```

#### 5. Payment via CPI with PDA Signer

The most advanced pattern: PDA from starter_program pays for counter increment.

```rust
#[derive(Accounts)]
pub struct IncrementWithPaymentFromPda<'info> {
    #[account(mut)]
    pub counter: Account<'info, Counter>,
    
    #[account(
        mut,
        seeds = [SEED_TOKEN_VAULT],
        bump
    )]
    pub pda_vault: SystemAccount<'info>,
    
    #[account(mut)]
    /// CHECK: Fee collector can be any account
    pub fee_collector: AccountInfo<'info>,
    
    pub counter_program: Program<'info, CounterProgram>,
    pub system_program: Program<'info, System>,
}

pub fn increment_with_payment_from_pda_handler(
    ctx: Context<IncrementWithPaymentFromPda>,
    payment: u64
) -> Result<()> {
    let seeds = &[
        SEED_TOKEN_VAULT,
        &[ctx.bumps.pda_vault],
    ];
    let signer = &[&seeds[..]];
    
    let cpi_accounts = IncrementWithPayment {
        counter: ctx.accounts.counter.to_account_info(),
        payer: ctx.accounts.pda_vault.to_account_info(),
        fee_collector: ctx.accounts.fee_collector.to_account_info(),
        system_program: ctx.accounts.system_program.to_account_info(),
    };
    
    let cpi_program = ctx.accounts.counter_program.to_account_info();
    let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
    
    counter_program::cpi::increment_with_payment(cpi_ctx, payment)?;
    
    msg!("Counter incremented with payment of {} lamports from PDA", payment);
    Ok(())
}
```

**Usage:**
```typescript
const [pdaVault] = anchor.web3.PublicKey.findProgramAddressSync(
  [Buffer.from('token_vault')],
  starterProgram.programId
)

await starterProgram.methods
  .incrementWithPaymentFromPda(new anchor.BN(100_000))
  .accounts({
    counter: counterPda,
    pdaVault: pdaVault,
    feeCollector: feeCollectorPubkey,
    counterProgram: counterProgram.programId,
    systemProgram: anchor.web3.SystemProgram.programId,
  })
  .rpc()
```

**Key Points:**
- PDA acts as payer using `CpiContext::new_with_signer`
- Seeds must match PDA derivation in account constraints
- PDA must have sufficient balance for payment + rent
- Demonstrates program-controlled treasury pattern

---

## CPI Best Practices

### 1. Account Validation

**❌ Don't validate in caller program:**
```rust
// Avoid this in CPI context
#[account(
    mut,
    seeds = [b"counter", authority.key().as_ref()],
    bump = counter.bump,  // Caller doesn't know the bump!
)]
pub counter: Account<'info, Counter>,
```

**✅ Let the target program validate:**
```rust
// Caller just passes the account
#[account(mut)]
pub counter: Account<'info, Counter>,
```

The target program (counter_program) will validate seeds and constraints.

### 2. Account Ownership

```rust
// Target program automatically checks:
// - Account is owned by its program ID
// - Account discriminator matches expected type
// - Seeds match for PDAs

// Caller doesn't need to duplicate these checks
```

### 3. Signer Propagation

```rust
// Signers in the caller are automatically signers in CPI
#[account(mut)]
pub authority: Signer<'info>,  // Available to CPI

// CPI context preserves signer status
let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
```

### 4. PDA Signing

If your program needs to sign as a PDA:

```rust
let seeds = &[
    b"authority",
    ctx.accounts.authority.key().as_ref(),
    &[ctx.bumps.pda_account],
];
let signer = &[&seeds[..]];

let cpi_ctx = CpiContext::new_with_signer(
    cpi_program,
    cpi_accounts,
    signer  // PDA can now sign
);
```

### 5. Error Handling

```rust
// CPI errors propagate automatically
match counter_program::cpi::increment(cpi_ctx) {
    Ok(_) => msg!("CPI succeeded"),
    Err(e) => return Err(e),  // Automatically handled by ?
}

// Or just use ? operator
counter_program::cpi::increment(cpi_ctx)?;
```

---

## Testing Cross-Program Interactions

### Test Structure

```typescript
describe('Cross-Program Interaction', () => {
  let counterProgram: Program<CounterProgram>
  let starterProgram: Program<StarterProgram>
  let user: anchor.web3.Keypair
  let counterPda: anchor.web3.PublicKey

  before(async () => {
    const provider = anchor.AnchorProvider.env()
    anchor.setProvider(provider)

    counterProgram = anchor.workspace.CounterProgram
    starterProgram = anchor.workspace.StarterProgram
    user = provider.wallet as anchor.Wallet

    // Derive counter PDA
    ;[counterPda] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from('counter'), user.publicKey.toBuffer()],
      counterProgram.programId
    )

    // Initialize counter
    await counterProgram.methods
      .initialize()
      .accounts({
        counter: counterPda,
        authority: user.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc()
  })

  it('Should increment via CPI', async () => {
    const before = await counterProgram.account.counter.fetch(counterPda)

    await starterProgram.methods
      .incrementCounter()
      .accounts({
        counter: counterPda,
        authority: user.publicKey,
        counterProgram: counterProgram.programId,
      })
      .rpc()

    const after = await counterProgram.account.counter.fetch(counterPda)
    expect(after.count.toNumber()).to.equal(before.count.toNumber() + 1)
  })
})
```

### Test Results

```
Cross-Program Interaction
  Direct Counter Program Operations
    ✓ Should initialize counter directly (533ms)
    ✓ Should increment counter directly (473ms)
    ✓ Should add value to counter directly (470ms)
    ✓ Should reset counter (476ms)
  Cross-Program Invocation via Starter Program
    ✓ Should increment counter via CPI (477ms)
    ✓ Should add value to counter via CPI (471ms)
    ✓ Should increment multiple times via CPI (470ms)
    ✓ Should handle arithmetic overflow (945ms)
  Complex Cross-Program Scenarios
    ✓ Should chain multiple CPI operations (1423ms)
    ✓ Should read counter state after CPI (466ms)
    ✓ Should verify counter state consistency (934ms)
  Payment Function with PDA Signer
    ✓ Should increment with payment directly (1471ms)
    ✓ Should increment with payment from PDA via CPI (1435ms)
    ✓ Should fail with insufficient PDA balance

14 passing (19s)
```

---

## Common Patterns

### Pattern 1: Simple CPI (Read-Only)

```rust
// Query another program's state
let counter_data = counter_program::Counter::try_from(
    &ctx.accounts.counter.to_account_info()
)?;
msg!("Current count: {}", counter_data.count);
```

### Pattern 2: CPI with State Mutation

```rust
// Modify state in another program
counter_program::cpi::increment(cpi_ctx)?;

// Verify the change
let updated = Counter::try_from(&ctx.accounts.counter)?;
require!(updated.count > 0, ErrorCode::InvalidState);
```

### Pattern 3: Conditional CPI

```rust
pub fn conditional_increment(ctx: Context<ConditionalOp>) -> Result<()> {
    let counter = Counter::try_from(&ctx.accounts.counter)?;
    
    if counter.count < 100 {
        let cpi_accounts = Increment {
            counter: ctx.accounts.counter.to_account_info(),
        };
        
        let cpi_ctx = CpiContext::new(
            ctx.accounts.counter_program.to_account_info(),
            cpi_accounts
        );
        
        counter_program::cpi::increment(cpi_ctx)?;
    }
    
    Ok(())
}
```

### Pattern 4: Chained CPI

```rust
pub fn chain_operations(ctx: Context<ChainOps>) -> Result<()> {
    // First CPI
    counter_program::cpi::add(cpi_ctx1, 5)?;
    
    // Second CPI
    counter_program::cpi::increment(cpi_ctx2)?;
    
    // Third CPI
    counter_program::cpi::increment(cpi_ctx3)?;
    
    Ok(())
}
```

---

## Program Deployment

### 1. Build Both Programs

```bash
cd starter_program
anchor build
```

### 2. Deploy to Devnet

```bash
# Deploy counter_program first
anchor deploy --program-name counter_program --provider.cluster devnet

# Then deploy starter_program
anchor deploy --program-name starter_program --provider.cluster devnet
```

### 3. Update Program IDs

```bash
# Get deployed program IDs
anchor keys list

# Update in Anchor.toml and declare_id!() macros
```

### 4. Verify Deployment

```bash
# Check counter_program
solana program show <COUNTER_PROGRAM_ID> --url devnet

# Check starter_program
solana program show <STARTER_PROGRAM_ID> --url devnet
```

---

## Security Considerations

### 1. Program ID Verification

```rust
// Always verify the program ID in CPI
require!(
    ctx.accounts.counter_program.key() == counter_program::ID,
    ErrorCode::InvalidProgram
);
```

### 2. Account Ownership

```rust
// Verify account is owned by expected program
require!(
    ctx.accounts.counter.owner == ctx.accounts.counter_program.key(),
    ErrorCode::InvalidOwner
);
```

### 3. Reentrancy Protection

```rust
// Anchor automatically prevents reentrancy
// Each program can only be invoked once in the call stack
```

### 4. Authority Checks

```rust
// Target program validates authority
// Caller should pass correct authority
pub authority: Signer<'info>,  // Must be counter owner
```

---

## Troubleshooting

### Error: ConstraintSeeds

**Problem:** Seeds validation fails in CPI caller.

**Solution:** Remove seeds constraint in caller, let target program validate:

```rust
// ❌ Wrong
#[account(mut, seeds = [...], bump)]
pub counter: Account<'info, Counter>,

// ✅ Correct
#[account(mut)]
pub counter: Account<'info, Counter>,
```

### Error: InvalidProgram

**Problem:** Wrong program ID passed to CPI.

**Solution:** Use correct program account:

```typescript
counterProgram: counterProgram.programId,  // Not starterProgram.programId!
```

### Error: MissingAccount

**Problem:** Required account not passed to CPI.

**Solution:** Include all accounts needed by target instruction:

```rust
let cpi_accounts = Initialize {
    counter: ctx.accounts.counter.to_account_info(),
    authority: ctx.accounts.authority.to_account_info(),
    system_program: ctx.accounts.system_program.to_account_info(),  // Don't forget!
};
```

### Error: InsufficientFundsForRent

**Problem:** Fee collector account not rent-exempt after receiving payment.

**Solution:** Initialize fee collector with rent-exempt balance first:

```typescript
const feeCollector = anchor.web3.Keypair.generate()

const rentExemptBalance = await provider.connection.getMinimumBalanceForRentExemption(0)
const airdropTx = await provider.connection.requestAirdrop(
  feeCollector.publicKey,
  rentExemptBalance + 1_000_000
)
await provider.connection.confirmTransaction(airdropTx)
```

### Error: PDA Cannot Sign

**Problem:** PDA used as signer but no seeds provided.

**Solution:** Use `CpiContext::new_with_signer` with correct seeds:

```rust
let seeds = &[SEED_TOKEN_VAULT, &[ctx.bumps.pda_vault]];
let signer = &[&seeds[..]];

let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
```

---

## Resources

- [Anchor CPI Documentation](https://book.anchor-lang.com/anchor_in_depth/CPIs.html)
- [Solana CPI Guide](https://solana.com/docs/core/cpi)
- [Counter Program Source](./programs/counter_program/src/lib.rs)
- [Cross-Program Instructions](./programs/starter_program/src/instructions/cross_program.rs)
- [Integration Tests](./tests/cross_program.ts)

---

## Summary

This guide demonstrated:

✅ Creating a CPI-enabled program (counter_program)  
✅ Implementing CPI callers (starter_program)  
✅ Best practices for account validation in CPI  
✅ Testing cross-program interactions  
✅ Common CPI patterns and error handling  
✅ Security considerations for CPI  
✅ Payment functions with SOL transfers  
✅ PDA signing for CPI operations  

**Key Patterns Covered:**
1. **Simple CPI** - Basic cross-program calls
2. **CPI with Parameters** - Passing data between programs
3. **Multiple CPIs** - Multiple calls in one transaction
4. **Payment Functions** - SOL transfers via CPI
5. **PDA Signer** - Program-controlled treasury pattern

**Next Steps:**
- Explore more complex CPI patterns (nested CPIs, program-owned PDAs)
- Implement bi-directional program communication
- Study real-world CPI examples (SPL Token, Metaplex)
