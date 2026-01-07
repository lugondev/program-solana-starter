import * as anchor from '@coral-xyz/anchor';
import {Program} from '@coral-xyz/anchor';
import {StarterProgram} from '../target/types/starter_program';
import {CounterProgram} from '../target/types/counter_program';
import {PublicKey, SystemProgram} from '@solana/web3.js';
import {expect} from 'chai';

describe('Cross-Program Interaction', () => {
	const provider = anchor.AnchorProvider.env();
	anchor.setProvider(provider);

	const starterProgram = anchor.workspace.StarterProgram as Program<StarterProgram>;
	const counterProgram = anchor.workspace.CounterProgram as Program<CounterProgram>;
	const user = provider.wallet as anchor.Wallet;

	let counterPda: PublicKey;

	before(async () => {
		[counterPda] = PublicKey.findProgramAddressSync([Buffer.from('counter'), user.publicKey.toBuffer()], counterProgram.programId);
	});

	describe('Direct Counter Program Operations', () => {
		it('Should initialize counter directly', async () => {
			await counterProgram.methods
				.initialize()
				.accounts({
					counter: counterPda,
					authority: user.publicKey,
					systemProgram: SystemProgram.programId,
				})
				.rpc();

			const counter = await counterProgram.account.counter.fetch(counterPda);
			expect(counter.authority.toString()).to.equal(user.publicKey.toString());
			expect(counter.count.toNumber()).to.equal(0);
		});

		it('Should increment counter directly', async () => {
			await counterProgram.methods
				.increment()
				.accounts({
					counter: counterPda,
				})
				.rpc();

			const counter = await counterProgram.account.counter.fetch(counterPda);
			expect(counter.count.toNumber()).to.equal(1);
		});

		it('Should add value to counter directly', async () => {
			await counterProgram.methods
				.add(new anchor.BN(5))
				.accounts({
					counter: counterPda,
				})
				.rpc();

			const counter = await counterProgram.account.counter.fetch(counterPda);
			expect(counter.count.toNumber()).to.equal(6);
		});

		it('Should reset counter', async () => {
			await counterProgram.methods
				.reset()
				.accounts({
					counter: counterPda,
					authority: user.publicKey,
				})
				.rpc();

			const counter = await counterProgram.account.counter.fetch(counterPda);
			expect(counter.count.toNumber()).to.equal(0);
		});
	});

	describe('Cross-Program Invocation via Starter Program', () => {
		it('Should increment counter via CPI from starter program', async () => {
			const beforeCounter = await counterProgram.account.counter.fetch(counterPda);

			await starterProgram.methods
				.incrementCounter()
				.accounts({
					counter: counterPda,
					authority: user.publicKey,
					counterProgram: counterProgram.programId,
				})
				.rpc();

			const afterCounter = await counterProgram.account.counter.fetch(counterPda);
			expect(afterCounter.count.toNumber()).to.equal(beforeCounter.count.toNumber() + 1);
		});

		it('Should add value to counter via CPI', async () => {
			const beforeCounter = await counterProgram.account.counter.fetch(counterPda);

			await starterProgram.methods
				.addToCounter(new anchor.BN(10))
				.accounts({
					counter: counterPda,
					authority: user.publicKey,
					counterProgram: counterProgram.programId,
				})
				.rpc();

			const afterCounter = await counterProgram.account.counter.fetch(counterPda);
			expect(afterCounter.count.toNumber()).to.equal(beforeCounter.count.toNumber() + 10);
		});

		it('Should increment multiple times via CPI', async () => {
			const beforeCounter = await counterProgram.account.counter.fetch(counterPda);

			const times = 3;
			await starterProgram.methods
				.incrementMultiple(times)
				.accounts({
					counter: counterPda,
					authority: user.publicKey,
					counterProgram: counterProgram.programId,
				})
				.rpc();

			const afterCounter = await counterProgram.account.counter.fetch(counterPda);
			expect(afterCounter.count.toNumber()).to.equal(beforeCounter.count.toNumber() + times);
		});

		it('Should handle arithmetic overflow', async () => {
			await counterProgram.methods
				.reset()
				.accounts({
					counter: counterPda,
					authority: user.publicKey,
				})
				.rpc();

			const maxValue = new anchor.BN('18446744073709551615');

			await counterProgram.methods
				.add(maxValue)
				.accounts({
					counter: counterPda,
				})
				.rpc();

			try {
				await starterProgram.methods
					.addToCounter(new anchor.BN(1))
					.accounts({
						counter: counterPda,
						authority: user.publicKey,
						counterProgram: counterProgram.programId,
					})
					.rpc();
				expect.fail('Should have thrown overflow error');
			} catch (error) {
				expect(error.message).to.include('Overflow');
			}
		});
	});

	describe('Complex Cross-Program Scenarios', () => {
		beforeEach(async () => {
			await counterProgram.methods
				.reset()
				.accounts({
					counter: counterPda,
					authority: user.publicKey,
				})
				.rpc();
		});

		it('Should chain multiple CPI operations', async () => {
			await starterProgram.methods
				.incrementCounter()
				.accounts({
					counter: counterPda,
					authority: user.publicKey,
					counterProgram: counterProgram.programId,
				})
				.rpc();

			await starterProgram.methods
				.addToCounter(new anchor.BN(5))
				.accounts({
					counter: counterPda,
					authority: user.publicKey,
					counterProgram: counterProgram.programId,
				})
				.rpc();

			await starterProgram.methods
				.incrementMultiple(3)
				.accounts({
					counter: counterPda,
					authority: user.publicKey,
					counterProgram: counterProgram.programId,
				})
				.rpc();

			const counter = await counterProgram.account.counter.fetch(counterPda);
			expect(counter.count.toNumber()).to.equal(9);
		});

		it('Should read counter state after CPI', async () => {
			await starterProgram.methods
				.addToCounter(new anchor.BN(42))
				.accounts({
					counter: counterPda,
					authority: user.publicKey,
					counterProgram: counterProgram.programId,
				})
				.rpc();

			const counter = await counterProgram.account.counter.fetch(counterPda);
			expect(counter.count.toNumber()).to.equal(42);
			expect(counter.authority.toString()).to.equal(user.publicKey.toString());
		});

		it('Should verify counter state consistency across programs', async () => {
			const initialCounter = await counterProgram.account.counter.fetch(counterPda);

			await counterProgram.methods
				.increment()
				.accounts({
					counter: counterPda,
				})
				.rpc();

			await starterProgram.methods
				.incrementCounter()
				.accounts({
					counter: counterPda,
					authority: user.publicKey,
					counterProgram: counterProgram.programId,
				})
				.rpc();

			const finalCounter = await counterProgram.account.counter.fetch(counterPda);
			expect(finalCounter.count.toNumber()).to.equal(initialCounter.count.toNumber() + 2);
		});
	});

	describe('Payment Function with PDA Signer', () => {
		let feeCollector: anchor.web3.Keypair;
		let pdaVault: anchor.web3.PublicKey;

		before(async () => {
			feeCollector = anchor.web3.Keypair.generate();

			const rentExemptBalance = await provider.connection.getMinimumBalanceForRentExemption(0);
			const airdropFee = await provider.connection.requestAirdrop(
				feeCollector.publicKey,
				rentExemptBalance + 1_000_000
			);
			await provider.connection.confirmTransaction(airdropFee);

			;[pdaVault] = anchor.web3.PublicKey.findProgramAddressSync(
				[Buffer.from('token_vault')],
				starterProgram.programId
			);

			const airdropTx = await provider.connection.requestAirdrop(pdaVault, 10_000_000);
			await provider.connection.confirmTransaction(airdropTx);

			await counterProgram.methods
				.reset()
				.accounts({
					counter: counterPda,
					authority: user.publicKey,
				})
				.rpc();
		});

		beforeEach(async () => {
			const userBalance = await provider.connection.getBalance(user.publicKey);
			if (userBalance < 10_000_000) {
				const airdropTx = await provider.connection.requestAirdrop(user.publicKey, 10_000_000);
				await provider.connection.confirmTransaction(airdropTx);
			}
		});

		it('Should increment with payment directly', async () => {
			const payment = new anchor.BN(50_000);
			const beforeCount = await counterProgram.account.counter.fetch(counterPda);
			const beforeFeeBalance = await provider.connection.getBalance(feeCollector.publicKey);

			await counterProgram.methods
				.incrementWithPayment(payment)
				.accounts({
					counter: counterPda,
					payer: user.publicKey,
					feeCollector: feeCollector.publicKey,
					systemProgram: anchor.web3.SystemProgram.programId,
				})
				.rpc({skipPreflight: true});

			await new Promise(resolve => setTimeout(resolve, 1000));

			const afterCount = await counterProgram.account.counter.fetch(counterPda);
			const afterFeeBalance = await provider.connection.getBalance(feeCollector.publicKey);

			expect(afterCount.count.toNumber()).to.equal(beforeCount.count.toNumber() + 1);
			expect(afterFeeBalance).to.equal(beforeFeeBalance + payment.toNumber());
		});

		it('Should increment with payment from PDA via CPI', async () => {
			const payment = new anchor.BN(100_000);
			const beforeCount = await counterProgram.account.counter.fetch(counterPda);
			const beforePdaBalance = await provider.connection.getBalance(pdaVault);
			const beforeFeeBalance = await provider.connection.getBalance(feeCollector.publicKey);

			await starterProgram.methods
				.incrementWithPaymentFromPda(payment)
				.accounts({
					counter: counterPda,
					pdaVault: pdaVault,
					feeCollector: feeCollector.publicKey,
					counterProgram: counterProgram.programId,
					systemProgram: anchor.web3.SystemProgram.programId,
				})
				.rpc({skipPreflight: true});

			await new Promise(resolve => setTimeout(resolve, 1000));

			const afterCount = await counterProgram.account.counter.fetch(counterPda);
			const afterPdaBalance = await provider.connection.getBalance(pdaVault);
			const afterFeeBalance = await provider.connection.getBalance(feeCollector.publicKey);

			expect(afterCount.count.toNumber()).to.equal(beforeCount.count.toNumber() + 1);
			expect(afterPdaBalance).to.be.lessThan(beforePdaBalance);
			expect(afterFeeBalance).to.equal(beforeFeeBalance + payment.toNumber());
		});

		it('Should fail with insufficient PDA balance', async () => {
			const excessivePayment = new anchor.BN(100_000_000);

			try {
				await starterProgram.methods
					.incrementWithPaymentFromPda(excessivePayment)
					.accounts({
						counter: counterPda,
						pdaVault: pdaVault,
						feeCollector: feeCollector.publicKey,
						counterProgram: counterProgram.programId,
						systemProgram: anchor.web3.SystemProgram.programId,
					})
					.rpc();
				expect.fail('Should have failed with insufficient funds');
			} catch (error) {
				expect(error.message).to.match(/insufficient/i);
			}
		});
	});
});
