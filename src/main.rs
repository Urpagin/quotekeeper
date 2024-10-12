use core::panic;
use serde::{Deserialize, Serialize};
use std::{
    env::var,
    fs::File,
    io::{BufReader, BufWriter, Read, Write},
    process::{exit, Command},
};

use chrono::Local;
use tempfile::NamedTempFile;

const PROGRAM_DATA_DIRECTORY: &str = ".quotekeeper";
const QUOTES_FILE_NAME: &str = "quotes.json";

fn main() {
    let quote: String = get_quote_editor();
    let author: String = get_author();
    let date: String = get_date();

    if let Err(e) = update_quotes(&quote, &author, &date, QUOTES_FILE_NAME) {
        eprintln!("Failed to update quotes: {e}");
        exit(-1);
    }
}

/// Adds a new quote to the quotes file.
fn update_quotes(
    quote: &str,
    author: &str,
    date: &str,
    file_name: &str,
) -> Result<(), std::io::Error> {
    let home_dir = dirs::home_dir().expect("Home directory not found.");
    let config_dir = &home_dir.join(PROGRAM_DATA_DIRECTORY);

    if !config_dir.exists() {
        std::fs::create_dir(config_dir)?;
    }

    let quotes_file = config_dir.join(file_name);
    if !quotes_file.exists() {
        File::create(&quotes_file)?;
    }

    update_json(quote, author, date, &quotes_file.to_string_lossy())
}

#[derive(Serialize, Deserialize)]
struct Quote {
    quote: String,
    author: String,
    date: String,
}

impl std::fmt::Display for Quote {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "quote: {}\nauthor: {}\ndate: {}",
            self.quote, self.author, self.date
        )
    }
}

#[derive(Serialize, Deserialize)]
struct Quotes {
    quotes: Vec<Quote>,
}

impl Default for Quote {
    fn default() -> Self {
        Self {
            quote: String::default(),
            author: String::default(),
            date: String::default(),
        }
    }
}

impl Default for Quotes {
    fn default() -> Self {
        Self {
            quotes: std::vec![Quote::default()],
        }
    }
}

/// Adds a new quote to the json file.
fn update_json(quote: &str, author: &str, date: &str, filepath: &str) -> std::io::Result<()> {
    // Parse the file
    let file = File::open(filepath)?;
    let reader = BufReader::new(file);

    let new_quote: Quote = Quote {
        quote: quote.to_string(),
        author: author.to_string(),
        date: date.to_string(),
    };
    println!("\n{new_quote}");

    let mut quotes = Quotes::default();

    let parsed_data: Result<Quotes, _> = serde_json::from_reader(reader);
    match parsed_data {
        Ok(read_quotes) => {
            quotes = read_quotes;
            quotes.quotes.push(new_quote);
        }
        Err(_) => {
            // TODO: Backup the current quotes file. Because we're overwritting the old file here.
            //
            //backup_quotes(filepath)?;
            eprintln!("Failed to parse quotes, overwritting quotes file.");
            quotes.quotes = vec![new_quote];
        }
    }

    // Open the file again to overrite the file and add the new quote.
    let file = File::create(filepath)?;
    let writer = BufWriter::new(file);
    // Beautiful pretty JSON with indent!
    serde_json::to_writer_pretty(writer, &quotes)?;

    println!("\nSaved quote in '{filepath}'");

    Ok(())
}

/// Gets the quote author from the user.
///
/// # Returns
/// A `String` containing the quote author from the user.
fn get_author() -> String {
    // TODO: Make a "recent authors" selection system. (save all authors in a file)
    prompt_user("Quote author\n-> ")
}

/// Gets a quote from the user from stdin.
///
/// # Returns
/// A `String` containing a quote from the user
fn get_quote_stdin() -> String {
    prompt_user("Quote\n-> ")
}

/// Gets a quote from the user from his default editor.
///
/// # Returns
/// A `String` containing a quote from the user.
///
/// # Panics
/// Panics on error.
fn get_quote_editor() -> String {
    // Temp file that auto-deletes
    let mut file = NamedTempFile::new().expect("Failed to create tempfile.");

    let file_path = file.path();
    let editor = &var("EDITOR").expect("Failed to read 'EDITOR' env variable.");

    // This assumes the default editor works like: <editor> <file_path> to open a file
    let status = Command::new(editor)
        .arg(file_path)
        .status()
        .expect(&format!("Failed to open {editor}"));

    if !status.success() {
        panic!("{editor} exited with error.");
    }

    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .expect("Failed to read tempfile.");

    contents = contents.trim().to_string();

    if contents.is_empty() {
        std::io::stdout().flush().unwrap();
        let answer = prompt_yes_or_no(
            "The quote is blank, do you want to re-enter a quote (Y/n) ",
            YesOrNo::Yes,
        );

        match answer {
            YesOrNo::Yes => {
                // Use of recursion
                return get_quote_editor();
            }
            YesOrNo::No => {
                println!("Quote empty, program aborted.");
                exit(0);
            }
        }
    }
    contents
}

#[derive(Debug)]
enum YesOrNo {
    Yes,
    No,
}

/// Prompts the user and reads his input, returns a Yes or No.
///
/// # Returns
/// A `YesOrNo` enum read from stdin.
fn prompt_yes_or_no(prompt: &str, default: YesOrNo) -> YesOrNo {
    let user_input: &str = &prompt_user_allow_empty(prompt).to_lowercase();
    match user_input {
        "y" | "yes" => YesOrNo::Yes,
        "n" | "no" => YesOrNo::No,
        _ => default,
    }
}

/// Gets a date from the user
///
/// # Returns
/// A `String` of the current date by asking the user, so it could be anything.
fn get_date() -> String {
    loop {
        match prompt_yes_or_no("Do you want to set a custom date (N/y) ", YesOrNo::No) {
            YesOrNo::Yes => return prompt_user("Date\n-> "),
            YesOrNo::No => return get_machine_date(),
        }
    }
}

/// Returns the current date.
///
/// # Returns
/// A `String` of the current date in the format:
/// "day-month-year hour:month:second"
fn get_machine_date() -> String {
    let current_local = Local::now();
    current_local.format("%d-%m-%Y %H:%M:%S").to_string()
}

/// Returns a trimmed string of what the user inputted.
///
/// # Returns
/// A trimmed `String` read from stdin.
///
/// # Panics
/// Panics if stdin read or stdout flush fail.
fn prompt_user_allow_empty(prompt: &str) -> String {
    print!("{prompt}");
    std::io::stdout().flush().expect("Failed to flush stdout.");

    let mut buffer = String::new();
    std::io::stdin()
        .read_line(&mut buffer)
        .expect("Failed to read from stdin.");

    buffer.trim().to_string()
}

/// Returns a trimmed and non-empty string from the user.
///
/// # Returns
/// A `String`, trimmed, non-empty from stdin.
///
/// # Panics
/// Panics if stdin read or stdout flush fail.
fn prompt_user(prompt: &str) -> String {
    loop {
        let answer = prompt_user_allow_empty(prompt).trim().to_string();
        if !answer.is_empty() {
            return answer;
        }
    }
}

enum ConfigArgs {
    /// Whether the program reads quotes from stdin.
    UseStdin(bool),
    /// If not from stdin, using custom editor. By default, uses the EDITOR env var.
    CustomEditor(String),
}

#[derive(Serialize)]
struct ConfigTomlArgs {
    editor: String,
}

struct Config<'a> {
    file_path: &'a str,
}

impl<'a> Config<'a> {
    fn new(file_path: &'a str) -> Self {
        Self { file_path }
    }

    /// Creates the initial config file and populates it with defaults.
    fn init_config_file(&self) {

        let default = r#"[settings]
        # Set this to your favorite editor to write your quotes.
        # You can also set it to 'stdin' to read from the standard input,
        # or 'default' to use the default editor on your machine.
        editor = default"#;

        let mut file = File::create(&self.file_path).expect("Failed to create new config file.");
        file.write_all(default.as_bytes()).expect("Failed to populate newly created config file.");
    }

    fn load_config(&self) -> ConfigArgs {
        let mut file = File::open(&self.file_path).expect("Failed to open config file.")
    }
}
