#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::util::BASIS_POINTS_DIVISOR;
    use allocation::*;
    use anchor_lang::prelude::{msg, Clock, Pubkey};
    use curve::BondingCurve;
    use once_cell::sync::Lazy;
    use std::{
        ops::Mul,
        time::{SystemTime, UNIX_EPOCH},
    };
    use structs::CreateBondingCurveParams;
    static START_TIME: Lazy<i64> = Lazy::new(|| {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64
    });
    static SOL_LAUNCH_THRESHOLD: Lazy<u64> = Lazy::new(|| 70u64.mul(10u64.pow(9)));
    static CLOCK: Lazy<Clock> = Lazy::new(|| Clock {
        unix_timestamp: START_TIME.clone(),
        ..Clock::default()
    });

    static SIMPLE_SEGMENTS: Lazy<Vec<CurveSegmentDef>> = Lazy::new(|| {
        vec![CurveSegmentDef {
            segment_type: SegmentType::Exponential(2, 3, 5),
            start_supply_bps: 0,
            end_supply_bps: BASIS_POINTS_DIVISOR,
        }]
    });
    #[test]
    fn test_buy_and_sell_too_much() {
        let creator = Pubkey::default();
        let mint = Pubkey::default();
        let allocation = AllocationDataParams::default();

        let params = CreateBondingCurveParams {
            curve_segments: SIMPLE_SEGMENTS.to_vec(),
            name: "test".to_string(),
            symbol: "test".to_string(),
            uri: "test".to_string(),
            start_time: Some(*START_TIME),

            token_total_supply: 5000,
            sol_launch_threshold: *SOL_LAUNCH_THRESHOLD,

            // virtual_token_multiplier_bps: 730,
            // virtual_sol_reserves: 600,
            allocation,
            vesting_terms: None,
        };
        let mut curve =
            BondingCurve::create_from_params(mint, creator, creator, creator, &params, &CLOCK, 0);

        let curve_initial = curve.clone();

        println!("curve_initial: {:#?}", curve_initial);

        // Attempt to buy more tokens than available in reserves
        let buy_result = curve.apply_buy(2000).unwrap();

        println!("buy_result: {:#?}", curve);
        println!("{:?} \n", buy_result);
        assert_eq!(buy_result.token_amount, 189); // Adjusted based on available tokens
        assert_eq!(buy_result.sol_amount, 2000);
        assert_eq!(
            curve.real_token_reserves,
            curve_initial.real_token_reserves - buy_result.token_amount
        );
        assert_eq!(
            curve.real_sol_reserves,
            curve_initial.real_sol_reserves + buy_result.sol_amount
        );
        // assert_eq!(
        //     curve.virtual_token_reserves,
        //     curve_initial.virtual_token_reserves - buy_result.token_amount as u128
        // );
        // assert_eq!(
        //     curve.virtual_sol_reserves,
        //     curve_initial.virtual_sol_reserves + buy_result.sol_amount
        // );
        println!("{} \n", curve);
        println!("{:?} \n", buy_result);

        // Attempt to sell more tokens than available in reserves
        let sell_result = curve.apply_sell(2000);
        println!("{} \n", curve);
        println!("{:?} \n", sell_result);
        assert!(sell_result.is_none());
    }

    #[test]
    fn test_apply_sell() {
        let creator = Pubkey::default();
        let mint = Pubkey::default();
        let allocation = AllocationDataParams::default();

        let params = CreateBondingCurveParams {
            curve_segments: SIMPLE_SEGMENTS.to_vec(),
            name: "test".to_string(),
            symbol: "test".to_string(),
            uri: "test".to_string(),
            start_time: Some(*START_TIME),

            token_total_supply: 5000,
            sol_launch_threshold: *SOL_LAUNCH_THRESHOLD,

            allocation,
            vesting_terms: None,
        };
        let mut curve =
            BondingCurve::create_from_params(mint, creator, creator, creator, &params, &CLOCK, 0);
        // first apply buy
        curve.apply_buy(5000).unwrap();

        // let curve_initial = curve.clone();
        let result = curve.apply_sell(160).unwrap();
        println!("{:?} \n", result);
        assert_eq!(result.token_amount, 160);
        assert_eq!(result.sol_amount, 1688);
        assert_eq!(curve.real_token_reserves, 1187);
        // assert_eq!(curve.virtual_token_reserves, 89);
        // assert_eq!(curve.virtual_sol_reserves, 3981);
        assert_eq!(curve.real_sol_reserves, 3312);
    }

    #[test]
    fn test_get_sell_price() {
        let creator = Pubkey::default();
        let mint = Pubkey::default();
        let allocation = AllocationDataParams::default();

        let params = CreateBondingCurveParams {
            curve_segments: SIMPLE_SEGMENTS.to_vec(),
            name: "test".to_string(),
            symbol: "test".to_string(),
            uri: "test".to_string(),
            start_time: Some(*START_TIME),

            token_total_supply: 5000,
            sol_launch_threshold: *SOL_LAUNCH_THRESHOLD,

            // virtual_token_multiplier_bps: 730,
            // virtual_sol_reserves: 600,
            allocation,
            vesting_terms: None,
        };
        let mut curve =
            BondingCurve::create_from_params(mint, creator, creator, creator, &params, &CLOCK, 0);
        // first apply buy
        let res = curve.apply_buy(3000).unwrap();

        // let curve_initial = curve.clone();
        // Edge case: zero tokens
        assert_eq!(curve.get_sell_price(0), None);

        // Normal case -- some tokens are sold
        assert_eq!(curve.get_sell_price(res.token_amount - 100), Some(1942));

        // Normal case -- all tokens are sold, curve keeps spread as profit
        assert!(curve.get_sell_price(res.token_amount).unwrap() < res.sol_amount);

        // Should not exceed real sol reserves
        assert!(curve.get_sell_price(5000) <= Some(curve.real_sol_reserves));
    }

    #[test]
    fn test_apply_buy() {
        let creator = Pubkey::default();
        let mint = Pubkey::default();
        let allocation = AllocationDataParams::default();

        let params = CreateBondingCurveParams {
            curve_segments: SIMPLE_SEGMENTS.to_vec(),
            name: "test".to_string(),
            symbol: "test".to_string(),
            uri: "test".to_string(),
            start_time: Some(*START_TIME),

            token_total_supply: 5000,
            sol_launch_threshold: *SOL_LAUNCH_THRESHOLD,

            // virtual_token_multiplier_bps: 730,
            // virtual_sol_reserves: 600,
            allocation,
            vesting_terms: None,
        };
        let mut curve =
            BondingCurve::create_from_params(mint, creator, creator, creator, &params, &CLOCK, 0);
        let curve_initial = curve.clone();

        let purchase_amount = 1000;

        let result = curve.apply_buy(purchase_amount).unwrap();
        println!("{:?} \n", result);
        assert_eq!(result.sol_amount, purchase_amount);
        assert_eq!(result.token_amount, 94);
        // assert_eq!(
        //     curve.virtual_token_reserves,
        //     curve_initial.virtual_token_reserves - result.token_amount as u128
        // );
        // assert_eq!(curve.virtual_sol_reserves, 700); // Adjusted based on purchased SOL
        assert_eq!(
            curve.real_token_reserves,
            curve_initial.real_token_reserves - result.token_amount
        );
        assert_eq!(curve.real_sol_reserves, purchase_amount); // Adjusted based on purchased SOL
    }

    #[test]
    fn test_get_buy_price() {
        let creator = Pubkey::default();
        let mint = Pubkey::default();
        let allocation = AllocationDataParams::default();

        let params = CreateBondingCurveParams {
            curve_segments: SIMPLE_SEGMENTS.to_vec(),
            name: "test".to_string(),
            symbol: "test".to_string(),
            uri: "test".to_string(),
            start_time: Some(*START_TIME),

            token_total_supply: 5000,
            sol_launch_threshold: *SOL_LAUNCH_THRESHOLD,

            // virtual_token_multiplier_bps: 730,
            // virtual_sol_reserves: 600,
            allocation,
            vesting_terms: None,
        };
        let curve =
            BondingCurve::create_from_params(mint, creator, creator, creator, &params, &CLOCK, 0);
        // let _curve_initial = curve.clone();
        assert_eq!(curve.get_buy_price(0), None);

        // Normal case
        assert_eq!(curve.get_buy_price(500), Some(5278));

        // Edge case: very large token amount
        assert_eq!(curve.get_buy_price(2000), None);
    }

    #[test]
    fn test_get_tokens_for_buy_sol() {
        let creator = Pubkey::default();
        let mint = Pubkey::default();
        let allocation = AllocationDataParams::default();

        let params = CreateBondingCurveParams {
            curve_segments: SIMPLE_SEGMENTS.to_vec(),
            name: "test".to_string(),
            symbol: "test".to_string(),
            uri: "test".to_string(),
            start_time: Some(*START_TIME),

            token_total_supply: 5000,
            sol_launch_threshold: *SOL_LAUNCH_THRESHOLD,

            allocation,
            vesting_terms: None,
        };
        let curve =
            BondingCurve::create_from_params(mint, creator, creator, creator, &params, &CLOCK, 0);

        // Test case 1: Normal case
        assert_eq!(curve.get_tokens_for_buy_sol(1000), Some(94));

        // Test case 2: Edge case - zero SOL
        assert_eq!(curve.get_tokens_for_buy_sol(0), None);

        // Test case 4: Large SOL amount (but within limits)
        assert_eq!(curve.get_tokens_for_buy_sol(3000), Some(284));

        // Test case 5: SOL amount that would exceed real token reserves
        assert_eq!(
            curve.get_tokens_for_buy_sol(900000),
            Some(curve.supply_allocation.bonding_supply)
        );
    }

    #[test]
    fn test_get_tokens_for_sell_sol() {
        let creator = Pubkey::default();
        let mint = Pubkey::default();
        let allocation = AllocationDataParams::default();

        let params = CreateBondingCurveParams {
            curve_segments: SIMPLE_SEGMENTS.to_vec(),
            name: "test".to_string(),
            symbol: "test".to_string(),
            uri: "test".to_string(),
            start_time: Some(*START_TIME),

            token_total_supply: 5000,
            sol_launch_threshold: *SOL_LAUNCH_THRESHOLD,

            // virtual_token_multiplier_bps: 730,
            // virtual_sol_reserves: 600,
            allocation,
            vesting_terms: None,
        };
        let mut curve =
            BondingCurve::create_from_params(mint, creator, creator, creator, &params, &CLOCK, 0);
        // let _curve_initial = curve.clone();
        // first apply buy
        curve.apply_buy(1000).unwrap();

        // Test case 1: Normal case
        assert_eq!(curve.get_tokens_for_sell_sol(100), Some(9));

        // Test case 2: Edge case - zero SOL
        assert_eq!(curve.get_tokens_for_sell_sol(0), None);

        // Test case 3: Edge case - more SOL than virtual reserves
        assert_eq!(curve.get_tokens_for_sell_sol(1001), None);

        // Test case 4: Large SOL amount (but within limits)
        assert_eq!(curve.get_tokens_for_sell_sol(500), Some(47));
    }

    // FUZZ TESTS
    use proptest::prelude::*;

    // proptest! {
    //     #![proptest_config(ProptestConfig::with_cases(10000))]

    //     #[test]
    //     fn fuzz_test_default_alloc_simple_curve_apply_buy(
    //         token_total_supply in 1..u64::MAX,
    //         sol_amount in 1..u64::MAX,
    //         // virtual_token_reserves in 1..u64::MAX,
    //         // real_sol_reserves in 1..u64::MAX,
    //         // initial_virtual_token_reserves in 1..u64::MAX,
    //     ) {
    //         let creator = Pubkey::default();
    //         let mint = Pubkey::default();
    //         let allocation = AllocationDataParams::default();

    //         let params = CreateBondingCurveParams {
    //             curve_segments:SIMPLE_SEGMENTS.to_vec(),
    //             name: "test".to_string(),
    //             symbol: "test".to_string(),
    //             uri: "test".to_string(),
    //             start_time: Some(*START_TIME),

    //             token_total_supply,
    //             sol_launch_threshold: *SOL_LAUNCH_THRESHOLD,

    //             // virtual_token_multiplier_bps,
    //             // virtual_sol_reserves,

    //             allocation,
    //             vesting_terms:None,
    //         };
    //             let mut curve = BondingCurve::create_from_params(mint,creator,creator, creator, &params, &CLOCK, 0);
    //         let _curve_initial = curve.clone();

    //         if let Some(result) = curve.apply_buy(sol_amount) {
    //             prop_assert!(result.token_amount <= _curve_initial.real_token_reserves, "Token amount bought should not exceed real token reserves");
    //         }
    //     }

    //     #[test]
    //     fn fuzz_test_default_alloc_simple_curve_apply_sell(
    //         token_total_supply in 1..u64::MAX,

    //         token_amount in 1..u64::MAX,
    //         buy_sol_amount in 1..u64::MAX,
    //         // virtual_token_reserves in 1..u64::MAX,
    //         // real_sol_reserves in 1..u64::MAX,
    //         // initial_virtual_token_reserves in 1..u64::MAX,
    //     ) {
    //         let creator = Pubkey::default();
    //         let mint = Pubkey::default();
    //         let allocation = AllocationDataParams::default();

    //         let params = CreateBondingCurveParams {
    //             curve_segments:SIMPLE_SEGMENTS.to_vec(),
    //             name: "test".to_string(),
    //             symbol: "test".to_string(),
    //             uri: "test".to_string(),
    //             start_time: Some(*START_TIME),

    //             token_total_supply,
    //             sol_launch_threshold: *SOL_LAUNCH_THRESHOLD,

    //             // virtual_token_multiplier_bps,
    //             // virtual_sol_reserves,

    //             allocation,
    //             vesting_terms:None,
    //         };
    //             let mut curve = BondingCurve::create_from_params(mint,creator,creator, creator, &params, &CLOCK, 0);
    //         let buy_result = curve.apply_buy(buy_sol_amount);
    //         if buy_result.is_none() {
    //             return Ok(())
    //         }
    //         let _curve_after_buy = curve.clone();
    //         if let Some(result) = curve.apply_sell(token_amount) {
    //             prop_assert!(result.sol_amount <= _curve_after_buy.real_sol_reserves, "SOL amount to send to seller should not exceed real SOL reserves");
    //         }
    //     }

    // }
}
