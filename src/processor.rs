pub fn get_words(string: &String) -> Vec<String> {
    string.split_whitespace().map(|s| s.to_string()).collect()
}

pub fn split_word(word: &mut String) -> (String, String, String) {
    let mut starting_punctuations: String = String::new();
    let mut middle_word: String = String::new();
    let mut ending_punctuations: String = String::new();

    for i in 0..word.len() {
        if !word.chars().nth(i).unwrap().is_alphanumeric() {
            starting_punctuations.push(word.chars().nth(i).unwrap());
        } else {
            middle_word = word.chars().skip(i).collect();
            break;
        }
    }
    for i in (0..middle_word.len()).rev() {
        if !middle_word.chars().nth(i).unwrap().is_alphanumeric() {
            ending_punctuations.push(middle_word.chars().nth(i).unwrap());
        } else {
            middle_word = middle_word.chars().take(i + 1).collect();
            break;
        }
    }

    (starting_punctuations, middle_word, ending_punctuations)
}

#[cfg(test)]
mod tests {
    use super::{get_words, split_word};

    #[test]
    fn test_get_words() {
        let string: String = "Hello, world!".to_string();
        let words: Vec<String> = get_words(&string);
        assert_eq!(words, vec!["Hello,", "world!"]);
    }

    #[test]
    fn test_split_word() {
        let mut word: String = "!!!Hello,".to_string();
        let (starting_punctuations, middle_word, ending_punctuations): (String, String, String) = split_word(&mut word);
        assert_eq!(starting_punctuations, "!!!");
        assert_eq!(middle_word, "Hello");
        assert_eq!(ending_punctuations, ",");

        word = "world!!!".to_string();
        let (starting_punctuations, middle_word, ending_punctuations): (String, String, String) = split_word(&mut word);
        assert_eq!(starting_punctuations, "");
        assert_eq!(middle_word, "world");
        assert_eq!(ending_punctuations, "!!!");
    }
}
