# Kioku

A CLI to generate human-readable names and associated metadata for labelling software experiments.

The first three letters of each word in the default word list are unique to support easy tab completion and words contain only ascii lowercase letters.
The default word list has been manually checked for profanity or otherwise triggering language, but this check is subject to human error. 
If you do find a word that you believe to be disrespectful, derogatory or otherwise inappropriate please file an issue.

## Installation
Download the binary for your distribution from the Github releases page and put it in your PATH.
Alternatively, run the installation script on the releases page, which will perform these actions for you.

## Usage

Use the default word list to generate a name of chosen length.
```bash
$ kioku
gene-ruin-note
$ kioku -l 5
robe-speed-fake-wedge-sash
```

Generate a metadata file with time stamp and git commit hash.
```
# overwrites meta.json
$ kioku -o meta.json
gene-ruin-note
$ cat meta.json
{
  "label": "fund-nose-cord",
  "revision": "84cf86e230009fefe779a47b92052b90f83bf504",
  "timestamp": "2026-01-15T07:40:09.310648479+00:00"
}
```
If you prefer to have a single file with multiple metadata entries instead of multiple small files, use the [jsonlines](https://jsonlines.org/) format.
```
# appends to meta.jsonl
$ kioku -o meta.jsonl
gene-ruin-note
```

Use a custom word list
```
$ echo "beetlejuice" > mywords.txt
$ kioku -w mywords.txt
beetlejuice-beetlejuice-beetlejuice
```
Word lists should be text files with one word per line.
Trailing whitespace is allowed, but otherwise only uppercase and lowercase ascii characters may be used for words.


## Origin of the name
Kioku (記憶) is Japanese for [memory, remembrance](https://jisho.org/search/kioku).

This tool is not affiliated with the many other similarly named software projects.
