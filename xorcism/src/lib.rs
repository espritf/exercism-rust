use std::{borrow::Borrow, usize};

/// A munger which XORs a key with some data
#[derive(Clone)]
pub struct Xorcism<'a> {
    key: &'a [u8],
    index: usize,
}

// TODO why self.next() is not works?
fn next(key: &[u8], index: &mut usize) -> u8 {
    let r = key[*index];
    *index = (*index + 1) % key.len();
    r
}

impl<'a> Xorcism<'a> {
    /// Create a new Xorcism munger from a key
    ///
    /// Should accept anything which has a cheap conversion to a byte slice.
    pub fn new<Key>(key: &'a Key) -> Xorcism<'a> 
        where 
            Key: AsRef<[u8]> + ?Sized,
    {
        Self { key: key.as_ref(), index: 0 }
    }

    /// XOR each byte of the input buffer with a byte from the key.
    ///
    /// Note that this is stateful: repeated calls are likely to produce different results,
    /// even with identical inputs.
    pub fn munge_in_place(&mut self, data: &mut [u8]) {
        for d in data.iter_mut() {
            *d ^= next(&self.key, &mut self.index);
        }
    }

    /// XOR each byte of the data with a byte from the key.
    ///
    /// Note that this is stateful: repeated calls are likely to produce different results,
    /// even with identical inputs.
    ///
    /// Should accept anything which has a cheap conversion to a byte iterator.
    /// Shouldn't matter whether the byte iterator's values are owned or borrowed.
    pub fn munge<'b, Data>(&'b mut self, data: Data) -> impl Iterator<Item = u8> + 'b
        where
            Data: IntoIterator + 'b,
            Data::Item: Borrow<u8>,
    {
        data.into_iter().map(|b| {
            b.borrow() ^ next(&self.key, &mut self.index)
        })
    }
}
