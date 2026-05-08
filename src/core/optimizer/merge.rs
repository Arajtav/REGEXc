use crate::core::{optimizer::flatten::flatten, processor::InlinedExpression};

// TODO: what is that
pub fn merge(input: Vec<InlinedExpression>) -> Vec<InlinedExpression> {
    let mut output = Vec::with_capacity(input.len());

    'outer: for new in input.into_iter().map(flatten) {
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

            if let (InlinedExpression::Builtin(new), InlinedExpression::Literal(existing)) =
                (&new, &item)
                && existing.len() == 1
                && let Some(c) = existing.chars().next()
                && new.contains(c)
            {
                *item = InlinedExpression::Builtin(*new);
                continue 'outer;
            }

            if let (InlinedExpression::Literal(new), InlinedExpression::Builtin(existing)) =
                (&new, &item)
                && new.len() == 1
                && let Some(c) = new.chars().next()
                && existing.contains(c)
            {
                continue 'outer;
            }
        }

        output.push(new);
    }

    output
}
