// import { InitializeInstructionArgs } from "./generated";
import { LAMPORTS_PER_SOL } from "@solana/web3.js";
import BN from "bn.js";
import { none } from "@metaplex-foundation/umi";
import { CreateBondingCurveInstructionArgs, ProgramStatus } from "./generated";

import { AllocationData } from './generated/types/allocationData';


export const TOKEN_DECIMALS = 6;
export const INIT_ALLOCATIONS_PCS = {
    creator:10.0,
    cex:10.0,
    launchBrandkit:10.0,
    lifetimeBrandkit:10.0,
    platform:10.0,
    presale:0.0,
    poolReserve:50.0,
}

export const DECIMALS_MULTIPLIER = 10 ** TOKEN_DECIMALS;
export const TOKEN_SUPPLY_AMOUNT = 2_000* 1_000_000;
export const VIRTUAL_TOKEN_MULTIPLIER = 7.3 // +7.3%
export const DEFAULT_TOKEN_SUPPLY= TOKEN_SUPPLY_AMOUNT * DECIMALS_MULTIPLIER;
export const POOL_INITIAL_TOKEN_SUPPLY = DEFAULT_TOKEN_SUPPLY * INIT_ALLOCATIONS_PCS.poolReserve/100;

export const SIMPLE_DEFAULT_BONDING_CURVE_PRESET:CreateBondingCurveInstructionArgs ={
    name: "simpleBondingCurve",
    symbol: "SBC",
    uri: "https://www.simpleBondingCurve.com",

    // startTime: Date.now(),
    startTime: none(),
    tokenTotalSupply: DEFAULT_TOKEN_SUPPLY,
    solLaunchThreshold: 300 *LAMPORTS_PER_SOL,
    virtualTokenMultiplier: VIRTUAL_TOKEN_MULTIPLIER,
    virtualSolReserves: 30 * LAMPORTS_PER_SOL,
    allocation: INIT_ALLOCATIONS_PCS,

}

export const INIT_DEFAULTS={
    tradeFeeBps: 100,
    launchFeeLamports: 0.5*LAMPORTS_PER_SOL,
    createdMintDecimals: TOKEN_DECIMALS,

    status: ProgramStatus.Running,
}

export const INIT_DEFAULTS_ANCHOR={
    tradeFeeBps: 100,
    launchFeeLamports: new BN(0.5*LAMPORTS_PER_SOL),
    createdMintDecimals: TOKEN_DECIMALS,

    status: ProgramStatus.Running,
}
