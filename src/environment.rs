use std::collections::HashMap;

use crate::ast::LiteralValue;

pub struct Environment {
    values: HashMap<String, LiteralValue>,
}

impl Environment {
    pub fn new() -> Self {
        return Environment {
            values: HashMap::new(),
        };
    }

    pub fn define(&mut self, name: String, value: LiteralValue) {
        self.values.insert(name, value);
    }

    pub fn get(&self, name: String) -> &LiteralValue {
        return self.values.get(&name).unwrap();
    }
}
