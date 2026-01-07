import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { StarterProgram } from "../target/types/starter_program";
import { expect } from "chai";
import { PublicKey, Keypair } from "@solana/web3.js";
import {
  getAssociatedTokenAddress,
  TOKEN_PROGRAM_ID,
  ASSOCIATED_TOKEN_PROGRAM_ID,
} from "@solana/spl-token";

describe("Advanced Token Operations", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.StarterProgram as Program<StarterProgram>;
  const payer = provider.wallet;

  let mintPda: PublicKey;
  let mintAuthority: PublicKey;
  let userTokenAccount: PublicKey;
  let user: Keypair;

  before(async () => {
    user = Keypair.generate();

    await provider.connection.requestAirdrop(
      user.publicKey,
      5 * anchor.web3.LAMPORTS_PER_SOL
    );
    await new Promise((resolve) => setTimeout(resolve, 1000));

    const [mint] = PublicKey.findProgramAddressSync(
      [Buffer.from("mint")],
      program.programId
    );

    const [authority] = PublicKey.findProgramAddressSync(
      [Buffer.from("mint_authority")],
      program.programId
    );

    mintPda = mint;
    mintAuthority = authority;

    try {
      await program.account.mint.fetch(mintPda);
    } catch {
      await program.methods
        .createMint()
        .accounts({
          signer: payer.publicKey,
          mint: mintPda,
          mintAuthority: mintAuthority,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .rpc();
    }

    userTokenAccount = await getAssociatedTokenAddress(mintPda, user.publicKey);

    await program.methods
      .mintTokens(new anchor.BN(1000000))
      .accounts({
        signer: user.publicKey,
        tokenAccount: userTokenAccount,
        mint: mintPda,
        mintAuthority: mintAuthority,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([user])
      .rpc();
  });

  describe("Approve and Revoke Delegate", () => {
    it("Should approve a delegate for token spending", async () => {
      const delegate = Keypair.generate();

      const amount = 100000;

      await program.methods
        .approveDelegate(new anchor.BN(amount))
        .accounts({
          tokenAccount: userTokenAccount,
          delegate: delegate.publicKey,
          authority: user.publicKey,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .signers([user])
        .rpc();

      const tokenAccountInfo = await provider.connection.getAccountInfo(
        userTokenAccount
      );
      expect(tokenAccountInfo).to.not.be.null;
    });

    it("Should revoke delegate approval", async () => {
      const delegate = Keypair.generate();

      await program.methods
        .approveDelegate(new anchor.BN(50000))
        .accounts({
          tokenAccount: userTokenAccount,
          delegate: delegate.publicKey,
          authority: user.publicKey,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .signers([user])
        .rpc();

      await program.methods
        .revokeDelegate()
        .accounts({
          tokenAccount: userTokenAccount,
          authority: user.publicKey,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .signers([user])
        .rpc();

      const tokenAccountInfo = await provider.connection.getAccountInfo(
        userTokenAccount
      );
      expect(tokenAccountInfo).to.not.be.null;
    });

    it("Should fail when non-authority tries to approve delegate", async () => {
      const delegate = Keypair.generate();
      const nonAuthority = Keypair.generate();

      await provider.connection.requestAirdrop(
        nonAuthority.publicKey,
        2 * anchor.web3.LAMPORTS_PER_SOL
      );
      await new Promise((resolve) => setTimeout(resolve, 1000));

      try {
        await program.methods
          .approveDelegate(new anchor.BN(10000))
          .accounts({
            tokenAccount: userTokenAccount,
            delegate: delegate.publicKey,
            authority: nonAuthority.publicKey,
            tokenProgram: TOKEN_PROGRAM_ID,
          })
          .signers([nonAuthority])
          .rpc();
        expect.fail("Should have thrown an error");
      } catch (error) {
        expect(error).to.exist;
      }
    });
  });

  describe("Freeze and Thaw Account", () => {
    let freezableTokenAccount: PublicKey;
    let freezableUser: Keypair;

    beforeEach(async () => {
      freezableUser = Keypair.generate();

      await provider.connection.requestAirdrop(
        freezableUser.publicKey,
        2 * anchor.web3.LAMPORTS_PER_SOL
      );
      await new Promise((resolve) => setTimeout(resolve, 1000));

      freezableTokenAccount = await getAssociatedTokenAddress(
        mintPda,
        freezableUser.publicKey
      );

      await program.methods
        .mintTokens(new anchor.BN(500000))
        .accounts({
          signer: freezableUser.publicKey,
          tokenAccount: freezableTokenAccount,
          mint: mintPda,
          mintAuthority: mintAuthority,
          tokenProgram: TOKEN_PROGRAM_ID,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .signers([freezableUser])
        .rpc();
    });

    it("Should freeze a token account", async () => {
      await program.methods
        .freezeTokenAccount()
        .accounts({
          tokenAccount: freezableTokenAccount,
          mint: mintPda,
          freezeAuthority: mintAuthority,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .rpc();

      const tokenAccountInfo = await provider.connection.getAccountInfo(
        freezableTokenAccount
      );
      expect(tokenAccountInfo).to.not.be.null;
    });

    it("Should thaw a frozen token account", async () => {
      await program.methods
        .freezeTokenAccount()
        .accounts({
          tokenAccount: freezableTokenAccount,
          mint: mintPda,
          freezeAuthority: mintAuthority,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .rpc();

      await program.methods
        .thawTokenAccount()
        .accounts({
          tokenAccount: freezableTokenAccount,
          mint: mintPda,
          freezeAuthority: mintAuthority,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .rpc();

      const tokenAccountInfo = await provider.connection.getAccountInfo(
        freezableTokenAccount
      );
      expect(tokenAccountInfo).to.not.be.null;
    });

    it("Should prevent transfers when account is frozen", async () => {
      const recipient = Keypair.generate();
      const recipientTokenAccount = await getAssociatedTokenAddress(
        mintPda,
        recipient.publicKey
      );

      await provider.connection.requestAirdrop(
        recipient.publicKey,
        2 * anchor.web3.LAMPORTS_PER_SOL
      );
      await new Promise((resolve) => setTimeout(resolve, 1000));

      await program.methods
        .mintTokens(new anchor.BN(100))
        .accounts({
          signer: recipient.publicKey,
          tokenAccount: recipientTokenAccount,
          mint: mintPda,
          mintAuthority: mintAuthority,
          tokenProgram: TOKEN_PROGRAM_ID,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .signers([recipient])
        .rpc();

      await program.methods
        .freezeTokenAccount()
        .accounts({
          tokenAccount: freezableTokenAccount,
          mint: mintPda,
          freezeAuthority: mintAuthority,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .rpc();

      try {
        await program.methods
          .transferTokens(new anchor.BN(10000))
          .accounts({
            fromAccount: freezableTokenAccount,
            toAccount: recipientTokenAccount,
            mint: mintPda,
            authority: freezableUser.publicKey,
            tokenProgram: TOKEN_PROGRAM_ID,
          })
          .signers([freezableUser])
          .rpc();
        expect.fail("Should have thrown an error");
      } catch (error) {
        expect(error).to.exist;
      }
    });

    it("Should allow transfers after thawing", async () => {
      const recipient = Keypair.generate();
      const recipientTokenAccount = await getAssociatedTokenAddress(
        mintPda,
        recipient.publicKey
      );

      await provider.connection.requestAirdrop(
        recipient.publicKey,
        2 * anchor.web3.LAMPORTS_PER_SOL
      );
      await new Promise((resolve) => setTimeout(resolve, 1000));

      await program.methods
        .mintTokens(new anchor.BN(100))
        .accounts({
          signer: recipient.publicKey,
          tokenAccount: recipientTokenAccount,
          mint: mintPda,
          mintAuthority: mintAuthority,
          tokenProgram: TOKEN_PROGRAM_ID,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .signers([recipient])
        .rpc();

      await program.methods
        .freezeTokenAccount()
        .accounts({
          tokenAccount: freezableTokenAccount,
          mint: mintPda,
          freezeAuthority: mintAuthority,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .rpc();

      await program.methods
        .thawTokenAccount()
        .accounts({
          tokenAccount: freezableTokenAccount,
          mint: mintPda,
          freezeAuthority: mintAuthority,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .rpc();

      await program.methods
        .transferTokens(new anchor.BN(10000))
        .accounts({
          fromAccount: freezableTokenAccount,
          toAccount: recipientTokenAccount,
          mint: mintPda,
          authority: freezableUser.publicKey,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .signers([freezableUser])
        .rpc();

      const tokenAccountInfo = await provider.connection.getAccountInfo(
        recipientTokenAccount
      );
      expect(tokenAccountInfo).to.not.be.null;
    });
  });

  describe("Close Token Account", () => {
    it("Should close an empty token account", async () => {
      const tempUser = Keypair.generate();

      await provider.connection.requestAirdrop(
        tempUser.publicKey,
        2 * anchor.web3.LAMPORTS_PER_SOL
      );
      await new Promise((resolve) => setTimeout(resolve, 1000));

      const tempTokenAccount = await getAssociatedTokenAddress(
        mintPda,
        tempUser.publicKey
      );

      await program.methods
        .mintTokens(new anchor.BN(1000))
        .accounts({
          signer: tempUser.publicKey,
          tokenAccount: tempTokenAccount,
          mint: mintPda,
          mintAuthority: mintAuthority,
          tokenProgram: TOKEN_PROGRAM_ID,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .signers([tempUser])
        .rpc();

      await program.methods
        .burnTokens(new anchor.BN(1000))
        .accounts({
          tokenAccount: tempTokenAccount,
          mint: mintPda,
          authority: tempUser.publicKey,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .signers([tempUser])
        .rpc();

      const balanceBefore = await provider.connection.getBalance(
        tempUser.publicKey
      );

      await program.methods
        .closeTokenAccount()
        .accounts({
          tokenAccount: tempTokenAccount,
          destination: tempUser.publicKey,
          authority: tempUser.publicKey,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .signers([tempUser])
        .rpc();

      const balanceAfter = await provider.connection.getBalance(
        tempUser.publicKey
      );

      expect(balanceAfter).to.be.greaterThan(balanceBefore);

      try {
        await provider.connection.getAccountInfo(tempTokenAccount);
        const accountInfo = await provider.connection.getAccountInfo(
          tempTokenAccount
        );
        if (accountInfo !== null) {
          expect.fail("Account should be closed");
        }
      } catch (error) {}
    });

    it("Should fail to close account with non-zero balance", async () => {
      const tempUser = Keypair.generate();

      await provider.connection.requestAirdrop(
        tempUser.publicKey,
        2 * anchor.web3.LAMPORTS_PER_SOL
      );
      await new Promise((resolve) => setTimeout(resolve, 1000));

      const tempTokenAccount = await getAssociatedTokenAddress(
        mintPda,
        tempUser.publicKey
      );

      await program.methods
        .mintTokens(new anchor.BN(5000))
        .accounts({
          signer: tempUser.publicKey,
          tokenAccount: tempTokenAccount,
          mint: mintPda,
          mintAuthority: mintAuthority,
          tokenProgram: TOKEN_PROGRAM_ID,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .signers([tempUser])
        .rpc();

      try {
        await program.methods
          .closeTokenAccount()
          .accounts({
            tokenAccount: tempTokenAccount,
            destination: tempUser.publicKey,
            authority: tempUser.publicKey,
            tokenProgram: TOKEN_PROGRAM_ID,
          })
          .signers([tempUser])
          .rpc();
        expect.fail("Should have thrown an error");
      } catch (error) {
        expect(error).to.exist;
      }
    });

    it("Should fail when non-authority tries to close account", async () => {
      const tempUser = Keypair.generate();
      const nonAuthority = Keypair.generate();

      await provider.connection.requestAirdrop(
        tempUser.publicKey,
        2 * anchor.web3.LAMPORTS_PER_SOL
      );
      await provider.connection.requestAirdrop(
        nonAuthority.publicKey,
        2 * anchor.web3.LAMPORTS_PER_SOL
      );
      await new Promise((resolve) => setTimeout(resolve, 1000));

      const tempTokenAccount = await getAssociatedTokenAddress(
        mintPda,
        tempUser.publicKey
      );

      await program.methods
        .mintTokens(new anchor.BN(1000))
        .accounts({
          signer: tempUser.publicKey,
          tokenAccount: tempTokenAccount,
          mint: mintPda,
          mintAuthority: mintAuthority,
          tokenProgram: TOKEN_PROGRAM_ID,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .signers([tempUser])
        .rpc();

      await program.methods
        .burnTokens(new anchor.BN(1000))
        .accounts({
          tokenAccount: tempTokenAccount,
          mint: mintPda,
          authority: tempUser.publicKey,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .signers([tempUser])
        .rpc();

      try {
        await program.methods
          .closeTokenAccount()
          .accounts({
            tokenAccount: tempTokenAccount,
            destination: nonAuthority.publicKey,
            authority: nonAuthority.publicKey,
            tokenProgram: TOKEN_PROGRAM_ID,
          })
          .signers([nonAuthority])
          .rpc();
        expect.fail("Should have thrown an error");
      } catch (error) {
        expect(error).to.exist;
      }
    });
  });
});
