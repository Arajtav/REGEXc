mod core;

use std::{fs, path::Path};

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
pub enum RegexKind {
    PosixEre,
    Pcre,
    Ecmascript,
    Python,
    Java,
    Re2,
}

fn main() {
    let cli = Cli::parse();

    let input = match fs::read_to_string(cli.input) {
        Ok(input) => input,
        Err(err) => {
            eprintln!("Failed to read the input file: {err}");
            return;
        }
    };

    let regex = match core::compile(&input, cli.kind) {
        Ok(res) => res,
        Err(err) => {
            eprintln!("{err}");
            return;
        }
    };

    if let Some(output) = cli.out {
        if let Err(err) = fs::write(output, regex) {
            eprintln!("Failed to write to the output file: {err}");
        }
    } else {
        println!("{regex}");
    }
}
