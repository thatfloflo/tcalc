use crate::core::decimals::Decimal;
use crate::core::values::{Value, ValueStore};

pub struct Environment {
    pub variables: ValueStore,
}

impl Default for Environment {
    fn default() -> Self {
        let mut vs = ValueStore::with_protected_keys(vec!["pi", "tau", "e"]);
        vs.set_readonly("pi", Value::from(Decimal::PI));
        vs.set_readonly("tau", Value::from(Decimal::TAU));
        vs.set_readonly("e", Value::from(Decimal::E));
        Self { variables: vs }
    }
}
