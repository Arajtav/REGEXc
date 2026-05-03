use std::collections::HashMap;

use crate::{RegexKind, core::parser};

pub fn compile<'a>(
    expression: &'a parser::Expression,
    definitions: &'a HashMap<&'a str, parser::Expression>,
    regex_kind: RegexKind,
) -> String {
    if regex_kind != RegexKind::Re2 {
        todo!()
    }

    match expression {
        parser::Expression::Literal(literal) => escape_whitespace(&regex::escape(literal)),
        parser::Expression::Joined(a, b) => {
            compile(a, definitions, regex_kind) + &compile(b, definitions, regex_kind)
        }
    }
}

fn escape_whitespace(text: &str) -> String {
    let mut out = String::with_capacity(text.len());

    for c in text.chars() {
        match c {
            '\n' => out.push_str(r"\n"),
            '\r' => out.push_str(r"\r"),
            '\t' => out.push_str(r"\t"),
            '\x0b' => out.push_str(r"\v"),
            '\x0c' => out.push_str(r"\f"),
            _ => out.push(c),
        }
    }

    out
}
