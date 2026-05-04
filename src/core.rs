use chumsky::Parser;

use crate::{
    RegexKind,
    core::{lexer::lex, parser::parser, processor::process},
};

mod compiler;
mod lexer;
mod parser;
mod processor;

pub fn compile(input: &str, kind: RegexKind) -> String {
    let tokens = lex(input);

    let parsed = parser().parse(&tokens).into_result().unwrap();

    compiler::compile("EXPORT", process(parsed), kind)
}
