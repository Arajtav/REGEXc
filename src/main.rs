use std::path::Path;

use clap::{Parser, ValueEnum};

#[derive(Debug, Parser)]
#[command(arg_required_else_help = true)]
struct Cli {
    /// Output file location, if not present the output will be printed to stdout.
    #[arg(short = 'o', long)]
    out: Option<Box<Path>>,

    /// Regex kind, different regex engines support different features and use different syntax.
    #[arg(long, value_enum, default_value_t = RegexKind::PosixEre)]
    kind: RegexKind,

    /// Input file.
    input: Box<Path>,
}

#[derive(Debug, ValueEnum, Clone, Copy, PartialEq, Eq)]
#[clap(rename_all = "snake_case")]
enum RegexKind {
    PosixEre,
    Pcre,
    Ecmascript,
    Python,
    Java,
    Re2,
}

fn main() {
    let cli = Cli::parse();
    dbg!(cli);
}
