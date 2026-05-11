use crate::{RegexKind, core::processor::InlinedExpression};

fn generate_re2(expr: InlinedExpression) -> Result<String, String> {
    match expr {
        InlinedExpression::Oneof(inner) => Ok({
            let mut str = String::from("[");

            for c in inner {
                match c {
                    '\n' => str.push_str(r"\n"),
                    '\r' => str.push_str(r"\r"),
                    '\t' => str.push_str(r"\t"),
                    '\x0b' => str.push_str(r"\v"),
                    '\x0c' => str.push_str(r"\f"),
                    _ => {
                        if regex_syntax::is_meta_character(c) {
                            str.push('\\');
                        }
                        str.push(c)
                    }
                }
            }

            str + "]"
        }),
        InlinedExpression::Optional(inner) => Ok(format!("(?:{})?", generate_re2(*inner)?)),
        InlinedExpression::Multiple(inner) => Ok(format!("(?:{})+", generate_re2(*inner)?)),
        InlinedExpression::Some(inner) => Ok(format!("(?:{})*", generate_re2(*inner)?)),
        InlinedExpression::Literal(literal) => Ok(escape_whitespace(&regex::escape(&literal))),
        InlinedExpression::Joined(v) => v.into_iter().map(generate_re2).collect(),
        InlinedExpression::Alternative(v) => Ok(format!(
            "(?:{})",
            v.into_iter()
                .map(generate_re2)
                .collect::<Result<Vec<_>, _>>()?
                .join("|")
        )),
        InlinedExpression::Builtin(b) => Ok(b.generate().to_owned()),
        InlinedExpression::Char(c) => Ok(String::from(c)),
        InlinedExpression::Nothing => Ok(String::new()),
    }
}

pub fn generate(expr: InlinedExpression, regex_kind: RegexKind) -> Result<String, String> {
    if regex_kind != RegexKind::Re2 {
        todo!("only re2 is supported as of now")
    }

    generate_re2(expr)
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
