use chumsky::prelude::*;

use crate::core::lexer::Token;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expression {
    Literal(String),
    Joined(Box<Expression>, Box<Expression>),
    Alternative(Box<Expression>, Box<Expression>),
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

    let literal = select! {
        Token::Literal(val) => Expression::Literal(val),
    };

    let alt_expr = literal.clone().foldl(
        just(Token::Alt)
            .ignore_then(literal.clone())
            .repeated(),
        |lhs, rhs| Expression::Alternative(Box::new(lhs), Box::new(rhs)),
    );

    let expr = alt_expr.clone().foldl(
        just(Token::Join).ignore_then(alt_expr.clone()).repeated(),
        |lhs, rhs| Expression::Joined(Box::new(lhs), Box::new(rhs)),
    );

    let definition = ident
        .then_ignore(just(Token::Define))
        .then(expr)
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
            Expression::Joined(
                Box::new(lit("a")),
                Box::new(Expression::Alternative(
                    Box::new(lit("b")),
                    Box::new(lit("c"))
                ))
            )
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
            Expression::Joined(
                Box::new(Expression::Alternative(
                    Box::new(lit("a")),
                    Box::new(lit("b"))
                )),
                Box::new(lit("c"))
            )
        )
    }
}
