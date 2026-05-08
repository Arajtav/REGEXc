use crate::core::{optimizer::flatten::flatten, inliner::InlinedExpression};

mod flatten;

pub fn optimize(expr: InlinedExpression) -> InlinedExpression {
    flatten(expr)
}
