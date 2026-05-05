use crate::{
    RegexKind,
    core::{lexer::lex, parser::parse, processor::process},
};

mod compiler;
mod lexer;
mod parser;
mod processor;

pub fn compile(input: &str, kind: RegexKind) -> Result<String, String> {
    let tokens = lex(input)?;
    let parsed = parse(&tokens)?;
    compiler::compile("EXPORT", process(parsed), kind)
}
