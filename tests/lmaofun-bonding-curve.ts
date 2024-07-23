import { Amman } from "@metaplex-foundation/amman-client";


import {
  keypairIdentity,
  createAmount,
  none,
} from "@metaplex-foundation/umi";
import { createUmi } from "@metaplex-foundation/umi-bundle-defaults";
import {
  createSplAssociatedTokenProgram,
  createSplTokenProgram,
  SPL_SYSTEM_PROGRAM_ID,
} from "@metaplex-foundation/mpl-toolbox";
import {
  Connection,
  Keypair,
  LAMPORTS_PER_SOL,
  PublicKey as Web3JsPublicKey,
} from "@solana/web3.js";
import {
  createLmaofunBondingCurveProgram,
  fetchGlobal,
  findGlobalPda,
  initialize,
  LMAOFUN_BONDING_CURVE_PROGRAM_ID,
  ProgramStatus,
} from "../clients/js/src";
import {
  fromWeb3JsKeypair,
} from "@metaplex-foundation/umi-web3js-adapters";
import assert from "assert";
import * as anchor from "@coral-xyz/anchor";
import {
  INIT_DEFAULTS,
} from "../clients/js/src/constants";
import { Program } from "@coral-xyz/anchor";
import { LmaofunBondingCurve } from "../target/types/lmaofun_bonding_curve";
import { findEvtAuthorityPda, getTransactionEventsFromDetails, getTxDetails, getTxEventsFromTxBuilderResponse, logEvent } from "../clients/js/src/utils";
import {
  setParams,
} from "../clients/js/src/generated/instructions/setParams";
import { assertGlobal } from "./utils";

const amman = Amman.instance();


const keypair = Keypair.fromSecretKey(
  Uint8Array.from(require("../keys/test-kp.json"))
);

amman.addr.addLabel("master", keypair.publicKey);
amman.addr.addLabel("LmaofunBondingCurveProgram", LMAOFUN_BONDING_CURVE_PROGRAM_ID);

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
  amman.addr.addLabel("global", globalPda[0]);

  let eventAuthorityPda = findEvtAuthorityPda(umi);
  let eventAuthority = eventAuthorityPda[0];
  const evtAuthorityAccs = {
    eventAuthority,
    program: LMAOFUN_BONDING_CURVE_PROGRAM_ID,
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
    //  ANCHOR
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

    const txBuilder = initialize(umi, {
      global: globalPda,
      authority: umi.identity,
      params: INIT_DEFAULTS,
      systemProgram: SPL_SYSTEM_PROGRAM_ID,
      ...evtAuthorityAccs,
    });


    const txRes= await txBuilder.sendAndConfirm(umi);
    const events =await getTxEventsFromTxBuilderResponse(connection, program, txRes);
    events.forEach(logEvent)



    const global = await fetchGlobal(umi, globalPda);
    assertGlobal( global, INIT_DEFAULTS);
  });

  it("set_params in SwapOnly", async () => {
    const txBuilder =  setParams(umi, {
        global: globalPda[0],
        authority: umi.identity,
        params:{
          initialTokenSupply:none(),
          initialRealSolReserves:none(),
          initialRealTokenReserves:none(),
          initialVirtualSolReserves:none(),
          initialVirtualTokenReserves:none(),
          solLaunchThreshold:none(),
          feeBasisPoints:none(),
          createdMintDecimals:none(),

          status: ProgramStatus.SwapOnly,
        },

        ...evtAuthorityAccs,
      })


    const txRes = await txBuilder.sendAndConfirm(umi);
    const events = await getTxEventsFromTxBuilderResponse(connection, program, txRes);
    events.forEach(logEvent)
    const global = await fetchGlobal(umi, globalPda);


    assertGlobal(global, {
      ...INIT_DEFAULTS,
      status: ProgramStatus.SwapOnly,
    });
  });

  it("set_params back", async () => {
    const txBuilder =   setParams(umi, {
      global: globalPda[0],
      authority: umi.identity,
      params:{
        initialTokenSupply:none(),
        initialRealSolReserves:none(),
        initialRealTokenReserves:none(),
        initialVirtualSolReserves:none(),
        initialVirtualTokenReserves:none(),
        solLaunchThreshold:none(),
        feeBasisPoints:none(),
        createdMintDecimals:none(),

        status: ProgramStatus.Running,
      },
      ...evtAuthorityAccs,
    })

    const txRes= await txBuilder.sendAndConfirm(umi);
    const events = await getTxEventsFromTxBuilderResponse(connection, program, txRes);
    events.forEach(logEvent)
    const global = await fetchGlobal(umi, globalPda);

    assertGlobal(global,INIT_DEFAULTS);
  });
});
