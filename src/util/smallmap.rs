pub struct SmallMap<K, V>(Vec<(K, V)>);
impl<K: PartialEq, V> SmallMap<K, V> {
    pub fn new() -> Self {
        Self(vec![])
    }
    pub fn insert(&mut self, key: K, value: V) {
        self.0.push((key, value))
    }
    pub fn get(&self, key: &K) -> Option<&V> {
        self.0.iter().find_map(|(k, v)| (k == key).then_some(v))
    }
    pub fn len(&self) -> usize {
        self.0.len()
    }
}
