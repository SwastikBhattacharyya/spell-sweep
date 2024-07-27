use std::{
    error::Error,
    fs,
    io::{self, BufRead, BufReader, Read},
    path::PathBuf,
    process,
};

use clap::{command, value_parser, Arg};

type Result<T> = std::result::Result<T, Box<dyn Error>>;

enum Data {
    File(PathBuf),
    Pipe(Option<String>),
}

fn read_stdin() -> Result<Option<String>> {
    if !atty::is(atty::Stream::Stdin) {
        let stdin = io::stdin();
        let mut reader = BufReader::new(stdin);

        let mut buffer = String::new();
        reader
            .read_line(&mut buffer)
            .expect("whatever I don't care");

        return Ok(Some(buffer.trim().to_string()));
    }

    Ok(None)
}

fn handle_input_data(input: Data) -> Result<String> {
    match input {
        Data::File(file_path) => handle_file(file_path),
        Data::Pipe(piped_data) => Ok(handle_piped_data(piped_data)),
    }
}

fn handle_file(file_path: PathBuf) -> Result<String> {
    let mut file = fs::File::open(file_path)?;

    let mut buffer = Vec::<u8>::new();

    file.read_to_end(&mut buffer)?;

    Ok(String::from_utf8_lossy(buffer.as_slice())
        .trim()
        .to_string())
}

fn handle_piped_data(piped_data: Option<String>) -> String {
    match piped_data {
        None => {
            eprintln!("Error: Provide file path or pipe some data in.");
            process::exit(1);
        }
        Some(data) => data,
    }
}

pub fn parse_cmd_args() -> Result<String> {
    let matches = command!()
        .arg(
            Arg::new("filepath")
                .short('f')
                .long("file")
                .help("Path to the source file")
                .value_parser(value_parser!(PathBuf)),
        )
        .get_matches();

    let data = match matches.get_one::<PathBuf>("filepath") {
        Some(file_path) => handle_input_data(Data::File(file_path.to_path_buf()))?,
        None => handle_input_data(Data::Pipe(read_stdin()?))?,
    };

    Ok(data)
}

#[cfg(test)]
mod tests {
    use std::process::Command;

    // use super::*;
    #[test]
    fn file_not_found() {
        let output = Command::new("cargo")
            .arg("run")
            .arg("--")
            .arg("--file")
            .arg("/home/technoflask/.bashr")
            .output()
            .expect("Failed to execute command");

        assert_eq!(output.status.success(), false);
    }
}
