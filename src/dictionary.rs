use std::{fs::File, io::{BufRead, BufReader}};

#[derive(Debug)]
#[readonly::make]
pub struct Dictionary {
    pub words: Vec<String>,
    pub max_word_length: usize,
    pub alphabet_length: usize,
}

impl Dictionary {
    pub fn new(max_word_length: usize, alphabet_length: usize) -> Self {
        Dictionary {
            words: Vec::new(),
            max_word_length,
            alphabet_length,
        }
    }

    pub fn from_file(file_path: &str, alphabet_length: usize) -> Option<Self> {
        let file: File = File::open(file_path).ok()?;
        let buf_reader: BufReader<File> = BufReader::new(file);

        let mut words: Vec<String> = Vec::new();
        let mut max_word_length: usize = 0;
        
        for line in buf_reader.lines() {
            let word: String = line.ok()?;
            max_word_length = std::cmp::max(word.len(), max_word_length);
            words.push(word.to_lowercase());
        }

        Some(Dictionary {
            words,
            max_word_length,
            alphabet_length,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::Dictionary;

    #[test]
    fn test_new() {
        let dictionary: Dictionary = Dictionary::new(0, 0);
        assert_eq!(dictionary.max_word_length, 0);
        assert_eq!(dictionary.alphabet_length, 0);
    }

    #[test]
    fn test_from_file() {
        let dictionary: Dictionary = Dictionary::from_file("dictionary.txt", 255).unwrap();
        assert_ne!(dictionary.words.len(), 0);
        assert_eq!(dictionary.alphabet_length, 255);

        let max_word_length: Option<usize> = dictionary.words.iter().map(|word| word.len()).max();
        max_word_length.map(|max| assert_eq!(max, dictionary.max_word_length));
    }
}
