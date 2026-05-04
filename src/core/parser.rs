use chumsky::prelude::*;

use crate::core::lexer::Token;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expression {
    Literal(String),
    Alternative(Vec<Self>),
    Joined(Vec<Self>),
}

#[derive(Debug, PartialEq, Eq)]
pub struct Definition<'a> {
    pub name: &'a str,
    pub value: Expression,
}

pub fn parser<'a>()
-> impl Parser<'a, &'a [Token<'a>], Vec<Definition<'a>>, extra::Err<Rich<'a, Token<'a>>>> {
    let ident = select! {
        Token::Definition(name) => name,
    };

    let atom = select! {
        Token::Literal(val) => Expression::Literal(val),
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

#[cfg(test)]
mod tests {
    use super::*;

    fn lit(literal: &str) -> Expression {
        Expression::Literal(format!("\"{literal}\""))
    }

    #[test]
    fn join_lower_precedence_than_alt() {
        // a := "a" + "b" / "c"
        let tokens = vec![
            Token::Definition("a"),
            Token::Define,
            Token::Literal(format!("\"a\"")),
            Token::Join,
            Token::Literal(format!("\"b\"")),
            Token::Alt,
            Token::Literal(format!("\"c\"")),
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
            Token::Definition("a"),
            Token::Define,
            Token::Literal(format!("\"a\"")),
            Token::Alt,
            Token::Literal(format!("\"b\"")),
            Token::Join,
            Token::Literal(format!("\"c\"")),
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
