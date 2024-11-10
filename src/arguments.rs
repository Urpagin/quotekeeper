use crate::{prompt_yes_or_no, YesOrNo, PROGRAM_DATA_DIRECTORY, QUOTES_FILE_NAME};
use clap::Parser;
use std::{fs, process};
#[derive(Parser)]
#[command(name = "quotekeeper")]
#[command(
    about = "QuoteKeeper is a simple app that captures and organizes quotes in a JSON format",
    long_about = "Imagine you're in a history class with your eccentric teacher, and they say something funny that you'd like to save for later. Instead of wasting 45 seconds opening a new file and filling out the details, QuoteKeeper allows you to jot it down instantly and keep it neatly organized."
)]
struct Cli {
    /// Greet the user
    #[arg(short, long)]
    greet: bool,
    #[arg(short = 'r', long)]
    remove: bool,
}

pub fn init() {
    let args = Cli::parse();
    if args.greet {
        println!("Hello, welcome to QuoteKeeper!");
    } else {
        println!("Run with --greet to receive a greeting!");
    }

    if args.remove {
        remove_file();
    }
}
fn remove_file() -> () {
    let home_dir = dirs::home_dir().expect("Home directory not found.");
    let quote_path = home_dir.join(PROGRAM_DATA_DIRECTORY).join(QUOTES_FILE_NAME);
    let input = prompt_yes_or_no(
        "Are you sure you want to remove the quotes? [y/N]",
        YesOrNo::No,
    );
    match input {
        YesOrNo::Yes => {
            match fs::remove_file(&quote_path) {
                Ok(_) => println!("File removed successfully"),
                Err(e) => eprintln!("Error while deleting the file: {e}"),
            }

            println!("Exiting...");
            process::exit(1);
        }
        YesOrNo::No => {
            println!("Exiting...");
            process::exit(1);
        }
    };
}
