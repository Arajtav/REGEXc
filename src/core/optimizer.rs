use crate::core::{optimizer::flatten::flatten, processor::InlinedExpression};

mod flatten;

pub fn optimize(expr: InlinedExpression) -> InlinedExpression {
    flatten(expr)
}
