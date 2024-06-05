use std::collections::{HashMap, HashSet};
use std::hash::Hash;

pub fn values_to_set<K, V>(map: &HashMap<K, Vec<V>>) -> HashSet<V>
where
    V: Hash + Eq + Clone,
{
    map.values().flat_map(|v| v.iter().cloned()).collect()
}
