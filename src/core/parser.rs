use chumsky::prelude::*;

use crate::core::lexer::Token;

#[derive(Debug, Clone)]
pub enum Expression {
    Literal(String),
    Joined(Box<Expression>, Box<Expression>),
}

#[derive(Debug)]
pub struct Definition<'a> {
    pub name: &'a str,
    pub value: Expression,
}

pub fn parser<'a>()
-> impl Parser<'a, &'a [Token<'a>], Vec<Definition<'a>>, extra::Err<Rich<'a, Token<'a>>>> {
    let ident = select! {
        Token::Definition(name) => name,
    };

    let literal = select! {
        Token::Literal(val) => Expression::Literal(val),
    };

    let expr = literal.clone().foldl(
        just(Token::Join).ignore_then(literal.clone()).repeated(),
        |lhs, rhs| Expression::Joined(Box::new(lhs), Box::new(rhs)),
    );

    let definition = ident
        .then_ignore(just(Token::Define))
        .then(expr)
        .then_ignore(just(Token::Newline))
        .map(|(name, value)| Definition { name, value });

    definition.repeated().collect()
}
