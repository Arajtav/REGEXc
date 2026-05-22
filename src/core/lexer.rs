use std::str::FromStr;

use logos::Logos;

#[derive(Logos, Debug, Clone, PartialEq, Eq)]
#[logos(skip r"[ \t]+")]
#[logos(skip(r";[^\r\n]*", allow_greedy = true))]
pub enum Token<'a> {
    #[regex(r"\r\n|\n|\r")]
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

    #[token("OPTIONAL")]
    Optional,

    #[token("MULTIPLE")]
    Multiple,

    #[token("ONEOF")]
    Oneof,

    #[token("NOTHING")]
    Nothing,
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

impl Builtin {
    pub fn build(self) -> &'static str {
        match self {
            Self::Digit => "\\d",
            Self::WordChar => "\\w",
            Self::WhiteSpace => "\\s",
            Self::Tab => "\\t",
            Self::CarriageReturn => "\\r",
            Self::Linefeed => "\\n",
            Self::VerticalTab => "\\v",
            Self::FormFeed => "\\f",
            Self::Nul => "\\0",
            Self::Space => " ",
        }
    }

    pub fn contains(self, c: char) -> bool {
        match self {
            Builtin::Digit => c.is_ascii_digit(),
            Builtin::WordChar => c.is_ascii_alphanumeric(),
            Builtin::WhiteSpace => c.is_whitespace(),
            Builtin::Space => c == ' ',
            Builtin::Tab => c == '\t',
            Builtin::CarriageReturn => c == '\r',
            Builtin::Linefeed => c == '\n',
            Builtin::VerticalTab => c == '\x0b',
            Builtin::FormFeed => c == '\x0c',
            Builtin::Nul => c == '\0',
        }
    }

    pub fn contains_builtin(self, c: Builtin) -> bool {
        (match self {
            Builtin::WordChar => c == Builtin::Digit,
            Builtin::WhiteSpace => {
                c == Builtin::Space
                    || c == Builtin::Tab
                    || c == Builtin::CarriageReturn
                    || c == Builtin::Linefeed
                    || c == Builtin::VerticalTab
                    || c == Builtin::FormFeed
            }
            _ => false,
        }) || c == self
    }
}

impl FromStr for Builtin {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
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
    Builtin::from_str(s)
        .map(Ident::Builtin)
        .unwrap_or(Ident::Definition(s))
}

fn parse_literal<'a>(lex: &mut logos::Lexer<'a, Token<'a>>) -> Option<String> {
    let slice = lex.slice();
    let inner = &slice[1..slice.len() - 1];

    unescape::unescape(inner)
}

pub fn lex(input: &str) -> Result<Vec<Token<'_>>, String> {
    let mut tokens = Vec::new();

    let mut lexer = Token::lexer(input);

    while let Some(result) = lexer.next() {
        if let Ok(tok) = result {
            tokens.push(tok);
        } else {
            let span = lexer.span();
            let bad = &input[span.clone()];

            let (line, col) = byte_to_line_col(input, span.start);

            return Err(format!("unexpected {bad:?} at {line}:{col}"));
        }
    }

    Ok(tokens)
}

fn byte_to_line_col(input: &str, byte_idx: usize) -> (usize, usize) {
    let mut line = 1;
    let mut col = 1;

    for (i, ch) in input.char_indices() {
        if i >= byte_idx {
            break;
        }

        if ch == '\n' {
            line += 1;
            col = 1;
        } else {
            col += 1;
        }
    }

    (line, col)
}
