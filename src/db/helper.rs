use std::collections::HashMap;

pub struct Cacher<T> {
    cache: HashMap<String, Option<T>>,
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
    pub fn get<F>(&mut self, key: &String, req: F) -> &Option<T>
    where
        F: Fn(&String) -> Option<T>,
    {
        self.cache.entry(key.clone()).or_insert_with(|| req(key))
    }
}
