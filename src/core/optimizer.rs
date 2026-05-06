use crate::core::processor::ProcessedExpression;

pub fn merge_nested(expr: ProcessedExpression) -> ProcessedExpression {
    match expr {
        ProcessedExpression::Optional(inner) => {
            let mut merged = merge_nested(*inner);

            if let ProcessedExpression::Optional(next) = merged {
                merged = *next;
            }

            ProcessedExpression::Optional(Box::new(merged))
        }

        ProcessedExpression::Alternative(v) => {
            ProcessedExpression::Alternative(v.into_iter().map(merge_nested).collect())
        }

        ProcessedExpression::Joined(v) => {
            ProcessedExpression::Joined(v.into_iter().map(merge_nested).collect())
        }
        _ => expr,
    }
}
