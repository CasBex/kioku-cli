use clap::Parser;
use rand::prelude::*;
use std::fmt;
use std::fs;
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
enum WordlistErr {
    FileErr(String, io::Error),
}

impl fmt::Display for WordlistErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FileErr(fil, e) => {
                write!(f, "{}: ", fil)?;
                e.fmt(f)
            }
        }
    }
}

impl std::error::Error for WordlistErr {}

#[derive(serde::Serialize)]
struct MetaData {
    label: String,
    revision: Option<String>,
    timestamp: String,
}

static WORDLIST: &str = include_str!("../assets/wordlist.txt");

fn wordlist_filter_map<'a>(word: &'a str, dowarn: &mut bool) -> Option<&'a str> {
    let tw = word.trim();
    if tw.contains(char::is_whitespace) {
        if *dowarn {
            eprintln!("Wordlist contains invalid words, discarding");
            *dowarn = false;
        }
        None
    } else {
        Some(tw)
    }
}

fn parse_wordlist(filename: &std::path::PathBuf) -> Result<Vec<String>, WordlistErr> {
    let to_err = |e| WordlistErr::FileErr(filename.to_string_lossy().into_owned(), e);
    let mut dowarn = true;
    Ok(
        io::BufReader::new(fs::File::open(filename).map_err(to_err)?)
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

fn generate_metadata(filename: &str, slug: &str) -> Result<(), io::Error> {
    let revision = git2::Repository::discover(".").ok().and_then(|rep| {
        rep.head()
            .ok()
            .and_then(|head| head.target())
            .map(|oid| oid.to_string())
    });
    let timestamp = chrono::Local::now().to_rfc3339();
    let meta = MetaData {
        label: slug.to_string(),
        revision,
        timestamp,
    };
    let mut opener = std::fs::OpenOptions::new();
    opener.create(true);
    if filename.ends_with(".jsonl") {
        opener.append(true);
    } else {
        opener.write(true);
    }
    let writer = if filename.ends_with(".jsonl") || filename.ends_with(".json") {
        io::BufWriter::new(opener.open(filename)?)
    } else {
        let mut tmp = String::from(filename);
        tmp.push_str(".json");
        io::BufWriter::new(opener.open(tmp)?)
    };
    serde_json::to_writer_pretty(writer, &meta).unwrap();
    Ok(())
}

fn inner_main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let filepath = cli.words;
    let wordlist = if let Some(fpath) = filepath {
        parse_wordlist(&fpath)?
    } else {
        ensure_wordlist()
    };
    let name = generate_name(&wordlist, cli.length);
    if let Some(timestamp) = cli.output {
        generate_metadata(timestamp.as_str(), name.as_str())?;
    }
    println!("{}", name);
    Ok(())
}

fn main() {
    if let Err(e) = inner_main() {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}
