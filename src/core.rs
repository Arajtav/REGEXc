use chumsky::Parser;
use logos::Logos;

use crate::{
    RegexKind,
    core::{lexer::Token, parser::parser, processor::process},
};

mod compiler;
mod lexer;
mod parser;
mod processor;

pub fn compile(input: &str, kind: RegexKind) -> String {
    let mut tokens = Vec::new();
    for token in Token::lexer(input) {
        let token = token.unwrap();

        if tokens.last().is_none_or(|f| *f == Token::Newline) && token == Token::Newline {
            continue;
        }

        tokens.push(token);
    }

    if tokens.last() != Some(&Token::Newline) {
        tokens.push(Token::Newline);
    }

    let parsed = parser().parse(&tokens).into_result().unwrap();

    let definitions = process(parsed);
    let export = definitions.get("EXPORT").expect("missing EXPORT");

    compiler::compile(export, &definitions, kind)
}
