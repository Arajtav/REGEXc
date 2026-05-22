use crate::core::lexer::{Builtin, Ident, Token};
use chumsky::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expression<'a> {
    Literal(String),
    Oneof(String),
    Builtin(Builtin),
    Ident(&'a str),
    Alternative(Vec<Self>),
    Joined(Vec<Self>),
    Optional(Box<Self>),
    Multiple(Box<Self>),
    Nothing,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Definition<'a> {
    pub name: &'a str,
    pub value: Expression<'a>,
}

fn parser<'a>()
-> impl Parser<'a, &'a [Token<'a>], Vec<Definition<'a>>, extra::Err<Rich<'a, Token<'a>>>> {
    let ident = select! {
        Token::Ident(Ident::Definition(name)) => name,
    };

    let literal = select! {
        Token::Literal(lit) => lit,
    };

    let literal_or_oneof = just(Token::Oneof)
        .or_not()
        .then(literal)
        .map(|(oneof, content)| {
            if oneof.is_some() {
                Expression::Oneof(content)
            } else {
                Expression::Literal(content)
            }
        });

    let atom = select! {
        Token::Ident(Ident::Builtin(b)) => Expression::Builtin(b),
        Token::Ident(Ident::Definition(name)) => Expression::Ident(name),
        Token::Nothing => Expression::Nothing,
    }
    .or(literal_or_oneof);

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
        .map(|(name, value)| Definition { name, value });

    definition
        .separated_by(just(Token::Newline).repeated().at_least(1))
        .allow_leading()
        .allow_trailing()
        .collect()
}

pub fn parse<'a>(tokens: &'a [Token<'a>]) -> Result<Vec<Definition<'a>>, String> {
    parser().parse(tokens).into_result().map_err(|err| {
        err.iter()
            .map(|err| format!("{err:?}"))
            .collect::<Vec<String>>()
            .join("\n")
    })
}
