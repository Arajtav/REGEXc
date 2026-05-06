use std::collections::{HashMap, HashSet};

use crate::core::{
    lexer::Builtin,
    optimizer,
    parser::{self, Expression},
};

#[derive(Debug, PartialEq, Eq)]
pub enum ProcessedExpression {
    Literal(String),
    Builtin(Builtin),
    Alternative(Vec<Self>),
    Joined(Vec<Self>),
    Optional(Box<Self>),
}

pub fn process(source: Vec<parser::Definition<'_>>) -> Result<ProcessedExpression, String> {
    let mut definitions = HashMap::new();

    for definition in source {
        if definitions.contains_key(definition.name) {
            return Err(format!("duplicate definition: {}", definition.name));
        }
        definitions.insert(definition.name, definition.value);
    }

    let root = definitions.get("EXPORT").ok_or("EXPORT is not defined")?;

    let expanded = expand(root, &definitions, &mut HashSet::new())?;

    let out = optimizer::merge_nested(expanded);

    Ok(out)
}

fn expand<'a>(
    expr: &parser::Expression<'a>,
    defs: &HashMap<&'a str, parser::Expression<'a>>,
    rec: &mut HashSet<&'a str>,
) -> Result<ProcessedExpression, String> {
    match expr {
        Expression::Literal(s) => Ok(ProcessedExpression::Literal(s.to_owned())),
        Expression::Builtin(b) => Ok(ProcessedExpression::Builtin(*b)),
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
        Expression::Optional(o) => Ok(ProcessedExpression::Optional(Box::new(expand(
            o, defs, rec,
        )?))),
        Expression::Alternative(v) => {
            let mut vec = Vec::with_capacity(v.len());
            for expr in v {
                vec.push(expand(expr, defs, rec)?);
            }
            Ok(ProcessedExpression::Alternative(vec))
        }
        Expression::Joined(v) => {
            let mut vec = Vec::with_capacity(v.len());
            for expr in v {
                vec.push(expand(expr, defs, rec)?);
            }
            Ok(ProcessedExpression::Joined(vec))
        }
    }
}
