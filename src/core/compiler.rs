use std::collections::HashMap;

use crate::{
    RegexKind,
    core::{lexer::Builtin, parser::Expression},
};

enum LazyCompiled<'a> {
    Raw(Expression<'a>),
    Compiled(String),
}

fn compile_expr_re2<'a>(
    definition: &mut HashMap<&'a str, LazyCompiled<'a>>,
    expr: Expression<'a>,
) -> String {
    match expr {
        Expression::Ident(ident) => compile_re2(definition, ident),
        Expression::Literal(literal) => escape_whitespace(&regex::escape(&literal)),
        Expression::Joined(v) => v
            .into_iter()
            .map(|e| compile_expr_re2(definition, e))
            .collect(),
        Expression::Alternative(v) => format!(
            "(?:{})",
            v.into_iter()
                .map(|e| compile_expr_re2(definition, e))
                .collect::<Vec<_>>()
                .join("|")
        ),
        Expression::Builtin(b) => match b {
            Builtin::Digit => String::from("\\d"),
            Builtin::WordChar => String::from("\\w"),
            Builtin::WhiteSpace => String::from("\\s"),
            Builtin::Tab => String::from("\\t"),
            Builtin::CarriageReturn => String::from("\\r"),
            Builtin::Linefeed => String::from("\\n"),
            Builtin::VerticalTab => String::from("\\v"),
            Builtin::FormFeed => String::from("\\f"),
            Builtin::Nul => String::from("\\0"),
            Builtin::Space => String::from(" "),
        },
    }
}

// I could swear this can be clone free but I cannot get it to work.
fn compile_re2<'a>(definition: &mut HashMap<&'a str, LazyCompiled<'a>>, name: &'a str) -> String {
    if let Some(LazyCompiled::Compiled(v)) = definition.get(name) {
        return v.clone();
    }

    let expr = match definition
        .get(name)
        .unwrap_or_else(|| panic!("Unknown symbol: {name}"))
    {
        LazyCompiled::Raw(expr) => expr.clone(),
        LazyCompiled::Compiled(v) => return v.clone(),
    };

    let value = compile_expr_re2(definition, expr);

    definition.insert(name, LazyCompiled::Compiled(value.clone()));
    value
}

pub fn compile<'a>(
    entry: &'a str,
    definitions: HashMap<&'a str, Expression<'a>>,
    regex_kind: RegexKind,
) -> String {
    if regex_kind != RegexKind::Re2 {
        todo!("only re2 is supported as of now")
    }

    let mut definitions = definitions
        .into_iter()
        .map(|(k, v)| (k, LazyCompiled::Raw(v)))
        .collect();

    compile_re2(&mut definitions, entry)
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
