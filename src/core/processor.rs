use std::collections::{HashMap, HashSet};

use crate::core::{
    lexer::Builtin,
    optimizer::optimize,
    parser::{self, Expression},
};

#[derive(Debug, PartialEq, Eq)]
pub enum InlinedExpression {
    Literal(String),
    Builtin(Builtin),
    Alternative(Vec<Self>),
    Joined(Vec<Self>),
    Optional(Box<Self>),
    Multiple(Box<Self>),
    Some(Box<Self>),
}

pub fn process(source: Vec<parser::Definition<'_>>) -> Result<InlinedExpression, String> {
    let mut definitions = HashMap::new();

    for definition in source {
        if definitions.contains_key(definition.name) {
            return Err(format!("duplicate definition: {}", definition.name));
        }
        definitions.insert(definition.name, definition.value);
    }

    let root = definitions.get("EXPORT").ok_or("EXPORT is not defined")?;

    let expanded = expand(root, &definitions, &mut HashSet::new())?;

    let out = optimize(expanded);

    Ok(out)
}

fn expand<'a>(
    expr: &parser::Expression<'a>,
    defs: &HashMap<&'a str, parser::Expression<'a>>,
    rec: &mut HashSet<&'a str>,
) -> Result<InlinedExpression, String> {
    match expr {
        Expression::Literal(s) => Ok(InlinedExpression::Literal(s.to_owned())),
        Expression::Builtin(b) => Ok(InlinedExpression::Builtin(*b)),
        Expression::Ident(name) => {
            if !rec.insert(name) {
                return Err(format!(
                    "recursive loop detected: {} all depend on each other",
                    rec.iter().copied().collect::<Vec<&'a str>>().join(", ")
                ));
            }

            let expr = defs.get(name).ok_or(format!("{name} is not defined"))?;
            expand(expr, defs, rec)
        }
        Expression::Optional(o) => Ok(InlinedExpression::Optional(Box::new(expand(o, defs, rec)?))),
        Expression::Multiple(o) => Ok(InlinedExpression::Multiple(Box::new(expand(o, defs, rec)?))),
        Expression::Alternative(v) => {
            let mut vec = Vec::with_capacity(v.len());
            for expr in v {
                vec.push(expand(expr, defs, rec)?);
            }
            Ok(InlinedExpression::Alternative(vec))
        }
        Expression::Joined(v) => {
            let mut vec = Vec::with_capacity(v.len());
            for expr in v {
                vec.push(expand(expr, defs, rec)?);
            }
            Ok(InlinedExpression::Joined(vec))
        }
    }
}
