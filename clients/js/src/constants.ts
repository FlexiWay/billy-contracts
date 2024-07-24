// import { InitializeInstructionArgs } from "./generated";
import { LAMPORTS_PER_SOL } from "@solana/web3.js";
import BN from "bn.js";
import { ProgramStatus } from "./generated";
export const TOKEN_DECIMALS = 6;

export const DECIMALS_MULTIPLIER = 10 ** TOKEN_DECIMALS;

export const DEFAULT_TOKEN_SUPPLY= 1_000* 1_000_000 * DECIMALS_MULTIPLIER;

export const INIT_DEFAULTS={
    initialRealSolReserves: 0,
    initialRealTokenReserves: DEFAULT_TOKEN_SUPPLY,
    initialVirtualSolReserves: 30 * LAMPORTS_PER_SOL,
    initialVirtualTokenReserves: 1_073 * 1_000_000 * DECIMALS_MULTIPLIER,
    initialTokenSupply:DEFAULT_TOKEN_SUPPLY,
    solLaunchThreshold: 100*LAMPORTS_PER_SOL,
    tradeFeeBps: 100,
    launchFeeLamports: 0.5*LAMPORTS_PER_SOL,
    createdMintDecimals: TOKEN_DECIMALS,

    status: ProgramStatus.Running,
}

export const INIT_DEFAULTS_ANCHOR={
    initialRealSolReserves: new BN(0),
    initialRealTokenReserves: new BN(DEFAULT_TOKEN_SUPPLY),
    initialVirtualSolReserves: new BN(30 * LAMPORTS_PER_SOL),
    initialVirtualTokenReserves: new BN(1_073 * 1_000_000 * DECIMALS_MULTIPLIER),
    initialTokenSupply:new BN(DEFAULT_TOKEN_SUPPLY),
    solLaunchThreshold: new BN(100*LAMPORTS_PER_SOL),
    tradeFeeBps: 100,
    launchFeeLamports: new BN(0.5*LAMPORTS_PER_SOL),
    createdMintDecimals: TOKEN_DECIMALS,

    status: ProgramStatus.Running,
}
