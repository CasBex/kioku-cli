use std::fs;
use std::io::{self, BufRead};

#[derive(Debug)]
enum WordlistErr {
    FileErr(io::Error),
    NotWordList,
}

impl From<io::Error> for WordlistErr {
    fn from(value: io::Error) -> Self {
        Self::FileErr(value)
    }
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

fn main() -> Result<(), WordlistErr> {
    let args: Vec<String> = std::env::args().collect();
    let filepath = &args[1];
    let wordlist = parse_wordlist(filepath)?;
    println!("Read wordlist of length {}", wordlist.len());
    Ok(())
}
