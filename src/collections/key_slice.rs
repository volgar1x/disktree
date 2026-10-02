use std::{ops::Deref, slice};

use crate::KeyRef;

#[derive(Debug)]
pub struct KeySlice([KeyRef]);

impl Deref for KeySlice {
    type Target = [KeyRef];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl KeySlice {
    pub fn new_unchecked(keys: &[KeyRef]) -> &Self {
        unsafe { std::mem::transmute(keys) }
    }

    pub fn contains(&mut self, key: impl AsRef<KeyRef>) -> bool {
        let key = key.as_ref();
        self.0.binary_search_by(|item| item.cmp(key)).is_ok()
    }
}

impl<'a> IntoIterator for &'a KeySlice {
    type Item = &'a KeyRef;
    type IntoIter = slice::Iter<'a, KeyRef>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}
