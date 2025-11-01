use std::borrow::Borrow;

#[derive(Debug)]
pub struct SmallMap<K, V>(Vec<(K, V)>);

impl<K: PartialEq, V> SmallMap<K, V> {
    pub const fn new() -> Self {
        Self(Vec::new())
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self(Vec::with_capacity(capacity))
    }

    pub fn insert(&mut self, key: K, value: V) {
        self.0.push((key, value))
    }

    pub fn get<Q: ?Sized>(&self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: PartialEq,
    {
        self.0.iter().find_map(|(k, v)| (k.borrow() == key).then_some(v))
    }

    pub fn get_mut<Q: ?Sized>(&mut self, key: &Q) -> Option<&mut V>
    where
        K: Borrow<Q>,
        Q: PartialEq,
    {
        self.0
            .iter_mut()
            .find_map(|(k, v)| ((&*k).borrow() == key).then_some(v))
    }

    pub fn contains_key<Q: ?Sized>(&self, key: &Q) -> bool
    where
        K: Borrow<Q>,
        Q: PartialEq,
    {
        self.0.iter().any(|(k, _)| k.borrow() == key)
    }

    pub fn remove<Q: ?Sized>(&mut self, key: &Q) -> Option<V>
    where
        K: Borrow<Q>,
        Q: PartialEq,
    {
        let pos = self.0.iter().position(|(k, _)| k.borrow() == key)?;
        Some(self.0.remove(pos).1)
    }

    pub fn keys(&self) -> impl Iterator<Item = &K> {
        self.0.iter().map(|(k, _)| k)
    }

    pub fn values(&self) -> impl Iterator<Item = &V> {
        self.0.iter().map(|(_, v)| v)
    }

    pub fn values_mut(&mut self) -> impl Iterator<Item = &mut V> {
        self.0.iter_mut().map(|(_, v)| v)
    }

    pub fn into_keys(self) -> Vec<K> {
        self.0.into_iter().map(|(k, _)| k).collect()
    }

    pub fn into_values(self) -> Vec<V> {
        self.0.into_iter().map(|(_, v)| v).collect()
    }

    pub const fn len(&self) -> usize {
        self.0.len()
    }

    pub const fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn clear(&mut self) {
        self.0.clear();
    }

    pub fn iter(&self) -> impl Iterator<Item = (&K, &V)> {
        self.0.iter().map(|(k, v)| (k, v))
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&K, &mut V)> {
        self.0.iter_mut().map(|(k, v)| (&*k, v))
    }
}
