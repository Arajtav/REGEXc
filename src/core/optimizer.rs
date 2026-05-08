use crate::core::{optimizer::flatten::flatten, processor::InlinedExpression};

mod flatten;
mod merge;

pub fn optimize(expr: InlinedExpression) -> InlinedExpression {
    flatten(expr)
}
