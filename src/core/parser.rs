use crate::core::lexer::{Builtin, Ident, Token};
use chumsky::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expression<'a> {
    Literal(String),
    Builtin(Builtin),
    Ident(&'a str),
    Alternative(Vec<Self>),
    Joined(Vec<Self>),
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

    let atom = select! {
        Token::Literal(val) => Expression::Literal(val),
        Token::Ident(Ident::Builtin(b)) => Expression::Builtin(b),
        Token::Ident(Ident::Definition(name)) => Expression::Ident(name),
    };

    let alt = atom
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
        .then_ignore(just(Token::Newline))
        .map(|(name, value)| Definition { name, value });

    definition.repeated().collect()
}

pub fn parse<'a>(tokens: &'a [Token<'a>]) -> Result<Vec<Definition<'a>>, String> {
    parser().parse(tokens).into_result().map_err(|err| {
        err.iter()
            .map(|err| format!("{err:?}"))
            .collect::<Vec<String>>()
            .join("\n")
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lit(literal: &str) -> Expression<'_> {
        Expression::Literal(format!("\"{literal}\""))
    }

    #[test]
    fn join_lower_precedence_than_alt() {
        // a := "a" + "b" / "c"
        let tokens = vec![
            Token::Ident(Ident::Definition("a")),
            Token::Define,
            Token::Literal("\"a\"".to_owned()),
            Token::Join,
            Token::Literal("\"b\"".to_owned()),
            Token::Alt,
            Token::Literal("\"c\"".to_owned()),
            Token::Newline,
        ];

        let ast = parser().parse(&tokens).unwrap();
        let expr = &ast[0].value;

        assert_eq!(
            *expr,
            Expression::Joined(vec![
                lit("a"),
                Expression::Alternative(vec![lit("b"), lit("c")])
            ])
        );

        // b := 1 / 2 + 3
        let tokens = vec![
            Token::Ident(Ident::Definition("a")),
            Token::Define,
            Token::Literal("\"a\"".to_owned()),
            Token::Alt,
            Token::Literal("\"b\"".to_owned()),
            Token::Join,
            Token::Literal("\"c\"".to_owned()),
            Token::Newline,
        ];

        let ast = parser().parse(&tokens).unwrap();
        let expr = &ast[0].value;

        assert_eq!(
            *expr,
            Expression::Joined(vec![
                Expression::Alternative(vec![lit("a"), lit("b")]),
                lit("c")
            ])
        );
    }
}
