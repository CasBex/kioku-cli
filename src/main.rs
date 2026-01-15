use chrono::prelude::*;
use rand::prelude::*;
use std::fmt;
use std::fs;
use std::io::{self, BufRead};

#[derive(Debug)]
enum WordlistErr {
    FileErr(io::Error),
    NotWordList,
}

impl fmt::Display for WordlistErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FileErr(e) => e.fmt(f),
            Self::NotWordList => write!(f, "Wordlist badly formatted"),
        }
    }
}

impl std::error::Error for WordlistErr {}

impl From<io::Error> for WordlistErr {
    fn from(value: io::Error) -> Self {
        Self::FileErr(value)
    }
}

#[derive(serde::Serialize)]
struct MetaData {
    label: String,
    revision: Option<String>,
    timestamp: String,
}

fn parse_wordlist(filename: &str) -> Result<Vec<String>, WordlistErr> {
    io::BufReader::new(fs::File::open(filename)?)
        .lines()
        .map(|x| {
            x.map_err(WordlistErr::from).and_then(|y| {
                let ty = y.trim();
                if ty.contains(char::is_whitespace) {
                    Err(WordlistErr::NotWordList)
                } else {
                    Ok(String::from(ty))
                }
            })
        })
        .collect()
}

fn generate_name<'a>(wordlist: &'a Vec<String>, num_words: usize) -> String {
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

fn generate_metadata(filename: &str, slug: &str, use_utc: bool) -> Result<(), io::Error> {
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let filepath = &args[1];
    let wordlist = parse_wordlist(filepath)?;
    // generate_metadata("tmp.jsonl", "test", true)?;
    println!("{}", generate_name(&wordlist, 3));
    Ok(())
}
