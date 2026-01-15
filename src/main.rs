use rand::prelude::*;
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

fn main() -> Result<(), WordlistErr> {
    let args: Vec<String> = std::env::args().collect();
    let filepath = &args[1];
    let wordlist = parse_wordlist(filepath)?;
    println!("{}", generate_name(&wordlist, 3));
    Ok(())
}
