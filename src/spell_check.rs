use std::{fs::File, path::Path};
use crate::{bk_tree::BKTree, bloom_filter::BloomFilter, dictionary::Dictionary, processor};

#[readonly::make]
pub struct SpellCheck {
    pub dictionary: Option<Dictionary>,
    pub bk_tree: Option<BKTree>,
    pub bloom_filter: Option<BloomFilter>
}

impl SpellCheck {
    pub fn new() -> Self {
       Self { dictionary: None, bk_tree: None, bloom_filter: None } 
    }

    pub fn populate_bk_tree(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if Path::new("bk_tree.bin").exists() {
            self.bk_tree = match BKTree::from_file("bk_tree.bin") {
                Ok(t) => Some(t),
                Err(_) => None
            };
        }
        else {
            if self.dictionary.is_none() {
                self.get_dictionary();
            }
            match &self.dictionary {
                Some(dictionary) => {
                    for word in dictionary.words.iter() {
                        match &mut self.bk_tree {
                            Some(tree) => tree.add(word.clone()),
                            None => return Err(Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, "BKTree not initialized")))
                        }
                    }
                },
                None => return Err(Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, "Dictionary not initialized")))
            }
        }
        match &self.bk_tree {
            Some(tree) => {
                match tree.to_file("bk_tree.bin") {
                    Ok(_) => Ok(()),
                    Err(_) => Err(Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, "Failed to write BKTree to file")))
                }
            },
            None => Err(Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, "BKTree not initialized")))
        }
    }

    pub fn populate_bloom_filter(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if Path::new("bloom_filter.bin").exists() {
            self.bloom_filter = match BloomFilter::from_file("bloom_filter.bin") {
                Ok(bloom_filter) => Some(bloom_filter),
                Err(_) => None,
            };
        }
        else {
            if self.dictionary.is_none() {
                self.get_dictionary();
            }
            self.bloom_filter = match &self.dictionary {
                Some(dictionary) => {
                    let words = dictionary.words.clone();
                    Some(BloomFilter::from(words))
                },
                None => None,
            }
        }

        match &self.bloom_filter {
            Some(bloom_filter) => {
                match bloom_filter.to_file("bloom_filter.bin") {
                    Ok(_) => Ok(()),
                    Err(_) => Err(Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, "Failed to write BloomFilter to file")))
                }
            },
            None => Err(Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, "BloomFilter not initialized"))),
        }
    }

    fn get_dictionary(&mut self) {
        let file: File = match File::open("dictionary.txt") {
            Ok(file) => file,
            Err(_) => panic!("Failed to open file")
        };

        self.dictionary = match Dictionary::from((file, 255)) {
            dictionary => Some(dictionary),
        };
    }

    pub fn run(&self) {
        loop {
            println!("Enter a paragraph to spell check: ");
            let mut paragraph: String = String::new();
            std::io::stdin().read_line(&mut paragraph).unwrap();

            let mut words: Vec<String> = processor::get_words(&paragraph);
            let split_words: Vec<(String, String, String)> = words.iter_mut().map(|word| processor::split_word(word)).collect();

            for (starting_punctuations, middle_word, ending_punctuations) in split_words {
                let bloom_filter: &BloomFilter = self.bloom_filter.as_ref().unwrap();
                let bk_tree: &BKTree = self.bk_tree.as_ref().unwrap();

                let lowercase_word = middle_word.to_lowercase();
                if bloom_filter.lookup(&lowercase_word) {
                    let valid_word = bk_tree.does_contain(lowercase_word.clone());
                    if !valid_word {
                        println!("{}{}{} is misspelled", &starting_punctuations, &middle_word, &ending_punctuations);
                        let suggestions = bk_tree.get_similar_words(lowercase_word.clone(), 1);
                        println!("Suggestions: {:?}", suggestions);
                    }
                }
                else {
                    println!("{}{}{} is misspelled", &starting_punctuations, &middle_word, &ending_punctuations);
                    let suggestions = bk_tree.get_similar_words(lowercase_word.clone(), 1);
                    println!("Suggestions: {:?}", suggestions);
                }
            }
        }
    }
}
