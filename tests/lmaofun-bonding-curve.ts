import { Amman } from "@metaplex-foundation/amman-client";

import {
  keypairIdentity,
  createAmount,
  none,
  Keypair,
  createSignerFromKeypair,
  generateSigner,
} from "@metaplex-foundation/umi";
import { createUmi } from "@metaplex-foundation/umi-bundle-defaults";
import {
  createMint,
  createSplAssociatedTokenProgram,
  createSplTokenProgram,
  findAssociatedTokenPda,
  SPL_SYSTEM_PROGRAM_ID,
  SPL_ASSOCIATED_TOKEN_PROGRAM_ID,
} from "@metaplex-foundation/mpl-toolbox";
import {
  Connection,
  Keypair as Web3JsKeypair,
  LAMPORTS_PER_SOL,
  PublicKey as Web3JsPublicKey,
  SYSVAR_CLOCK_PUBKEY,
} from "@solana/web3.js";
import {
  createLmaofunBondingCurveProgram,
  fetchGlobal,
  findGlobalPda,
  initialize,
  LMAOFUN_BONDING_CURVE_PROGRAM_ID,
  ProgramStatus,
  createBondingCurve,
  safeFetchBondingCurve,
  fetchBondingCurve,
  findBondingCurvePda,
  withdrawFees,
} from "../clients/js/src";
import {
  fromWeb3JsKeypair,
  fromWeb3JsPublicKey,
} from "@metaplex-foundation/umi-web3js-adapters";
import { findMetadataPda } from "@metaplex-foundation/mpl-token-metadata";
import assert from "assert";
import * as anchor from "@coral-xyz/anchor";
import { INIT_DEFAULTS } from "../clients/js/src/constants";
import { Program } from "@coral-xyz/anchor";
import { LmaofunBondingCurve } from "../target/types/lmaofun_bonding_curve";
import {
  findEvtAuthorityPda,
  getTransactionEventsFromDetails,
  getTxDetails,
  getTxEventsFromTxBuilderResponse,
  logEvent,
} from "../clients/js/src/utils";
import { setParams } from "../clients/js/src/generated/instructions/setParams";
import { assertBondingCurve, assertGlobal } from "./utils";
import { getGlobalSize } from "../clients/js/src/generated/accounts/global";

const amman = Amman.instance({
  ammanClientOpts: { autoUnref: false, ack: true },
  knownLabels: {
    [LMAOFUN_BONDING_CURVE_PROGRAM_ID.toString()]: "LmaofunBondingCurveProgram",
  },
});

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
const connection = new Connection(rpcUrl, {
  commitment: "confirmed",
});

let umi = createUmi(rpcUrl);

const keypair = Web3JsKeypair.fromSecretKey(
  Uint8Array.from(require("../keys/test-kp.json"))
);

let simpleMintKp = generateSigner(umi);
let creator = generateSigner(umi);
let withdrawAuthority = generateSigner(umi);

amman.addr.addLabel("master", keypair.publicKey);
amman.addr.addLabel("withdrawAuthority", withdrawAuthority.publicKey);

amman.addr.addLabel("simpleMint", simpleMintKp.publicKey);
amman.addr.addLabel("creator", creator.publicKey);

describe("lmaofun-bonding", () => {
  const bondingCurveProgram = createLmaofunBondingCurveProgram();
  umi.programs.add(createSplAssociatedTokenProgram());
  umi.programs.add(createSplTokenProgram());
  umi.programs.add(bondingCurveProgram);

  umi.use(keypairIdentity(fromWeb3JsKeypair(keypair)));

  let globalPda = findGlobalPda(umi);
  amman.addr.addLabel("global", globalPda[0]);

  let eventAuthorityPda = findEvtAuthorityPda(umi);
  let eventAuthority = eventAuthorityPda[0];
  const evtAuthorityAccs = {
    eventAuthority,
    program: LMAOFUN_BONDING_CURVE_PROGRAM_ID,
  };
  before(async () => {
    try {
      await Promise.all(
        [
          umi.identity.publicKey,
          creator.publicKey,
          withdrawAuthority.publicKey,
        ].map((pk) =>
          umi.rpc.airdrop(pk, createAmount(100 * LAMPORTS_PER_SOL, "SOL", 9), {
            commitment: "finalized",
          })
        )
      );
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

    const txRes = await txBuilder.sendAndConfirm(umi);
    // const events = await getTxEventsFromTxBuilderResponse(
    //   connection,
    //   program,
    //   txRes
    // );
    // events.forEach(logEvent);

    const global = await fetchGlobal(umi, globalPda);
    assertGlobal(global, INIT_DEFAULTS);
  });

  it("creates simple bonding curve", async () => {
    await createMint(umi, {
      mint: createSignerFromKeypair(umi, simpleMintKp),
      decimals: INIT_DEFAULTS.createdMintDecimals,
      mintAuthority: globalPda[0],
      freezeAuthority: globalPda[0],
    });

    const simpleMintBondingCurvePda = await findBondingCurvePda(umi, {
      mint: simpleMintKp.publicKey,
    });
    const simpleMintBondingCurveTknAcc = await findAssociatedTokenPda(umi, {
      mint: simpleMintKp.publicKey,
      owner: simpleMintBondingCurvePda[0],
    });

    const metadataPda = await findMetadataPda(umi, {
      mint: simpleMintKp.publicKey,
    });

    const mintMeta = {
      name: "simpleMint",
      symbol: "simpleMint",
      uri: "https://www.simpleMint.com",
    };

    const txBuilder = createBondingCurve(umi, {
      global: globalPda[0],
      creator: createSignerFromKeypair(umi, creator),
      mint: simpleMintKp,

      bondingCurve: simpleMintBondingCurvePda[0],
      bondingCurveTokenAccount: simpleMintBondingCurveTknAcc[0],
      metadata: metadataPda[0],

      ...mintMeta,
      ...evtAuthorityAccs,

      associatedTokenProgram: SPL_ASSOCIATED_TOKEN_PROGRAM_ID,
      clock: fromWeb3JsPublicKey(SYSVAR_CLOCK_PUBKEY),
      startTime: none(),
    });
    const txRes = await txBuilder.sendAndConfirm(umi);

    // const events = await getTxEventsFromTxBuilderResponse(
    //   connection,
    //   program,
    //   txRes
    // );
    // events.forEach(logEvent);

    const bondingCurveData = await fetchBondingCurve(
      umi,
      simpleMintBondingCurvePda[0]
    );
    assertBondingCurve(bondingCurveData, {
      virtualSolReserves: INIT_DEFAULTS.initialVirtualSolReserves,
      virtualTokenReserves: INIT_DEFAULTS.initialVirtualTokenReserves,
      realSolReserves: INIT_DEFAULTS.initialRealSolReserves,
      realTokenReserves: INIT_DEFAULTS.initialRealTokenReserves,
      tokenTotalSupply: INIT_DEFAULTS.initialTokenSupply,
      complete: false,
    });

    // assert launch fee collection
    const globalBalance = await umi.rpc.getBalance(globalPda[0]);
    const globalBalanceInt = parseInt(globalBalance.basisPoints.toString());
    const startingBalance = await connection.getMinimumBalanceForRentExemption(
      getGlobalSize()
    );
    const accruedFees = globalBalanceInt - startingBalance;

    assert(accruedFees == INIT_DEFAULTS.launchFeeLamports);
  });

  it("set_params: status:SwapOnly, withdrawAuthority", async () => {
    const txBuilder = setParams(umi, {
      global: globalPda[0],
      authority: umi.identity,
      params: {
        launchFeeLamports: none(),
        initialTokenSupply: none(),
        initialRealSolReserves: none(),
        initialRealTokenReserves: none(),
        initialVirtualSolReserves: none(),
        initialVirtualTokenReserves: none(),
        solLaunchThreshold: none(),
        tradeFeeBps: none(),
        createdMintDecimals: none(),
        status: ProgramStatus.SwapOnly,
      },
      newWithdrawAuthority: withdrawAuthority.publicKey,
      ...evtAuthorityAccs,
    });

    const txRes = await txBuilder.sendAndConfirm(umi);
    // const events = await getTxEventsFromTxBuilderResponse(connection, program, txRes);
    // events.forEach(logEvent)
    const global = await fetchGlobal(umi, globalPda);

    assertGlobal(global, {
      ...INIT_DEFAULTS,
      status: ProgramStatus.SwapOnly,
      withdrawAuthority: withdrawAuthority.publicKey,
    });
  });

  it("withdraw_fees using withdraw_authority", async () => {
    const globalBalance = await umi.rpc.getBalance(globalPda[0]);
    const globalBalanceInt = parseInt(globalBalance.basisPoints.toString());
    const startingBalance = await connection.getMinimumBalanceForRentExemption(
      getGlobalSize()
    );
    const accruedFees = globalBalanceInt - startingBalance;

    assert(accruedFees > 0);
    const txBuilder = withdrawFees(umi, {
      global: globalPda[0],
      authority: withdrawAuthority,
      ...evtAuthorityAccs,
    });

    const txRes = await txBuilder.sendAndConfirm(umi);
    // const events = await getTxEventsFromTxBuilderResponse(
    //   connection,
    //   program,
    //   txRes
    // );
    // events.forEach(logEvent);

    const global = await fetchGlobal(umi, globalPda);

    assertGlobal(global, {
      ...INIT_DEFAULTS,
      status: ProgramStatus.SwapOnly,
      withdrawAuthority: withdrawAuthority.publicKey,
    });

    const globalBalancePost = await umi.rpc.getBalance(globalPda[0]);
    const globalBalanceIntPost = parseInt(
      globalBalancePost.basisPoints.toString()
    );
    console.log({ globalBalancePost, globalBalanceIntPost, startingBalance });
    assert(globalBalanceIntPost == startingBalance);
  });

  it("set_params: status:Running", async () => {
    const txBuilder = setParams(umi, {
      global: globalPda[0],
      authority: umi.identity,
      params: {
        launchFeeLamports: none(),
        initialTokenSupply: none(),
        initialRealSolReserves: none(),
        initialRealTokenReserves: none(),
        initialVirtualSolReserves: none(),
        initialVirtualTokenReserves: none(),
        solLaunchThreshold: none(),
        tradeFeeBps: none(),
        createdMintDecimals: none(),

        status: ProgramStatus.Running,
      },
      ...evtAuthorityAccs,
    });

    const txRes = await txBuilder.sendAndConfirm(umi);
    //   const events = await getTxEventsFromTxBuilderResponse(connection, program, txRes);
    //   events.forEach(logEvent)
    const global = await fetchGlobal(umi, globalPda);

    assertGlobal(global, {
      ...INIT_DEFAULTS,
    });
  });
});
