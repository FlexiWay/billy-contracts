use crate::util::bps_mul;
use crate::util::BASIS_POINTS_DIVISOR;
use anchor_lang::prelude::*;

use std::vec::Vec;

#[derive(AnchorSerialize, AnchorDeserialize, InitSpace, Clone, Debug)]
pub enum CurveType {
    Constant,
    Linear,
    Exponential,
}

#[derive(AnchorSerialize, AnchorDeserialize, InitSpace, Clone, Debug)]
pub struct CurveSegmentDef {
    pub curve_type: CurveType,
    pub start_supply_bps: u64,
    pub end_supply_bps: u64,
    pub params: [u64; 3], // Parameters for the curve (meaning depends on curve type)
}

#[derive(AnchorSerialize, AnchorDeserialize, InitSpace, Clone, Debug)]
pub struct CurveSegment {
    pub curve_type: CurveType,
    pub start_supply: u64,
    pub end_supply: u64,
    pub params: [u64; 3], // Parameters for the curve (meaning depends on curve type)
}

pub trait CurveSegmentInput {
    fn is_valid(&self) -> bool;
    fn into_segment_data(&self, bonding_supply: u64) -> Vec<CurveSegment>;
}

impl CurveSegmentInput for Vec<CurveSegmentDef> {
    fn is_valid(&self) -> bool {
        // Must check that all end and supply bps are in consecutive order,
        // all add up to 10_000, there's no overlap, and no gaps
        // First start supply must be 0
        self.first().map(|segment| segment.start_supply_bps == 0).unwrap_or(false)
            && // Last end supply must be 10_000
            self.last().map(|segment| segment.end_supply_bps == BASIS_POINTS_DIVISOR).unwrap_or(false)
            && // All segments must be valid
            self.iter().all(|segment| {
                segment.start_supply_bps <= segment.end_supply_bps
                    && segment.start_supply_bps < BASIS_POINTS_DIVISOR
                    && segment.end_supply_bps <= BASIS_POINTS_DIVISOR
            })
            && // No overlapping segments
            self.iter().enumerate().all(|(i, segment)| {
                let next_segment = self.get(i + 1);
                if let Some(next_segment) = next_segment {
                    segment.end_supply_bps == next_segment.start_supply_bps
                } else {
                    true
                }
            })
    }

    fn into_segment_data(&self, bonding_supply: u64) -> Vec<CurveSegment> {
        // Map each segment def basis points to actual tokens
        let mut segments = Vec::with_capacity(self.len());
        for segment in self.iter() {
            let start_supply = bps_mul(segment.start_supply_bps, bonding_supply).unwrap();
            let end_supply = bps_mul(segment.end_supply_bps, bonding_supply).unwrap();
            segments.push(CurveSegment {
                curve_type: segment.curve_type.clone(),
                start_supply,
                end_supply,
                params: segment.params,
            });
        }
        segments
    }
}
