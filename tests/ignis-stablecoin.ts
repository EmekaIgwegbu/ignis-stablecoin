import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { IgnisStablecoin } from "../target/types/ignis_stablecoin";
import { Keypair, PublicKey, SendTransactionError } from '@solana/web3.js'
import { readFileSync } from 'fs';

import { burn, createAccount, getAssociatedTokenAddress, getAccount, getMint, getOrCreateAssociatedTokenAccount, TOKEN_PROGRAM_ID, ASSOCIATED_TOKEN_PROGRAM_ID } from '@solana/spl-token';
import { assert } from "chai";


describe("ignis-stablecoin", function () {

  // Configure the client to use the local (in-memory) cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.IgnisStablecoin as Program<IgnisStablecoin>;
  const provider = anchor.getProvider() as anchor.AnchorProvider;
  const wallet = provider.wallet;
  const walletKeypair = loadKeypairFromSeedFile('/Users/emekaigwegbu/.config/solana/id.json');
  const connection = provider.connection;

  // Program accounts
  let ignisStablecoinPDA: PublicKey;
  let venturaCoinPDA: PublicKey;
  let ignisMintPDA: PublicKey;
  let venturaMintPDA: PublicKey;
  let ignisReservePubKey: PublicKey;
  let venturaReservePubKey: PublicKey;
  let signingPDA: PublicKey;

  // User accounts
  let user: Keypair;
  let userIgnisATAPubKey: PublicKey;
  let userVenturaATAPubKey: PublicKey;

  before("Generate keypairs and PDAs", async function () {
    // Assign wallets
    user = Keypair.generate();

    // Derive PDAs
    [ignisStablecoinPDA] = await PublicKey.findProgramAddress(
      [Buffer.from("ignis_stablecoin")],
      program.programId
    );
    [venturaCoinPDA] = await PublicKey.findProgramAddress(
      [Buffer.from("ventura_coin")],
      program.programId
    );
    [ignisMintPDA] = await PublicKey.findProgramAddress(
      [Buffer.from("ignis_mint")],
      program.programId
    );
    [venturaMintPDA] = await PublicKey.findProgramAddress(
      [Buffer.from("ventura_mint")],
      program.programId
    );
    [signingPDA] = await PublicKey.findProgramAddress(
      [],
      program.programId
    );

    // Get associated token account addresses
    ignisReservePubKey = await getAssociatedTokenAddress(ignisMintPDA, signingPDA, true);
    venturaReservePubKey = await getAssociatedTokenAddress(venturaMintPDA, signingPDA, true);
  });

  it("Initialise instruction", async function () {
    try {
      await program.methods
        .initialise()
        .accounts({
          ignisStablecoin: ignisStablecoinPDA,
          venturaCoin: venturaCoinPDA,
          ignisMint: ignisMintPDA,
          venturaMint: venturaMintPDA,
          ignisReserve: ignisReservePubKey,
          venturaReserve: venturaReservePubKey,
          reserveWallet: wallet.publicKey,
          signingPda: signingPDA,
          tokenProgram: TOKEN_PROGRAM_ID,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .rpc();
    }
    catch (error) {
      if (error instanceof SendTransactionError) {
        const logs = await error.getLogs(connection);
        console.error('Transaction failed with logs:', logs);
      } else {
        console.error('Transaction failed:', error);
      }
      throw error;
    }

    const ignisStablecoinAccount = await program.account.ignisStablecoin.fetch(ignisStablecoinPDA);
    const venturaCoinAccount = await program.account.venturaCoin.fetch(venturaCoinPDA);
    const ignisMintAccount = await getMint(connection, ignisMintPDA);
    const venturaMintAccount = await getMint(connection, venturaMintPDA);
    const ignisReserveAccount = await getAccount(connection, ignisReservePubKey);
    const venturaReserveAccount = await getAccount(connection, venturaReservePubKey);

    // Assertions on ignisStablecoin properties
    assert.equal(ignisStablecoinAccount.peg, 1.0);
    assert.isTrue(ignisStablecoinAccount.mint.equals(ignisMintPDA));
    assert.isTrue(ignisStablecoinAccount.ignisReserve.equals(ignisReservePubKey));
    assert.isTrue(ignisStablecoinAccount.reserveWallet.equals(wallet.publicKey));

    // Assertions on venturaCoin properties
    assert.isTrue(venturaCoinAccount.mint.equals(venturaMintPDA));
    assert.isTrue(venturaCoinAccount.venturaReserve.equals(venturaReservePubKey));
    assert.isTrue(venturaCoinAccount.reserveWallet.equals(wallet.publicKey));

    // Assertions on ignis mint account properties
    assert.isTrue(ignisMintAccount.isInitialized);
    assert.equal(ignisMintAccount.decimals, 6);
    assert.isTrue(ignisMintAccount.mintAuthority.equals(signingPDA));
    assert.isTrue(ignisMintAccount.freezeAuthority.equals(signingPDA));
    assert.equal(ignisMintAccount.supply, BigInt(0));

    // Assertions on ventura mint account properties
    assert.isTrue(venturaMintAccount.isInitialized);
    assert.equal(venturaMintAccount.decimals, 6);
    assert.isTrue(venturaMintAccount.mintAuthority.equals(signingPDA));
    assert.isTrue(venturaMintAccount.freezeAuthority.equals(signingPDA));
    assert.equal(venturaMintAccount.supply, BigInt(0));

    // Assertions on ignis reserve account properties
    assert.isTrue(ignisReserveAccount.isInitialized);
    assert.isTrue(ignisReserveAccount.mint.equals(ignisMintPDA));
    assert.isTrue(ignisReserveAccount.owner.equals(signingPDA));
    assert.equal(ignisReserveAccount.amount, BigInt(0));

    // Assertions on ventura reserve account properties
    assert.isTrue(venturaReserveAccount.isInitialized);
    assert.isTrue(venturaReserveAccount.mint.equals(venturaMintPDA));
    assert.isTrue(venturaReserveAccount.owner.equals(signingPDA));
    assert.equal(venturaReserveAccount.amount, BigInt(0));

  });

  describe("ignis instructions", function () {
    before("Create user token accounts", async function () {
      userIgnisATAPubKey = await createAccount(connection, walletKeypair, ignisMintPDA, user.publicKey);
      userVenturaATAPubKey = await createAccount(connection, walletKeypair, venturaMintPDA, user.publicKey);
    })

    after("Reset token balances to 0", async function () {
      const ignisReserve = await getAccount(connection, ignisReservePubKey);
      const ignisReserveAmount = ignisReserve.amount.toString();
      const venturaReserve = await getAccount(connection, venturaReservePubKey);
      const venturaReserveAmount = venturaReserve.amount.toString();
      const userIgnisATA = await getOrCreateAssociatedTokenAccount(connection, walletKeypair, ignisMintPDA, user.publicKey);
      const userVenturaATA = await getOrCreateAssociatedTokenAccount(connection, walletKeypair, venturaMintPDA, user.publicKey);
      const userIgnisAmount = userIgnisATA.amount;
      const userVenturaAmount = userVenturaATA.amount;

      // Burn all reserve ignis
      await program.methods
        .burnReserveIgnis(new anchor.BN(ignisReserveAmount))
        .accounts({
          ignisStableCoin: ignisStablecoinPDA,
          ignisMint: ignisMintPDA,
          ignisReserve: ignisReservePubKey,
          signingPda: signingPDA,
          reserveWallet: wallet.publicKey,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .rpc();

      // Burn all reserve ventura
      await program.methods
        .burnReserveVentura(new anchor.BN(venturaReserveAmount))
        .accounts({
          venturaCoin: venturaCoinPDA,
          venturaMintPDA: venturaMintPDA,
          venturaReserve: venturaReservePubKey,
          signingPda: signingPDA,
          reserveWallet: wallet.publicKey,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .rpc();

      // Burn all user ignis and ventura
      await burn(connection, walletKeypair, userIgnisATA.address, ignisMintPDA, user, userIgnisAmount);
      await burn(connection, walletKeypair, userVenturaATA.address, venturaMintPDA, user, userVenturaAmount);
    })

    describe('Mint ignis to user then redeem it', async function () {
      const ignisAmount = 2;

      it('Mint ignis to user', async function () {
        await program.methods
          .mintIgnisTo(new anchor.BN(ignisAmount))
          .accounts({
            ignisStableCoin: ignisStablecoinPDA,
            to: userIgnisATAPubKey,
            ignisMint: ignisMintPDA,
            signingPda: signingPDA,
            reserveWallet: wallet.publicKey,
            tokenProgram: TOKEN_PROGRAM_ID,
          })
          .rpc();

        const ignisMintAccount = await getMint(connection, ignisMintPDA);
        const userIgnisATA = await getAccount(connection, userIgnisATAPubKey);

        // Assertions on ignis and user ignis ATA properties
        assert.equal(ignisMintAccount.supply, BigInt(ignisAmount));
        assert.equal(userIgnisATA.amount, BigInt(ignisAmount));
      });

      it('Redeem user ignis', async function () {
        await program.methods
          .redeemIgnis(new anchor.BN(ignisAmount))
          .accounts({
            ignisStablecoin: ignisStablecoinPDA,
            venturaCoin: venturaCoinPDA,
            userIgnisAta: userIgnisATAPubKey,
            userVenturaAta: userVenturaATAPubKey,
            ignisMint: ignisMintPDA,
            venturaMint: venturaMintPDA,
            signingPda: signingPDA,
            user: user.publicKey,
            tokenProgram: TOKEN_PROGRAM_ID,
          })
          .signers([user])
          .rpc();

        const venturaCoinAccount = await program.account.venturaCoin.fetch(venturaCoinPDA);
        const ignisMintAccount = await getMint(connection, ignisMintPDA);
        const venturaMintAccount = await getMint(connection, venturaMintPDA);
        const userIgnisATA = await getAccount(connection, userIgnisATAPubKey);
        const userVenturaATA = await getAccount(connection, userVenturaATAPubKey);

        // Assertions on ignis and user ignis ATA properties
        assert.equal(ignisMintAccount.supply, BigInt(0));
        assert.equal(userIgnisATA.amount, BigInt(0));

        // Assertions on ventura and user ventura ATA properties
        assert.isTrue(venturaMintAccount.supply > 0);
        assert.isTrue(userVenturaATA.amount > 0);
      })
    })

    describe("Mint ignis to the reserve then burn it", function () {
      const ignisAmount = 2;

      it('Mint ignis to the reserve', async function () {
        await program.methods
          .mintIgnisTo(new anchor.BN(ignisAmount))
          .accounts({
            ignisStableCoin: ignisStablecoinPDA,
            to: ignisReservePubKey,
            ignisMint: ignisMintPDA,
            signingPda: signingPDA,
            reserveWallet: wallet.publicKey,
            tokenProgram: TOKEN_PROGRAM_ID,
          })
          .rpc();

        const ignisMintAccount = await getMint(connection, ignisMintPDA);
        const ignisReserve = await getAccount(connection, ignisReservePubKey);

        // Assertions on ignis properties and ignis reserve properties
        assert.equal(ignisMintAccount.supply, BigInt(ignisAmount));
        assert.equal(ignisReserve.amount, BigInt(ignisAmount));
      });

      it('Burn reserve ignis', async function () {
        await program.methods
          .burnReserveIgnis(new anchor.BN(ignisAmount))
          .accounts({
            ignisStableCoin: ignisStablecoinPDA,
            ignisMint: ignisMintPDA,
            ignisReserve: ignisReservePubKey,
            signingPda: signingPDA,
            reserveWallet: wallet.publicKey,
            tokenProgram: TOKEN_PROGRAM_ID,
          })
          .rpc();

        const ignisMintAccount = await getMint(connection, ignisMintPDA);
        const ignisReserve = await getAccount(connection, ignisReservePubKey);

        // Assertions on ignis properties and ignis reserve properties
        assert.equal(ignisMintAccount.supply, BigInt(0));
        assert.equal(ignisReserve.amount, BigInt(0));
      });
    })
  })

  describe("ventura instructions", function () {

    describe('Mint ventura to user then redeem it', async function () {
      const venturaAmount = 2;

      it('Mint ventura to user', async function () {
        await program.methods
          .mintVenturaTo(new anchor.BN(venturaAmount))
          .accounts({
            venturaCoin: venturaCoinPDA,
            to: userVenturaATAPubKey,
            venturaMint: venturaMintPDA,
            signingPda: signingPDA,
            reserveWallet: wallet.publicKey,
            tokenProgram: TOKEN_PROGRAM_ID,
          })
          .rpc();

        const venturaMintAccount = await getMint(connection, venturaMintPDA);
        const userVenturaATA = await getAccount(connection, userVenturaATAPubKey);

        // Assertions on ventura and user ventura ATA properties
        assert.equal(venturaMintAccount.supply, BigInt(venturaAmount));
        assert.equal(userVenturaATA.amount, BigInt(venturaAmount));
      });

      it('Redeem user ventura', async function () {
        await program.methods
          .redeemVentura(new anchor.BN(venturaAmount))
          .accounts({
            ignisStablecoin: ignisStablecoinPDA,
            venturaCoin: venturaCoinPDA,
            userIgnisAta: userIgnisATAPubKey,
            userVenturaAta: userVenturaATAPubKey,
            ignisMint: ignisMintPDA,
            venturaMint: venturaMintPDA,
            signingPda: signingPDA,
            user: user.publicKey,
            tokenProgram: TOKEN_PROGRAM_ID,
          })
          .signers([user])
          .rpc();

        const ignisMintAccount = await getMint(connection, ignisMintPDA);
        const venturaMintAccount = await getMint(connection, venturaMintPDA);
        const userIgnisATA = await getAccount(connection, userIgnisATAPubKey);
        const userVenturaATA = await getAccount(connection, userVenturaATAPubKey);

        // Assertions on ventura and user ventura ATA properties
        assert.equal(venturaMintAccount.supply, BigInt(0));
        assert.equal(userVenturaATA.amount, BigInt(0));

        // Assertions on ignis and user ignis ATA properties
        assert.isTrue(ignisMintAccount.supply > 0);
        assert.isTrue(userIgnisATA.amount > 0);
      })
    })

    describe("Mint ventura to the reserve then burn it", function () {
      const venturaAmount = 2;

      it('Mint ventura to the reserve', async function () {
        await program.methods
          .mintVenturaTo(new anchor.BN(venturaAmount))
          .accounts({
            venturaCoin: venturaCoinPDA,
            to: venturaReservePubKey,
            venturaMint: venturaMintPDA,
            signingPda: signingPDA,
            reserveWallet: wallet.publicKey,
            tokenProgram: TOKEN_PROGRAM_ID,
          })
          .rpc();

        const venturaMintAccount = await getMint(connection, venturaMintPDA);
        const venturaReserve = await getAccount(connection, venturaReservePubKey);

        // Assertions on ventura properties and ventura reserve properties
        assert.equal(venturaMintAccount.supply, BigInt(venturaAmount));
        assert.equal(venturaReserve.amount, BigInt(venturaAmount));
      });

      it('Burn reserve ventura', async function () {
        await program.methods
          .burnReserveVentura(new anchor.BN(venturaAmount))
          .accounts({
            venturaCoin: venturaCoinPDA,
            venturaMintPDA: venturaMintPDA,
            venturaReserve: venturaReservePubKey,
            signingPda: signingPDA,
            reserveWallet: wallet.publicKey,
            tokenProgram: TOKEN_PROGRAM_ID,
          })
          .rpc();

        const venturaMintAccount = await getMint(connection, venturaMintPDA);
        const venturaReserve = await getAccount(connection, venturaReservePubKey);

        // Assertions on ventura properties and ventura reserve properties
        assert.equal(venturaMintAccount.supply, BigInt(0));
        assert.equal(venturaReserve.amount, BigInt(0));
      });
    })
  });
});

function loadKeypairFromSeedFile(filePath: string): Keypair {
  const fileContent = readFileSync(filePath);
  const seedArray = Uint8Array.from(JSON.parse(fileContent.toString()));
  return Keypair.fromSecretKey(seedArray);
}
