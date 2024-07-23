import { Context, Pda, RpcConfirmTransactionResult , TransactionSignature} from '@metaplex-foundation/umi';
import { string } from "@metaplex-foundation/umi/serializers";
import { LMAOFUN_BONDING_CURVE_PROGRAM_ID } from './generated/programs/lmaofunBondingCurve';
import * as anchor from "@coral-xyz/anchor";
import { LmaofunBondingCurve } from './idls/lmaofun_bonding_curve';
import { Connection, PublicKey, } from '@solana/web3.js';

import {
    toWeb3JsPublicKey,
  } from "@metaplex-foundation/umi-web3js-adapters";
import { bs58 } from '@coral-xyz/anchor/dist/cjs/utils/bytes';

const EVENT_AUTHORITY_PDA_SEED = "__event_authority";
export function findEvtAuthorityPda(
    context: Pick<Context, 'eddsa' | 'programs'>,
    ): Pda {
    const programId = context.programs.getPublicKey('bondingCurve', LMAOFUN_BONDING_CURVE_PROGRAM_ID);
    return context.eddsa.findPda(programId, [
                    string({ size: 'variable' }).serialize(EVENT_AUTHORITY_PDA_SEED),
              ]);
  }


export function findEvtAuthorityPdaRaw(

    ): [PublicKey, number] {
    const programId = toWeb3JsPublicKey(LMAOFUN_BONDING_CURVE_PROGRAM_ID);
   const pda = PublicKey.findProgramAddressSync([Buffer.from(EVENT_AUTHORITY_PDA_SEED)], programId);
   return pda
  }



type EventKeys = keyof anchor.IdlEvents<LmaofunBondingCurve>;

const validEventNames: Array<keyof anchor.IdlEvents<LmaofunBondingCurve>> = [
  "GlobalUpdateEvent",
  "CreateEvent",
];

export const getTxEventsFromTxBuilderResponse = async (conn:Connection, program: anchor.Program<LmaofunBondingCurve>, txBuilderRes:{
    signature: TransactionSignature;
    result: RpcConfirmTransactionResult;
}) => {
    const sig = bs58.encode(txBuilderRes.signature)
    return await getTransactionEvents(conn, program, sig);
}

export const getTransactionEvents = async (conn:Connection, program: anchor.Program<LmaofunBondingCurve>, sig: string) => {
    const txDetails = await getTxDetails(conn, sig);
    return getTransactionEventsFromDetails(program, txDetails);
}

export const getTransactionEventsFromDetails = (
  program: anchor.Program<LmaofunBondingCurve>,
  txResponse: anchor.web3.VersionedTransactionResponse | null
) => {
  if (!txResponse) {
    return [];
  }

  let eventPDA= findEvtAuthorityPdaRaw()[0];

  let indexOfEventPDA =
    txResponse.transaction.message.staticAccountKeys.findIndex((key) =>
      key.equals(eventPDA)
    );

  if (indexOfEventPDA === -1) {
    return [];
  }

  const matchingInstructions = txResponse.meta?.innerInstructions
    ?.flatMap((ix) => ix.instructions)
    .filter(
      (instruction) =>
        instruction.accounts.length === 1 &&
        instruction.accounts[0] === indexOfEventPDA
    );

  if (matchingInstructions) {
    let events = matchingInstructions.map((instruction) => {
      const ixData = anchor.utils.bytes.bs58.decode(instruction.data);
      const eventData = anchor.utils.bytes.base64.encode(ixData.slice(8));
      const event = program.coder.events.decode(eventData);
      return event;
    });
    const isNotNull = <T>(value: T | null): value is T => {
      return value !== null;
    };
    return events.filter(isNotNull);
  } else {
    return [];
  }
};

const isEventName = (
  eventName: string
): eventName is keyof anchor.IdlEvents<LmaofunBondingCurve> => {
  return validEventNames.includes(
    eventName as keyof anchor.IdlEvents<LmaofunBondingCurve>
  );
};

export const toEvent = <E extends EventKeys>(
  eventName: E,
  event: any
): anchor.IdlEvents<LmaofunBondingCurve>[E] | null => {
  if (isEventName(eventName)) {
    return getEvent(eventName, event.data);
  }
  return null;
};

const getEvent = <E extends EventKeys>(
  eventName: E,
  event: anchor.IdlEvents<LmaofunBondingCurve>[E]
): anchor.IdlEvents<LmaofunBondingCurve>[E] => {
  return event;
};

export const getTxDetails = async (connection: anchor.web3.Connection, sig: string) => {
  const latestBlockHash = await connection.getLatestBlockhash("processed");

  await connection.confirmTransaction(
    {
      blockhash: latestBlockHash.blockhash,
      lastValidBlockHeight: latestBlockHash.lastValidBlockHeight,
      signature: sig,
    },
    "confirmed"
  );

  return await connection.getTransaction(sig, {
    maxSupportedTransactionVersion: 0,
    commitment: "confirmed",
  });
};
