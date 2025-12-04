use std::borrow::Borrow;

#[derive(Debug, Clone)]
pub struct SmallMap<K, V>(Vec<(K, V)>);

impl<K, V> Default for SmallMap<K, V> {
    fn default() -> Self {
        Self(vec![])
    }
}

#[allow(unused)]
impl<K: PartialEq, V> SmallMap<K, V> {
    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        Self(Vec::new())
    }

    #[inline]
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self(Vec::with_capacity(capacity))
    }

    #[inline]
    pub fn insert(&mut self, key: K, value: V) {
        self.0.push((key, value));
    }

    #[inline]
    pub fn insert_popular(&mut self, key: K, value: V) {
        self.0.insert(0, (key, value));
    }

    #[inline]
    #[must_use]
    pub fn get<Q>(&self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: ?Sized + PartialEq,
    {
        self.0
            .iter()
            .find_map(|(k, v)| (k.borrow() == key).then_some(v))
    }

    #[inline]
    #[must_use]
    pub fn get_mut<Q>(&mut self, key: &Q) -> Option<&mut V>
    where
        K: Borrow<Q>,
        Q: ?Sized + PartialEq,
    {
        self.0
            .iter_mut()
            .find_map(|(k, v)| ((*k).borrow() == key).then_some(v))
    }

    #[inline]
    #[must_use]
    pub fn contains_key<Q>(&self, key: &Q) -> bool
    where
        K: Borrow<Q>,
        Q: ?Sized + PartialEq,
    {
        self.0.iter().any(|(k, _)| k.borrow() == key)
    }

    #[inline]
    pub fn remove<Q>(&mut self, key: &Q) -> Option<V>
    where
        K: Borrow<Q>,
        Q: ?Sized + PartialEq,
    {
        let pos = self.0.iter().position(|(k, _)| k.borrow() == key)?;
        Some(self.0.remove(pos).1)
    }

    #[inline]
    pub fn keys(&self) -> impl Iterator<Item = &K> {
        self.0.iter().map(|(k, _)| k)
    }

    #[inline]
    pub fn values(&self) -> impl Iterator<Item = &V> {
        self.0.iter().map(|(_, v)| v)
    }

    #[inline]
    pub fn values_mut(&mut self) -> impl Iterator<Item = &mut V> {
        self.0.iter_mut().map(|(_, v)| v)
    }

    #[inline]
    #[must_use]
    pub fn into_keys(self) -> Vec<K> {
        self.0.into_iter().map(|(k, _)| k).collect()
    }

    #[must_use]
    pub fn into_values(self) -> Vec<V> {
        self.0.into_iter().map(|(_, v)| v).collect()
    }

    #[inline]
    #[must_use]
    pub const fn len(&self) -> usize {
        self.0.len()
    }

    #[inline]
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    #[inline]
    pub fn clear(&mut self) {
        self.0.clear();
    }

    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = (&K, &V)> {
        self.0.iter().map(|(k, v)| (k, v))
    }

    #[inline]
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&K, &mut V)> {
        self.0.iter_mut().map(|(k, v)| (&*k, v))
    }
}
