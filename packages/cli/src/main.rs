use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Seek};
use std::path::Path;
use std::sync::Once;

use clap::Parser;
use kitoken::{Definition, DeserializationError, Kitoken};

#[derive(Parser)]
enum Command {
    #[clap(name = "convert", about = "Convert a tokenizer model to a kitoken definition")]
    Convert {
        #[arg(name = "path", help = "Path to the tokenizer model")]
        path: String,
    },
    #[clap(name = "compare", about = "Compare two tokenizer models")]
    Compare {
        #[arg(name = "one", help = "Path to the first tokenizer model")]
        one: String,
        #[arg(name = "two", help = "Path to the second tokenizer model")]
        two: String,
    },
    #[clap(name = "inspect", about = "Inspect a tokenizer model")]
    Inspect {
        #[arg(name = "path", help = "Path to the tokenizer model")]
        path: String,
    },
    #[clap(name = "encode", about = "Encode text into tokens")]
    Encode {
        #[arg(name = "model", help = "Path to the tokenizer model")]
        model: String,
        #[arg(name = "path", help = "Path to the input file")]
        input: String,
    },
    #[clap(name = "decode", about = "Decode tokens into text")]
    Decode {
        #[arg(name = "model", help = "Path to the tokenizer model")]
        model: String,
        #[arg(name = "path", help = "Path to the input file")]
        input: String,
    },
}

#[derive(Parser)]
struct Args {
    #[clap(subcommand)]
    command: Command,
}

static INIT_ENV: Once = Once::new();

pub fn init_env() {
    INIT_ENV.call_once(|| {
        simple_logger::SimpleLogger::new()
            .with_level(log::Level::Info.to_level_filter())
            .env()
            .init()
            .unwrap();
    });
}

pub fn main() {
    init_env();

    let args = Args::parse();
    match args.command {
        Command::Convert { path } => {
            let path = Path::new(&path);
            let mut paths = Vec::new();
            if path.is_dir() {
                for entry in std::fs::read_dir(path).unwrap() {
                    let entry = entry.unwrap();
                    let path = entry.path();
                    if path.is_file() {
                        paths.push(path);
                    }
                }
            } else if path.is_file() {
                paths.push(path.to_path_buf());
            } else {
                eprintln!("Invalid path: {}", path.display());
                std::process::exit(1);
            }
            for path in paths {
                convert(&path, true).unwrap_or_else(|error| {
                    eprintln!("{}", error);
                    std::process::exit(1);
                });
            }
        }
        Command::Compare { one, two } => {
            let one = Path::new(&one);
            let two = Path::new(&two);
            let one = convert(one, false).unwrap_or_else(|error| {
                eprintln!("{}", error);
                std::process::exit(1);
            });
            let two = convert(two, false).unwrap_or_else(|error| {
                eprintln!("{}", error);
                std::process::exit(1);
            });
            if one != two {
                eprintln!("Models are different");
                if one.model.vocab() != two.model.vocab() {
                    let num_diff = one
                        .model
                        .vocab()
                        .iter()
                        .zip(two.model.vocab())
                        .filter(|(a, b)| a != b)
                        .count();
                    eprintln!("Vocabs are different: {} entries", num_diff);
                }
                if one.specials != two.specials {
                    let num_diff = one
                        .specials
                        .iter()
                        .zip(two.specials.iter())
                        .filter(|(a, b)| a != b)
                        .count();
                    eprintln!("Specials are different: {} entries", num_diff);
                }
                if one.config != two.config {
                    eprintln!("Configs are different");
                }
                std::process::exit(1);
            } else {
                println!("Models are the same");
            }
        }
        Command::Inspect { path } => {
            let path = Path::new(&path);
            let model = convert(path, false).unwrap_or_else(|error| {
                eprintln!("{}", error);
                std::process::exit(1);
            });
            println!("Specials: {:#?}", model.specials);
            println!("{:#?}", model);
        }
        Command::Encode { model, input } => {
            let model = Path::new(&model);
            let inputp = Path::new(&input);
            let model = convert(model, false).unwrap_or_else(|error| {
                eprintln!("{}", error);
                std::process::exit(1);
            });
            let encoder = Kitoken::from_definition(model).unwrap_or_else(|error| {
                eprintln!("{}", error);
                std::process::exit(1);
            });
            let mut buffer = String::with_capacity(1024);
            if inputp.is_file() {
                let mut reader = BufReader::new(File::open(inputp).unwrap());
                reader.read_to_string(&mut buffer).unwrap();
            } else {
                println!("No such file \"{}\", assuming literal input", input);
                buffer.push_str(&input);
            }
            let result = encoder.encode(&buffer, true).unwrap_or_else(|error| {
                eprintln!("{}", error);
                std::process::exit(1);
            });
            for token in result {
                print!("{} ", token);
            }
            println!()
        }
        Command::Decode { model, input } => {
            let model = Path::new(&model);
            let inputp = Path::new(&input);
            let model = convert(model, false).unwrap_or_else(|error| {
                eprintln!("{}", error);
                std::process::exit(1);
            });
            let encoder = Kitoken::from_definition(model).unwrap_or_else(|error| {
                eprintln!("{}", error);
                std::process::exit(1);
            });
            let mut buffer = String::with_capacity(1024);
            if inputp.is_file() {
                let mut reader = BufReader::new(File::open(input).unwrap());
                reader.read_to_string(&mut buffer).unwrap();
            } else {
                println!("No such file \"{}\", assuming literal input", input);
                buffer.push_str(&input);
            }
            let tokens = buffer
                .split(&[' ', ',', '\n'])
                .filter(|s| !s.is_empty())
                .map(str::parse)
                .collect::<Result<Vec<_>, _>>()
                .unwrap_or_else(|error| {
                    eprintln!("{}", error);
                    std::process::exit(1);
                });
            let result = encoder.decode(&tokens, true).unwrap_or_else(|error| {
                eprintln!("{}", error);
                std::process::exit(1);
            });
            println!("{}", String::from_utf8(result).unwrap());
        }
    }
}

pub fn convert(path: &Path, write: bool) -> Result<Definition, DeserializationError> {
    let mut reader = BufReader::new(File::open(path)?);
    let definition = Definition::from_reader(&mut reader)?;
    eprintln!("Definition loaded from {}", path.display());
    match definition.model {
        kitoken::Model::BytePair { .. } => eprintln!("Model type: BPE"),
        kitoken::Model::Unigram { .. } => eprintln!("Model type: Unigram"),
        kitoken::Model::WordPiece { .. } => eprintln!("Model type: WordPiece"),
        _ => {}
    }
    eprintln!("Vocab size: {}", definition.model.vocab().len());
    eprintln!("Specials size: {}", definition.specials.len());
    eprintln!("Input size: {} bytes", reader.stream_position()?);
    if write {
        let out = path.with_extension("kit");
        let mut writer = BufWriter::new(File::create(&out)?);
        definition.to_writer(&mut writer)?;
        eprintln!("Definition written to {}", out.display());
        eprintln!("Output size: {} bytes", writer.stream_position()?);
    }
    eprintln!();
    Ok(definition)
}
