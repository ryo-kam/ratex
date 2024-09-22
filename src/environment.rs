use std::{cell::RefCell, collections::HashMap, fmt, rc::Rc};

use crate::{
    ast::Object,
    error::{RatexError, RatexErrorType},
};

#[derive(Clone, PartialEq)]
pub struct Environment {
    values: HashMap<String, Object>,
    enclosing: Option<Rc<RefCell<Environment>>>,
}

impl fmt::Debug for Environment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut builder = f.debug_struct("Environment");
        builder.field("values", &self.values);

        if let Some(env) = &self.enclosing {
            let value = env.borrow();
            builder.field("enclosing", &value);
        }

        builder.finish()
    }
}

impl Environment {
    pub fn new() -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Environment {
            values: HashMap::new(),
            enclosing: None,
        }))
    }

    pub fn new_child(parent: Rc<RefCell<Environment>>) -> Rc<RefCell<Environment>> {
        Rc::new(RefCell::new(Environment {
            values: HashMap::new(),
            enclosing: Some(parent.clone()),
        }))
    }

    pub fn define(&mut self, name: String, value: Object) {
        self.values.insert(name, value);
    }

    pub fn get(&self, name: String) -> Result<Object, RatexError> {
        match self.values.get(&name) {
            Some(value) => Ok(value.clone()),
            None => match &self.enclosing {
                Some(parent) => {
                    let a = parent.borrow().get(name)?.clone();
                    return Ok(a);
                }
                None => Err(RatexError {
                    source: RatexErrorType::UndefinedIdentifier(name),
                }),
            },
        }
    }

    pub fn get_at(env: Rc<RefCell<Self>>, distance: usize, name: String) -> Object {
        println!("{:?}", &env);

        Self::ancestor(env, distance)
            .borrow()
            .values
            .get(&name)
            .unwrap()
            .clone()
    }

    pub fn assign(&mut self, name: String, value: Object) -> Result<(), RatexError> {
        if self.values.contains_key(&name) {
            self.values.insert(name, value);
        } else {
            match &mut self.enclosing {
                Some(parent) => {
                    return parent.borrow_mut().assign(name, value);
                }
                None => {
                    return Err(RatexError {
                        source: RatexErrorType::UndefinedIdentifier(name),
                    })
                }
            }
        }

        Ok(())
    }

    pub fn assign_at(env: Rc<RefCell<Self>>, distance: usize, name: String, value: Object) {
        Self::ancestor(env, distance)
            .borrow_mut()
            .values
            .insert(name, value);
    }

    fn ancestor(env: Rc<RefCell<Self>>, distance: usize) -> Rc<RefCell<Self>> {
        let mut env_ref = Rc::clone(&env);

        for _ in 0..distance {
            env_ref = Rc::clone(Rc::clone(&env_ref).borrow_mut().enclosing.as_ref().unwrap());
        }

        return env_ref;
    }
}
