extern crate clap;
extern crate chrono;
extern crate regex;

mod account;
mod parser;
use {
    account::Account,
    clap::App,
    parser::parse,
    std::{
        fmt::Display,
        fs::File,
        io::Read,
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
        .args_from_usage("<FILE>    'The file to read the budget from'")
        .get_matches();
    let path = err_to_str(matches.value_of("FILE").ok_or("Missing required paramter file to read the budget from"))?;
    let mut file = err_to_str(File::open(path))?;
    let mut contents = String::new();
    err_to_str(file.read_to_string(&mut contents))?;
    let mut account = Account::new_root();
    for action in parse(&contents) {
        account.apply(action?)?;
    }
    println!("{}", account);
    Ok(())
}

