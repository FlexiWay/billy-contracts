import { Context, Pda } from '@metaplex-foundation/umi';
import { string } from "@metaplex-foundation/umi/serializers";
import { LMAOFUN_BONDING_CURVE_PROGRAM_ID } from './generated/programs/lmaofunBondingCurve';

export function findEvtAuthorityPda(
    context: Pick<Context, 'eddsa' | 'programs'>,
    ): Pda {
    const programId = context.programs.getPublicKey('bondingCurve', LMAOFUN_BONDING_CURVE_PROGRAM_ID);
    return context.eddsa.findPda(programId, [
                    string({ size: 'variable' }).serialize("__event_authority"),
              ]);
  }
