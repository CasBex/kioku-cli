use anyhow::Context;
use clap::Parser;
use rand::prelude::*;
use std::fmt;
use std::fs;
use std::io::Write;
use std::io::{self, BufRead};

#[derive(Parser)]
#[command(version, about="Generate random human-readable strings for naming experiments and log associated metadata", long_about = None)] // Read from `Cargo.toml`

struct Cli {
    /// Length of the generated name in words
    #[arg(short, long, value_name = "LENGTH", default_value = "3")]
    length: usize,
    /// Output metadata in JSON format to <FILE>
    #[arg(short, long, value_name = "FILE")]
    output: Option<String>,
    /// Specify wordlist to use
    #[arg(short, long, value_name = "WORDLIST")]
    words: Option<std::path::PathBuf>,
}

#[derive(Debug)]
enum KiokuErr {
    BrokenPipe,
    ApplicationErr(anyhow::Error),
    IOErr(io::Error),
}

impl From<anyhow::Error> for KiokuErr {
    fn from(value: anyhow::Error) -> Self {
        KiokuErr::ApplicationErr(value)
    }
}

impl From<io::Error> for KiokuErr {
    fn from(value: io::Error) -> Self {
        match value.kind() {
            io::ErrorKind::BrokenPipe => KiokuErr::BrokenPipe,
            _ => KiokuErr::IOErr(value),
        }
    }
}

impl fmt::Display for KiokuErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            KiokuErr::BrokenPipe => Ok(()),
            KiokuErr::ApplicationErr(e) => write!(f, "{:#}", e),
            KiokuErr::IOErr(e) => e.fmt(f),
        }
    }
}

impl std::error::Error for KiokuErr {}

#[derive(serde::Serialize)]
struct MetaData<'a> {
    label: &'a str,
    revision: Option<String>,
    timestamp: String,
}

static WORDLIST: &str = include_str!("../assets/wordlist.txt");

fn wordlist_filter_map<'a>(word: &'a str, dowarn: &mut bool) -> Option<&'a str> {
    let tw = word.trim();
    if tw
        .chars()
        .all(|x| char::is_ascii_lowercase(&x) || char::is_ascii_uppercase(&x))
    {
        Some(tw)
    } else {
        if *dowarn {
            eprintln!("Wordlist contains invalid words, discarding");
            *dowarn = false;
        }
        None
    }
}

fn parse_wordlist(filename: &std::path::PathBuf) -> anyhow::Result<Vec<String>> {
    let mut dowarn = true;
    Ok(
        io::BufReader::new(fs::File::open(filename).with_context(|| {
            format!(
                "Failed to read wordlist file {}",
                filename.to_string_lossy()
            )
        })?)
        .lines()
        .filter_map(|x| x.ok())
        .filter_map(|x| wordlist_filter_map(x.as_str(), &mut dowarn).map(|y| y.to_string()))
        .collect(),
    )
}

fn ensure_wordlist() -> Vec<String> {
    WORDLIST
        .split_whitespace()
        .filter_map(|x| wordlist_filter_map(x, &mut false).map(|y| y.to_string()))
        .collect()
}

fn generate_name<'a>(wordlist: &'a [String], num_words: usize) -> String {
    let mut rng = rand::rng();
    let mut output = String::new();
    for word in (0..num_words).map(|_| wordlist[rng.random_range(0..wordlist.len())].as_str()) {
        if output.len() == 0 {
            output.push_str(word);
        } else {
            output.push('-');
            output.push_str(word);
        }
    }
    output
}

fn generate_metadata(filename: &str, slug: &str) -> anyhow::Result<()> {
    let revision = git2::Repository::discover(".").ok().and_then(|rep| {
        rep.head()
            .ok()
            .and_then(|head| head.target())
            .map(|oid| oid.to_string())
    });
    let timestamp = chrono::Local::now().to_rfc3339();
    let meta = MetaData {
        label: slug,
        revision,
        timestamp,
    };
    let mut opener = fs::OpenOptions::new();
    opener.create(true);
    if filename.ends_with(".jsonl") {
        opener.append(true);
    } else {
        opener.write(true).truncate(true);
    }
    let fname = if filename.ends_with(".jsonl") || filename.ends_with(".json") {
        filename.to_string()
    } else {
        format!("{}.json", filename)
    };
    let mut writer = io::BufWriter::new(
        opener
            .open(fname.as_str())
            .with_context(|| format!("Failed to write metadata file {}", fname))?,
    );

    serde_json::to_writer_pretty(&mut writer, &meta).unwrap();
    writer.write("\n".as_bytes()).unwrap();
    Ok(())
}

fn inner_main() -> Result<(), KiokuErr> {
    let cli = Cli::parse();
    let filepath = cli.words;
    let wordlist = if let Some(fpath) = filepath {
        parse_wordlist(&fpath)?
    } else {
        ensure_wordlist()
    };
    let name = generate_name(&wordlist, cli.length);
    writeln!(io::stdout(), "{}", name)?;
    if let Some(timestamp) = cli.output {
        generate_metadata(timestamp.as_str(), name.as_str())?;
    }
    Ok(())
}

fn main() {
    if let Err(e) = inner_main() {
        match e {
            KiokuErr::BrokenPipe => {
                std::process::exit(141);
            }
            e => {
                eprintln!("{}", e);
                std::process::exit(1);
            }
        }
    }
}
