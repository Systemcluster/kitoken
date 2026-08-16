//! Byte-key hash map with length-sharding.

use alloc::vec::Vec;
use core::iter::FromIterator;

use hashbrown::HashMap;

use crate::TokenBytes;

#[derive(Clone)]
pub(crate) struct ByteMap<V> {
    map: Vec<HashMap<TokenBytes, V>>,
    len: usize,
}
impl<V: Clone + Copy> ByteMap<V> {
    #[inline(always)]
    pub(crate) fn get<'a, K: Into<&'a [u8]>>(&'a self, key: K) -> Option<&'a V> {
        let key = key.into();
        self.map.get(key.len()).and_then(|m| m.get(key))
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        self.len
    }

    #[inline(always)]
    pub(crate) fn iter(&self) -> impl Iterator<Item = (&[u8], V)> + '_ {
        self.map.iter().flatten().map(|(k, v)| (k.as_slice(), *v))
    }

    #[inline(always)]
    pub(crate) fn keys(&self) -> impl Iterator<Item = &TokenBytes> + '_ {
        self.map.iter().flatten().map(|(v, _)| v)
    }
}
impl<V> FromIterator<(TokenBytes, V)> for ByteMap<V> {
    fn from_iter<T: IntoIterator<Item = (TokenBytes, V)>>(iter: T) -> Self {
        let entries = iter.into_iter().collect::<Vec<_>>();
        let max = entries.iter().map(|(key, _)| key.len()).max().unwrap_or_default();
        let mut map = Vec::with_capacity(max + 1);
        map.resize_with(max + 1, || HashMap::with_capacity(0));
        for (key, value) in entries {
            map[key.len()].insert(key, value);
        }
        let len = map.iter().map(HashMap::len).sum();
        Self { map, len }
    }
}
