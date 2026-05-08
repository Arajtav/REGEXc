use std::{
    collections::{HashMap, HashSet},
    path::Path,
};

use crate::{
    core::{
        lexer::Builtin,
        optimizer::optimize,
        parser::{self, Expression},
    },
    diagnostic::Diagnostic,
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

pub fn inline<'a>(
    path: &'a Path,
    source: &'a str,
    root: &'a str,
    expressions: Vec<(parser::Definition<'a>, std::ops::Range<usize>)>,
) -> Result<InlinedExpression, Vec<Diagnostic<'a>>> {
    let mut definitions = HashMap::new();
    let mut diagnostics = Vec::new();

    for (expression, span) in expressions {
        if definitions.contains_key(expression.name) {
            diagnostics.push(Diagnostic {
                path,
                source,
                error: format!("Redefinition of {}", expression.name),
                span: Some(span.into()),
            });
        } else {
            definitions.insert(expression.name, expression.value);
        }
    }

    if !diagnostics.is_empty() {
        return Err(diagnostics);
    }

    let Some(root) = definitions.get(root) else {
        return Err(vec![Diagnostic {
            path,
            source,
            error: format!("{root} is not defined"),
            span: None,
        }]);
    };

    expand(path, source, root, &definitions, &mut HashSet::new()).map_err(|e| vec![e])
}

fn expand<'a>(
    path: &'a Path,
    source: &'a str,
    expr: &parser::Expression<'a>,
    defs: &HashMap<&'a str, parser::Expression<'a>>,
    rec: &mut HashSet<&'a str>,
) -> Result<InlinedExpression, Diagnostic<'a>> {
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
