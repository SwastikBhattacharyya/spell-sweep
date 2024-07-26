use std::{fs::File, io::{BufRead, BufReader}};

#[derive(Debug)]
#[readonly::make]
pub struct Dictionary {
    pub words: Vec<String>,
    pub max_word_length: u16,
    pub alphabet_length: u16,
}

impl From<(File, u16)> for Dictionary {
    fn from(value: (File, u16)) -> Self {
        let buf_reader: BufReader<File> = BufReader::new(value.0);

        let mut words: Vec<String> = Vec::new();
        let mut max_word_length: u16 = 0;

        for line in buf_reader.lines().filter_map(Result::ok) {
            let word: String = line;
            max_word_length = std::cmp::max(word.len() as u16, max_word_length);
            words.push(word.to_lowercase());
        }

        Self {
            words,
            max_word_length,
            alphabet_length: value.1
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs::File;
    use super::Dictionary;
    
    #[test]
    fn test_from_file() {
        let file: File = File::open("dictionary.txt").expect("File not found");
        let dictionary: Dictionary = Dictionary::from((file, 255));

        assert_ne!(dictionary.words.len(), 0);
        assert_eq!(dictionary.alphabet_length, 255);

        let max_word_length: u16 = dictionary.words.iter().map(|word| word.len()).max().expect("No max word length found") as u16;

        assert_eq!(dictionary.max_word_length, max_word_length);
    }
}
