mod binary_search;

use std::{borrow::Cow, slice};

use crate::{
    Item, Key, Value,
    collections::{
        GetByKey, InsertItem, ItemSlice, ItemVec, KeySlice, item_store::binary_search::SearchResult,
    },
    functional::Setter,
    item::{ITEM_LENGTH, ItemRef},
};

pub struct ItemStore<'a>(Vec<Cow<'a, [ItemRef]>>);

impl<'a, 'b> IntoIterator for &'b ItemStore<'a>
where
    'a: 'b,
{
    type Item = &'b ItemRef;
    type IntoIter = std::iter::FlatMap<
        std::slice::Iter<'b, Cow<'a, [ItemRef]>>,
        std::slice::Iter<'b, ItemRef>,
        fn(&'b Cow<'a, [ItemRef]>) -> std::slice::Iter<'b, ItemRef>,
    >;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter().flat_map(|group| group.iter())
    }
}

impl<'a> ItemStore<'a> {
    pub fn with_chunk_size(items: &'a ItemSlice, chunk_size: usize) -> Self {
        let items = items.chunks(chunk_size).map(Cow::Borrowed).collect();
        Self(items)
    }

    pub fn new(items: &'a ItemSlice) -> Self {
        // 0x200 * ITEM_LENGTH <= 0x8000
        Self::with_chunk_size(items, 0x200)
    }

    pub fn into_chunks(self) -> impl Iterator<Item = ItemStoreChunk<'a>> {
        self.0.into_iter().map(|chunk| match chunk {
            Cow::Borrowed(chunk) => ItemStoreChunk::Borrowed(ItemSlice::new(chunk)),
            Cow::Owned(chunk) => ItemStoreChunk::Owned(chunk),
        })
    }
}

pub enum ItemStoreChunk<'a> {
    Borrowed(&'a ItemSlice),
    Owned(Vec<ItemRef>),
}

impl ItemStore<'_> {
    pub fn chunks(&self) -> impl Iterator<Item = &ItemSlice> {
        self.0.iter().map(|group| ItemSlice::new(group.as_ref()))
    }

    pub fn items(&self) -> impl Iterator<Item = &ItemRef> {
        self.into_iter()
    }
}

impl GetByKey for ItemStore<'_> {
    fn is_empty(&self) -> bool {
        self.0.iter().all(|group| group.is_empty())
    }

    fn len(&self) -> usize {
        self.0.iter().map(|group| group.len()).sum()
    }

    fn keys(&self) -> impl Iterator<Item = &Key> {
        self.0
            .iter()
            .flat_map(|group| group.iter())
            .map(|item| item.key())
    }

    fn contains(&self, key: &Key) -> bool {
        self.binary_search(key, None).is_found()
    }

    fn get(&self, key: &Key) -> Option<Value> {
        if let SearchResult::Found { index, group_index } = self.binary_search(key, None) {
            Some(self.0[index][group_index].value())
        } else {
            None
        }
    }
}

impl InsertItem for ItemStore<'_> {
    fn insert(&mut self, key: Key, value: Value) -> bool {
        if let Some(mut setter) = self.insert_mut(key) {
            setter.set(value);
            true
        } else {
            false
        }
    }

    fn insert_mut(&mut self, key: Key) -> Option<impl Setter<Value>> {
        if self.0.is_empty() {
            let group = self.0.push_mut(Cow::Owned(vec![])).to_mut();
            let item = group.push_mut(ItemRef::new(key, Value::default()));
            Some(item)
        } else {
            match self.binary_search(&key, None) {
                SearchResult::Found { .. } => None,
                SearchResult::GroupFound { index, group_index } => {
                    let group = self.0[index].to_mut();
                    let item = group.insert_mut(group_index, ItemRef::new(key, Value::default()));
                    Some(item)
                }
                SearchResult::NotFound { index } => match index.checked_sub(1) {
                    Some(index) => {
                        let group = self.0[index].to_mut();
                        let item = group.push_mut(ItemRef::new(key, Value::default()));
                        Some(item)
                    }
                    None => {
                        let group = self.0[0].to_mut();
                        let item = group.insert_mut(0, ItemRef::new(key, Value::default()));
                        Some(item)
                    }
                },
            }
        }
    }
}

impl<'a> ItemStore<'a> {
    pub fn append<'b>(
        &'b mut self,
        items: ItemVec,
        new_offset: u32,
    ) -> impl Iterator<Item = (Key, Value)>
    where
        'a: 'b,
    {
        ItemStoreAppender {
            store: self,
            items: items.into_iter(),
            prev_needle: None,
            offset: new_offset,
        }
    }
}

impl ItemStore<'_> {
    #[cfg(test)]
    fn debug_sorted(&self, index: usize, group_index: usize) {
        let cur_item = self.0[index][group_index].key();
        let prev_item = self.0[index]
            .get(group_index - 1)
            .map(|item| item.key())
            .or_else(|| Some(self.0.get(index - 1)?.last()?.key()));
        let next_item = self.0[index]
            .get(group_index + 1)
            .map(|item| item.key())
            .or_else(|| Some(self.0.get(index + 1)?.first()?.key()));
        let ctx_items = [prev_item, Some(cur_item), next_item];

        let items = self.0.iter().enumerate().flat_map(|(index, group)| {
            group
                .iter()
                .enumerate()
                .map(move |(group_index, item)| (index, group_index, item))
        });
        let tmp0 = items.clone().find_map(|(index, group_index, item)| {
            (item.key() >= cur_item).then_some((index, group_index))
        });
        let tmp1 = tmp0.and_then(|(index, group_index)| {
            group_index.checked_sub(1).map_or_else(
                || self.0.get(index - 1)?.last(),
                |group_index| Some(&self.0[index][group_index]),
            )
        });

        assert!(
            self.keys().is_sorted(),
            "items={ctx_items:?} needle={:?} tmp0={tmp0:?} tmp1={tmp1:?}",
            (index, group_index)
        );
    }
}

impl<'a> ItemStore<'a> {
    pub fn remove(&mut self, key: &Key) -> Option<Value> {
        let SearchResult::Found { index, group_index } = self.binary_search(key, None) else {
            return None;
        };
        let group = self.0[index].to_mut();
        let item = group.remove(group_index);
        if group.is_empty() {
            self.0.remove(index);
        }
        Some(item.value())
    }

    pub fn remove_all<'b>(
        &'b mut self,
        keys: &'b KeySlice,
    ) -> impl Iterator<Item = (Key, Value)> + use<'a, 'b> {
        ItemStoreRemover {
            store: self,
            keys: keys.iter(),
            prev_needle: None,
        }
    }

    pub fn into_bytes(self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(self.len() * ITEM_LENGTH);
        for group in self.0 {
            let items = ItemSlice::new(&group);
            bytes.extend_from_slice(items.as_bytes());
        }
        bytes
    }
}

#[must_use]
struct ItemStoreAppender<'a, 'b> {
    store: &'b mut ItemStore<'a>,
    items: std::vec::IntoIter<Item>,
    prev_needle: Option<(usize, usize)>,
    offset: u32,
}

impl<'a, 'b> Iterator for ItemStoreAppender<'a, 'b> {
    type Item = (Key, Value);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let mut item = self.items.next()?;

            if self.store.is_empty() {
                let key = item.key.clone();
                let prev_value = item.value;
                item.value.offset = self.offset;
                self.offset += item.value.length;
                self.store.insert(item.key, item.value);
                return Some((key, prev_value));
            }

            match self.store.binary_search(&item.key, self.prev_needle) {
                SearchResult::Found { index, group_index } => {
                    self.prev_needle = Some((index, group_index));
                    continue;
                }

                SearchResult::GroupFound { index, group_index } => {
                    let key = item.key.clone();
                    let prev_value = item.value;
                    item.value.offset = self.offset;
                    self.offset += item.value.length;

                    let group = self.store.0[index].to_mut();
                    group.insert(group_index, item.into_ref());
                    let needle = (index, group_index);
                    self.prev_needle = Some(needle);

                    #[cfg(test)]
                    self.store.debug_sorted(index, group_index);

                    return Some((key, prev_value));
                }

                SearchResult::NotFound { index } => {
                    let key = item.key.clone();
                    let prev_value = item.value;
                    item.value.offset = self.offset;
                    self.offset += item.value.length;

                    match index.checked_sub(1) {
                        Some(index) => {
                            let group = self.store.0[index].to_mut();
                            let group_index = group.len();
                            group.push(item.into_ref());
                            self.prev_needle = Some((index, group_index));

                            #[cfg(test)]
                            self.store.debug_sorted(index, group_index);
                        }
                        None => {
                            let group = self.store.0[0].to_mut();
                            let group_index = 0;
                            group.insert(0, item.into_ref());
                            self.prev_needle = Some((index, group_index));

                            #[cfg(test)]
                            self.store.debug_sorted(index, group_index);
                        }
                    }

                    return Some((key, prev_value));
                }
            }
        }
    }
}

#[must_use]
struct ItemStoreRemover<'a, 'b> {
    store: &'b mut ItemStore<'a>,
    keys: slice::Iter<'b, Key>,
    prev_needle: Option<(usize, usize)>,
}

impl<'a, 'b> Iterator for ItemStoreRemover<'a, 'b> {
    type Item = (Key, Value);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let key = self.keys.next()?;

            let SearchResult::Found { index, group_index } =
                self.store.binary_search(key, self.prev_needle)
            else {
                continue;
            };

            let group = self.store.0[index].to_mut();
            let item = group.remove(group_index);
            if group.is_empty() {
                self.store.0.remove(index);
            }

            self.prev_needle = Some((index, group_index));

            return Some((item.key().to_owned(), item.value()));
        }
    }
}
