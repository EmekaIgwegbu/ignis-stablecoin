import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { IgnisStablecoin } from "../target/types/ignis_stablecoin";
import { PublicKey, SendTransactionError } from '@solana/web3.js'

import Token, { TOKEN_PROGRAM_ID } from '@solana/spl-token';
import { assert } from "chai";


describe("ignis-stablecoin", () => {

  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.IgnisStablecoin as Program<IgnisStablecoin>;
  const provider = anchor.getProvider() as anchor.AnchorProvider;
  const reserveWallet = provider.wallet;
  const connection = provider.connection;

  let user: anchor.web3.Keypair;
  let userIgnisAccountPubKey: PublicKey;
  let userVenturaAccountPubKey: PublicKey;
  let ignisStablecoinPDA: PublicKey;
  let venturaCoinPDA: PublicKey;
  let ignisMintPDA: PublicKey;
  let venturaMintPDA: PublicKey;
  let ignisReservePDA: PublicKey;
  let venturaReservePDA: PublicKey;
  let signingPDA: PublicKey;

  let initialUserIgnisBalance: bigint;
  let initialUserVenturaBalance: bigint;
  let initialIgnisStablecoinCirculatingAmount: anchor.BN;
  let initialVenturaCoinCirculatingAmount: anchor.BN;
  let initialIgnisMintAmount: bigint;
  let initialVenturaMintAmount: bigint;

  // TODO: manually set up user ignis and ventura accounts for testing then use them here
  before("Initialise test accounts", async function () {
    // Assign wallets
    user = anchor.web3.Keypair.generate();

    // Generate user token account keypairs
    userIgnisAccountPubKey = anchor.web3.Keypair.generate().publicKey;
    userVenturaAccountPubKey = anchor.web3.Keypair.generate().publicKey;

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
    [ignisReservePDA] = await PublicKey.findProgramAddress(
      [Buffer.from("ignis_reserve")],
      program.programId
    );
    [venturaReservePDA] = await PublicKey.findProgramAddress(
      [Buffer.from("ventura_reserve")],
      program.programId
    );
    [signingPDA] = await PublicKey.findProgramAddress(
      [],
      program.programId
    );

    // Get initial account balances
    // initialUserIgnisBalance = (await Token.getAccount(connection, userIgnisAccount)).amount;
    // initialUserVenturaBalance = (await Token.getAccount(connection, userVenturaAccount)).amount;
    // initialIgnisStablecoinCirculatingAmount = (await program.account.ignisStablecoin.fetch(ignisStablecoinPDA)).circulatingSupply;
    // initialVenturaCoinCirculatingAmount = (await program.account.ignisStablecoin.fetch(venturaCoinPDA)).circulatingSupply;
    // initialIgnisMintAmount = (await Token.getMint(connection, ignisMint)).supply;
    // initialVenturaMintAmount = (await Token.getMint(connection, venturaMint)).supply;
  });

  // TODO: consider just setting the initial values of relevant amounts instead then updating tests
  after("Reset accounts to their initial state", async function () {

  });

  it.only("Initialise instruction", async function () {
    // console.log("ignisStablecoin", ignisStablecoinPDA);
    // console.log("venturaCoin", venturaCoinPDA);
    // console.log("ignisMintPDA", ignisMintPDA);
    // console.log("venturaMintPDA", venturaMintPDA);
    // console.log("ignisReservePDA", ignisReservePDA);
    // console.log("venturaReservePDA", venturaReservePDA);
    // console.log("reserveWallet", reserveWallet.publicKey);

    try {
      await program.methods
        .initialise()
        .accounts({
          ignisStablecoin: ignisStablecoinPDA,
          venturaCoin: venturaCoinPDA,
          ignisMint: ignisMintPDA,
          venturaMint: venturaMintPDA,
          ignisReserve: ignisReservePDA,
          venturaReserve: venturaReservePDA,
          reserveWallet: reserveWallet.publicKey,
          signingPda: signingPDA,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: anchor.web3.SystemProgram.programId,
          rent: anchor.web3.SYSVAR_RENT_PUBKEY
        })
        .rpc();
    }
    catch (error) {
      if (error instanceof SendTransactionError) {
        // Catch the SendTransactionError and retrieve logs
        const logs = await error.getLogs(connection);
        console.error('Transaction failed with logs:', logs);
      } else {
        console.error('Transaction failed:', error);
      }
      throw error;
    }


    console.log("ignisStablecoinPDA", ignisStablecoinPDA);
    console.log("venturaCoinPDA", venturaCoinPDA);

    const ignisStablecoinAccount = await program.account.ignisStablecoin.fetch(ignisStablecoinPDA);
    const venturaCoinAccount = await program.account.venturaCoin.fetch(venturaCoinPDA);
    const ignisMintAccount = await Token.getMint(connection, ignisMintPDA);
    const venturaMintAccount = await Token.getMint(connection, venturaMintPDA);
    const ignisReserveAccount = await Token.getAccount(connection, ignisReservePDA);
    const venturaReserveAccount = await Token.getAccount(connection, venturaReservePDA);

    // Assertions on ignisStablecoin properties
    assert.equal(ignisStablecoinAccount.reserveAmount, new anchor.BN(0));
    assert.equal(ignisStablecoinAccount.circulatingSupply, new anchor.BN(0));
    assert.equal(ignisStablecoinAccount.peg, 1.0);
    assert.isTrue(ignisStablecoinAccount.mint.equals(ignisMintPDA));
    assert.isTrue(ignisStablecoinAccount.ignisReserve.equals(ignisReservePDA));
    assert.isTrue(ignisStablecoinAccount.reserveWallet.equals(reserveWallet.publicKey));

    // Assertions on venturaCoin properties
    assert.equal(venturaCoinAccount.reserveAmount, new anchor.BN(0));
    assert.equal(venturaCoinAccount.circulatingSupply, new anchor.BN(0));
    assert.isTrue(venturaCoinAccount.mint.equals(venturaMintPDA));
    assert.isTrue(venturaCoinAccount.venturaReserve.equals(venturaReservePDA));
    assert.isTrue(venturaCoinAccount.reserveWallet.equals(reserveWallet.publicKey));

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

  it('Mint ignis to user account then redeem it', async function () {
    const amount = 2;

    // mint ignis to user account
    await program.methods
      .mintIgnisTo(new anchor.BN(amount))
      .accounts({
        ignisStableCoin: ignisStablecoinPDA,
        to: userIgnisAccountPubKey,
        ignisMint: ignisMintPDA,
        signingPda: signingPDA,
        reserveWallet: reserveWallet.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .rpc();

    const ignisStablecoinAccount = await program.account.ignisStablecoin.fetch(ignisStablecoinPDA);
    const venturaCoinAccount = await program.account.venturaCoin.fetch(venturaCoinPDA);
    const ignisMintAccount = await Token.getMint(connection, ignisMintPDA);
    const venturaMintAccount = await Token.getMint(connection, venturaMintPDA);
    const userIgnisAccount = await Token.getAccount(connection, userIgnisAccountPubKey);
    const userVenturaAccount = await Token.getAccount(connection, userVenturaAccountPubKey);

    // Assertions on ignis and user ignis account properties
    assert.equal(ignisStablecoinAccount.circulatingSupply, initialIgnisStablecoinCirculatingAmount.add(new anchor.BN(amount)));
    assert.equal(ignisMintAccount.supply, initialIgnisMintAmount + BigInt(amount));
    assert.equal(userIgnisAccount.amount, initialUserIgnisBalance + BigInt(amount));

    // redeem user ignis
    await program.methods
      .redeemIgnis(new anchor.BN(amount))
      .accounts({
        ignisStablecoin: ignisStablecoinPDA,
        venturaCoin: venturaCoinPDA,
        userIgnisAccountPubKey: userIgnisAccount,
        userVenturaAccountPubKey: userVenturaAccount,
        ignisMint: ignisMintPDA,
        venturaMint: venturaMintPDA,
        signingPda: signingPDA,
        user: user.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([user])
      .rpc();

    // Assertions on ignis and user ignis account properties
    assert.equal(ignisStablecoinAccount.circulatingSupply, initialIgnisStablecoinCirculatingAmount);
    assert.equal(ignisMintAccount.supply, initialIgnisMintAmount);
    assert.equal(userIgnisAccount.amount, initialUserIgnisBalance);

    // Assertions on ventura and user ventura account properties
    assert.isTrue(venturaCoinAccount.circulatingSupply > initialVenturaCoinCirculatingAmount);
    assert.isTrue(venturaMintAccount.supply > initialVenturaMintAmount);
    assert.isTrue(userVenturaAccount.amount > initialUserVenturaBalance);
    console.log("venturaCoinAccount.circulatingSupply", venturaCoinAccount.circulatingSupply);
  });
});
