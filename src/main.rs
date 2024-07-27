mod bk_tree;
mod bloom_filter;
mod cmd;
mod dictionary;
mod processor;
mod spell_check;
mod utils;

use std::process;

use spell_check::SpellCheck;

fn main() {
    let cmd_data = cmd::parse_cmd_args().unwrap_or_else(|err| {
        eprintln!("Error: {}", err);
        process::exit(1);
    });

    let bk_tree_path: &str = "bk_tree.bin";
    let bloom_filter_path: &str = "bloom_filter.bin";
    let dictionary_path: &str = "dictionary.txt";
    let alphabet_length: u16 = 255;

    let spell_check: SpellCheck = SpellCheck::new(
        bk_tree_path,
        bloom_filter_path,
        dictionary_path,
        alphabet_length,
    );
    spell_check.run(cmd_data);
}
