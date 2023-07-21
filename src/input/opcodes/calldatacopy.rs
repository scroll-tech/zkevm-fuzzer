use crate::input::{FromRng, RandomBytes, RandomWord};
use rand::Rng;
use serde::{Deserialize, Serialize};

pub const MAX_CALLDATA_LENGTH: usize = 64;

#[derive(Debug, Serialize, Deserialize)]
pub struct CalldataCopyRootArgs {
    pub calldata: RandomBytes<MAX_CALLDATA_LENGTH>,
    pub length: RandomWord<0, {(MAX_CALLDATA_LENGTH * 2) as u64}>,
    pub data_offset: RandomWord<0, {(MAX_CALLDATA_LENGTH * 2) as u64}>,
    pub memory_offset: RandomWord<0, {(MAX_CALLDATA_LENGTH * 2) as u64}>,
}

impl FromRng for CalldataCopyRootArgs {
    fn from_rng<R: Rng>(rng: &mut R) -> Self {
        Self {
            calldata: RandomBytes::from_rng(rng),
            length: RandomWord::from_rng(rng),
            data_offset: RandomWord::from_rng(rng),
            memory_offset: RandomWord::from_rng(rng),
        }
    }
}
