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
    pub fn new_with(cache: HashMap<String, Option<T>>) -> Self {
        Self { cache }
    }
}

impl<T> Cacher<T>
where
    T: Clone,
{
    pub fn get<F>(&mut self, key: &str, req: F) -> &Option<T>
    where
        F: Fn(&str) -> Option<T>,
    {
        self.cache
            .entry(key.to_string())
            .or_insert_with(|| req(key))
    }
}
