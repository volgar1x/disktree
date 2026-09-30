use std::{fmt, ops::Deref};

use crate::{
    Item, Key, Value,
    collections::{GetByKey, InsertItem},
    functional::Setter,
    item::ITEM_LENGTH,
};

#[derive(Clone)]
pub struct ItemVec(pub(crate) Vec<Item>);

impl FromIterator<Item> for ItemVec {
    fn from_iter<T: IntoIterator<Item = Item>>(iter: T) -> Self {
        let mut vec = iter.into_iter().collect::<Vec<_>>();
        vec.sort_unstable_by(|a, b| a.key.cmp(&b.key));
        vec.dedup_by(|a, b| a.key == b.key);
        Self(vec)
    }
}

impl GetByKey for ItemVec {
    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    fn len(&self) -> usize {
        self.0.len()
    }

    fn keys(&self) -> impl Iterator<Item = &Key> {
        self.0.iter().map(|item| &item.key)
    }

    fn contains(&self, key: &Key) -> bool {
        self.0.binary_search_by(|item| item.key.cmp(key)).is_ok()
    }

    fn get(&self, key: &Key) -> Option<Value> {
        if let Ok(index) = self.0.binary_search_by(|item| item.key.cmp(key)) {
            Some(self.0[index].value)
        } else {
            None
        }
    }
}

impl InsertItem for ItemVec {
    fn insert(&mut self, key: Key, value: Value) -> bool {
        let Err(index) = self.0.binary_search_by(|item| item.key.cmp(&key)) else {
            return false;
        };

        self.0.insert(index, Item { key, value });
        true
    }

    fn insert_mut(&mut self, key: Key) -> Option<impl Setter<Value>> {
        let Err(index) = self.0.binary_search_by(|item| item.key.cmp(&key)) else {
            return None;
        };

        let item = self.0.insert_mut(
            index,
            Item {
                key,
                value: Value::default(),
            },
        );
        Some(&mut item.value)
    }
}

impl fmt::Debug for ItemVec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_map()
            .entries(self.0.iter().map(|item| (&item.key, &item.value)))
            .finish()
    }
}

impl ItemVec {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn with_capacity(cap: usize) -> Self {
        Self(Vec::with_capacity(cap))
    }

    pub fn into_bytes(self) -> Box<[u8]> {
        let mut bytes = Box::new_uninit_slice(self.0.len() * ITEM_LENGTH);
        for (index, item) in self.0.into_iter().enumerate() {
            unsafe {
                bytes
                    .as_mut_ptr()
                    .add(index * ITEM_LENGTH)
                    .cast::<[u8; ITEM_LENGTH]>()
                    .write(item.into_bytes());
            }
        }

        unsafe { bytes.assume_init() }
    }
}

impl Default for ItemVec {
    fn default() -> Self {
        Self::new()
    }
}

impl Deref for ItemVec {
    type Target = [Item];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl IntoIterator for ItemVec {
    type Item = Item;
    type IntoIter = std::vec::IntoIter<Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a> IntoIterator for &'a ItemVec {
    type Item = &'a Item;
    type IntoIter = std::slice::Iter<'a, Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}
