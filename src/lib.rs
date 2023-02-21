use std::collections::hash_map::RandomState;
use std::hash::{BuildHasher, Hash, Hasher};

pub mod sparse_array;
pub use sparse_array::SparseArray;

#[derive(Clone, Default)]
pub struct Map<K, V, H: BuildHasher = RandomState> {
    table: SparseArray<(K, V)>,
    hasher_builder: H,
}

impl<K: Eq + Hash, V> Map<K, V> {
    fn raw_entry(&self, key: &K) -> (u64, Option<&V>) {
        let mut hasher = self.hasher_builder.build_hasher();
        key.hash(&mut hasher);
        let hash = hasher.finish();

        let mut index = hash;
        while let Some((k, v)) = self.table.get(index) {
            if k == key {
                return (index, Some(v));
            }
            index = index.wrapping_add(1);
            debug_assert_ne!(index, hash);
        }
        (index, None)
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        self.raw_entry(key).1
    }
}

impl<K: Clone + Eq + Hash, V: Clone> Map<K, V> {
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        let (index, _) = self.raw_entry(&key);
        self.table.set(index, (key, value)).map(|(_, v)| v)
    }
}
