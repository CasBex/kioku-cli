use clap::Parser;
use directories;
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
    /// Remove default word list from system
    #[arg(long)]
    remove_wordlist: bool,
}

#[derive(Debug)]
enum WordlistErr {
    FileErrStripped(io::Error),
    FileErr(String, io::Error),
    NotWordList,
}

impl WordlistErr {
    fn strip_filename(self) -> Self {
        match self {
            Self::FileErr(_, b) => Self::FileErrStripped(b),
            e => e,
        }
    }
}

impl fmt::Display for WordlistErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FileErrStripped(e) => e.fmt(f),
            Self::FileErr(fil, e) => {
                write!(f, "{}: ", fil)?;
                e.fmt(f)
            }
            Self::NotWordList => write!(f, "Wordlist badly formatted"),
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

fn parse_wordlist(filename: &std::path::PathBuf) -> Result<Vec<String>, WordlistErr> {
    let to_err = |e| WordlistErr::FileErr(filename.to_string_lossy().into_owned(), e);
    io::BufReader::new(fs::File::open(filename).map_err(to_err)?)
        .lines()
        .map(|x| {
            x.map_err(to_err).and_then(|y| {
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

fn default_wordlist_path() -> Option<std::path::PathBuf> {
    directories::ProjectDirs::from("com", "CasBex", "kioku")
        .map(|dirs| dirs.data_local_dir().join("wordlist.txt"))
}

fn ensure_wordlist() -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
    const URL: &str = "http://localhost:8080/assets/wordlist.txt";

    let path = default_wordlist_path().ok_or("cannot determine default wordlist location")?;

    if !path.exists() {
        print!(
            "Could not find default wordlist. Install it from {}?\nY/n> ",
            URL
        );
        let mut input = String::new();
        io::stdout().flush().unwrap();
        io::stdin().read_line(&mut input).unwrap();
        println!("");
        if input.trim().to_lowercase().starts_with("n") {
            return Err("Not installing default wordlist".into());
        }
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        println!("Downloading wordlist...");
        let txt = reqwest::blocking::get(URL)?.text()?;
        std::fs::write(&path, txt)?;
        println!("Saved wordlist to {}", path.display());
    }

    Ok(path)
}

fn remove_wordlist() -> Result<(), Box<dyn std::error::Error>> {
    let path = default_wordlist_path().ok_or("cannot determine default wordlist location")?;
    Ok(fs::remove_file(path)?)
}

fn inner_main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    if cli.remove_wordlist {
        return remove_wordlist();
    }
    let filepath = cli.words;
    let wordlist = if let Some(fpath) = filepath {
        parse_wordlist(&fpath)?
    } else {
        let fpath = ensure_wordlist()?;
        parse_wordlist(&fpath)
            .map_err(|e| format!("Error loading default wordlist: {}", e.strip_filename()))?
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
