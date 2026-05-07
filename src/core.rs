use crate::{
    RegexKind,
    core::{lexer::lex, parser::parse, processor::process},
};

mod generator;
mod lexer;
mod optimizer;
mod parser;
mod processor;

pub fn compile(input: &str, kind: RegexKind) -> Result<String, String> {
    let tokens = lex(input)?;
    let parsed = parse(&tokens)?;
    let processed = process(parsed)?;
    generator::generate(processed, kind)
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! re2_test {
        ($name:ident, $input:literal, $expected:literal) => {
            #[test]
            fn $name() {
                let result = compile($input, RegexKind::Re2).expect("Failed to compile");
                assert_eq!(result, $expected);
            }
        };
    }

    re2_test!(lit, r#"EXPORT := "abc""#, r"abc");
    re2_test!(lit_escape_newline, r#"EXPORT := "\n""#, r"\n");
    re2_test!(lit_escape_tab, r#"EXPORT := "    ""#, r"\t");
    re2_test!(lit_escape_dot, r#"EXPORT := ".""#, r"\.");
    re2_test!(lit_join, r#"EXPORT := "a" + DIGIT"#, r"a\d");
    re2_test!(alt_chars_chain, r#"EXPORT := "a" / "b" / DIGIT"#, r"[ab\d]");
    re2_test!(alt_chars_hyphen, r#"EXPORT := "-" / "b"#, r"[-b]");
    re2_test!(
        alt_chars_hyphen_escape,
        r#"EXPORT := "a" / "-" / "b"#,
        r"[-ab]"
    );
    re2_test!(
        alt_with_join_a,
        r#"EXPORT := "a" + "bc" / "cd""#,
        r"a(?:bc|cd)"
    );
    re2_test!(
        alt_with_join_b,
        r#"EXPORT := "ab" / "bc" + "c""#,
        r"(?:ab|bc)c"
    );
    re2_test!(alt_merge, r#"EXPORT := "a" / "a""#, r"a");
    re2_test!(
        alt_complex_merge,
        r#"EXPORT := "a" / "0" / DIGIT"#,
        r"[a\d]"
    );
    re2_test!(oneof_basic, r#"EXPORT := ONEOF "abc""#, r"[abc]");
    re2_test!(oneof_single, r#"EXPORT := ONEOF "a""#, r"a");
    re2_test!(oneof_escaped, r#"EXPORT := ONEOF ".-""#, r"[-\.]");
    re2_test!(oneof_join, r#"EXPORT := ONEOF "ab" + "c""#, r"[ab]c");
    re2_test!(opt_literal, r#"EXPORT := OPTIONAL "a""#, r"a?");
    re2_test!(opt_oneof, r#"EXPORT := OPTIONAL ONEOF "ab""#, r"[ab]?");
    re2_test!(opt_digit, r"EXPORT := OPTIONAL DIGIT", r"\d?");
    re2_test!(opt_join, r#"EXPORT := "a" + OPTIONAL "b""#, r"ab?");
    re2_test!(
        opt_group,
        r#"
        def := "a" / "b" / "cd"
        EXPORT := OPTIONAL def
        "#,
        r"(?:[ab]|cd)?"
    );

    re2_test!(mul_literal, r#"EXPORT := MULTIPLE "a""#, r"a+");
    re2_test!(mul_oneof, r#"EXPORT := MULTIPLE ONEOF "ab""#, r"[ab]+");
    re2_test!(mul_digit, r"EXPORT := MULTIPLE DIGIT", r"\d+");
    re2_test!(mul_join, r#"EXPORT := MULTIPLE "a" + "b""#, r"a+b");
    re2_test!(opt_mul_char, r#"EXPORT := OPTIONAL MULTIPLE "a""#, r"a*");
    re2_test!(
        opt_mul_literal,
        r#"EXPORT := OPTIONAL MULTIPLE "ab""#,
        r"(?:ab)*"
    );
    re2_test!(
        opt_mul_oneof,
        r#"EXPORT := OPTIONAL MULTIPLE ONEOF "ab""#,
        r"[ab]*"
    );
    re2_test!(nothing_alt, r#"EXPORT := "a" / NOTHING"#, r"a?");
    re2_test!(
        nothing_alt_chain,
        r#"EXPORT := "a" / "b" / NOTHING"#,
        r"#[ab]?#"
    );
    re2_test!(nothing_join, r#"EXPORT := "a" + NOTHING"#, r"a");
}
