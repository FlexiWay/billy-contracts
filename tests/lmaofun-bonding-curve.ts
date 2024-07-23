import {
  keypairIdentity,
  createAmount,
  TransactionBuilder,
} from "@metaplex-foundation/umi";
import { createUmi } from "@metaplex-foundation/umi-bundle-defaults";
import {
  createAssociatedToken,
  createSplAssociatedTokenProgram,
  createSplTokenProgram,
  SPL_SYSTEM_PROGRAM_ID,
} from "@metaplex-foundation/mpl-toolbox";
import {
  Connection,
  Keypair,
  LAMPORTS_PER_SOL,
  sendAndConfirmTransaction,
  SystemProgram,
  Transaction,
  TransactionMessage,
  VersionedTransaction,
  PublicKey as Web3JsPublicKey,
} from "@solana/web3.js";
import {
  createLmaofunBondingCurveProgram,
  findGlobalPda,
  initialize,
  InitializeInstructionAccounts,
  LMAOFUN_BONDING_CURVE_PROGRAM_ID,
  ProgramStatus,
  safeFetchGlobal,
  safeFetchGlobalFromSeeds,
} from "../clients/js/src";
import {
  createMint,
  createMintToInstruction,
  getOrCreateAssociatedTokenAccount,
  TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import {
  fromWeb3JsKeypair,
  fromWeb3JsPublicKey,
  toWeb3JsKeypair,
  toWeb3JsPublicKey,
  toWeb3JsTransaction,
} from "@metaplex-foundation/umi-web3js-adapters";
import assert from "assert";
import * as anchor from "@coral-xyz/anchor";
import {
  DECIMALS_MULTIPLIER,
  DEFAULT_TOKEN_SUPPLY,
  INIT_DEFAULTS,
  INIT_DEFAULTS_ANCHOR,
} from "../clients/js/src/constants";
import { Program } from "@coral-xyz/anchor";
import { LmaofunBondingCurve } from "../target/types/lmaofun_bonding_curve";
import { findEvtAuthorityPda } from "../clients/js/src/utils";
import {
  setParams,
  SetParamsInstructionAccounts,
} from "../clients/js/src/generated/instructions/setParams";

const keypair = Keypair.fromSecretKey(
  Uint8Array.from(require("../keys/test-kp.json"))
);

describe("lmaofun-bonding", () => {
  let rpcUrl;
  if (process.env.ANCHOR_PROVIDER_URL) {
    rpcUrl = process.env.ANCHOR_PROVIDER_URL;
  } else {
    rpcUrl = "http://127.0.0.1:8899";
    process.env.ANCHOR_PROVIDER_URL = rpcUrl;
    process.env.ANCHOR_WALLET = "./keys/test-kp.json";
  }

  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const program = anchor.workspace
    .LmaofunBondingCurve as Program<LmaofunBondingCurve>;

  let umi = createUmi(rpcUrl);
  const bondingCurveProgram = createLmaofunBondingCurveProgram();
  umi.programs.add(createSplAssociatedTokenProgram());
  umi.programs.add(createSplTokenProgram());
  umi.programs.add(bondingCurveProgram);
  const connection = new Connection(rpcUrl, {
    commitment: "confirmed",
  });

  umi.use(keypairIdentity(fromWeb3JsKeypair(keypair)));

  let globalPda = findGlobalPda(umi);
  let eventAuthorityPda = findEvtAuthorityPda(umi);
  let eventAuthority = eventAuthorityPda[0];
  const evtAuthorityAccs = {
    eventAuthority,
    program: LMAOFUN_BONDING_CURVE_PROGRAM_ID,
  };
  const quoteMintDecimals = 6;
  const initAccs = {
    globalAuthority: toWeb3JsPublicKey(umi.identity.publicKey),
    feeRecipient: toWeb3JsPublicKey(umi.identity.publicKey),
    withdrawAuthority: toWeb3JsPublicKey(umi.identity.publicKey),
  };
  before(async () => {
    try {
      const solBal = await umi.rpc.getBalance(umi.identity.publicKey);
      if (parseInt(solBal.basisPoints.toString()) < 10 * LAMPORTS_PER_SOL) {
        const sig = await umi.rpc.airdrop(
          umi.identity.publicKey,
          createAmount(100 * LAMPORTS_PER_SOL, "SOL", 9),
          { commitment: "finalized" }
        );
      }
    } catch (error) {
      console.log(error);
    }
  });

  it("is initialized", async () => {
    const txBuilder = initialize(umi, {
      global: globalPda,
      authority: umi.identity,
      params: INIT_DEFAULTS,
      systemProgram: SPL_SYSTEM_PROGRAM_ID,
      ...evtAuthorityAccs,
    });


    const { signature, result } = await txBuilder.sendAndConfirm(umi);
    console.log({ signature, val: result.value });

    // const tx = await program.methods
    //   .initialize(INIT_DEFAULTS_ANCHOR)
    //   .accounts({
    //     authority: keypair.publicKey,
    //     global: globalPda[0],
    //   })
    //   // .signers([keypair])
    //   .transaction();

    // const sig = await connection.sendTransaction(tx, [keypair]);
    // console.log({ sig });
    // const res = await connection.confirmTransaction(sig, "finalized");
    // console.log(res);

    // console.log({ sig });

    const global = await safeFetchGlobal(umi, globalPda);
    if(!global) {
      throw new Error("Global acc not found");
    }
    assert.equal(
      global.initialRealSolReserves,
      INIT_DEFAULTS.initialRealSolReserves
    );
    assert.equal(
      global.initialRealTokenReserves,
      INIT_DEFAULTS.initialRealTokenReserves
    );
    assert.equal(
      global.initialVirtualSolReserves,
      INIT_DEFAULTS.initialVirtualSolReserves
    );
    assert.equal(
      global.initialVirtualTokenReserves,
      INIT_DEFAULTS.initialVirtualTokenReserves
    );
    assert.equal(global.initialTokenSupply, INIT_DEFAULTS.initialTokenSupply);
    assert.equal(global.solLaunchThreshold, INIT_DEFAULTS.solLaunchThreshold);
    assert.equal(global.feeBasisPoints, INIT_DEFAULTS.feeBasisPoints);

    assert.equal(global.status, ProgramStatus.Running);
  });

  it("set_params in SwapOnly", async () => {
    const txBuilder = new TransactionBuilder();
    txBuilder.add(
      setParams(umi, {
        global: globalPda[0],
        status: ProgramStatus.SwapOnly,

        settingsParams: INIT_DEFAULTS,
        authorityParams: initAccs,

        ...evtAuthorityAccs,
      })
    );

    const tx = await txBuilder.buildAndSign(umi);
    const _tx = toWeb3JsTransaction(tx);
    const simRes = await connection.simulateTransaction(_tx);
    console.log(simRes);
    const { ...a } = await txBuilder.sendAndConfirm(umi);
    console.log(a);
    const global = await safeFetchGlobal(umi, globalPda);
    console.log({ global });

    assert.equal(global.status, ProgramStatus.SwapOnly);
  });

  it("set_params back", async () => {
    const txBuilder = new TransactionBuilder();
    txBuilder.add(
      setParams(umi, {
        global: globalPda[0],
        status: ProgramStatus.Running,

        settingsParams: INIT_DEFAULTS,
        authorityParams: initAccs,
        ...evtAuthorityAccs,
      })
    );

    const tx = await txBuilder.buildAndSign(umi);
    const _tx = toWeb3JsTransaction(tx);
    const simRes = await connection.simulateTransaction(_tx);
    console.log(simRes);
    const { ...a } = await txBuilder.sendAndConfirm(umi);
    console.log(a);
    const global = await safeFetchGlobal(umi, globalPda);
    console.log({ global });

    assert.equal(global.status, ProgramStatus.Running);
  });
});
