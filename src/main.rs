extern crate clap;
extern crate chrono;
extern crate num_rational;
extern crate regex;

mod account;
mod parser;
use {
    account::Account,
    clap::App,
    parser::parse,
    std::{
        fs::File,
        io::Read,
    },
};

fn main() {
    let matches = App::new("budget")
        .version("0.1.0")
        .author("Ryan Goldstein")
        .about("Manage your money through space and time")
        .args_from_usage("<FILE>    'The file to read the budget from'")
        .get_matches();
    let mut file = File::open(matches.value_of("FILE").unwrap()).unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
    let mut account = Account::new_root();
    for action in parse(&contents) {
        account.apply(action.unwrap()).unwrap();
    }
    println!("{}", account);
}

