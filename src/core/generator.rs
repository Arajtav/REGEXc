use std::borrow::Cow;

use crate::{
    RegexKind,
    core::{lexer::Builtin, processor::InlinedExpression},
};

fn make_atom(s: Cow<'static, str>, a: bool) -> String {
    if a {
        s.into_owned()
    } else {
        format!("(?:{s})")
    }
}

impl InlinedExpression {
    pub fn build_re2(self) -> (Cow<'static, str>, bool) {
        match self {
            InlinedExpression::Nothing => (Cow::Borrowed(""), true),
            InlinedExpression::Literal(literal) => (
                Cow::Owned(escape_whitespace(&regex::escape(&literal))),
                false,
            ),
            InlinedExpression::Char(c) => (
                Cow::Owned(escape_whitespace(&regex::escape(&String::from(c)))),
                true,
            ),
            InlinedExpression::Optional(inner) => {
                let (inner, atom) = inner.build_re2();
                (Cow::Owned(make_atom(inner, atom) + "?"), true)
            }
            InlinedExpression::Multiple(inner) => {
                let (inner, atom) = inner.build_re2();
                (Cow::Owned(make_atom(inner, atom) + "+"), true)
            }
            InlinedExpression::Some(inner) => {
                let (inner, atom) = inner.build_re2();
                (Cow::Owned(make_atom(inner, atom) + "*"), true)
            }
            InlinedExpression::Joined(v) => (
                Cow::Owned(v.into_iter().map(|a| a.build_re2().0).collect()),
                false,
            ),
            InlinedExpression::Alternative(v) => (
                Cow::Owned(format!(
                    "(?:{})",
                    v.into_iter()
                        .map(|a| a.build_re2().0)
                        .collect::<Vec<_>>()
                        .join("|")
                )),
                false,
            ),
            InlinedExpression::Builtin(b) => (
                Cow::Borrowed(match b {
                    Builtin::Digit => "\\d",
                    Builtin::WordChar => "\\w",
                    Builtin::WhiteSpace => "\\s",
                    Builtin::Tab => "\\t",
                    Builtin::CarriageReturn => "\\r",
                    Builtin::Linefeed => "\\n",
                    Builtin::VerticalTab => "\\v",
                    Builtin::FormFeed => "\\f",
                    Builtin::Nul => "\\0",
                    Builtin::Space => " ",
                }),
                true,
            ),
        }
    }
}

pub fn generate(expr: InlinedExpression, regex_kind: RegexKind) -> Result<String, String> {
    if regex_kind == RegexKind::Re2 {
        Ok(expr.build_re2().0.into_owned())
    } else {
        Err(String::from("Only RE2 is supported as of now"))
    }
}

fn escape_whitespace(text: &str) -> String {
    let mut out = String::with_capacity(text.len());

    for c in text.chars() {
        match c {
            '\n' => out.push_str(r"\n"),
            '\r' => out.push_str(r"\r"),
            '\t' => out.push_str(r"\t"),
            '\x0b' => out.push_str(r"\v"),
            '\x0c' => out.push_str(r"\f"),
            _ => out.push(c),
        }
    }

    out
}
