extern crate clap;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate num_rational;

mod account;
use {
    account::{
        Account,
        Inflow::{self, *},
    },
    clap::{App, SubCommand},
    num_rational::Rational,
    serde_json::from_reader,
    std::fs::File
};

fn money_from(string: &str) -> Rational {
    Rational::new((string.parse::<f32>().unwrap() * 100.0) as isize, 100)
}

fn main() {
    let mut account: Account = serde_json::from_reader(File::open("budget.json").unwrap()).expect("There doesn't appear to be a budget");
    let matches = App::new("budget")
        .version("0.1.0")
        .author("Ryan Goldstein")
        .about("Manage your money")
        .subcommand(SubCommand::with_name("deposit")
                    .about("Add money into the system")
                    .args_from_usage("<AMOUNT> 'The amount of money to put in'"))
        .subcommand(SubCommand::with_name("withdraw")
                    .about("Take money from the system")
                    .args_from_usage("<NAME> 'The name of the account'
                                      <AMOUNT> 'The amount of money to put in'"))
        .subcommand(SubCommand::with_name("show")
                    .about("Show the status of your accounts"))
        .subcommand(SubCommand::with_name("add-account")
                    .about("Add a node")
                    .args_from_usage("<NAME> 'The name of the account'
                                      <TYPE> 'Either fixed or flex'
                                      <SPLIT> 'The total fixed amount or the flex amount'
                                      --amount=[AMOUNT] 'The amount of money in the account'
                                      --max=[MAX] 'The maximum value of the account'
                                      --parent=[PARENT] 'The parent to use for the account'"))
        .get_matches();
    if let Some(_) = matches.subcommand_matches("show") {
        account.print();
    }
    if let Some(matches) = matches.subcommand_matches("withdraw") {
        let name = matches.value_of("NAME").unwrap();
        let amount = money_from(matches.value_of("AMOUNT").unwrap());
        account.find_child(name).unwrap().withdraw(amount);
    }
    if let Some(matches) = matches.subcommand_matches("add-account") {
        let name = matches.value_of("NAME").unwrap().to_owned();
        let inner = match matches.value_of("amount") {
            Some(amount) => Account::new_leaf(name, money_from(amount)),
            None => Account::new_branch(name)
        };
        let split = money_from(matches.value_of("SPLIT").unwrap());
        let inflow = match matches.value_of("TYPE").unwrap() {
            "fixed" => Fixed(split),
            "flex" => Flex(split),
            _ => panic!("Not a valid account type")
        };
        let max = match matches.value_of("max") {
            Some(max) => Some(money_from(max)),
            None => None
        };
        let account = match matches.value_of("parent") {
            Some(parent) => account.find_child(parent).unwrap(),
            None => &mut account,
        };
        account.add_child(inner, inflow, max);
    }
    if let Some(matches) = matches.subcommand_matches("deposit") {
        let amount = matches.value_of("AMOUNT").unwrap().parse::<f32>().unwrap();
        account.deposit(Rational::new((amount * 100.0) as isize, 100));
    }
    serde_json::to_writer(File::create("budget.json").unwrap(), &account).unwrap();
}
