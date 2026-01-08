use crate::core::values::ValueStore;

pub struct Environment {
    pub variables: ValueStore,
}

impl Default for Environment {
    fn default() -> Self {
        Self {
            variables: ValueStore::default(),
        }
    }
}
