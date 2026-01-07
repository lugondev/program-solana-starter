import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { StarterProgram } from "../target/types/starter_program";
import { expect } from "chai";
import { PublicKey, Keypair } from "@solana/web3.js";

describe("Role-Based Access Control (RBAC)", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.StarterProgram as Program<StarterProgram>;
  const admin = provider.wallet;

  let programConfig: PublicKey;
  let feeDestination: Keypair;

  before(async () => {
    feeDestination = Keypair.generate();

    const [configPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("program_config")],
      program.programId
    );

    programConfig = configPda;

    try {
      await program.account.programConfig.fetch(configPda);
    } catch {
      await program.methods
        .initializeConfig(feeDestination.publicKey)
        .accounts({
          programConfig: configPda,
          authority: admin.publicKey,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .rpc();
    }
  });

  describe("Role Assignment", () => {
    it("Should assign Admin role to a user", async () => {
      const user = Keypair.generate();

      await provider.connection.requestAirdrop(
        user.publicKey,
        2 * anchor.web3.LAMPORTS_PER_SOL
      );
      await new Promise((resolve) => setTimeout(resolve, 1000));

      const [rolePda] = PublicKey.findProgramAddressSync(
        [Buffer.from("role"), user.publicKey.toBuffer()],
        program.programId
      );

      const tx = await program.methods
        .assignRole({ admin: {} })
        .accounts({
          role: rolePda,
          programConfig: programConfig,
          admin: admin.publicKey,
          targetAuthority: user.publicKey,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .rpc();

      const role = await program.account.role.fetch(rolePda);

      expect(role.authority.toString()).to.equal(user.publicKey.toString());
      expect(role.roleType).to.deep.equal({ admin: {} });
      expect(role.permissions).to.equal(0xff);
      expect(role.assignedBy.toString()).to.equal(admin.publicKey.toString());
    });

    it("Should assign Moderator role to a user", async () => {
      const user = Keypair.generate();

      await provider.connection.requestAirdrop(
        user.publicKey,
        2 * anchor.web3.LAMPORTS_PER_SOL
      );
      await new Promise((resolve) => setTimeout(resolve, 1000));

      const [rolePda] = PublicKey.findProgramAddressSync(
        [Buffer.from("role"), user.publicKey.toBuffer()],
        program.programId
      );

      await program.methods
        .assignRole({ moderator: {} })
        .accounts({
          role: rolePda,
          programConfig: programConfig,
          admin: admin.publicKey,
          targetAuthority: user.publicKey,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .rpc();

      const role = await program.account.role.fetch(rolePda);

      expect(role.authority.toString()).to.equal(user.publicKey.toString());
      expect(role.roleType).to.deep.equal({ moderator: {} });
      expect(role.permissions).to.equal(0x06);
    });

    it("Should assign User role with no permissions", async () => {
      const user = Keypair.generate();

      await provider.connection.requestAirdrop(
        user.publicKey,
        2 * anchor.web3.LAMPORTS_PER_SOL
      );
      await new Promise((resolve) => setTimeout(resolve, 1000));

      const [rolePda] = PublicKey.findProgramAddressSync(
        [Buffer.from("role"), user.publicKey.toBuffer()],
        program.programId
      );

      await program.methods
        .assignRole({ user: {} })
        .accounts({
          role: rolePda,
          programConfig: programConfig,
          admin: admin.publicKey,
          targetAuthority: user.publicKey,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .rpc();

      const role = await program.account.role.fetch(rolePda);

      expect(role.authority.toString()).to.equal(user.publicKey.toString());
      expect(role.roleType).to.deep.equal({ user: {} });
      expect(role.permissions).to.equal(0x00);
    });

    it("Should fail when non-admin tries to assign role", async () => {
      const nonAdmin = Keypair.generate();
      const targetUser = Keypair.generate();

      await provider.connection.requestAirdrop(
        nonAdmin.publicKey,
        2 * anchor.web3.LAMPORTS_PER_SOL
      );
      await new Promise((resolve) => setTimeout(resolve, 1000));

      const [rolePda] = PublicKey.findProgramAddressSync(
        [Buffer.from("role"), targetUser.publicKey.toBuffer()],
        program.programId
      );

      try {
        await program.methods
          .assignRole({ user: {} })
          .accounts({
            role: rolePda,
            programConfig: programConfig,
            admin: nonAdmin.publicKey,
            targetAuthority: targetUser.publicKey,
            systemProgram: anchor.web3.SystemProgram.programId,
          })
          .signers([nonAdmin])
          .rpc();
        expect.fail("Should have thrown an error");
      } catch (error) {
        expect(error.message).to.include("0x7d3");
      }
    });

    it("Should fail when role already exists", async () => {
      const user = Keypair.generate();

      await provider.connection.requestAirdrop(
        user.publicKey,
        2 * anchor.web3.LAMPORTS_PER_SOL
      );
      await new Promise((resolve) => setTimeout(resolve, 1000));

      const [rolePda] = PublicKey.findProgramAddressSync(
        [Buffer.from("role"), user.publicKey.toBuffer()],
        program.programId
      );

      await program.methods
        .assignRole({ user: {} })
        .accounts({
          role: rolePda,
          programConfig: programConfig,
          admin: admin.publicKey,
          targetAuthority: user.publicKey,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .rpc();

      try {
        await program.methods
          .assignRole({ admin: {} })
          .accounts({
            role: rolePda,
            programConfig: programConfig,
            admin: admin.publicKey,
            targetAuthority: user.publicKey,
            systemProgram: anchor.web3.SystemProgram.programId,
          })
          .rpc();
        expect.fail("Should have thrown an error");
      } catch (error) {
        expect(error.message).to.include("0x0");
      }
    });
  });

  describe("Permission Management", () => {
    let testUser: Keypair;
    let testRolePda: PublicKey;

    beforeEach(async () => {
      testUser = Keypair.generate();

      await provider.connection.requestAirdrop(
        testUser.publicKey,
        2 * anchor.web3.LAMPORTS_PER_SOL
      );
      await new Promise((resolve) => setTimeout(resolve, 1000));

      const [rolePda] = PublicKey.findProgramAddressSync(
        [Buffer.from("role"), testUser.publicKey.toBuffer()],
        program.programId
      );

      testRolePda = rolePda;

      await program.methods
        .assignRole({ user: {} })
        .accounts({
          role: rolePda,
          programConfig: programConfig,
          admin: admin.publicKey,
          targetAuthority: testUser.publicKey,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .rpc();
    });

    it("Should add permissions to existing role", async () => {
      const MANAGE_CONFIG = 1 << 0;
      const MANAGE_USERS = 1 << 1;

      await program.methods
        .updateRolePermissions(MANAGE_CONFIG | MANAGE_USERS, 0)
        .accounts({
          role: testRolePda,
          programConfig: programConfig,
          admin: admin.publicKey,
        })
        .rpc();

      const role = await program.account.role.fetch(testRolePda);

      expect(role.permissions).to.equal(MANAGE_CONFIG | MANAGE_USERS);
    });

    it("Should remove permissions from existing role", async () => {
      const MANAGE_CONFIG = 1 << 0;
      const MANAGE_USERS = 1 << 1;
      const MANAGE_TOKENS = 1 << 2;

      await program.methods
        .updateRolePermissions(MANAGE_CONFIG | MANAGE_USERS | MANAGE_TOKENS, 0)
        .accounts({
          role: testRolePda,
          programConfig: programConfig,
          admin: admin.publicKey,
        })
        .rpc();

      await program.methods
        .updateRolePermissions(0, MANAGE_USERS)
        .accounts({
          role: testRolePda,
          programConfig: programConfig,
          admin: admin.publicKey,
        })
        .rpc();

      const role = await program.account.role.fetch(testRolePda);

      expect(role.permissions).to.equal(MANAGE_CONFIG | MANAGE_TOKENS);
    });

    it("Should add and remove permissions in one call", async () => {
      const MANAGE_CONFIG = 1 << 0;
      const MANAGE_USERS = 1 << 1;
      const MANAGE_TOKENS = 1 << 2;

      await program.methods
        .updateRolePermissions(MANAGE_USERS, 0)
        .accounts({
          role: testRolePda,
          programConfig: programConfig,
          admin: admin.publicKey,
        })
        .rpc();

      await program.methods
        .updateRolePermissions(MANAGE_CONFIG | MANAGE_TOKENS, MANAGE_USERS)
        .accounts({
          role: testRolePda,
          programConfig: programConfig,
          admin: admin.publicKey,
        })
        .rpc();

      const role = await program.account.role.fetch(testRolePda);

      expect(role.permissions).to.equal(MANAGE_CONFIG | MANAGE_TOKENS);
    });
  });

  describe("Permission Checking", () => {
    let testUser: Keypair;
    let testRolePda: PublicKey;

    before(async () => {
      testUser = Keypair.generate();

      await provider.connection.requestAirdrop(
        testUser.publicKey,
        2 * anchor.web3.LAMPORTS_PER_SOL
      );
      await new Promise((resolve) => setTimeout(resolve, 1000));

      const [rolePda] = PublicKey.findProgramAddressSync(
        [Buffer.from("role"), testUser.publicKey.toBuffer()],
        program.programId
      );

      testRolePda = rolePda;

      await program.methods
        .assignRole({ moderator: {} })
        .accounts({
          role: rolePda,
          programConfig: programConfig,
          admin: admin.publicKey,
          targetAuthority: testUser.publicKey,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .rpc();
    });

    it("Should return true for permissions the role has", async () => {
      const MANAGE_USERS = 1 << 1;

      const result = await program.methods
        .checkPermission(MANAGE_USERS)
        .accounts({
          role: testRolePda,
          authority: testUser.publicKey,
        })
        .signers([testUser])
        .view();

      expect(result).to.be.true;
    });

    it("Should return false for permissions the role does not have", async () => {
      const PAUSE_PROGRAM = 1 << 3;

      const result = await program.methods
        .checkPermission(PAUSE_PROGRAM)
        .accounts({
          role: testRolePda,
          authority: testUser.publicKey,
        })
        .signers([testUser])
        .view();

      expect(result).to.be.false;
    });

    it("Should fail when wrong authority tries to check", async () => {
      const wrongUser = Keypair.generate();
      await provider.connection.requestAirdrop(
        wrongUser.publicKey,
        2 * anchor.web3.LAMPORTS_PER_SOL
      );
      await new Promise((resolve) => setTimeout(resolve, 1000));

      const MANAGE_USERS = 1 << 1;

      try {
        await program.methods
          .checkPermission(MANAGE_USERS)
          .accounts({
            role: testRolePda,
            authority: wrongUser.publicKey,
          })
          .signers([wrongUser])
          .view();
        expect.fail("Should have thrown an error");
      } catch (error) {
        expect(error.message).to.include("Unauthorized");
      }
    });
  });

  describe("Role Revocation", () => {
    it("Should revoke a role and close the account", async () => {
      const user = Keypair.generate();

      await provider.connection.requestAirdrop(
        user.publicKey,
        2 * anchor.web3.LAMPORTS_PER_SOL
      );
      await new Promise((resolve) => setTimeout(resolve, 1000));

      const [rolePda] = PublicKey.findProgramAddressSync(
        [Buffer.from("role"), user.publicKey.toBuffer()],
        program.programId
      );

      await program.methods
        .assignRole({ user: {} })
        .accounts({
          role: rolePda,
          programConfig: programConfig,
          admin: admin.publicKey,
          targetAuthority: user.publicKey,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .rpc();

      const adminBalanceBefore = await provider.connection.getBalance(
        admin.publicKey
      );

      await program.methods
        .revokeRole()
        .accounts({
          role: rolePda,
          programConfig: programConfig,
          admin: admin.publicKey,
        })
        .rpc();

      const adminBalanceAfter = await provider.connection.getBalance(
        admin.publicKey
      );

      try {
        await program.account.role.fetch(rolePda);
        expect.fail("Account should be closed");
      } catch (error) {
        expect(error.message).to.include("Account does not exist");
      }

      expect(adminBalanceAfter).to.be.greaterThan(adminBalanceBefore);
    });

    it("Should fail when non-admin tries to revoke role", async () => {
      const user = Keypair.generate();
      const nonAdmin = Keypair.generate();

      await provider.connection.requestAirdrop(
        user.publicKey,
        2 * anchor.web3.LAMPORTS_PER_SOL
      );
      await provider.connection.requestAirdrop(
        nonAdmin.publicKey,
        2 * anchor.web3.LAMPORTS_PER_SOL
      );
      await new Promise((resolve) => setTimeout(resolve, 1000));

      const [rolePda] = PublicKey.findProgramAddressSync(
        [Buffer.from("role"), user.publicKey.toBuffer()],
        program.programId
      );

      await program.methods
        .assignRole({ user: {} })
        .accounts({
          role: rolePda,
          programConfig: programConfig,
          admin: admin.publicKey,
          targetAuthority: user.publicKey,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .rpc();

      try {
        await program.methods
          .revokeRole()
          .accounts({
            role: rolePda,
            programConfig: programConfig,
            admin: nonAdmin.publicKey,
          })
          .signers([nonAdmin])
          .rpc();
        expect.fail("Should have thrown an error");
      } catch (error) {
        expect(error.message).to.include("0x7d3");
      }
    });
  });

  describe("Events", () => {
    it("Should emit RoleAssignedEvent when role is assigned", async () => {
      const user = Keypair.generate();

      await provider.connection.requestAirdrop(
        user.publicKey,
        2 * anchor.web3.LAMPORTS_PER_SOL
      );
      await new Promise((resolve) => setTimeout(resolve, 1000));

      const [rolePda] = PublicKey.findProgramAddressSync(
        [Buffer.from("role"), user.publicKey.toBuffer()],
        program.programId
      );

      const listener = program.addEventListener(
        "RoleAssignedEvent",
        (event) => {
          expect(event.authority.toString()).to.equal(
            user.publicKey.toString()
          );
          expect(event.roleType).to.deep.equal({ admin: {} });
          expect(event.assignedBy.toString()).to.equal(
            admin.publicKey.toString()
          );
        }
      );

      await program.methods
        .assignRole({ admin: {} })
        .accounts({
          role: rolePda,
          programConfig: programConfig,
          admin: admin.publicKey,
          targetAuthority: user.publicKey,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .rpc();

      await new Promise((resolve) => setTimeout(resolve, 1000));

      await program.removeEventListener(listener);
    });
  });
});
