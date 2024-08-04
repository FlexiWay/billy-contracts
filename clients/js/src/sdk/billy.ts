import { Pda, Program, PublicKey, Umi } from "@metaplex-foundation/umi";
import { createSplAssociatedTokenProgram, createSplTokenProgram } from '@metaplex-foundation/mpl-toolbox';
import { BILLY_BONDING_CURVE_PROGRAM_ID, createBillyBondingCurveProgram, fetchGlobal, findGlobalPda } from "../generated";
import { findEvtAuthorityPda } from "../utils";
import { AdminSDK } from "./admin";
import { CurveSDK } from "./curve";

export class BillySDK{
    umi:Umi;

    programId:PublicKey;

    program: Program;

    globalPda:Pda;

    evtAuthPda:Pda;

    evtAuthAccs:{
        eventAuthority:PublicKey,
        program:PublicKey
    }

    constructor (umi:Umi){
        const bondingCurveProgram = createBillyBondingCurveProgram();
        this.programId = BILLY_BONDING_CURVE_PROGRAM_ID;
        this.program = bondingCurveProgram;
        umi.programs.add(createSplAssociatedTokenProgram());
        umi.programs.add(createSplTokenProgram());
        umi.programs.add(bondingCurveProgram);
        this.umi = umi
        this.globalPda=findGlobalPda(this.umi);
        this.evtAuthPda = findEvtAuthorityPda(this.umi);
        this.evtAuthAccs = {
            eventAuthority: this.evtAuthPda[0],
            program: BILLY_BONDING_CURVE_PROGRAM_ID,
        };
    }

    fetchGlobalData(){
        return fetchGlobal(this.umi, this.globalPda);
    }

    getAdminSDK(){
        return new AdminSDK(this);
    }

    getCurveSDK(mint:PublicKey){
        return new CurveSDK(this, mint);
    }
}
