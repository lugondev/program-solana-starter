import * as anchor from '@coral-xyz/anchor';
import {Program} from '@coral-xyz/anchor';
import {StarterProgram} from '../target/types/starter_program';
import {PublicKey, Keypair, SystemProgram} from '@solana/web3.js';
import {
	TOKEN_PROGRAM_ID,
	ASSOCIATED_TOKEN_PROGRAM_ID,
	getAssociatedTokenAddress,
	createMint,
	getAccount,
} from '@solana/spl-token';
import {expect} from 'chai';

describe('starter_program', () => {
	const provider = anchor.AnchorProvider.env();
	anchor.setProvider(provider);

	const program = anchor.workspace.StarterProgram as Program<StarterProgram>;
	const user = provider.wallet as anchor.Wallet;

	describe('Initialize', () => {
		it('Should initialize successfully', async () => {
			const tx = await program.methods.initialize().rpc();
			console.log('Initialize tx:', tx);
		});
	});

	describe('Program Configuration', () => {
		const feeDestination = Keypair.generate().publicKey;

		it('Should initialize program config', async () => {
			const [configPda] = PublicKey.findProgramAddressSync([Buffer.from('program_config')], program.programId);

			await program.methods
				.initializeConfig(feeDestination)
				.accounts({
					programConfig: configPda,
					authority: user.publicKey,
					systemProgram: SystemProgram.programId,
				})
				.rpc();

			const config = await program.account.programConfig.fetch(configPda);
			expect(config.admin.toString()).to.equal(user.publicKey.toString());
			expect(config.feeDestination.toString()).to.equal(feeDestination.toString());
			expect(config.feeBasisPoints.toNumber()).to.equal(100);
			expect(config.paused).to.equal(false);
		});

		it('Should update program config', async () => {
			const [configPda] = PublicKey.findProgramAddressSync([Buffer.from('program_config')], program.programId);

			const newFeeDestination = Keypair.generate().publicKey;

			await program.methods
				.updateConfig(user.publicKey, newFeeDestination, new anchor.BN(200))
				.accounts({
					programConfig: configPda,
					admin: user.publicKey,
				})
				.rpc();

			const config = await program.account.programConfig.fetch(configPda);
			expect(config.admin.toString()).to.equal(user.publicKey.toString());
			expect(config.feeBasisPoints.toNumber()).to.equal(200);
		});

		it('Should toggle pause status', async () => {
			const [configPda] = PublicKey.findProgramAddressSync([Buffer.from('program_config')], program.programId);

			const configBefore = await program.account.programConfig.fetch(configPda);
			const pausedBefore = configBefore.paused;

			await program.methods
				.togglePause()
				.accounts({
					programConfig: configPda,
					admin: user.publicKey,
				})
				.rpc();

			const configAfter = await program.account.programConfig.fetch(configPda);
			expect(configAfter.paused).to.equal(!pausedBefore);
		});
	});

	describe('User Account (PDA)', () => {
		it('Should create user account', async () => {
			const [userPda] = PublicKey.findProgramAddressSync([Buffer.from('user_account'), user.publicKey.toBuffer()], program.programId);

			await program.methods
				.createUserAccount()
				.accounts({
					userAccount: userPda,
					authority: user.publicKey,
					systemProgram: SystemProgram.programId,
				})
				.rpc();

			const userAccount = await program.account.userAccount.fetch(userPda);
			expect(userAccount.authority.toString()).to.equal(user.publicKey.toString());
			expect(userAccount.points.toNumber()).to.equal(0);
		});

		it('Should update user account', async () => {
			const [userPda] = PublicKey.findProgramAddressSync([Buffer.from('user_account'), user.publicKey.toBuffer()], program.programId);

			await program.methods
				.updateUserAccount(new anchor.BN(100))
				.accounts({
					userAccount: userPda,
					authority: user.publicKey,
				})
				.rpc();

			const userAccount = await program.account.userAccount.fetch(userPda);
			expect(userAccount.points.toNumber()).to.equal(100);
		});

		it('Should close user account', async () => {
			const [userPda] = PublicKey.findProgramAddressSync([Buffer.from('user_account'), user.publicKey.toBuffer()], program.programId);

			await program.methods
				.closeUserAccount()
				.accounts({
					userAccount: userPda,
					authority: user.publicKey,
				})
				.rpc();

			try {
				await program.account.userAccount.fetch(userPda);
				expect.fail('Account should be closed');
			} catch (error) {
				expect(error.message).to.include('Account does not exist');
			}
		});
	});

	describe('SPL Token Operations', () => {
		it('Should create mint with PDA authority', async () => {
			const [mintPda] = PublicKey.findProgramAddressSync([Buffer.from('mint')], program.programId);

			const [mintAuthority] = PublicKey.findProgramAddressSync([Buffer.from('mint_authority')], program.programId);

			await program.methods
				.createMint()
				.accounts({
					signer: user.publicKey,
					mint: mintPda,
					mintAuthority: mintAuthority,
					tokenProgram: TOKEN_PROGRAM_ID,
					systemProgram: SystemProgram.programId,
				})
				.rpc();

			console.log('Mint created:', mintPda.toString());
		});

		it('Should mint tokens to user', async () => {
			const [mintPda] = PublicKey.findProgramAddressSync([Buffer.from('mint')], program.programId);

			const [mintAuthority] = PublicKey.findProgramAddressSync([Buffer.from('mint_authority')], program.programId);

			const userTokenAccount = await getAssociatedTokenAddress(mintPda, user.publicKey);

			const amount = new anchor.BN(1000000);

			await program.methods
				.mintTokens(amount)
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

			const tokenAccountInfo = await getAccount(provider.connection, userTokenAccount);
			expect(tokenAccountInfo.amount.toString()).to.equal(amount.toString());
		});

		it('Should transfer tokens between accounts', async () => {
			const [mintPda] = PublicKey.findProgramAddressSync([Buffer.from('mint')], program.programId);

			const recipient = Keypair.generate();
			const fromTokenAccount = await getAssociatedTokenAddress(mintPda, user.publicKey);
			const toTokenAccount = await getAssociatedTokenAddress(mintPda, recipient.publicKey);

			const airdropTx = await provider.connection.requestAirdrop(recipient.publicKey, 1000000000);
			await provider.connection.confirmTransaction(airdropTx);

			await program.methods
				.mintTokens(new anchor.BN(1000000))
				.accounts({
					signer: recipient.publicKey,
					tokenAccount: toTokenAccount,
					mint: mintPda,
					mintAuthority: PublicKey.findProgramAddressSync([Buffer.from('mint_authority')], program.programId)[0],
					tokenProgram: TOKEN_PROGRAM_ID,
					associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
					systemProgram: SystemProgram.programId,
				})
				.signers([recipient])
				.rpc();

			const amount = new anchor.BN(500000);

			await program.methods
				.transferTokens(amount)
				.accounts({
					fromAccount: fromTokenAccount,
					toAccount: toTokenAccount,
					mint: mintPda,
					authority: user.publicKey,
					tokenProgram: TOKEN_PROGRAM_ID,
				})
				.rpc();

			const toAccountInfo = await getAccount(provider.connection, toTokenAccount);
			expect(Number(toAccountInfo.amount)).to.be.greaterThanOrEqual(amount.toNumber());
		});

		it('Should burn tokens', async () => {
			const [mintPda] = PublicKey.findProgramAddressSync([Buffer.from('mint')], program.programId);

			const userTokenAccount = await getAssociatedTokenAddress(mintPda, user.publicKey);

			const beforeBalance = await getAccount(provider.connection, userTokenAccount);

			const burnAmount = new anchor.BN(100000);

			await program.methods
				.burnTokens(burnAmount)
				.accounts({
					tokenAccount: userTokenAccount,
					mint: mintPda,
					authority: user.publicKey,
					tokenProgram: TOKEN_PROGRAM_ID,
				})
				.rpc();

			const afterBalance = await getAccount(provider.connection, userTokenAccount);
			expect(Number(beforeBalance.amount) - Number(afterBalance.amount)).to.equal(burnAmount.toNumber());
		});
	});

	describe('Cross-Program Invocation (CPI)', () => {
		it('Should transfer SOL', async () => {
			const recipient = Keypair.generate();

			const beforeBalance = await provider.connection.getBalance(recipient.publicKey);

			const amount = new anchor.BN(1000000);

			await program.methods
				.transferSol(amount)
				.accounts({
					from: user.publicKey,
					to: recipient.publicKey,
					systemProgram: SystemProgram.programId,
				})
				.rpc();

			const afterBalance = await provider.connection.getBalance(recipient.publicKey);
			expect(afterBalance - beforeBalance).to.equal(amount.toNumber());
		});

		it('Should transfer SOL with PDA', async () => {
			const [vaultPda] = PublicKey.findProgramAddressSync([Buffer.from('token_vault')], program.programId);

			const rentExemptBalance = await provider.connection.getMinimumBalanceForRentExemption(0);
			const airdropAmount = rentExemptBalance + 10000000;
			const airdropTx = await provider.connection.requestAirdrop(vaultPda, airdropAmount);
			await provider.connection.confirmTransaction(airdropTx);

			const recipient = Keypair.generate();
			
			const recipientAirdrop = await provider.connection.requestAirdrop(recipient.publicKey, rentExemptBalance);
			await provider.connection.confirmTransaction(recipientAirdrop);
			
			const beforeBalance = await provider.connection.getBalance(recipient.publicKey);

			const amount = new anchor.BN(500000);

			await program.methods
				.transferSolWithPda(amount)
				.accounts({
					vault: vaultPda,
					recipient: recipient.publicKey,
					systemProgram: SystemProgram.programId,
				})
				.rpc();

			const afterBalance = await provider.connection.getBalance(recipient.publicKey);
			expect(afterBalance - beforeBalance).to.equal(amount.toNumber());
		});
	});
});
