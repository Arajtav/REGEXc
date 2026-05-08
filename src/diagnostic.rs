use std::{fmt::Display, path::Path};

#[derive(Debug, Clone, Copy)]
pub struct Span {
    start: usize,
    end: usize,
}

impl From<std::ops::Range<usize>> for Span {
    fn from(value: std::ops::Range<usize>) -> Self {
        Self {
            start: value.start,
            end: value.end,
        }
    }
}

pub struct Diagnostic<'a> {
    pub path: &'a Path,
    pub source: &'a str,
    pub error: String,
    pub span: Option<Span>,
}

impl Diagnostic<'_> {
    fn offset_to_line_col(&self, offset: usize) -> (usize, usize) {
        let mut line = 1;
        let mut col = 1;

        for (i, ch) in self.source.char_indices() {
            if i >= offset {
                break;
            }

            if ch == '\n' {
                line += 1;
                col = 1;
            } else {
                col += 1;
            }
        }

        (line, col)
    }

    pub fn resolve_positions(&self) -> Option<((usize, usize), (usize, usize))> {
        let span = self.span?;
        Some((
            self.offset_to_line_col(span.start),
            self.offset_to_line_col(span.end),
        ))
    }
}

impl Display for Diagnostic<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(((start_line, start_col), (end_line, end_col))) = self.resolve_positions() {
            writeln!(
                f,
                "An error occurred in {} at {start_line}:{start_col} (to {end_line}:{end_col}):",
                self.path.display()
            )
        } else {
            writeln!(f, "An error occurred in {}:", self.path.display())
        }?;
        writeln!(f, "{}", self.error)
    }
}
