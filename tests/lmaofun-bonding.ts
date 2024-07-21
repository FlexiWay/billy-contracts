import {
  keypairIdentity,
  Pda,
  PublicKey,
  publicKey,
  TransactionBuilder,
  createAmount,
  some,
} from "@metaplex-foundation/umi";
import { createUmi } from "@metaplex-foundation/umi-bundle-defaults";
import {
  createAssociatedToken,
  createSplAssociatedTokenProgram,
  createSplTokenProgram,
  findAssociatedTokenPda,
  safeFetchMint,
  safeFetchToken,
  SPL_ASSOCIATED_TOKEN_PROGRAM_ID,
} from "@metaplex-foundation/mpl-toolbox";
import {
  Connection,
  Keypair,
  sendAndConfirmTransaction,
  Transaction,
  PublicKey as Web3JsPublicKey,
} from "@solana/web3.js";
import {
  BONDING_CURVE_PROGRAM_ID,
  createBondingCurveProgram,
  fetchTestState,
  fetchTestStateFromSeeds,
} from "../clients/js/src";
import {
  ASSOCIATED_TOKEN_PROGRAM_ID,
  createMint,
  createMintToInstruction,
  getOrCreateAssociatedTokenAccount,
  mintTo,
  TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import {
  fromWeb3JsKeypair,
  fromWeb3JsPublicKey,
  toWeb3JsKeypair,
} from "@metaplex-foundation/umi-web3js-adapters";
import assert from "assert";
describe("lmaofun-bonding", () => {
  let umi = createUmi("http://127.0.0.1:8899");
  umi.programs.add(createSplAssociatedTokenProgram());
  umi.programs.add(createSplTokenProgram());
  umi.programs.add(createBondingCurveProgram());
  const connection = new Connection("http://127.0.0.1:8899", {
    commitment: "finalized",
  });

  const keypair = Keypair.fromSecretKey(
    Uint8Array.from(require("../keys/test-kp.json"))
  );

  umi.use(keypairIdentity(fromWeb3JsKeypair(keypair)));
  const quoteMintDecimals = 6;

  before(async () => {
    try {
      await umi.rpc.airdrop(
        umi.identity.publicKey,
        createAmount(100_000 * 10 ** 9, "SOL", 9),
        { commitment: "finalized" }
      );

      const quoteMintWeb3js = await createMint(
        connection,
        keypair,
        keypair.publicKey,
        keypair.publicKey,
        quoteMintDecimals // Decimals
      );

      console.log("Created USDC: ", quoteMintWeb3js.toBase58());

      const userUsdcInfo = await getOrCreateAssociatedTokenAccount(
        connection,
        keypair,
        quoteMintWeb3js,
        keypair.publicKey,
        false,
        "confirmed"
      );
      console.log(
        keypair,
        quoteMintWeb3js,
        userUsdcInfo.address,
        keypair.publicKey
      );

      const mintToIx = createMintToInstruction(
        quoteMintWeb3js,
        userUsdcInfo.address,
        keypair.publicKey,
        100_000_000 * 10 ** quoteMintDecimals,
        [],
        TOKEN_PROGRAM_ID
      );

      const tx = new Transaction().add(mintToIx);
      const sig = await sendAndConfirmTransaction(connection, tx, [keypair]);
      console.log({ sig });
      const userQuote = fromWeb3JsPublicKey(userUsdcInfo.address);
      const quoteMint = fromWeb3JsPublicKey(quoteMintWeb3js);
    } catch (error) {
      console.log(error);
    }
  });

  it("passes", async () => {
    assert.equal(1, 1);
  });
});
