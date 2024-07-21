import { Context, Pda } from '@metaplex-foundation/umi';
import { string } from "@metaplex-foundation/umi/serializers";
import { BONDING_CURVE_PROGRAM_ID } from './generated/programs/bondingCurve';

export function findEvtAuthoritylPda(
    context: Pick<Context, 'eddsa' | 'programs'>,
    ): Pda {
    const programId = context.programs.getPublicKey('bondingCurve', BONDING_CURVE_PROGRAM_ID);
    return context.eddsa.findPda(programId, [
                    string({ size: 'variable' }).serialize("__eventAuthority"),
              ]);
  }
