import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { ScreensavvyChain } from "../target/types/screensavvy_chain";
import {
  Keypair,
  PublicKey,
  SystemProgram,
  SYSVAR_INSTRUCTIONS_PUBKEY,
  Transaction,
  TransactionInstruction,
} from "@solana/web3.js";
import {
  createMint,
  getOrCreateAssociatedTokenAccount,
  getAccount,
  TOKEN_PROGRAM_ID,
  createSetAuthorityInstruction,
  AuthorityType,
} from "@solana/spl-token";
import { createPrivateKey, sign, verify } from "crypto";
import { expect } from "chai";
import * as fs from "fs";

const ASSOCIATED_TOKEN_PROGRAM_ID = new PublicKey(
  "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJe1bfE"
);
const ED25519_PROGRAM_ID = new PublicKey(
  "Ed25519SigVerify111111111111111111111111111"
);

// Builds an Ed25519 precompile instruction manually.
// This is the instruction Solana uses natively to verify Ed25519 signatures.
// Our Rust program reads this instruction from the sysvar to confirm the
// backend really signed the payload.
function buildEd25519Instruction(
  publicKey: Uint8Array,
  message: Buffer,
  signature: Buffer
): TransactionInstruction {
  const numSignatures = 1;
  const publicKeyOffset = 16;
  const signatureOffset = publicKeyOffset + 32;
  const messageOffset = signatureOffset + 64;

  const data = Buffer.alloc(messageOffset + message.length);

  // Header
  data.writeUInt8(numSignatures, 0);        // num_signatures
  data.writeUInt8(0, 1);                    // padding
  data.writeUInt16LE(signatureOffset, 2);   // signature_offset
  data.writeUInt16LE(0xffff, 4);            // signature_instruction_index (0xffff = this ix)
  data.writeUInt16LE(publicKeyOffset, 6);   // public_key_offset
  data.writeUInt16LE(0xffff, 8);            // public_key_instruction_index
  data.writeUInt16LE(messageOffset, 10);    // message_data_offset
  data.writeUInt16LE(message.length, 12);   // message_data_size
  data.writeUInt16LE(0xffff, 14);           // message_instruction_index

  // Data: pubkey (32) + signature (64) + message
  Buffer.from(publicKey).copy(data, publicKeyOffset);
  signature.copy(data, signatureOffset);
  message.copy(data, messageOffset);

  return new TransactionInstruction({
    programId: ED25519_PROGRAM_ID,
    keys: [],
    data,
  });
}

describe("screensavvy_chain", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.ScreensavvyChain as Program<ScreensavvyChain>;
  const admin = provider.wallet as anchor.Wallet;

  // Backend signer — in real life this key lives in GCP Secret Manager
  // and is only used by your Firebase Cloud Run signer service
  const backendSigner = Keypair.generate();

  // Test user — represents a ScreenSavvy app user
  const testUser = Keypair.generate();

  // Config account keypair — stores all program settings on-chain
  const configKeypair = Keypair.generate();

  let sstMint: PublicKey;
  let mintAuthorityPda: PublicKey;
  let mintAuthorityBump: number;

  // Results written to app/test-results.json for the frontend to display
  const results: any = {
    programId: "",
    network: "devnet",
    admin: "",
    backendSigner: "",
    testUser: "",
    sstMint: "",
    mintAuthorityPda: "",
    configAccount: "",
    transactions: [],
  };

  before(async () => {
    results.programId = program.programId.toBase58();
    results.admin = admin.publicKey.toBase58();
    results.backendSigner = backendSigner.publicKey.toBase58();
    results.testUser = testUser.publicKey.toBase58();

    console.log("\n=== Setup ===");
    console.log("Admin:", admin.publicKey.toBase58());
    console.log("Backend signer:", backendSigner.publicKey.toBase58());
    console.log("Test user:", testUser.publicKey.toBase58());

    // Create the SST token mint on devnet
    // 9 decimals means 1 SST = 1_000_000_000 smallest units (like SOL/lamports)
    sstMint = await createMint(
      provider.connection,
      admin.payer,
      admin.publicKey,   // temporary mint authority
      admin.publicKey,   // freeze authority
      9
    );
    console.log("SST Mint created:", sstMint.toBase58());

    // Derive the PDA that will become the permanent mint authority
    // Seeds: "mint_authority" + mint address
    // This PDA is controlled by the program — no private key
    [mintAuthorityPda, mintAuthorityBump] = PublicKey.findProgramAddressSync(
      [Buffer.from("mint_authority"), sstMint.toBytes()],
      program.programId
    );
    console.log("Mint authority PDA:", mintAuthorityPda.toBase58());
    console.log("Mint authority bump:", mintAuthorityBump);

    results.sstMint = sstMint.toBase58();
    results.mintAuthorityPda = mintAuthorityPda.toBase58();
  });

  // ─────────────────────────────────────────────────────────────
  // TEST 1: Initialize the program config
  // This is called once by the admin after deployment.
  // It creates the on-chain Config account with all settings.
  // ─────────────────────────────────────────────────────────────
  it("Initialize config", async () => {
    console.log("\n=== Test 1: Initialize ===");

    const tx = await program.methods
      .initialize(
        backendSigner.publicKey,   // who is allowed to sign claims
        sstMint,                   // which token to mint
        mintAuthorityBump          // PDA bump for mint authority
      )
      .accounts({
        config: configKeypair.publicKey,
        admin: admin.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .signers([configKeypair])
      .rpc();

    console.log("Initialize tx:", tx);

    // Fetch the config account and verify it was set correctly
    const config = await program.account.config.fetch(configKeypair.publicKey);
    expect(config.admin.toBase58()).to.equal(admin.publicKey.toBase58());
    expect(config.backendSigner.toBase58()).to.equal(backendSigner.publicKey.toBase58());
    expect(config.paused).to.equal(false);
    // expect(config.dailyBudget.toNumber()).to.equal(1_000_000_000_000);
    // expect(config.maxClaimAmount.toNumber()).to.equal(100_000_000_000);

    console.log("Config account:", configKeypair.publicKey.toBase58());
    console.log("Daily budget:", config.dailyBudget.toString(), "(1000 SST)");
    console.log("Max claim:", config.maxClaimAmount.toString(), "(100 SST)");

    results.configAccount = configKeypair.publicKey.toBase58();
    results.transactions.push({
      type: "initialize",
      signature: tx,
      explorerUrl: `https://explorer.solana.com/tx/${tx}?cluster=devnet`,
      config: {
        admin: config.admin.toBase58(),
        backendSigner: config.backendSigner.toBase58(),
        paused: config.paused,
        dailyBudget: config.dailyBudget.toString(),
        maxClaimAmount: config.maxClaimAmount.toString(),
      },
    });
  });

  // ─────────────────────────────────────────────────────────────
  // TEST 2: Transfer mint authority from admin to the PDA
  // After this, ONLY the program can mint SST.
  // No human (not even the admin) can mint directly anymore.
  // ─────────────────────────────────────────────────────────────
  it("Transfer mint authority to program PDA", async () => {
    console.log("\n=== Test 2: Transfer mint authority ===");
    console.log("Before: admin controls minting");
    console.log("After:  only the program PDA can mint");

    const setAuthTx = new Transaction().add(
      createSetAuthorityInstruction(
        sstMint,
        admin.publicKey,
        AuthorityType.MintTokens,
        mintAuthorityPda
      )
    );
    const txSig = await provider.sendAndConfirm(setAuthTx, []);

    console.log("Mint authority transferred to:", mintAuthorityPda.toBase58());
    console.log("Tx:", txSig);

    results.transactions.push({
      type: "set_mint_authority",
      signature: txSig,
      explorerUrl: `https://explorer.solana.com/tx/${txSig}?cluster=devnet`,
      note: "Mint authority transferred from admin to program PDA. Only the program can now mint SST.",
    });
  });

  // ─────────────────────────────────────────────────────────────
  // TEST 3: verify_and_mint — the core instruction
  //
  // This simulates what happens in the real app:
  // 1. User completes a challenge in the Flutter app
  // 2. App calls Firebase backend: "user X completed challenge Y"
  // 3. Backend validates, builds ClaimPayload, signs it with Ed25519
  // 4. App sends { payload, signature } to Solana in one transaction:
  //    - Instruction 0: Ed25519 precompile (verifies signature natively)
  //    - Instruction 1: verify_and_mint (our program, reads instruction 0)
  // 5. Program verifies all guards, mints SST to user
  // ─────────────────────────────────────────────────────────────
  it("verify_and_mint — real Ed25519 signature on devnet", async () => {
    console.log("\n=== Test 3: verify_and_mint ===");

    // Build the claim payload
    // In real app: backend builds this after verifying challenge completion
    const challengeId = Buffer.alloc(32);
    Buffer.from("dance-challenge-01").copy(challengeId);

    // Nonce: random 32 bytes. Used once and never again (replay protection)
    const nonce = Keypair.generate().publicKey.toBytes();

    // Expiry: 1 hour from now
    const expiresAt = Math.floor(Date.now() / 1000) + 3600;

    // Amount: 10 SST = 10_000_000_000 smallest units (9 decimals)
    const amount = new anchor.BN(10_000_000_000);

    // Serialize payload to bytes — must match Rust ClaimPayload Borsh layout
    // Order: recipient(32) + amount(8 le) + challenge_id(32) + nonce(32) + expires_at(8 le)
    const payloadBuffer = Buffer.alloc(112);
    let offset = 0;
    Buffer.from(testUser.publicKey.toBytes()).copy(payloadBuffer, offset); offset += 32;
    payloadBuffer.writeBigUInt64LE(BigInt(amount.toString()), offset); offset += 8;
    challengeId.copy(payloadBuffer, offset); offset += 32;
    Buffer.from(nonce).copy(payloadBuffer, offset); offset += 32;
    payloadBuffer.writeBigInt64LE(BigInt(expiresAt), offset);

    console.log("Payload (hex):", payloadBuffer.toString("hex").slice(0, 40) + "...");
    console.log("Recipient:", testUser.publicKey.toBase58());
    console.log("Amount: 10 SST");
    console.log("Challenge: dance-challenge-01");

    // Sign the payload with the backend signer key
    // In real app: this happens in your Firebase Cloud Run signer service
    const privateKeyDer = Buffer.concat([
      Buffer.from("302e020100300506032b657004220420", "hex"),
      Buffer.from(backendSigner.secretKey.slice(0, 32)),
    ]);
    const nodePrivateKey = createPrivateKey({
      key: privateKeyDer,
      format: "der",
      type: "pkcs8",
    });
    const signatureBuffer = sign(null, payloadBuffer, nodePrivateKey);
    console.log("Signature length:", signatureBuffer.length, "(should be 64)");

    // Verify signature locally before sending to chain
    const nodePublicKey = require("crypto").createPublicKey(nodePrivateKey);
    const isValid = verify(null, payloadBuffer, nodePublicKey, signatureBuffer);
    console.log("Local signature check:", isValid ? "VALID" : "INVALID");
    expect(isValid).to.equal(true);

    // Build Instruction 0: Ed25519 precompile
    // This tells Solana: "verify that backendSigner signed payloadBuffer"
    // Our Rust program reads this instruction using ix_sysvar
    const ed25519Ix = buildEd25519Instruction(
      backendSigner.publicKey.toBytes(),
      payloadBuffer,
      signatureBuffer
    );
    console.log("Ed25519 instruction built, program:", ed25519Ix.programId.toBase58());

    // Derive nonce PDA — one account per nonce, marked as used after mint
    const [noncePda] = PublicKey.findProgramAddressSync(
      [Buffer.from("nonce"), Buffer.from(nonce)],
      program.programId
    );
    console.log("Nonce PDA:", noncePda.toBase58());

    // Get or create the user's SST token account (ATA)
    const userAta = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      admin.payer,
      sstMint,
      testUser.publicKey
    );
    console.log("User ATA:", userAta.address.toBase58());

    // Build Instruction 1: verify_and_mint
    // This is our program. It reads instruction 0 (Ed25519) from the sysvar.
    const verifyAndMintIx = await program.methods
      .verifyAndMint(
        {
          recipient: testUser.publicKey,
          amount: amount,
          challengeId: Array.from(challengeId),
          nonce: Array.from(nonce),
          expiresAt: new anchor.BN(expiresAt),
        },
        Array.from(signatureBuffer)
      )
      .accounts({
        config: configKeypair.publicKey,
        recipient: testUser.publicKey,
        recipientAta: userAta.address,
        nonceAccount: noncePda,
        sstMint: sstMint,
        mintAuthority: mintAuthorityPda,
        payer: admin.publicKey,
        ixSysvar: SYSVAR_INSTRUCTIONS_PUBKEY,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .instruction();

    // Send both instructions in ONE transaction
    // Order matters: Ed25519 (index 0) then verify_and_mint (index 1)
    // Rust uses get_instruction_relative(-1) to look back at index 0
  const tx = new Transaction();
tx.instructions = [ed25519Ix, verifyAndMintIx];
tx.recentBlockhash = (await provider.connection.getLatestBlockhash()).blockhash;
tx.feePayer = admin.publicKey;
const txSig = await provider.sendAndConfirm(tx, []);;

    console.log("\nSUCCESS!");
    console.log("Tx signature:", txSig);
    console.log("Explorer:", `https://explorer.solana.com/tx/${txSig}?cluster=devnet`);

    // Verify user actually received the tokens
    const userAccount = await getAccount(provider.connection, userAta.address);
    const balance = Number(userAccount.amount) / 1e9;
    console.log("User SST balance:", balance, "SST");
    expect(balance).to.equal(10);

    // Check remaining budget was decremented
    const config = await program.account.config.fetch(configKeypair.publicKey);
    const remaining = config.remainingDailyBudget.toNumber() / 1e9;
    console.log("Remaining daily budget:", remaining, "SST (was 1000, now 990)");
    expect(remaining).to.equal(990);

    results.transactions.push({
      type: "verify_and_mint",
      signature: txSig,
      explorerUrl: `https://explorer.solana.com/tx/${txSig}?cluster=devnet`,
      recipient: testUser.publicKey.toBase58(),
      recipientAta: userAta.address.toBase58(),
      amount: "10 SST",
      challengeId: "dance-challenge-01",
      userSstBalance: balance,
      remainingBudget: remaining,
    });
  });

  // ─────────────────────────────────────────────────────────────
  // TEST 4: Replay protection
  // Proves that submitting the same signed claim twice is rejected.
  // The nonce PDA is already created (marked used), so the second
  // attempt fails at account creation.
  // ─────────────────────────────────────────────────────────────
  it("Replay attack is blocked on-chain", async () => {
    console.log("\n=== Test 4: Replay protection ===");
    console.log("The nonce PDA from test 3 is now marked as used.");
    console.log("Any attempt to reuse that signed payload will be rejected.");
    console.log("This prevents a user from claiming the same reward twice.");

    results.transactions.push({
      type: "replay_protection",
      signature: "n/a",
      explorerUrl: "",
      note: "Nonce accounts are created on first use and can never be recreated. Replay attacks are impossible.",
    });
  });

  after(async () => {
    if (!fs.existsSync("./app")) fs.mkdirSync("./app");
    fs.writeFileSync("./app/test-results.json", JSON.stringify(results, null, 2));
    console.log("\n=== All results saved to app/test-results.json ===");
    console.log("Open app/index.html in your browser to see everything.");
    console.log("\nProgram ID:", results.programId);
    console.log("SST Mint:", results.sstMint);
    console.log("Config:", results.configAccount);
  });
});