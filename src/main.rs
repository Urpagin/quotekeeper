mod config;

use core::panic;
use serde::{Deserialize, Serialize};
use std::{
    env::var,
    fs::File,
    io::{BufReader, BufWriter, Read, Write},
    path::Path,
    process::{exit, Command},
};

use chrono::Local;
use config::get_config;
use tempfile::NamedTempFile;

const PROGRAM_DATA_DIRECTORY: &str = ".quotekeeper";
const QUOTES_FILE_NAME: &str = "quotes.json";
const CONFIG_FILE_NAME: &str = "config.conf";
const BACKUP_DIRECTORY: &str = "recovered_quotes";

// TODO: Make cli commands to change settings.

fn main() {
    init_app_fs();

    let quote: String = get_quote();
    let author: String = get_author();
    let date: String = get_date();

    if let Err(e) = update_quotes(&quote, &author, &date, QUOTES_FILE_NAME) {
        eprintln!("Failed to update quotes: {e}");
        exit(-1);
    }
}

/// Creates the directory of the program and files.
fn init_app_fs() {
    let home_dir = dirs::home_dir().expect("Home directory not found.");
    let program_root = Path::new(&home_dir).join(PROGRAM_DATA_DIRECTORY);

    if !program_root.exists() {
        std::fs::create_dir(&program_root).expect("Failed to create app directory.");
    }

    let quotes_path = program_root.join(QUOTES_FILE_NAME);

    init_file(&quotes_path.to_string_lossy(), r#"{"quotes": []}"#);
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

#[derive(Serialize, Deserialize, Default)]
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

impl Default for Quotes {
    fn default() -> Self {
        Self {
            quotes: std::vec![Quote::default()],
        }
    }
}

/// Creates a file and populates it with a default content only if it does not exist.
fn init_file(path: &str, content: &str) {
    let path = Path::new(path);

    if path.exists() {
        return;
    }

    let mut file =
        File::create(path).unwrap_or_else(|_| panic!("Failed to create file {:#?}", path));

    file.write_all(content.as_bytes())
        .unwrap_or_else(|_| panic!("Failed to populate file {:#?}", path));

    file.flush()
        .unwrap_or_else(|_| panic!("Failed to flush file {:#?}", path));

    file.sync_all()
        .unwrap_or_else(|_| panic!("Failed to sync file to fs {:#?}", path));
}
/// Adds a new quote to the json file.
fn update_json(quote: &str, author: &str, date: &str, filepath: &str) -> std::io::Result<()> {
    // Populate initial empty JSON if the file does not already exist

    // Parse the file
    let file = File::open(filepath)?;
    let reader = BufReader::new(file);

    let new_quote: Quote = Quote {
        quote: quote.to_string(),
        author: author.to_string(),
        date: date.to_string(),
    };

    let dashes = "--------------------";
    println!("{dashes}\n{new_quote}\n{dashes}");

    let mut quotes = Quotes::default();

    let parsed_data: Result<Quotes, _> = serde_json::from_reader(reader);
    match parsed_data {
        Ok(read_quotes) => {
            quotes = read_quotes;
            quotes.quotes.push(new_quote);
        }
        Err(_) => {
            eprintln!("Failed to parse quotes file '{filepath}', reinitializing file: corrupt JSON or quotes file not existing.");

            if let Err(e) = backup_quotes(filepath) {
                eprintln!("Failed to backup the quotes file: {e}")
            }

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

/// Makes a copy of the inputted file.
fn backup_quotes(filepath: &str) -> std::io::Result<()> {
    let path = Path::new(filepath);
    let backup_path = Path::new(filepath)
        .parent()
        .expect("Failed to get parent dir of bakcup quotes file.")
        .join(BACKUP_DIRECTORY);

    // Read the file's contents.
    let mut file = File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    // mkdir if it does not exist.
    if !backup_path.exists() {
        std::fs::create_dir(&backup_path)?;
    }

    // file_name is quotes.json
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("default_filename")
        .to_string();

    // Split "quotes.json" into "quotes" and "json" into `String`.
    let mut split_file_name = file_name.split(".");
    let name: String = split_file_name
        .next()
        .map(|s| s.to_string())
        .unwrap_or("ERR_quotes".to_string());
    let ext: String = split_file_name
        .next()
        .map(|s| s.to_string())
        .unwrap_or("ERR_.json".to_string());

    // Create the new filename "[date]_quotes.json"
    let now = Local::now();
    let formatted = now.format("%d-%m-%Y_%H:%M:%S").to_string();
    let new_file_name = format!("{name}_{formatted}.{ext}");

    // Make the full path, create the file and populate it.
    let bak_file_path = backup_path.join(new_file_name);
    let mut bak_file = File::create(&bak_file_path)?;
    bak_file.write_all(contents.as_bytes())?;

    println!(
        "Successfully backed up the quotes file due to malformed JSON. Backup at {:#?}",
        bak_file_path
    );

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

/// Decices how to get the quote from the user.
///
/// # Returns
/// A `String`, the quote from the user.
fn get_quote() -> String {
    let home_dir = dirs::home_dir().expect("Home directory not found.");

    let config_path = home_dir.join(PROGRAM_DATA_DIRECTORY).join(CONFIG_FILE_NAME);

    let config = get_config(&config_path.to_string_lossy());
    let editor: &str = &config.settings.editor;

    match editor {
        "stdin" => get_quote_stdin(),
        "default" => {
            let default_editor = var("EDITOR").expect("Failed to read 'EDITOR' env variable.");
            get_quote_editor(&default_editor)
        }
        _ => get_quote_editor(editor),
    }
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
fn get_quote_editor(editor: &str) -> String {
    // Temp file that auto-deletes
    let mut file = NamedTempFile::new().expect("Failed to create tempfile.");
    let file_path = file.path();

    // This assumes the default editor works like: <editor> <file_path> to open a file
    let status = Command::new(editor)
        .arg(file_path)
        .status()
        .unwrap_or_else(|_| panic!("Failed to open {editor}"));

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
                return get_quote_editor(editor);
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
    match prompt_yes_or_no("Do you want to set a custom date (N/y) ", YesOrNo::No) {
        YesOrNo::Yes => prompt_user("Date\n-> "),
        YesOrNo::No => get_machine_date(),
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
