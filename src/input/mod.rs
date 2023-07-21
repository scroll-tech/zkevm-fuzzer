use eth_types::Word;
use rand::{Rng, RngCore};
use serde::{Deserialize, Serialize};

pub mod opcodes;

#[derive(Debug, Serialize, Deserialize)]
pub struct RandomBytes<const MAX: usize>(pub Vec<u8>);

pub trait FromRng: Sized {
    fn from_rng<R: Rng>(rng: &mut R) -> Self;
}

impl<const MAX: usize> FromRng for RandomBytes<MAX> {
    fn from_rng<R: Rng>(rng: &mut R) -> Self {
        let mut bytes = vec![0u8; MAX];
        rng.fill_bytes(&mut bytes);
        Self(bytes)
    }
}

impl FromRng for Word {
    fn from_rng<R: Rng>(rng: &mut R) -> Self {
        let mut buf = [0u8; 32];
        rng.fill_bytes(&mut buf);
        Word::from_big_endian(&buf)
    }
}
