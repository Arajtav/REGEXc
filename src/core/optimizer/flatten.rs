use crate::core::{optimizer::optimize, processor::InlinedExpression};

pub fn flatten(expr: InlinedExpression) -> InlinedExpression {
    match expr {
        InlinedExpression::Optional(v) => {
            let inner = optimize(*v);

            match inner {
                InlinedExpression::Optional(i) => InlinedExpression::Optional(i),
                InlinedExpression::Multiple(i) | InlinedExpression::Some(i) => {
                    InlinedExpression::Some(i)
                }
                other => InlinedExpression::Optional(Box::new(other)),
            }
        }

        InlinedExpression::Multiple(v) => {
            let inner = optimize(*v);

            match inner {
                InlinedExpression::Multiple(i) => InlinedExpression::Multiple(i),
                InlinedExpression::Optional(i) | InlinedExpression::Some(i) => {
                    InlinedExpression::Some(i)
                }
                other => InlinedExpression::Multiple(Box::new(other)),
            }
        }

        InlinedExpression::Some(v) => {
            let inner = optimize(*v);

            match inner {
                InlinedExpression::Optional(i)
                | InlinedExpression::Multiple(i)
                | InlinedExpression::Some(i) => InlinedExpression::Some(i),
                other => InlinedExpression::Some(Box::new(other)),
            }
        }

        InlinedExpression::Alternative(v) => {
            InlinedExpression::Alternative(v.into_iter().map(optimize).collect())
        }

        InlinedExpression::Joined(v) => {
            InlinedExpression::Joined(v.into_iter().map(optimize).collect())
        }

        other => other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lit() -> InlinedExpression {
        InlinedExpression::Literal("x".to_string())
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
                let result = flatten($input);
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

    gen_test!(
        alt_opt_mul,
        alt(vec![opt(mul(lit()))]),
        alt(vec![some(lit())])
    );

    gen_test!(
        join_mul_opt,
        join(vec![mul(opt(lit()))]),
        join(vec![some(lit())])
    );
}
