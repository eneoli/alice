use serde::{Deserialize, Serialize};
use tsify_next::Tsify;

use crate::util::counter::Counter;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct Identifier {
    name: String,
    unique_id: usize,
}

impl Identifier {
    pub fn new(name: String, unique_id: usize) -> Self {
        Self { name, unique_id }
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn unique_id(&self) -> usize {
        self.unique_id
    }

    pub fn sorry() -> Self {
        Self {
            name: "sorry".to_string(),
            unique_id: -1,
        }
    }
}

pub struct IdentifierFactory {
    counter: Counter,
}

impl IdentifierFactory {
    pub fn new(counter: Counter) -> Self {
        Self { counter }
    }

    pub fn create(&mut self, name: String) -> Identifier {
        Identifier::new(name, self.counter.next_value())
    }
}
