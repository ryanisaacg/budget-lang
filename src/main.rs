extern crate clap;
extern crate chrono;
extern crate regex;

mod account;
mod parser;
use {
    account::{Account, Action::*},
    clap::App,
    parser::{parse, parse_date},
    std::{
        fmt::Display,
        fs::File,
        io::Read,
        iter::once,
    },
};

fn err_to_str<T>(result: Result<T, impl Display>) -> Result<T, String> {
    result.map_err(|e| format!("{}", e))
}

fn main() -> Result<(), String> {
    let matches = App::new("budget")
        .version("0.1.0")
        .author("Ryan Goldstein")
        .about("Manage your money through space and time")
        .args_from_usage("<FILE>    'The file to read the budget from'
                         -u,--until=[DATE] 'The date to go until'")
        .get_matches();
    let path = err_to_str(matches.value_of("FILE").ok_or("Missing required paramter file to read the budget from"))?;
    let mut file = err_to_str(File::open(path))?;
    let mut contents = String::new();
    err_to_str(file.read_to_string(&mut contents))?;
    let mut account = Account::new_root();
    let date = match matches.value_of("until") {
        Some(date) => match parse_date(0, &mut once(date)) {
            Ok(date) => Some(date),
            Err(error) => return Err(format!("Error reading date parameter: {}", error))
        },
        None => None
    };
    for action in parse(&contents) {
        let action = action?;
        match (&action, date) {
            (&Deposit { date, .. }, Some(stop)) | (&Withdraw { date, .. }, Some(stop))
                if date > stop => break,
            _ => ()
        }
        account.apply(action)?;
    }
    match date {
        Some(date) => println!("The budget as of {}:", date),
        None => println!("The current budget:")
    }
    println!("{}", account);
    Ok(())
}

