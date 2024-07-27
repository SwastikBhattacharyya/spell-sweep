use std::{
    fs::{File, OpenOptions},
    io::{self, BufRead, BufReader, Write},
    path::Path,
};

use crate::{bk_tree::BKTree, bloom_filter::BloomFilter, dictionary::Dictionary, processor};

#[readonly::make]
pub struct SpellCheck {
    bk_tree: BKTree,
    bloom_filter: BloomFilter,
}

impl SpellCheck {
    pub fn new(
        bk_tree_path: &str,
        bloom_filter_path: &str,
        dictionary_path: &str,
        alphabet_length: u16,
    ) -> Self {
        let bk_tree: BKTree;
        let bloom_filter: BloomFilter;
        let mut dictionary: Option<Dictionary> = None;

        if Path::new(bk_tree_path).exists() {
            bk_tree = BKTree::from(File::open(bk_tree_path).expect("Failed to open BKTree file"));
        } else {
            if dictionary.is_none() {
                dictionary = Some(Dictionary::from((
                    File::open(dictionary_path).expect("Failed to open dictionary file"),
                    alphabet_length,
                )));
            }
            bk_tree = BKTree::from(dictionary.as_ref().unwrap());
            bk_tree
                .to_file(bk_tree_path)
                .expect("Failed to write BKTree to file");
        }

        if Path::new(bloom_filter_path).exists() {
            bloom_filter = BloomFilter::from(
                File::open(bloom_filter_path).expect("Failed to open BloomFilter file"),
            );
        } else {
            if dictionary.is_none() {
                dictionary = Some(Dictionary::from((
                    File::open(dictionary_path).expect("Failed to open dictionary file"),
                    alphabet_length,
                )));
            }
            bloom_filter = BloomFilter::from(dictionary.as_ref().unwrap());
            bloom_filter
                .to_file(bloom_filter_path)
                .expect("Failed to write BloomFilter to file");
        }

        Self {
            bk_tree,
            bloom_filter,
        }
    }

    fn handle_suggestions(word: &str, suggestions: Vec<&str>) -> String {
        println!("{} is incorrect.", word);
        for (idx, suggestion) in suggestions.iter().enumerate() {
            println!("Suggestion: {} -> {}", idx + 1, suggestion);
        }

        let idx = take_input();

        return suggestions[(idx - 1) as usize].to_string();
    }

    fn insert_suggestion(
        bk_tree: &BKTree,
        word: &str,
        lower_word: &str,
        joinable_vec: &mut Vec<(String, String, String)>,
        data: (String, String),
    ) {
        let mut tol_value = 1;
        let mut suggestions;
        loop {
            suggestions = bk_tree.get_similar_words(&lower_word, tol_value).unwrap();
            if suggestions.len() > 0 {
                break;
            }
            tol_value += 1;
        }
        joinable_vec.push((
            data.0,
            convert_case(
                SpellCheck::handle_suggestions(&lower_word, suggestions).as_str(),
                &word,
            ),
            data.1,
        ));
    }

    pub fn run(&self, cmd_data: String) {
        let mut joinable_vec = Vec::<(String, String, String)>::new();

        for (start_punc, word, end_punc) in processor::split_input(&cmd_data) {
            let lower_word = word.to_lowercase();
            if !self.bloom_filter.lookup(&lower_word) {
                SpellCheck::insert_suggestion(
                    &self.bk_tree,
                    &word,
                    &lower_word,
                    &mut joinable_vec,
                    (start_punc, end_punc),
                );
            } else {
                if self.bk_tree.does_contain(&lower_word).unwrap() {
                    joinable_vec.push((start_punc, convert_case(&lower_word, &word), end_punc));
                } else {
                    SpellCheck::insert_suggestion(
                        &self.bk_tree,
                        &word,
                        &lower_word,
                        &mut joinable_vec,
                        (start_punc, end_punc),
                    );
                }
            }
        }

        println!("{}", processor::join_input(joinable_vec));
    }
}

fn convert_case(sugg: &str, orig: &str) -> String {
    let mut result = String::new();

    for i in 0..sugg.len() {
        if orig.chars().nth(i).is_none() {
            result.push(sugg.chars().nth(i).unwrap());
        } else if !sugg.chars().nth(i).unwrap().is_alphanumeric() {
            result.push(sugg.chars().nth(i).unwrap());
        } else if orig.chars().nth(i).unwrap().is_lowercase() {
            result.push(sugg.chars().nth(i).unwrap().to_lowercase().next().unwrap());
        } else if orig.chars().nth(i).unwrap().is_uppercase() {
            result.push(sugg.chars().nth(i).unwrap().to_uppercase().next().unwrap());
        }
    }

    result
}

fn take_input() -> u32 {
    print!("Enter the suggestion number: ");
    io::stdout().flush().unwrap();

    let fd = OpenOptions::new()
        .read(true)
        .write(true)
        .open("/dev/tty")
        .unwrap();

    let mut reader = BufReader::new(fd);

    let mut input = String::new();
    reader.read_line(&mut input).unwrap();

    input.trim().parse::<u32>().unwrap()
}

#[cfg(test)]
mod tests {
    use super::SpellCheck;

    #[test]
    fn test_new() {
        let bk_tree_path: &str = "bk_tree_test.bin";
        let bloom_filter_path: &str = "bloom_filter_test.bin";
        let dictionary_path: &str = "dictionary.txt";
        let alphabet_length: u16 = 255;

        let spell_check: SpellCheck = SpellCheck::new(
            bk_tree_path,
            bloom_filter_path,
            dictionary_path,
            alphabet_length,
        );

        assert_ne!(spell_check.bk_tree.tree.len(), 0);
        assert_eq!(spell_check.bk_tree.alphabet_length, alphabet_length);

        let words_absent = ["clesr", "erroe", "hel;", "rivee", "jokeq", "fathep"];
        for word in words_absent {
            assert_eq!(spell_check.bloom_filter.lookup(word), false);
        }

        std::fs::remove_file(bk_tree_path).expect("Failed to remove BKTree file");
        std::fs::remove_file(bloom_filter_path).expect("Failed to remove BloomFilter file");
    }
}
