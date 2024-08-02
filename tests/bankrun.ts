import { Amman } from "@metaplex-foundation/amman-client";
import {
  keypairIdentity,
  createAmount,
  none,
  Keypair,
  createSignerFromKeypair,
  generateSigner,
  TransactionBuilder,
  Umi,
} from "@metaplex-foundation/umi";
import { createUmi } from "@metaplex-foundation/umi-bundle-defaults";
import {
  createMint,
  createSplAssociatedTokenProgram,
  createSplTokenProgram,
  findAssociatedTokenPda,
  SPL_SYSTEM_PROGRAM_ID,
  SPL_ASSOCIATED_TOKEN_PROGRAM_ID,
  SPL_TOKEN_PROGRAM_ID,
} from "@metaplex-foundation/mpl-toolbox";
import {
  Connection,
  Keypair as Web3JsKeypair,
  LAMPORTS_PER_SOL,
  PublicKey as Web3JsPublicKey,
  SYSVAR_CLOCK_PUBKEY,
  Transaction,
  Keypair as Web3JsKp,
  VersionedTransaction,
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
  swap,
  findBrandDistributorPda,
  findCreatorDistributorPda,
  findPlatformDistributorPda,
  findPresaleDistributorPda,
  claimCreatorVesting,
  fetchCreatorDistributor,
} from "../clients/js/src";
import {
  fromWeb3JsKeypair,
  fromWeb3JsPublicKey,
  toWeb3JsPublicKey,
  toWeb3JsTransaction,
} from "@metaplex-foundation/umi-web3js-adapters";
import { BankrunProvider } from "anchor-bankrun";
import {
  findMetadataPda,
  MPL_TOKEN_METADATA_PROGRAM_ID,
} from "@metaplex-foundation/mpl-token-metadata";
import assert from "assert";
import * as anchor from "@coral-xyz/anchor";
import {
  INIT_DEFAULTS,
  SIMPLE_DEFAULT_BONDING_CURVE_PRESET,
} from "../clients/js/src/constants";
import { Program } from "@coral-xyz/anchor";
import { LmaofunBondingCurve } from "../target/types/lmaofun_bonding_curve";
import {
  calculateFee,
  findEvtAuthorityPda,
  getTransactionEventsFromDetails,
  getTxDetails,
  getTxEventsFromTxBuilderResponse,
  logEvent,
} from "../clients/js/src/utils";
import { setParams } from "../clients/js/src/generated/instructions/setParams";
import { assertBondingCurve, assertGlobal } from "../tests/utils";
import { getGlobalSize } from "../clients/js/src/generated/accounts/global";
import { AMM } from "../clients/js/src/amm";
import { Pda, PublicKey, unwrapOption } from "@metaplex-foundation/umi";
import {
  BanksClient,
  Clock,
  ProgramTestContext,
  start,
  startAnchor,
} from "solana-bankrun";
import { web3JsRpc } from "@metaplex-foundation/umi-rpc-web3js";
import { AccountLayout } from "@solana/spl-token";
import { readFileSync } from "fs";
import path from "path";
import { MPL_SYSTEM_EXTRAS_PROGRAM_ID } from "@metaplex-foundation/mpl-toolbox";

const USE_BANKRUN = true;
const INITIAL_SOL = 100 * LAMPORTS_PER_SOL;

const amman = Amman.instance({
  ammanClientOpts: { autoUnref: false, ack: true },
  knownLabels: {
    [LMAOFUN_BONDING_CURVE_PROGRAM_ID.toString()]: "LmaofunBondingCurveProgram",
  },
});

// --- KEYPAIRS
const masterKp = fromWeb3JsKeypair(
  Web3JsKeypair.fromSecretKey(Uint8Array.from(require("../keys/test-kp.json")))
);
const simpleMintKp = fromWeb3JsKeypair(Web3JsKeypair.generate());
const creator = fromWeb3JsKeypair(Web3JsKeypair.generate());
const trader = fromWeb3JsKeypair(Web3JsKeypair.generate());
const withdrawAuthority = fromWeb3JsKeypair(Web3JsKeypair.generate());

amman.addr.addLabel("withdrawAuthority", withdrawAuthority.publicKey);
amman.addr.addLabel("simpleMint", simpleMintKp.publicKey);
amman.addr.addLabel("creator", creator.publicKey);
amman.addr.addLabel("trader", trader.publicKey);

// --- PROVIDERS
let bankrunContext: ProgramTestContext;
let bankrunClient: BanksClient;
let bankrunProvider: BankrunProvider;
let connection: Connection;
let rpcUrl = "http://127.0.0.1:8899";

let umi: Umi;

const bondingCurveProgram = createLmaofunBondingCurveProgram();

const programBinDir = path.join(__dirname, "..", ".programsBin");

function getProgram(programBinary) {
  return path.join(programBinDir, programBinary);
}
const loadProviders = async () => {
  process.env.ANCHOR_WALLET = "./keys/test-kp.json";
  console.log("using bankrun");
  bankrunContext = await startAnchor(
    "./",
    [
      // even though the program is loaded into the test validator, we need
      // to tell banks test client to load it as well
      // {
      //   name: "mpl_token_metadata",
      //   programId: toWeb3JsPublicKey(MPL_TOKEN_METADATA_PROGRAM_ID),
      // },
      // {
      //   name: "mpl_system_extras",
      //   programId: toWeb3JsPublicKey(MPL_SYSTEM_EXTRAS_PROGRAM_ID),
      // },
      // {
      //   name: "system_program",
      //   programId: toWeb3JsPublicKey(SPL_SYSTEM_PROGRAM_ID),
      // },
      // {
      //   name: "associated_token_program",
      //   programId: toWeb3JsPublicKey(SPL_ASSOCIATED_TOKEN_PROGRAM_ID),
      // },
      // {
      //   name: "token_program",
      //   programId: toWeb3JsPublicKey(SPL_TOKEN_PROGRAM_ID),
      // },
      // {
      //   name: "lmaofun_bonding_curve",
      //   programId: toWeb3JsPublicKey(LMAOFUN_BONDING_CURVE_PROGRAM_ID),
      // },
    ],
    [
      {
        address: toWeb3JsPublicKey(masterKp.publicKey),
        info: {
          lamports: INITIAL_SOL,
          executable: false,
          data: Buffer.from([]),
          owner: toWeb3JsPublicKey(SPL_SYSTEM_PROGRAM_ID),
        },
      },
      {
        address: toWeb3JsPublicKey(creator.publicKey),
        info: {
          lamports: INITIAL_SOL,
          executable: false,
          data: Buffer.from([]),
          owner: toWeb3JsPublicKey(SPL_SYSTEM_PROGRAM_ID),
        },
      },
      {
        address: toWeb3JsPublicKey(trader.publicKey),
        info: {
          lamports: INITIAL_SOL,
          executable: false,
          data: Buffer.from([]),
          owner: toWeb3JsPublicKey(SPL_SYSTEM_PROGRAM_ID),
        },
      },
      {
        address: toWeb3JsPublicKey(MPL_TOKEN_METADATA_PROGRAM_ID),
        info: await loadBin(getProgram("mpl_token_metadata.so")),
      },
      {
        address: toWeb3JsPublicKey(MPL_SYSTEM_EXTRAS_PROGRAM_ID),
        info: await loadBin(getProgram("mpl_system_extras.so")),
      },
    ]
  );
  // console.log("bankrunCtx: ", bankrunContext);
  bankrunClient = bankrunContext.banksClient;
  // console.log("bankrunClient: ", bankrunClient);
  bankrunProvider = new BankrunProvider(bankrunContext);
  // console.log("provider: ", provider);
  // console.log(provider.connection.rpcEndpoint);

  console.log("anchor connection: ", bankrunProvider.connection.rpcEndpoint);

  //@ts-ignore
  bankrunProvider.connection.rpcEndpoint = rpcUrl;
  const conn = bankrunProvider.connection;

  // rpcUrl = anchor.AnchorProvider.env().connection.rpcEndpoint;
  umi = createUmi(rpcUrl).use(web3JsRpc(conn));
  connection = conn;
  console.log("using bankrun payer");
  umi.use(keypairIdentity(fromWeb3JsKeypair(bankrunContext.payer)));

  umi.programs.add(createSplAssociatedTokenProgram());
  umi.programs.add(createSplTokenProgram());
  umi.programs.add(bondingCurveProgram);
};

export const loadBin = async (binPath: string) => {
  const programBytes = readFileSync(binPath);
  const executableAccount = {
    lamports: INITIAL_SOL,
    executable: true,
    owner: new Web3JsPublicKey("BPFLoader2111111111111111111111111111111111"),
    data: programBytes,
  };
  return executableAccount;
};

// pdas and util accs
let globalPda: Pda;
let eventAuthorityPda: Pda;
let eventAuthority: PublicKey;
let evtAuthorityAccs: {
  eventAuthority: PublicKey;
  program: PublicKey;
};

const GLOBAL_STARTING_BALANCE_INT = 1524240; // cant getMinimumBalanceForRentExemption on bankrun
const PLATFORM_DISTRIBUTOR_STARTING_BALANCE_INT = 1183200;
const loadKeypairs = async (umi) => {
  amman.addr.addLabel("master", umi.identity.publicKey);

  globalPda = findGlobalPda(umi);
  amman.addr.addLabel("global", globalPda[0]);
  eventAuthorityPda = findEvtAuthorityPda(umi);
  eventAuthority = eventAuthorityPda[0];
  amman.addr.addLabel("eventAuthority", eventAuthority);
  evtAuthorityAccs = {
    eventAuthority,
    program: LMAOFUN_BONDING_CURVE_PROGRAM_ID,
  };
};

import { transactionBuilder } from "@metaplex-foundation/umi";
import { setComputeUnitLimit } from "@metaplex-foundation/mpl-toolbox";

async function processTransaction(umi, txBuilder: TransactionBuilder) {
  let txWithBudget = await transactionBuilder().add(
    setComputeUnitLimit(umi, { units: 600_000 })
  );
  const fullBuilder = txBuilder.prepend(txWithBudget);
  if (USE_BANKRUN) {
    let tx: VersionedTransaction;
    try {
      const bhash = await bankrunClient.getLatestBlockhash();
      tx = toWeb3JsTransaction(
        await fullBuilder.setBlockhash(bhash?.[0] || "").build(umi)
      );
    } catch (error) {
      console.log("error: ", error);
      throw error;
    }
    const simRes = await bankrunClient.simulateTransaction(tx);
    // console.log("simRes: ", simRes);
    // console.log("simRes.logs: ", simRes.meta?.logMessages);
    // console.log(simRes.result);
    return await bankrunClient.processTransaction(tx);
  } else {
    return await fullBuilder.sendAndConfirm(umi);
  }
}

const getBalance = async (umi: Umi, pubkey: PublicKey) => {
  // cannot use umi helpers in bankrun
  if (USE_BANKRUN) {
    const balance = await bankrunClient.getBalance(toWeb3JsPublicKey(pubkey));
    return balance;
  } else {
    const umiBalance = await umi.rpc.getBalance(pubkey);
    return umiBalance.basisPoints;
  }
};
const getTknAmount = async (umi: Umi, pubkey: PublicKey) => {
  // cannot use umi helpers and some rpc methods in bankrun
  if (USE_BANKRUN) {
    const accInfo = await bankrunClient.getAccount(toWeb3JsPublicKey(pubkey));
    const info = AccountLayout.decode(accInfo?.data || Buffer.from([]));
    return info.amount;
  } else {
    const umiBalance = await connection.getAccountInfo(
      toWeb3JsPublicKey(pubkey)
    );
    const info = AccountLayout.decode(umiBalance?.data || Buffer.from([]));
    return info.amount;
  }
};

describe("lmaofun-bonding", () => {
  before(async () => {
    await loadProviders();
    await loadKeypairs(umi);
    try {
      if (!USE_BANKRUN) {
        await Promise.all(
          [
            umi.identity.publicKey,
            creator.publicKey,
            withdrawAuthority.publicKey,
            trader.publicKey,
          ].map(async (pk) => {
            const res = await umi.rpc.airdrop(
              pk,
              createAmount(INITIAL_SOL, "SOL", 9),
              {
                commitment: "finalized",
              }
            );
          })
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

    await processTransaction(umi, txBuilder);

    const global = await fetchGlobal(umi, globalPda);
    assertGlobal(global, INIT_DEFAULTS);
  });

  it("creates simple bonding curve", async () => {
    // const mintTx = await createMint(umi, {
    // //   mint: createSignerFromKeypair(umi, simpleMintKp),
    // //   decimals: INIT_DEFAULTS.createdMintDecimals,
    // //   mintAuthority: globalPda[0],
    // //   freezeAuthority: globalPda[0],
    // // });
    // // await processTransaction(umi, mintTx);

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

    console.log("ALLPUBKEYS:");
    console.log("masterKp.publicKey", masterKp.publicKey);
    console.log("creator.publicKey", creator.publicKey);
    console.log("trader.publicKey", trader.publicKey);

    console.log("simpleMintKp.publicKey", simpleMintKp.publicKey);
    console.log("withdrawAuthority.publicKey", withdrawAuthority.publicKey);

    console.log("globalPda[0]", globalPda[0]);
    console.log("bondingCurvePda[0]", simpleMintBondingCurvePda[0]);
    console.log("bondingCurveTknAcc[0]", simpleMintBondingCurveTknAcc[0]);
    console.log("metadataPda[0]", metadataPda[0]);

    // THIS SHIT WILL BE MOVED
    const creatorDistributor = await findCreatorDistributorPda(umi, {
      mint: simpleMintKp.publicKey,
    });
    const creatorDistributorTknAcc = await findAssociatedTokenPda(umi, {
      mint: simpleMintKp.publicKey,
      owner: creatorDistributor[0],
    });

    const brandDistributor = await findBrandDistributorPda(umi, {
      mint: simpleMintKp.publicKey,
    });
    const brandDistributorTknAcc = await findAssociatedTokenPda(umi, {
      mint: simpleMintKp.publicKey,
      owner: brandDistributor[0],
    });

    const platformDistributor = await findPlatformDistributorPda(umi, {
      mint: simpleMintKp.publicKey,
    });
    const platformDistributorTknAcc = await findAssociatedTokenPda(umi, {
      mint: simpleMintKp.publicKey,
      owner: platformDistributor[0],
    });

    const presaleDistributor = await findPresaleDistributorPda(umi, {
      mint: simpleMintKp.publicKey,
    });
    const presaleDistributorTknAcc = await findAssociatedTokenPda(umi, {
      mint: simpleMintKp.publicKey,
      owner: presaleDistributor[0],
    });

    const txBuilder = createBondingCurve(umi, {
      global: globalPda[0],

      creator: createSignerFromKeypair(umi, creator),
      mint: createSignerFromKeypair(umi, simpleMintKp),

      bondingCurve: simpleMintBondingCurvePda[0],
      bondingCurveTokenAccount: simpleMintBondingCurveTknAcc[0],

      creatorDistributor: creatorDistributor[0],
      creatorDistributorTokenAccount: creatorDistributorTknAcc[0],

      presaleDistributor: presaleDistributor[0],
      presaleDistributorTokenAccount: presaleDistributorTknAcc[0],

      brandAuthority: creator.publicKey,
      brandDistributor: brandDistributor[0],
      brandDistributorTokenAccount: brandDistributorTknAcc[0],
      platformDistributor: platformDistributor[0],
      platformDistributorTokenAccount: platformDistributorTknAcc[0],

      ...SIMPLE_DEFAULT_BONDING_CURVE_PRESET,

      metadata: metadataPda[0],
      ...mintMeta,

      ...evtAuthorityAccs,
      associatedTokenProgram: SPL_ASSOCIATED_TOKEN_PROGRAM_ID,
      clock: fromWeb3JsPublicKey(SYSVAR_CLOCK_PUBKEY),
    });

    await processTransaction(umi, txBuilder);

    const bondingCurveData = await fetchBondingCurve(
      umi,
      simpleMintBondingCurvePda[0]
    );
    console.log("bondingCurveData", bondingCurveData);
    // assertBondingCurve(bondingCurveData, {
    //   ...SIMPLE_DEFAULT_BONDING_CURVE_PRESET,
    //   complete: false,
    // });

    // // assert launch fee collection
    // const globalBalanceInt = await getBalance(umi, globalPda[0]);
    // const startingBalance = GLOBAL_STARTING_BALANCE_INT;
    // const accruedFees = Number(globalBalanceInt) - startingBalance;

    // assert(accruedFees == INIT_DEFAULTS.launchFeeLamports);
  });

  it("swap: buy", async () => {
    const traderSigner = createSignerFromKeypair(umi, trader);
    const simpleMintBondingCurvePda = await findBondingCurvePda(umi, {
      mint: simpleMintKp.publicKey,
    });

    const simpleMintBondingCurveTknAcc = await findAssociatedTokenPda(umi, {
      mint: simpleMintKp.publicKey,
      owner: simpleMintBondingCurvePda[0],
    });
    const platformDistributor = await findPlatformDistributorPda(umi, {
      mint: simpleMintKp.publicKey,
    });
    const traderAta = await findAssociatedTokenPda(umi, {
      mint: simpleMintKp.publicKey,
      owner: traderSigner.publicKey,
    });

    const bondingCurveData = await fetchBondingCurve(
      umi,
      simpleMintBondingCurvePda[0]
    );
    console.log("bondingCurveData", bondingCurveData);
    const amm = AMM.fromBondingCurve(bondingCurveData);
    let minBuyTokenAmount = 100_000_000_000n;
    let solAmount = amm.getBuyPrice(minBuyTokenAmount);

    // should use actual fee set on global when live
    let fee = calculateFee(solAmount, INIT_DEFAULTS.tradeFeeBps);
    const solAmountWithFee = solAmount + fee;
    console.log("solAmount", solAmount);
    console.log("fee", fee);
    console.log("solAmountWithFee", solAmountWithFee);
    console.log("buyTokenAmount", minBuyTokenAmount);
    let buyResult = amm.applyBuy(minBuyTokenAmount);
    console.log("buySimResult", buyResult);

    const txBuilder = swap(umi, {
      global: globalPda[0],
      user: traderSigner,

      baseIn: false, // buy
      exactInAmount: solAmount,
      minOutAmount: minBuyTokenAmount,

      mint: simpleMintKp.publicKey,
      bondingCurve: simpleMintBondingCurvePda[0],
      bondingCurveTokenAccount: simpleMintBondingCurveTknAcc[0],
      userTokenAccount: traderAta[0],
      platformDistributor: platformDistributor[0],
      clock: fromWeb3JsPublicKey(SYSVAR_CLOCK_PUBKEY),
      associatedTokenProgram: SPL_ASSOCIATED_TOKEN_PROGRAM_ID,
      ...evtAuthorityAccs,
    });

    await processTransaction(umi, txBuilder);

    // const events = await getTxEventsFromTxBuilderResponse(connection, program, txRes);
    // events.forEach(logEvent);

    const bondingCurveDataPost = await fetchBondingCurve(
      umi,
      simpleMintBondingCurvePda[0]
    );
    const traderAtaBalancePost = await getTknAmount(umi, traderAta[0]);
    console.log("pre.realTokenReserves", bondingCurveData.realTokenReserves);
    console.log(
      "post.realTokenReserves",
      bondingCurveDataPost.realTokenReserves
    );
    console.log("buyTokenAmount", minBuyTokenAmount);
    const tknAmountDiff =
      bondingCurveData.realTokenReserves -
      bondingCurveDataPost.realTokenReserves;
    console.log("real difference", tknAmountDiff);
    console.log(
      "buyAmount-tknAmountDiff",
      tknAmountDiff - minBuyTokenAmount,
      tknAmountDiff > minBuyTokenAmount
    );
    assert(tknAmountDiff > minBuyTokenAmount);
    assert(
      bondingCurveDataPost.realSolReserves ==
        bondingCurveData.realSolReserves + solAmount
    );
    assert(traderAtaBalancePost >= minBuyTokenAmount);
  });
  it("swap: sell", async () => {
    const traderSigner = createSignerFromKeypair(umi, trader);
    const simpleMintBondingCurvePda = await findBondingCurvePda(umi, {
      mint: simpleMintKp.publicKey,
    });

    const simpleMintBondingCurveTknAcc = await findAssociatedTokenPda(umi, {
      mint: simpleMintKp.publicKey,
      owner: simpleMintBondingCurvePda[0],
    });
    const platformDistributor = await findPlatformDistributorPda(umi, {
      mint: simpleMintKp.publicKey,
    });
    const traderAta = await findAssociatedTokenPda(umi, {
      mint: simpleMintKp.publicKey,
      owner: traderSigner.publicKey,
    });
    const traderAtaBalancePre = await getTknAmount(umi, traderAta[0]);
    const bondingCurveData = await fetchBondingCurve(
      umi,
      simpleMintBondingCurvePda[0]
    );
    const amm = AMM.fromBondingCurve(bondingCurveData);
    let sellTokenAmount = 100_000_000_000n;
    let solAmount = amm.getSellPrice(sellTokenAmount);

    // should use actual fee set on global when live
    let fee = calculateFee(solAmount, INIT_DEFAULTS.tradeFeeBps);
    const solAmountAfterFee = solAmount - fee;
    console.log("solAmount", solAmount);
    console.log("fee", fee);
    console.log("solAmountAfterFee", solAmountAfterFee);
    console.log("sellTokenAmount", sellTokenAmount);
    let sellResult = amm.applySell(sellTokenAmount);
    console.log("sellSimResult", sellResult);
    console.log({
      global: globalPda[0],
      user: traderSigner,

      baseIn: true, // sell
      exactInAmount: sellTokenAmount,
      minOutAmount: solAmountAfterFee,

      mint: simpleMintKp.publicKey,
      bondingCurve: simpleMintBondingCurvePda[0],

      bondingCurveTokenAccount: simpleMintBondingCurveTknAcc[0],
      userTokenAccount: traderAta[0],

      clock: fromWeb3JsPublicKey(SYSVAR_CLOCK_PUBKEY),
      associatedTokenProgram: SPL_ASSOCIATED_TOKEN_PROGRAM_ID,
    });
    const txBuilder = swap(umi, {
      global: globalPda[0],
      user: traderSigner,

      baseIn: true, // sell
      exactInAmount: sellTokenAmount,
      minOutAmount: solAmountAfterFee,

      mint: simpleMintKp.publicKey,
      bondingCurve: simpleMintBondingCurvePda[0],
      bondingCurveTokenAccount: simpleMintBondingCurveTknAcc[0],
      userTokenAccount: traderAta[0],
      platformDistributor: platformDistributor[0],
      clock: fromWeb3JsPublicKey(SYSVAR_CLOCK_PUBKEY),
      associatedTokenProgram: SPL_ASSOCIATED_TOKEN_PROGRAM_ID,
      ...evtAuthorityAccs,
    });

    await processTransaction(umi, txBuilder);

    // Post-transaction checks
    const bondingCurveDataPost = await fetchBondingCurve(
      umi,
      simpleMintBondingCurvePda[0]
    );
    const traderAtaBalancePost = await getTknAmount(umi, traderAta[0]);
    assert(
      bondingCurveDataPost.realTokenReserves ==
        bondingCurveData.realTokenReserves + sellTokenAmount
    );
    assert(
      bondingCurveDataPost.realSolReserves ==
        bondingCurveData.realSolReserves - solAmount
    );
    assert(traderAtaBalancePost == traderAtaBalancePre - sellTokenAmount);
  });

  it("set_params: status:SwapOnly, withdrawAuthority", async () => {
    const txBuilder = setParams(umi, {
      global: globalPda[0],
      authority: umi.identity,
      params: {
        launchFeeLamports: none(),
        tradeFeeBps: none(),
        createdMintDecimals: none(),
        status: ProgramStatus.SwapOnly,
      },
      newWithdrawAuthority: withdrawAuthority.publicKey,
      ...evtAuthorityAccs,
    });

    // const txRes = await txBuilder.sendAndConfirm(umi);
    await processTransaction(umi, txBuilder);
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
    const platformDistributor = await findPlatformDistributorPda(umi, {
      mint: simpleMintKp.publicKey,
    });
    const feeBalanceInt_total = await getBalance(umi, platformDistributor[0]);
    console.log("feeBalanceInt_total", feeBalanceInt_total);
    const startingBalance = PLATFORM_DISTRIBUTOR_STARTING_BALANCE_INT;
    const accruedFees = Number(feeBalanceInt_total) - startingBalance;
    assert(accruedFees > 0);
    const txBuilder = withdrawFees(umi, {
      global: globalPda[0],
      authority: createSignerFromKeypair(umi, withdrawAuthority),
      mint: simpleMintKp.publicKey,
      platformDistributor: platformDistributor[0],
      clock: fromWeb3JsPublicKey(SYSVAR_CLOCK_PUBKEY),
      ...evtAuthorityAccs,
    });

    // const txRes = await txBuilder.sendAndConfirm(umi);
    await processTransaction(umi, txBuilder);
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

    const feeBalancePost = await getBalance(umi, platformDistributor[0]);
    const feeBalancePost_int = Number(feeBalancePost);
    console.log("feeBalancePost_int", feeBalancePost_int);
    console.log("startingBalance", startingBalance);
    assert(feeBalancePost_int == startingBalance);
  });

  it("set_params: status:Running", async () => {
    const txBuilder = setParams(umi, {
      global: globalPda[0],
      authority: umi.identity,
      params: {
        launchFeeLamports: none(),
        tradeFeeBps: none(),
        createdMintDecimals: none(),

        status: ProgramStatus.Running,
      },
      ...evtAuthorityAccs,
    });

    await processTransaction(umi, txBuilder);
    //   const events = await getTxEventsFromTxBuilderResponse(connection, program, txRes);
    //   events.forEach(logEvent)
    const global = await fetchGlobal(umi, globalPda);

    assertGlobal(global, {
      ...INIT_DEFAULTS,
    });
  });

  it("cant claim creator vesting before cliff", async () => {
    const simpleMintBondingCurvePda = await findBondingCurvePda(umi, {
      mint: simpleMintKp.publicKey,
    });
    const simpleMintBondingCurveTknAcc = await findAssociatedTokenPda(umi, {
      mint: simpleMintKp.publicKey,
      owner: simpleMintBondingCurvePda[0],
    });
    const creatorAta = await findAssociatedTokenPda(umi, {
      mint: simpleMintKp.publicKey,
      owner: creator.publicKey,
    });
    const creatorDistributor = await findCreatorDistributorPda(umi, {
      mint: simpleMintKp.publicKey,
    });
    const creatorDistributorTknAcc = await findAssociatedTokenPda(umi, {
      mint: simpleMintKp.publicKey,
      owner: creatorDistributor[0],
    });
    const txBuilder = claimCreatorVesting(umi, {
      global: globalPda[0],
      mint: simpleMintKp.publicKey,
      creator: createSignerFromKeypair(umi, creator),
      bondingCurve: simpleMintBondingCurvePda[0],
      userTokenAccount: creatorAta[0],
      creatorDistributor: creatorDistributor[0],
      creatorDistributorTokenAccount: creatorDistributorTknAcc[0],
      clock: fromWeb3JsPublicKey(SYSVAR_CLOCK_PUBKEY),
      tokenProgram: SPL_TOKEN_PROGRAM_ID,
      associatedTokenProgram: SPL_ASSOCIATED_TOKEN_PROGRAM_ID,
      ...evtAuthorityAccs,
    });
    try {
      await processTransaction(umi, txBuilder);
      assert(false);
    } catch (e) {
      // console.log(e);
      assert(true);
    }
  });

  it("can claim creator vesting after cliff", async () => {
    const simpleMintBondingCurvePda = await findBondingCurvePda(umi, {
      mint: simpleMintKp.publicKey,
    });
    const simpleMintBondingCurveTknAcc = await findAssociatedTokenPda(umi, {
      mint: simpleMintKp.publicKey,
      owner: simpleMintBondingCurvePda[0],
    });
    const creatorAta = await findAssociatedTokenPda(umi, {
      mint: simpleMintKp.publicKey,
      owner: creator.publicKey,
    });
    const creatorDistributor = await findCreatorDistributorPda(umi, {
      mint: simpleMintKp.publicKey,
    });
    const creatorDistributorTknAcc = await findAssociatedTokenPda(umi, {
      mint: simpleMintKp.publicKey,
      owner: creatorDistributor[0],
    });
    const bondingCurveData = await fetchBondingCurve(
      umi,
      simpleMintBondingCurvePda[0]
    );
    const startTime = bondingCurveData.startTime;
    const cliff = bondingCurveData.vestingTerms.cliff;
    const secondToJumpTo = startTime + cliff + BigInt(24 * 60 * 60);

    const currentClock = await bankrunClient.getClock();
    bankrunContext.setClock(
      new Clock(
        currentClock.slot,
        currentClock.epochStartTimestamp,
        currentClock.epoch,
        currentClock.leaderScheduleEpoch,
        secondToJumpTo
      )
    );

    const txBuilder = claimCreatorVesting(umi, {
      global: globalPda[0],
      mint: simpleMintKp.publicKey,
      creator: createSignerFromKeypair(umi, creator),
      bondingCurve: simpleMintBondingCurvePda[0],
      userTokenAccount: creatorAta[0],
      creatorDistributor: creatorDistributor[0],
      creatorDistributorTokenAccount: creatorDistributorTknAcc[0],
      clock: fromWeb3JsPublicKey(SYSVAR_CLOCK_PUBKEY),
      tokenProgram: SPL_TOKEN_PROGRAM_ID,
      associatedTokenProgram: SPL_ASSOCIATED_TOKEN_PROGRAM_ID,
      ...evtAuthorityAccs,
    });

    await processTransaction(umi, txBuilder);

    const creatorDistributorData = await fetchCreatorDistributor(
      umi,
      creatorDistributor[0]
    );
    assert(
      unwrapOption(creatorDistributorData.lastDistribution) == secondToJumpTo
    );
  });
  it("can claim creator again vesting after cliff", async () => {
    const simpleMintBondingCurvePda = await findBondingCurvePda(umi, {
      mint: simpleMintKp.publicKey,
    });
    const simpleMintBondingCurveTknAcc = await findAssociatedTokenPda(umi, {
      mint: simpleMintKp.publicKey,
      owner: simpleMintBondingCurvePda[0],
    });
    const creatorAta = await findAssociatedTokenPda(umi, {
      mint: simpleMintKp.publicKey,
      owner: creator.publicKey,
    });
    const creatorDistributor = await findCreatorDistributorPda(umi, {
      mint: simpleMintKp.publicKey,
    });
    const creatorDistributorTknAcc = await findAssociatedTokenPda(umi, {
      mint: simpleMintKp.publicKey,
      owner: creatorDistributor[0],
    });
    const bondingCurveData = await fetchBondingCurve(
      umi,
      simpleMintBondingCurvePda[0]
    );
    const creatorDistributorData = await fetchCreatorDistributor(
      umi,
      creatorDistributor[0]
    );
    const lastDistribution = unwrapOption(
      creatorDistributorData.lastDistribution
    );

    const secondToJumpTo = Number(lastDistribution) + Number(24 * 60 * 60);

    const currentClock = await bankrunClient.getClock();
    bankrunContext.setClock(
      new Clock(
        currentClock.slot,
        currentClock.epochStartTimestamp,
        currentClock.epoch,
        currentClock.leaderScheduleEpoch,
        BigInt(secondToJumpTo)
      )
    );

    const txBuilder = claimCreatorVesting(umi, {
      global: globalPda[0],
      mint: simpleMintKp.publicKey,
      creator: createSignerFromKeypair(umi, creator),
      bondingCurve: simpleMintBondingCurvePda[0],
      userTokenAccount: creatorAta[0],
      creatorDistributor: creatorDistributor[0],
      creatorDistributorTokenAccount: creatorDistributorTknAcc[0],
      clock: fromWeb3JsPublicKey(SYSVAR_CLOCK_PUBKEY),
      tokenProgram: SPL_TOKEN_PROGRAM_ID,
      associatedTokenProgram: SPL_ASSOCIATED_TOKEN_PROGRAM_ID,
      ...evtAuthorityAccs,
    });

    await processTransaction(umi, txBuilder);

    const creatorDistributorDataPost = await fetchCreatorDistributor(
      umi,
      creatorDistributor[0]
    );
    assert(
      unwrapOption(creatorDistributorDataPost.lastDistribution) ==
        BigInt(secondToJumpTo)
    );
  });
});
