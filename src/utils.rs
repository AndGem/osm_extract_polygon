use std::collections::{HashMap, HashSet};

pub fn hashmap_values_to_set<K, V: std::hash::Hash + std::cmp::Eq + std::clone::Clone>(
    map: &HashMap<K, Vec<V>>,
) -> HashSet<V> {
    map.iter().flat_map(|(_, v)| v.clone()).collect()
}
