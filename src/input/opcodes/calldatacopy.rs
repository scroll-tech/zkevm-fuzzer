use crate::input::{FromRng, RandomBytes};
use eth_types::Word;
use rand::Rng;
use serde::{Deserialize, Serialize};

pub const MAX_CALLDATA_LENGTH: usize = 64;

#[derive(Debug, Serialize, Deserialize)]
pub struct CalldataCopyRootArgs {
    pub calldata: RandomBytes<MAX_CALLDATA_LENGTH>,
    pub length: Word,
    pub data_offset: Word,
    pub memory_offset: Word,
}

impl FromRng for CalldataCopyRootArgs {
    fn from_rng<R: Rng>(rng: &mut R) -> Self {
        Self {
            calldata: RandomBytes::from_rng(rng),
            length: Word::from_rng(rng),
            data_offset: Word::from_rng(rng),
            memory_offset: Word::from_rng(rng),
        }
    }
}
