use std::collections::HashMap;

use crate::{RegexKind, core::parser};

pub fn compile<'a>(
    expression: &'a parser::Expression,
    _definitions: &'a HashMap<&'a str, parser::Expression>,
    regex_kind: RegexKind,
) -> String {
    if regex_kind != RegexKind::Re2 {
        todo!()
    }

    match expression {
        parser::Expression::Literal(literal) => regex::escape(literal),
    }
}
