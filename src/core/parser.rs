use std::path::Path;

use crate::{
    core::lexer::{Builtin, Ident, Token},
    diagnostic::{Diagnostic, Span},
};
use chumsky::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expression<'a> {
    Literal(String),
    Builtin(Builtin),
    Ident(&'a str),
    Alternative(Vec<Self>),
    Joined(Vec<Self>),
    Optional(Box<Self>),
    Multiple(Box<Self>),
}

#[derive(Debug, PartialEq, Eq)]
pub struct Definition<'a> {
    pub name: &'a str,
    pub value: Expression<'a>,
}

fn parser<'a>() -> impl Parser<
    'a,
    &'a [Spanned<Token<'a>>],
    Vec<(Definition<'a>, Span)>,
    extra::Err<Rich<'a, Token<'a>>>,
> {
    let ident = select! {
        (Token::Ident(Ident::Definition(name)), _) => name,
    };

    let atom = select! {
        Token::Literal(val) => Expression::Literal(val),
        Token::Ident(Ident::Builtin(b)) => Expression::Builtin(b),
        Token::Ident(Ident::Definition(name)) => Expression::Ident(name),
    };

    let modified_atom = just(Token::Optional)
        .or_not()
        .then(just(Token::Multiple).or_not())
        .then(atom)
        .map(|((opt, mul), expr)| {
            let expr = if opt.is_some() {
                Expression::Optional(Box::new(expr))
            } else {
                expr
            };

            if mul.is_some() {
                Expression::Multiple(Box::new(expr))
            } else {
                expr
            }
        });

    let alt = modified_atom
        .separated_by(just(Token::Alt))
        .at_least(1)
        .collect::<Vec<_>>()
        .map(|sub| match sub.as_slice() {
            [only] => only.clone(),
            _ => Expression::Alternative(sub),
        });

    let join = alt
        .clone()
        .separated_by(just(Token::Join))
        .at_least(1)
        .collect::<Vec<_>>()
        .map(|sub| match sub.as_slice() {
            [only] => only.clone(),
            _ => Expression::Joined(sub),
        });

    let definition = ident
        .then_ignore(just(Token::Define))
        .then(join)
        .map(|(name, value)| Definition { name, value })
        .map_with(|def, e| {
            let span = e.span();
            (def, span)
        });

    definition
        .separated_by(just(Token::Newline).repeated().at_least(1))
        .allow_leading()
        .allow_trailing()
        .collect()
}

pub fn parse<'a>(
    path: &'a Path,
    source: &'a str,
    tokens: &'a [Token<'a>],
) -> Result<Vec<(Definition<'a>, Span)>, Vec<Diagnostic<'a>>> {
    parser().parse(tokens).into_result().map_err(|err| {
        err.into_iter()
            .map(|err| Diagnostic {
                path,
                source,
                error: format!("Parser error"),
                span: Some(err.span().into_range().into()),
            })
            .collect()
    })
}
