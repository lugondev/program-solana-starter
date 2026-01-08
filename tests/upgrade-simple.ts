import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { StarterProgram } from "../target/types/starter_program";
import { expect } from "chai";
import { PublicKey, Keypair, SystemProgram } from "@solana/web3.js";

describe("Program Upgrade System", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.StarterProgram as Program<StarterProgram>;
  const admin = provider.wallet as anchor.Wallet;

  let upgradeAuthorityPda: PublicKey;
  let upgradeAuthorityBump: number;

  before(async () => {
    [upgradeAuthorityPda, upgradeAuthorityBump] =
      PublicKey.findProgramAddressSync(
        [Buffer.from("upgrade_authority")],
        program.programId
      );
  });

  describe("Initialize Upgrade Authority", () => {
    it("Should initialize upgrade authority with voting parameters", async () => {
      const votingThreshold = 67;
      const votingPeriodSeconds = 86400;
      const executionDelaySeconds = 3600;

      await program.methods
        .initializeUpgradeAuthority(
          votingThreshold,
          new anchor.BN(votingPeriodSeconds),
          new anchor.BN(executionDelaySeconds)
        )
        .accounts({
          upgradeAuthority: upgradeAuthorityPda,
          admin: admin.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .rpc();

      const authorityAccount = await program.account.upgradeAuthority.fetch(
        upgradeAuthorityPda
      );

      expect(authorityAccount.authority.toString()).to.equal(
        admin.publicKey.toString()
      );
      expect(authorityAccount.votingThreshold).to.equal(votingThreshold);
      expect(authorityAccount.votingPeriodSeconds.toNumber()).to.equal(
        votingPeriodSeconds
      );
      expect(authorityAccount.executionDelaySeconds.toNumber()).to.equal(
        executionDelaySeconds
      );
      expect(authorityAccount.proposalCount.toNumber()).to.equal(0);
      expect(authorityAccount.isLocked).to.equal(false);
      expect(authorityAccount.pendingAuthority).to.be.null;

      console.log("✅ Upgrade authority initialized successfully");
    });

    it("Should fail with invalid voting threshold", async () => {
      const newAdmin = Keypair.generate();

      const [newAuthorityPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("upgrade_authority"), newAdmin.publicKey.toBuffer()],
        program.programId
      );

      try {
        await program.methods
          .initializeUpgradeAuthority(
            0,
            new anchor.BN(86400),
            new anchor.BN(3600)
          )
          .accounts({
            upgradeAuthority: newAuthorityPda,
            admin: newAdmin.publicKey,
            systemProgram: SystemProgram.programId,
          })
          .signers([newAdmin])
          .rpc();

        expect.fail("Should have thrown an error");
      } catch (error) {
        expect(error.message).to.include("InvalidAmount");
      }
    });
  });

  describe("Authority Transfer", () => {
    let newAuthority: Keypair;

    beforeEach(() => {
      newAuthority = Keypair.generate();
    });

    it("Should initiate authority transfer", async () => {
      await program.methods
        .transferUpgradeAuthority()
        .accounts({
          upgradeAuthority: upgradeAuthorityPda,
          authority: admin.publicKey,
          newAuthority: newAuthority.publicKey,
        })
        .rpc();

      const authorityAccount = await program.account.upgradeAuthority.fetch(
        upgradeAuthorityPda
      );

      expect(authorityAccount.pendingAuthority).to.not.be.null;
      expect(authorityAccount.pendingAuthority.toString()).to.equal(
        newAuthority.publicKey.toString()
      );
      expect(authorityAccount.authority.toString()).to.equal(
        admin.publicKey.toString()
      );

      console.log("✅ Authority transfer initiated");
    });

    it("Should accept pending authority", async () => {
      await program.methods
        .transferUpgradeAuthority()
        .accounts({
          upgradeAuthority: upgradeAuthorityPda,
          authority: admin.publicKey,
          newAuthority: newAuthority.publicKey,
        })
        .rpc();

      await program.methods
        .acceptUpgradeAuthority()
        .accounts({
          upgradeAuthority: upgradeAuthorityPda,
          newAuthority: newAuthority.publicKey,
        })
        .signers([newAuthority])
        .rpc();

      const authorityAccount = await program.account.upgradeAuthority.fetch(
        upgradeAuthorityPda
      );

      expect(authorityAccount.authority.toString()).to.equal(
        newAuthority.publicKey.toString()
      );
      expect(authorityAccount.pendingAuthority).to.be.null;

      console.log("✅ Authority transferred successfully");

      await program.methods
        .transferUpgradeAuthority()
        .accounts({
          upgradeAuthority: upgradeAuthorityPda,
          authority: newAuthority.publicKey,
          newAuthority: admin.publicKey,
        })
        .signers([newAuthority])
        .rpc();

      await program.methods
        .acceptUpgradeAuthority()
        .accounts({
          upgradeAuthority: upgradeAuthorityPda,
          newAuthority: admin.publicKey,
        })
        .rpc();

      console.log("✅ Authority restored to admin");
    });
  });

  describe("Upgrade Proposals", () => {
    let newProgramData: Keypair;
    let proposalPda: PublicKey;
    let proposalId: number;

    beforeEach(async () => {
      newProgramData = Keypair.generate();

      const authorityAccount = await program.account.upgradeAuthority.fetch(
        upgradeAuthorityPda
      );
      proposalId = authorityAccount.proposalCount.toNumber();
      [proposalPda] = PublicKey.findProgramAddressSync(
        [
          Buffer.from("upgrade_proposal"),
          Buffer.from(
            new Uint8Array(new BigUint64Array([BigInt(proposalId)]).buffer)
          ),
        ],
        program.programId
      );
    });

    it("Should create upgrade proposal", async () => {
      const description = "Upgrade to v2.0.0 with new features";

      await program.methods
        .createUpgradeProposal(new anchor.BN(proposalId), description)
        .accounts({
          upgradeAuthority: upgradeAuthorityPda,
          proposal: proposalPda,
          proposer: admin.publicKey,
          newProgramData: newProgramData.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .rpc();

      const proposal = await program.account.upgradeProposal.fetch(proposalPda);

      expect(proposal.proposalId.toNumber()).to.equal(proposalId);
      expect(proposal.proposer.toString()).to.equal(admin.publicKey.toString());
      expect(proposal.newProgramData.toString()).to.equal(
        newProgramData.publicKey.toString()
      );
      expect(proposal.description).to.equal(description);
      expect(proposal.votesFor.toNumber()).to.equal(0);
      expect(proposal.votesAgainst.toNumber()).to.equal(0);
      expect(proposal.status).to.deep.equal({ pending: {} });

      const authorityAccount = await program.account.upgradeAuthority.fetch(
        upgradeAuthorityPda
      );
      expect(authorityAccount.proposalCount.toNumber()).to.equal(
        proposalId + 1
      );

      console.log("✅ Proposal created successfully");
    });

    it("Should fail with description too long", async () => {
      const longDescription = "x".repeat(201);

      try {
        await program.methods
          .createUpgradeProposal(new anchor.BN(proposalId), longDescription)
          .accounts({
            upgradeAuthority: upgradeAuthorityPda,
            proposal: proposalPda,
            proposer: admin.publicKey,
            newProgramData: newProgramData.publicKey,
            systemProgram: SystemProgram.programId,
          })
          .rpc();

        expect.fail("Should have thrown an error");
      } catch (error) {
        expect(error.message).to.include("InvalidAmount");
      }
    });
  });

  describe("Voting", () => {
    let newProgramData: Keypair;
    let proposalPda: PublicKey;
    let proposalId: number;
    let voter1: Keypair;
    let voter2: Keypair;

    beforeEach(async () => {
      newProgramData = Keypair.generate();
      voter1 = Keypair.generate();
      voter2 = Keypair.generate();

      await provider.connection.requestAirdrop(
        voter1.publicKey,
        2 * anchor.web3.LAMPORTS_PER_SOL
      );
      await provider.connection.requestAirdrop(
        voter2.publicKey,
        2 * anchor.web3.LAMPORTS_PER_SOL
      );
      await new Promise((resolve) => setTimeout(resolve, 1000));

      const authorityAccount = await program.account.upgradeAuthority.fetch(
        upgradeAuthorityPda
      );
      proposalId = authorityAccount.proposalCount.toNumber();
      [proposalPda] = PublicKey.findProgramAddressSync(
        [
          Buffer.from("upgrade_proposal"),
          Buffer.from(
            new Uint8Array(new BigUint64Array([BigInt(proposalId)]).buffer)
          ),
        ],
        program.programId
      );

      await program.methods
        .createUpgradeProposal(
          new anchor.BN(proposalId),
          "Test proposal for voting"
        )
        .accounts({
          upgradeAuthority: upgradeAuthorityPda,
          proposal: proposalPda,
          proposer: admin.publicKey,
          newProgramData: newProgramData.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .rpc();
    });

    it("Should cast vote in favor", async () => {
      const [votePda] = PublicKey.findProgramAddressSync(
        [
          Buffer.from("vote"),
          Buffer.from(
            new Uint8Array(new BigUint64Array([BigInt(proposalId)]).buffer)
          ),
          voter1.publicKey.toBuffer(),
        ],
        program.programId
      );

      const votingPower = 100;

      await program.methods
        .castVote(new anchor.BN(proposalId), true, new anchor.BN(votingPower))
        .accounts({
          upgradeAuthority: upgradeAuthorityPda,
          proposal: proposalPda,
          vote: votePda,
          voter: voter1.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([voter1])
        .rpc();

      const vote = await program.account.vote.fetch(votePda);
      expect(vote.voter.toString()).to.equal(voter1.publicKey.toString());
      expect(vote.proposalId.toNumber()).to.equal(proposalId);
      expect(vote.inFavor).to.equal(true);
      expect(vote.votingPower.toNumber()).to.equal(votingPower);

      const proposal = await program.account.upgradeProposal.fetch(proposalPda);
      expect(proposal.votesFor.toNumber()).to.equal(votingPower);
      expect(proposal.votesAgainst.toNumber()).to.equal(0);

      console.log("✅ Vote cast successfully");
    });

    it("Should cast vote against", async () => {
      const [votePda] = PublicKey.findProgramAddressSync(
        [
          Buffer.from("vote"),
          Buffer.from(
            new Uint8Array(new BigUint64Array([BigInt(proposalId)]).buffer)
          ),
          voter1.publicKey.toBuffer(),
        ],
        program.programId
      );

      const votingPower = 50;

      await program.methods
        .castVote(new anchor.BN(proposalId), false, new anchor.BN(votingPower))
        .accounts({
          upgradeAuthority: upgradeAuthorityPda,
          proposal: proposalPda,
          vote: votePda,
          voter: voter1.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([voter1])
        .rpc();

      const proposal = await program.account.upgradeProposal.fetch(proposalPda);
      expect(proposal.votesFor.toNumber()).to.equal(0);
      expect(proposal.votesAgainst.toNumber()).to.equal(votingPower);

      console.log("✅ Vote against cast successfully");
    });

    it("Should handle multiple votes", async () => {
      const [vote1Pda] = PublicKey.findProgramAddressSync(
        [
          Buffer.from("vote"),
          Buffer.from(
            new Uint8Array(new BigUint64Array([BigInt(proposalId)]).buffer)
          ),
          voter1.publicKey.toBuffer(),
        ],
        program.programId
      );

      const [vote2Pda] = PublicKey.findProgramAddressSync(
        [
          Buffer.from("vote"),
          Buffer.from(
            new Uint8Array(new BigUint64Array([BigInt(proposalId)]).buffer)
          ),
          voter2.publicKey.toBuffer(),
        ],
        program.programId
      );

      await program.methods
        .castVote(new anchor.BN(proposalId), true, new anchor.BN(70))
        .accounts({
          upgradeAuthority: upgradeAuthorityPda,
          proposal: proposalPda,
          vote: vote1Pda,
          voter: voter1.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([voter1])
        .rpc();

      await program.methods
        .castVote(new anchor.BN(proposalId), false, new anchor.BN(30))
        .accounts({
          upgradeAuthority: upgradeAuthorityPda,
          proposal: proposalPda,
          vote: vote2Pda,
          voter: voter2.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([voter2])
        .rpc();

      const proposal = await program.account.upgradeProposal.fetch(proposalPda);
      expect(proposal.votesFor.toNumber()).to.equal(70);
      expect(proposal.votesAgainst.toNumber()).to.equal(30);

      console.log("✅ Multiple votes handled correctly");
    });
  });

  describe("Proposal Cancellation", () => {
    let newProgramData: Keypair;
    let proposalPda: PublicKey;
    let proposalId: number;

    beforeEach(async () => {
      newProgramData = Keypair.generate();

      const authorityAccount = await program.account.upgradeAuthority.fetch(
        upgradeAuthorityPda
      );
      proposalId = authorityAccount.proposalCount.toNumber();
      [proposalPda] = PublicKey.findProgramAddressSync(
        [
          Buffer.from("upgrade_proposal"),
          Buffer.from(
            new Uint8Array(new BigUint64Array([BigInt(proposalId)]).buffer)
          ),
        ],
        program.programId
      );

      await program.methods
        .createUpgradeProposal(new anchor.BN(proposalId), "Proposal to cancel")
        .accounts({
          upgradeAuthority: upgradeAuthorityPda,
          proposal: proposalPda,
          proposer: admin.publicKey,
          newProgramData: newProgramData.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .rpc();
    });

    it("Should cancel pending proposal", async () => {
      await program.methods
        .cancelProposal(new anchor.BN(proposalId))
        .accounts({
          upgradeAuthority: upgradeAuthorityPda,
          proposal: proposalPda,
          authority: admin.publicKey,
        })
        .rpc();

      const proposal = await program.account.upgradeProposal.fetch(proposalPda);
      expect(proposal.status).to.deep.equal({ cancelled: {} });

      console.log("✅ Proposal cancelled successfully");
    });

    it("Should fail to cancel if not authority", async () => {
      const notAuthority = Keypair.generate();
      await provider.connection.requestAirdrop(
        notAuthority.publicKey,
        2 * anchor.web3.LAMPORTS_PER_SOL
      );
      await new Promise((resolve) => setTimeout(resolve, 1000));

      try {
        await program.methods
          .cancelProposal(new anchor.BN(proposalId))
          .accounts({
            upgradeAuthority: upgradeAuthorityPda,
            proposal: proposalPda,
            authority: notAuthority.publicKey,
          })
          .signers([notAuthority])
          .rpc();

        expect.fail("Should have thrown an error");
      } catch (error) {
        expect(error.message).to.include("Unauthorized");
      }
    });
  });
});
