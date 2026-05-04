use logos::Logos;

#[derive(Logos, Debug, Clone, PartialEq, Eq)]
#[logos(skip r"[ \t]+")]
#[logos(skip(r";[^\r\n]*", allow_greedy = true))]
#[logos(error = String)]
pub enum Token<'a> {
    #[regex(r"\r\n|\n|\r", |_| ())]
    Newline,

    #[regex(r"\w+", |lex| classify_ident(lex.slice()))]
    Ident(Ident<'a>),

    #[regex(r#""([^"\\]|\\.)*""#, parse_literal)]
    Literal(String),

    #[token(":=")]
    Define,
    #[token("+")]
    Join,
    #[token("/")]
    Alt,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Builtin {
    Digit,
    WordChar,
    WhiteSpace,
    Space,
    Tab,
    CarriageReturn,
    Linefeed,
    VerticalTab,
    FormFeed,
    Nul,
}

impl TryFrom<&str> for Builtin {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "DIGIT" => Ok(Self::Digit),
            "WCHAR" => Ok(Self::WordChar),
            "WHITESPACE" => Ok(Self::WhiteSpace),
            "TAB" => Ok(Self::Tab),
            "CR" => Ok(Self::CarriageReturn),
            "LF" => Ok(Self::Linefeed),
            "VTAB" => Ok(Self::VerticalTab),
            "FORMFEED" => Ok(Self::FormFeed),
            "NUL" => Ok(Self::Nul),
            "SPACE" => Ok(Self::Space),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ident<'a> {
    Builtin(Builtin),
    Definition(&'a str),
}

fn classify_ident(s: &str) -> Ident<'_> {
    Builtin::try_from(s)
        .map(Ident::Builtin)
        .unwrap_or(Ident::Definition(s))
}

fn parse_literal<'a>(lex: &mut logos::Lexer<'a, Token<'a>>) -> Result<String, String> {
    let slice = lex.slice();
    let inner = &slice[1..slice.len() - 1];

    unescape::unescape(inner).ok_or("invalid string literal".into())
}

pub fn lex(input: &str) -> Vec<Token<'_>> {
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

    tokens
}
