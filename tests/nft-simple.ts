import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { StarterProgram } from "../target/types/starter_program";
import {
  Keypair,
  LAMPORTS_PER_SOL,
  PublicKey,
  SystemProgram,
} from "@solana/web3.js";
import {
  TOKEN_PROGRAM_ID,
  ASSOCIATED_TOKEN_PROGRAM_ID,
  getAssociatedTokenAddress,
  createMint,
} from "@solana/spl-token";
import { expect } from "chai";

describe("NFT System - Basic Tests", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.StarterProgram as Program<StarterProgram>;

  it("Should create NFT collection", async () => {
    const authority = Keypair.generate();
    const collectionMint = Keypair.generate();

    const airdropSig = await provider.connection.requestAirdrop(
      authority.publicKey,
      5 * LAMPORTS_PER_SOL
    );
    await provider.connection.confirmTransaction(airdropSig);

    await createMint(
      provider.connection,
      authority,
      authority.publicKey,
      authority.publicKey,
      0,
      collectionMint
    );

    const [collectionPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("nft_collection"), collectionMint.publicKey.toBuffer()],
      program.programId
    );

    await program.methods
      .createCollection(
        "Test Collection",
        "TEST",
        "https://example.com/collection.json",
        500,
        new anchor.BN(1000),
        true
      )
      .accounts({
        collection: collectionPda,
        collectionMint: collectionMint.publicKey,
        authority: authority.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .signers([authority])
      .rpc();

    const collection = await program.account.nftCollection.fetch(collectionPda);
    expect(collection.authority.toString()).to.equal(
      authority.publicKey.toString()
    );
    expect(collection.name).to.equal("Test Collection");
    expect(collection.symbol).to.equal("TEST");
    expect(collection.sellerFeeBasisPoints).to.equal(500);
    expect(collection.totalSupply.toNumber()).to.equal(1000);
    expect(collection.mintedCount.toNumber()).to.equal(0);
    expect(collection.isMutable).to.be.true;
  });

  it("Should mint NFT", async () => {
    const authority = Keypair.generate();
    const collectionMint = Keypair.generate();
    const nftMint = Keypair.generate();

    const airdropSig = await provider.connection.requestAirdrop(
      authority.publicKey,
      5 * LAMPORTS_PER_SOL
    );
    await provider.connection.confirmTransaction(airdropSig);

    await createMint(
      provider.connection,
      authority,
      authority.publicKey,
      authority.publicKey,
      0,
      collectionMint
    );

    const [collectionPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("nft_collection"), collectionMint.publicKey.toBuffer()],
      program.programId
    );

    await program.methods
      .createCollection(
        "NFT Collection",
        "NFT",
        "https://example.com/nft-collection.json",
        500,
        new anchor.BN(100),
        true
      )
      .accounts({
        collection: collectionPda,
        collectionMint: collectionMint.publicKey,
        authority: authority.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .signers([authority])
      .rpc();

    await createMint(
      provider.connection,
      authority,
      authority.publicKey,
      authority.publicKey,
      0,
      nftMint
    );

    const [nftMetadataPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("nft_metadata"), nftMint.publicKey.toBuffer()],
      program.programId
    );

    const recipientTokenAccount = await getAssociatedTokenAddress(
      nftMint.publicKey,
      authority.publicKey
    );

    const creators = [
      {
        address: authority.publicKey,
        verified: true,
        share: 100,
      },
    ];

    await program.methods
      .mintNft("My NFT #1", "https://example.com/nft1.json", creators)
      .accounts({
        collection: collectionPda,
        nftMetadata: nftMetadataPda,
        nftMint: nftMint.publicKey,
        recipientTokenAccount: recipientTokenAccount,
        recipient: authority.publicKey,
        authority: authority.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
      })
      .signers([authority])
      .rpc();

    const nftMetadata = await program.account.nftMetadata.fetch(nftMetadataPda);
    expect(nftMetadata.name).to.equal("My NFT #1");
    expect(nftMetadata.owner.toString()).to.equal(
      authority.publicKey.toString()
    );

    const collection = await program.account.nftCollection.fetch(collectionPda);
    expect(collection.mintedCount.toNumber()).to.equal(1);
  });
});
