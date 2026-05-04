use std::collections::HashMap;

use crate::core::parser::{self, Expression};

pub fn process(source: Vec<parser::Definition<'_>>) -> HashMap<&str, Expression<'_>> {
    let mut definitions = HashMap::new();

    for definition in source {
        assert!(
            !definitions.contains_key(definition.name),
            "duplicate definition: {}",
            definition.name
        );
        definitions.insert(definition.name, definition.value);
    }

    definitions
}
