use arbitrary::{Arbitrary, Unstructured};
use std::mem::size_of;

pub mod opcodes;

#[derive(Debug)]
pub struct ArbitraryVec<const MAX: usize>(pub Vec<u8>);

impl<'a, const MAX: usize> Arbitrary<'a> for ArbitraryVec<MAX> {
    fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        let length = u.arbitrary::<usize>()? % MAX;
        Ok(Self(u.arbitrary_iter()?.take(length).collect::<Result<Vec<u8>, _>>()?))
    }

    fn size_hint(_: usize) -> (usize, Option<usize>) {
        (size_of::<usize>(), Some(MAX + size_of::<usize>()))
    }
}