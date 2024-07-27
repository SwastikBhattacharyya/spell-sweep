# Spell Sweep

Spell Sweep is a spell checker for the English dictionary. It takes input from a file or piped input and asks the user for corrections for misspelled words.

## Description

Spell sweep is a simple spell checker that uses a dictionary file to generate a Bloom Filter and a BK Tree using Damerau-Levenshtein distance to suggest corrections for misspelled words. The dictionary is created using a text file obtained from [dwyl's repository](https://github.com/dwyl/english-words). Each word is processed into three parts, punctuations at the beginning, the word itself, and punctuations at the end. The punctuations are used to reconstruct the word after the correction is made. Each word is passed into the Bloom Filter to check if it is a valid word. If the Bloom filter returns that the word is not present, the BK Tree is used to suggest corrections. If the Bloom suggests that the word may be present, the BK Tree is used to confirm the presence of the word. If the word is not present in the dictionary, the user is asked for a correction.

## Usage

Spell Sweep can be used in two ways:

1. By providing a file as input:
```bash
./spell_sweep -f <file>
```

2. By piping input:
```bash
<command> | ./spell_sweep
```

## Installation

To install Spell Sweep, clone the repository and run the following commands:
```bash
cargo build --release
```
The executable will be present in the **"target/release/"** directory. Make sure that the dictionary.txt file is present in the same directory as the executable.
