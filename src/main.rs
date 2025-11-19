// Copyright 2021 Google LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     https://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use aarch64_esr_decoder::{FieldInfo, decode, decode_midr, decode_smccc, parse_number};
use colored::Colorize;
use std::env;
use std::ops::Deref;
use std::process::exit;

fn main() {
    let args = match parse_args() {
        Ok(args) => args,
        Err(error_code) => exit(error_code),
    };

    let value = parse_number(&args.value).unwrap();
    let decoded = match args.mode {
        Mode::Esr => {
            println!("{} {}:", "ESR".cyan().bold(), format!("{value:#034x}").yellow().bold());
            decode(value).unwrap()
        }
        Mode::Midr => {
            println!("{} {}:", "MIDR".cyan().bold(), format!("{value:#034x}").yellow().bold());
            decode_midr(value).unwrap()
        }
        Mode::Smccc => {
            println!("{} {}:", "SMC ID".cyan().bold(), format!("{value:#018x}").yellow().bold());
            decode_smccc(value).unwrap()
        }
    };
    print_decoded(&decoded, args.verbose, 0);
}

fn print_decoded(fields: &[FieldInfo], verbose: bool, level: usize) {
    let indentation = " ".repeat(level * 2);
    for field in fields {
        let verbose_name = match field.long_name {
            Some(long_name) if verbose => format!("{}", format!(" ({long_name})").white()),
            _ => "".to_string(),
        };

        if field.width == 1 {
            let idx = format!("{:02}", field.start).white();
            let name = field.name.cyan().bold();
            let val = if field.value == 1 {
                "true".green().bold()
            } else {
                "false".red().bold()
            };
            println!("{}{}     {}: {}{}", indentation, idx, name, val, verbose_name);
        } else {
            let range = format!("{:02}..{:02}", field.start, field.start + field.width - 1).white();
            let name = field.name.cyan().bold();
            let hex = field.value_string().green().bold();
            let bin = field.value_binary_string().white();
            println!("{}{} {}: {} {}{}", indentation, range, name, hex, bin, verbose_name);
        }
        if let Some(description) = &field.description {
            let neon = (255u8, 140u8, 0u8); // neon orange approximation
            let line = format!("# {}", description);
            let mut out = String::new();

            let mut cursor = 0usize;
            while let Some(rel_hash) = line[cursor..].find('#') {
                let hash_idx = cursor + rel_hash;
                if hash_idx > cursor {
                    out.push_str(&line[cursor..hash_idx].white().to_string());
                }

                // Find end of sentence (first occurrence of ., !, or ? after '#')
                let after_hash = &line[hash_idx..];
                let mut end_idx = line.len();
                for term in ['.', '!', '?'] {
                    if let Some(p) = after_hash.find(term) {
                        let candidate = hash_idx + p + 1; // include the terminator
                        if candidate < end_idx {
                            end_idx = candidate;
                        }
                    }
                }

                if end_idx == line.len() {
                    out.push_str(
                        &line[hash_idx..]
                            .truecolor(neon.0, neon.1, neon.2)
                            .bold()
                            .to_string(),
                    );
                    cursor = line.len();
                    break;
                } else {
                    out.push_str(
                        &line[hash_idx..end_idx]
                            .truecolor(neon.0, neon.1, neon.2)
                            .bold()
                            .to_string(),
                    );
                    cursor = end_idx;
                }
            }

            if cursor < line.len() {
                out.push_str(&line[cursor..].white().to_string());
            }

            println!("{}  {}", indentation, out);
        }

        print_decoded(&field.subfields, verbose, level + 1);
    }
}

/// Parse and return command-line arguments, or an error code to return.
fn parse_args() -> Result<Args, i32> {
    let args: Vec<String> = env::args().collect();
    let args: Vec<&str> = args.iter().map(Deref::deref).collect();
    match args.as_slice() {
        [_, esr] => Ok(Args {
            verbose: false,
            mode: Mode::Esr,
            value: esr.to_string(),
        }),
        [_, "-v", esr] => Ok(Args {
            verbose: true,
            mode: Mode::Esr,
            value: esr.to_string(),
        }),
        [_, "midr", midr] => Ok(Args {
            verbose: false,
            mode: Mode::Midr,
            value: midr.to_string(),
        }),
        [_, "-v", "midr", midr] => Ok(Args {
            verbose: true,
            mode: Mode::Midr,
            value: midr.to_string(),
        }),
        [_, "smccc", smccc] => Ok(Args {
            verbose: false,
            mode: Mode::Smccc,
            value: smccc.to_string(),
        }),
        [_, "-v", "smccc", smccc] => Ok(Args {
            verbose: true,
            mode: Mode::Smccc,
            value: smccc.to_string(),
        }),
        _ => {
            eprintln!("Usage:");
            eprintln!("  {} [-v] <ESR value>", args[0]);
            eprintln!("  {} [-v] midr <MIDR value>", args[0]);
            eprintln!("  {} [-v] smccc <SMCCC function ID>", args[0]);
            Err(1)
        }
    }
}

/// Command-line arguments.
#[derive(Clone, Debug, Eq, PartialEq)]
struct Args {
    verbose: bool,
    mode: Mode,
    value: String,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum Mode {
    Esr,
    Midr,
    Smccc,
}
