use logos::Logos;

#[derive(Logos, Debug, Clone, PartialEq, Eq)]
#[logos(skip r"[ \t]+")]
#[logos(skip(r";[^\r\n]*", allow_greedy = true))]
#[logos(error = String)]
pub enum Token<'a> {
    #[regex(r"\r\n|\n|\r", |_| ())]
    Newline,

    #[token(":=")]
    Define,

    #[regex(r"\w+")]
    Definition(&'a str),

    #[regex(r#""([^"\\]|\\.)*""#, parse_literal)]
    Literal(String),

    #[token("+")]
    Join,
}

fn parse_literal<'a>(lex: &mut logos::Lexer<'a, Token<'a>>) -> Result<String, String> {
    let slice = lex.slice();
    let inner = &slice[1..slice.len() - 1];

    unescape::unescape(inner).ok_or("invalid string literal".into())
}
