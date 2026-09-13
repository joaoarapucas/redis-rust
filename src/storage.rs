//hash maps

use std::collections::HashMap;

#[derive(Default)]
pub struct Store {
    data: HashMap<String, String>,
}

impl Store {
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }

    pub fn add(&mut self, key: String, value: String) {
        self.data.insert(key, value);
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.data.get(key)
    }

    // generic lookup by value: returns the key holding exactly this value,
    // if any. used by the Lua bridge for uniqueness checks (e.g. the same
    // cpf must not be stored under two different keys).
    pub fn find_key_by_value(&self, value: &str) -> Option<String> {
        for (key, stored_value) in self.data.iter() {
            if stored_value == value {
                return Some(key.clone());
            }
        }
        None
    }
}
