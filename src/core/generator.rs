use crate::{
    RegexKind,
    core::{lexer::Builtin, processor::ProcessedExpression},
};

fn generate_re2(expr: ProcessedExpression) -> Result<String, String> {
    match expr {
        ProcessedExpression::Optional(inner) => {
            assert!(
                !matches!(*inner, ProcessedExpression::Optional(_)),
                "OPTIONAL should not be nested"
            );
            Ok(format!("(?:{})?", generate_re2(*inner)?))
        }
        ProcessedExpression::Literal(literal) => Ok(escape_whitespace(&regex::escape(&literal))),
        ProcessedExpression::Joined(v) => v.into_iter().map(generate_re2).collect(),
        ProcessedExpression::Alternative(v) => Ok(format!(
            "(?:{})",
            v.into_iter()
                .map(generate_re2)
                .collect::<Result<Vec<_>, _>>()?
                .join("|")
        )),
        ProcessedExpression::Builtin(b) => Ok(match b {
            Builtin::Digit => String::from("\\d"),
            Builtin::WordChar => String::from("\\w"),
            Builtin::WhiteSpace => String::from("\\s"),
            Builtin::Tab => String::from("\\t"),
            Builtin::CarriageReturn => String::from("\\r"),
            Builtin::Linefeed => String::from("\\n"),
            Builtin::VerticalTab => String::from("\\v"),
            Builtin::FormFeed => String::from("\\f"),
            Builtin::Nul => String::from("\\0"),
            Builtin::Space => String::from(" "),
        }),
    }
}

pub fn generate(expr: ProcessedExpression, regex_kind: RegexKind) -> Result<String, String> {
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
