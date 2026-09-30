use crate::{Key, Value, functional::Setter};

mod item_slice;
mod item_store;
mod item_vec;
mod key_set;
mod key_slice;

pub use item_slice::ItemSlice;
pub use item_store::ItemStore;
pub use item_vec::ItemVec;
pub use key_set::KeySet;
pub use key_slice::KeySlice;

pub trait GetByKey {
    fn is_empty(&self) -> bool;
    fn len(&self) -> usize;
    fn keys(&self) -> impl Iterator<Item = &Key>;
    fn contains(&self, key: &Key) -> bool;
    fn get(&self, key: &Key) -> Option<Value>;
}

pub trait InsertItem {
    fn insert(&mut self, key: Key, value: Value) -> bool;
    fn insert_mut(&mut self, key: Key) -> Option<impl Setter<Value>>;
}
