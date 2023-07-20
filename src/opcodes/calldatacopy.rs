use crate::ArbitraryVec;
use arbitrary::Arbitrary;

pub const MAX_CALLDATA_LENGTH: usize = 64;

#[derive(Arbitrary, Debug)]
pub struct CalldataCopyRootArgs {
    pub calldata: ArbitraryVec<MAX_CALLDATA_LENGTH>,
    pub length: [u8; 32],
    pub data_offset: [u8; 32],
    pub memory_offset: [u8; 32],
}