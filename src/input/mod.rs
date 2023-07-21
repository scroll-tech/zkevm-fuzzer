use eth_types::Word;
use rand::Rng;
use serde::{Deserialize, Serialize};

pub mod opcodes;

#[derive(Debug, Serialize, Deserialize)]
pub struct RandomBytes<const MAX: usize>(pub Vec<u8>);

#[derive(Debug, Serialize, Deserialize)]
pub struct RandomWord<const MIN: u64, const MAX: u64>(pub Word);

pub trait FromRng: Sized {
    fn from_rng<R: Rng>(rng: &mut R) -> Self;
}

impl<const MAX: usize> FromRng for RandomBytes<MAX> {
    fn from_rng<R: Rng>(rng: &mut R) -> Self {
        let length = rng.gen::<usize>() % MAX;
        let mut bytes = Vec::with_capacity(length);
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

impl<const MIN: u64, const MAX: u64> FromRng for RandomWord<MIN, MAX> {
    fn from_rng<R: Rng>(rng: &mut R) -> Self {
        let v = rng.gen_range(MIN..MAX);
        Self(Word::from(v))
    }
}