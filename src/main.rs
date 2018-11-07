extern crate clap;
extern crate chrono;
extern crate num_rational;
extern crate regex;

mod account;
mod parser;
use {
    account::Account,
    clap::App,
    num_rational::Rational,
    parser::parse,
    std::{
        fs::File,
        io::Read,
    },
};

fn money_from(string: &str) -> Rational {
    Rational::new((string.parse::<f32>().unwrap() * 100.0) as isize, 100)
}

fn main() {
    let matches = App::new("budget")
        .version("0.1.0")
        .author("Ryan Goldstein")
        .about("Manage your money through space and time")
        .args_from_usage("<FILE>    'The file to read the budget from'")
        .get_matches();
    let mut file = File::open(matches.value_of("FILE").unwrap()).unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents);
    let mut account = Account::new_root();
    for action in parse(&contents) {
        account.apply(action.unwrap());
    }
    println!("{}", account);
}

