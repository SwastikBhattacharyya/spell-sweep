pub fn split_input(string: &String) -> Vec<(String, String, String)> {
    let mut words: Vec<String> = get_words(string);
    let split_words: Vec<(String, String, String)> = words.iter_mut().map(|word| split_word(word)).collect();

    split_words
}

fn get_words(string: &String) -> Vec<String> {
    string.split_whitespace().map(|s| s.to_string()).collect()
}

fn split_word(word: &mut String) -> (String, String, String) {
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

pub fn join_input(split_words: Vec<(String, String, String)>) -> String {
    let words: Vec<String> = split_words.iter().map(|parts| join_word(parts.clone())).collect();
    words.join(" ")
}

fn join_word(parts: (String, String, String)) -> String {
    format!("{}{}{}", parts.0, parts.1, parts.2)
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

    #[test]
    fn test_split_input() {
        let word: String = "Hello, how are you, my name is John.".to_string();
        let split_words: Vec<(String, String, String)> = super::split_input(&word);

        assert_eq!(split_words[0], ("".to_string(), "Hello".to_string(), ",".to_string()));
        assert_eq!(split_words[1], ("".to_string(), "how".to_string(), "".to_string()));
        assert_eq!(split_words[2], ("".to_string(), "are".to_string(), "".to_string()));
        assert_eq!(split_words[3], ("".to_string(), "you".to_string(), ",".to_string()));
        assert_eq!(split_words[4], ("".to_string(), "my".to_string(), "".to_string()));
        assert_eq!(split_words[5], ("".to_string(), "name".to_string(), "".to_string()));
        assert_eq!(split_words[6], ("".to_string(), "is".to_string(), "".to_string()));
        assert_eq!(split_words[7], ("".to_string(), "John".to_string(), ".".to_string()));
    }

    #[test]
    fn test_join_word() {
        let parts: (String, String, String) = ("".to_string(), "Hello".to_string(), ",".to_string());
        let word: String = super::join_word(parts);
        assert_eq!(word, "Hello,");
    }

    #[test]
    fn test_join_input() {
        let split_words: Vec<(String, String, String)> = vec![
            ("".to_string(), "Hello".to_string(), ",".to_string()),
            ("".to_string(), "how".to_string(), "".to_string()),
            ("".to_string(), "are".to_string(), "".to_string()),
            ("".to_string(), "you".to_string(), ",".to_string()),
            ("".to_string(), "my".to_string(), "".to_string()),
            ("".to_string(), "name".to_string(), "".to_string()),
            ("".to_string(), "is".to_string(), "".to_string()),
            ("".to_string(), "John".to_string(), ".".to_string()),
        ];
        let input: String = super::join_input(split_words);
        assert_eq!(input, "Hello, how are you, my name is John.");
    }
}
