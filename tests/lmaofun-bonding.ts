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
  createBondingCurveProgram,
  findGlobalPda,
  initialize,
  InitializeInstructionAccounts,
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
} from "../clients/js/src/constants";
import { Program } from "@coral-xyz/anchor";
import { BondingCurve } from "../target/types/bonding_curve";
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
  const program = anchor.workspace.BondingCurve as Program<BondingCurve>;

  let umi = createUmi(rpcUrl);
  umi.programs.add(createSplAssociatedTokenProgram());
  umi.programs.add(createSplTokenProgram());
  umi.programs.add(createBondingCurveProgram());
  const connection = new Connection(rpcUrl, {
    commitment: "finalized",
  });

  umi.use(keypairIdentity(fromWeb3JsKeypair(keypair)));

  let globalPda = findGlobalPda(umi);
  let eventAuthorityPda = findEventAuthorityPda(umi);
  const quoteMintDecimals = 6;

  before(async () => {
    try {
      await umi.rpc.airdrop(
        umi.identity.publicKey,
        createAmount(100_000 * 10 ** 9, "SOL", 9),
        { commitment: "confirmed" }
      );
    } catch (error) {
      console.log(error);
    }
  });

  it("is initialized", async () => {
    const initAccs: InitializeInstructionAccounts = {
      global: globalPda[0],
    };

    const txBuilder = new TransactionBuilder();
    txBuilder.add(
      initialize(umi, {
        ...INIT_DEFAULTS,
        ...initAccs,
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
    const initAccs: SetParamsInstructionAccounts = {
      global: globalPda[0],
    };

    const txBuilder = new TransactionBuilder();
    txBuilder.add(
      setParams(umi, {
        // ...INIT_DEFAULTS,
        // ...initAccs,
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
});
