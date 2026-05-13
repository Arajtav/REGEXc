use std::collections::{HashMap, HashSet};

use crate::core::{
    lexer::Builtin,
    parser::{self, Expression},
};

#[derive(Debug, PartialEq, Eq)]
pub enum InlinedExpression {
    Literal(String),
    Char(char),
    Builtin(Builtin),
    Alternative(Vec<Self>),
    Joined(Vec<Self>),
    Optional(Box<Self>),
    Multiple(Box<Self>),
    Some(Box<Self>),
    Nothing,
}

pub fn alt_merge(input: Vec<InlinedExpression>) -> Vec<InlinedExpression> {
    let mut output = Vec::with_capacity(input.len());

    'outer: for new in input {
        for item in &mut output {
            if &new == item {
                continue 'outer;
            }

            if let (InlinedExpression::Builtin(new), InlinedExpression::Builtin(existing)) =
                (&new, &item)
            {
                if existing.contains_builtin(*new) {
                    continue 'outer;
                }

                if new.contains_builtin(*existing) {
                    *item = InlinedExpression::Builtin(*new);
                    continue 'outer;
                }
            }

            if let (InlinedExpression::Builtin(new), InlinedExpression::Char(existing)) =
                (&new, &item)
                && new.contains(*existing)
            {
                *item = InlinedExpression::Builtin(*new);
                continue 'outer;
            }

            if let (InlinedExpression::Char(new), InlinedExpression::Builtin(existing)) =
                (&new, &item)
                && existing.contains(*new)
            {
                continue 'outer;
            }
        }

        output.push(new);
    }

    output
}

pub fn join_merge(input: Vec<InlinedExpression>) -> Vec<InlinedExpression> {
    enum Join3State {
        None,
        String(String),
        Char(char),
    }

    let mut output = Vec::new();

    let mut lacc = Join3State::None;
    for i in input
        .into_iter()
        .filter(|i| !matches!(i, InlinedExpression::Nothing))
    {
        match i {
            InlinedExpression::Literal(lit) => {
                lacc = match lacc {
                    Join3State::None => Join3State::String(lit),
                    Join3State::String(acc) => Join3State::String(acc + &lit),
                    Join3State::Char(acc) => Join3State::String(String::from(acc) + &lit),
                };
            }
            InlinedExpression::Char(c) => {
                lacc = match lacc {
                    Join3State::None => Join3State::Char(c),
                    Join3State::String(acc) => Join3State::String(format!("{acc}{c}")),
                    Join3State::Char(acc) => Join3State::String(format!("{acc}{c}")),
                };
            }
            _ => {
                match lacc {
                    Join3State::String(str) => output.push(InlinedExpression::Literal(str)),
                    Join3State::Char(c) => output.push(InlinedExpression::Char(c)),
                    Join3State::None => {}
                }
                lacc = Join3State::None;
                output.push(i);
            }
        }
    }

    match lacc {
        Join3State::String(str) => output.push(InlinedExpression::Literal(str)),
        Join3State::Char(c) => output.push(InlinedExpression::Char(c)),
        Join3State::None => {}
    }

    output
}

impl InlinedExpression {
    pub fn optimize(self) -> Self {
        match self {
            InlinedExpression::Literal(literal) if literal.is_empty() => InlinedExpression::Nothing,
            InlinedExpression::Literal(literal) if literal.len() == 1 => {
                InlinedExpression::Char(literal.chars().next().unwrap())
            }
            InlinedExpression::Optional(inner) => {
                let inner = inner.optimize();
                match inner {
                    InlinedExpression::Multiple(innerer) => InlinedExpression::Some(innerer),
                    InlinedExpression::Nothing
                    | InlinedExpression::Optional(_)
                    | InlinedExpression::Some(_) => inner,
                    _ => InlinedExpression::Optional(Box::new(inner)),
                }
            }
            InlinedExpression::Multiple(inner) => {
                let inner = inner.optimize();
                match inner {
                    InlinedExpression::Optional(innerer) => InlinedExpression::Some(innerer),
                    InlinedExpression::Nothing
                    | InlinedExpression::Multiple(_)
                    | InlinedExpression::Some(_) => inner,
                    _ => InlinedExpression::Multiple(Box::new(inner)),
                }
            }
            InlinedExpression::Some(inner) => {
                let inner = inner.optimize();
                match inner {
                    InlinedExpression::Optional(innerer) | InlinedExpression::Multiple(innerer) => {
                        InlinedExpression::Some(innerer)
                    }
                    InlinedExpression::Nothing | InlinedExpression::Some(_) => inner,
                    _ => InlinedExpression::Some(Box::new(inner)),
                }
            }
            // TODO: doesn't cover alt(x, opt(alt(y))) which should result in opt(alt(x, y))
            InlinedExpression::Alternative(alt) => {
                let mut flat = Vec::with_capacity(alt.len());
                for a in alt.into_iter().map(InlinedExpression::optimize) {
                    if let InlinedExpression::Alternative(inner) = a {
                        flat.extend(inner);
                    } else {
                        flat.push(a);
                    }
                }

                let clean = alt_merge(flat);

                if clean.len() == 1 {
                    clean.into_iter().next().unwrap()
                } else {
                    InlinedExpression::Alternative(clean)
                }
            }
            InlinedExpression::Joined(join) => {
                let mut flat = Vec::with_capacity(join.len());
                for j in join.into_iter().map(InlinedExpression::optimize) {
                    if let InlinedExpression::Joined(inner) = j {
                        flat.extend(inner);
                    } else {
                        flat.push(j);
                    }
                }

                let clean = join_merge(flat);

                if clean.is_empty() {
                    InlinedExpression::Nothing
                } else if clean.len() == 1 {
                    clean.into_iter().next().unwrap()
                } else {
                    InlinedExpression::Joined(clean)
                }
            }
            _ => self,
        }
    }
}

pub fn process(source: Vec<parser::Definition<'_>>) -> Result<InlinedExpression, String> {
    let mut definitions = HashMap::new();

    for definition in source {
        if definitions.contains_key(definition.name) {
            return Err(format!("duplicate definition: {}", definition.name));
        }
        definitions.insert(definition.name, definition.value);
    }

    let root = definitions.get("EXPORT").ok_or("EXPORT is not defined")?;

    expand(root, &definitions, &mut HashSet::new())
}

fn expand<'a>(
    expr: &parser::Expression<'a>,
    defs: &HashMap<&'a str, parser::Expression<'a>>,
    rec: &mut HashSet<&'a str>,
) -> Result<InlinedExpression, String> {
    match expr {
        Expression::Literal(s) => Ok(InlinedExpression::Literal(s.to_owned())),
        Expression::Builtin(b) => Ok(InlinedExpression::Builtin(*b)),
        Expression::Ident(name) => {
            if !rec.insert(name) {
                return Err(format!(
                    "recursive loop detected: {} all depend on each other",
                    rec.iter().copied().collect::<Vec<&'a str>>().join(", ")
                ));
            }

            let expr = defs.get(name).ok_or(format!("{name} is not defined"))?;
            expand(expr, defs, rec)
        }
        Expression::Optional(o) => Ok(InlinedExpression::Optional(Box::new(expand(o, defs, rec)?))),
        Expression::Multiple(o) => Ok(InlinedExpression::Multiple(Box::new(expand(o, defs, rec)?))),
        Expression::Alternative(v) => {
            let mut vec = Vec::with_capacity(v.len());
            for expr in v {
                vec.push(expand(expr, defs, rec)?);
            }
            Ok(InlinedExpression::Alternative(vec))
        }
        Expression::Joined(v) => {
            let mut vec = Vec::with_capacity(v.len());
            for expr in v {
                vec.push(expand(expr, defs, rec)?);
            }
            Ok(InlinedExpression::Joined(vec))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lit() -> InlinedExpression {
        InlinedExpression::Char('x')
    }

    fn lit_d(c: char) -> InlinedExpression {
        InlinedExpression::Char(c)
    }

    fn opt(x: InlinedExpression) -> InlinedExpression {
        InlinedExpression::Optional(Box::new(x))
    }

    fn mul(x: InlinedExpression) -> InlinedExpression {
        InlinedExpression::Multiple(Box::new(x))
    }

    fn some(x: InlinedExpression) -> InlinedExpression {
        InlinedExpression::Some(Box::new(x))
    }

    fn alt(x: Vec<InlinedExpression>) -> InlinedExpression {
        InlinedExpression::Alternative(x)
    }

    fn join(x: Vec<InlinedExpression>) -> InlinedExpression {
        InlinedExpression::Joined(x)
    }

    macro_rules! gen_test {
        ($name:ident, $input:expr, $expected:expr) => {
            #[test]
            fn $name() {
                let result = $input.optimize();
                assert_eq!(result, $expected);
            }
        };
    }

    gen_test!(opt_opt, opt(opt(lit())), opt(lit()));
    gen_test!(opt_mul, opt(mul(lit())), some(lit()));
    gen_test!(opt_some, opt(some(lit())), some(lit()));
    gen_test!(mul_opt, mul(opt(lit())), some(lit()));
    gen_test!(mul_mul, mul(mul(lit())), mul(lit()));
    gen_test!(mul_some, mul(some(lit())), some(lit()));
    gen_test!(some_opt, some(opt(lit())), some(lit()));
    gen_test!(some_mul, some(mul(lit())), some(lit()));
    gen_test!(some_some, some(some(lit())), some(lit()));

    gen_test!(opt_opt_mul, opt(opt(mul(lit()))), some(lit()));
    gen_test!(opt_mul_opt, opt(mul(opt(lit()))), some(lit()));
    gen_test!(mul_mul_opt, mul(mul(opt(lit()))), some(lit()));
    gen_test!(mul_opt_mul, mul(opt(mul(lit()))), some(lit()));

    gen_test!(opt_mul_mul_opt, opt(mul(mul(opt(lit())))), some(lit()));

    gen_test!(alt_opt_mul, alt(vec![opt(mul(lit()))]), some(lit()));

    gen_test!(join_mul_opt, join(vec![mul(opt(lit()))]), some(lit()));

    gen_test!(
        alt3,
        alt(vec![alt(vec![lit_d('a'), lit_d('b')]), lit_d('c')]),
        alt(vec![lit_d('a'), lit_d('b'), lit_d('c')])
    );
}
