use std::collections::{HashMap, HashSet};

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
    in_stack: &mut HashSet<&'a str>,
) -> String {
    match expr {
        Expression::Ident(ident) => compile_re2(definition, ident, in_stack),
        Expression::Literal(literal) => escape_whitespace(&regex::escape(&literal)),
        Expression::Joined(v) => v
            .into_iter()
            .map(|e| compile_expr_re2(definition, e, in_stack))
            .collect(),
        Expression::Alternative(v) => format!(
            "(?:{})",
            v.into_iter()
                .map(|e| compile_expr_re2(definition, e, in_stack))
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
fn compile_re2<'a>(
    definition: &mut HashMap<&'a str, LazyCompiled<'a>>,
    name: &'a str,
    in_stack: &mut HashSet<&'a str>,
) -> String {
    assert!(
        in_stack.insert(name),
        "Recursive definition found at: {name:?}"
    );

    if let Some(LazyCompiled::Compiled(v)) = definition.get(name) {
        in_stack.remove(name);
        return v.clone();
    }

    let expr = match definition
        .get(name)
        .unwrap_or_else(|| panic!("Unknown symbol: {name:?}"))
    {
        LazyCompiled::Raw(expr) => expr.clone(),
        LazyCompiled::Compiled(v) => {
            in_stack.remove(name);
            return v.clone();
        }
    };

    let value = compile_expr_re2(definition, expr, in_stack);

    definition.insert(name, LazyCompiled::Compiled(value.clone()));

    in_stack.remove(name);

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

    compile_re2(&mut definitions, entry, &mut HashSet::new())
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
