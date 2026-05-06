use std::collections::{HashMap, HashSet};

use crate::core::{
    lexer::Builtin,
    parser::{self, Expression},
};

#[derive(Debug, PartialEq, Eq)]
pub enum ProcessedExpression {
    Literal(String),
    Builtin(Builtin),
    Alternative(Vec<Self>),
    Joined(Vec<Self>),
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

    expand(root, &definitions, &mut HashSet::new())
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
