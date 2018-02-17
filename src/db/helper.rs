use std::collections::HashMap;

pub struct Cacher<T> {
    cache: HashMap<String, T>,
}

impl<T> Cacher<T> {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }
}

impl<T> Cacher<T>
where
    T: Clone,
{
    pub fn get<F>(&mut self, key: String, req: F) -> Option<T>
    where
        F: Fn(String) -> Option<T>,
    {
        if self.cache.contains_key(&key) {
            return self.cache.get(&key).cloned();
        }
        let new_value = req(key.clone());
        if let Some(value_found) = new_value.clone() {
            self.cache.insert(key, value_found);
        }
        new_value
    }
}
