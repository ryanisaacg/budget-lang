extern crate clap;
extern crate chrono;
extern crate regex;

mod account;
mod parser;
use crate::{
    account::{Account, Action::{self, *}},
    chrono::NaiveDate,
    clap::App,
    parser::{parse, parse_date},
};
use std::{
    fmt::Display,
    fs::File,
    io::Read,
    iter::once,
};

fn err_to_str<T>(result: Result<T, impl Display>) -> Result<T, String> {
    result.map_err(|e| format!("{}", e))
}

fn calculate(actions: &Vec<Action>, start: Option<NaiveDate>, end: Option<NaiveDate>) -> Result<Account, String> {
    let mut account = Account::new_root();
    for action in actions {
        match &action {
            &Deposit { date, .. } | &Withdraw { date, .. } => {
                match &start {
                    Some(start) if date < start => continue,
                    _ => ()
                }
                match &end {
                    Some(end) if date > end => continue,
                    _ => ()
                }
            }
            _ => ()
        }
        account.apply(action.clone())?;
    }

    Ok(account)
}

fn main() -> Result<(), String> {
    let matches = App::new("budget")
        .version("0.1.0")
        .author("Ryan Goldstein")
        .about("Manage your money through space and time")
        .args_from_usage("<FILE>    'The file to read the budget from'
                         -d,--diff=[DATE] 'The date to diff from'
                         -u,--until=[DATE] 'The date to go until'")
        .get_matches();

    let path = err_to_str(matches.value_of("FILE").ok_or("Missing required paramter file to read the budget from"))?;
    let until = matches.value_of("until")
        .map(|date| parse_date(0, &mut once(date)))
        .map_or(Ok(None), |date| date.map(Some))?;
    let diff = matches.value_of("diff")
        .map(|date| parse_date(0, &mut once(date)))
        .map_or(Ok(None), |date| date.map(Some))?;

    let mut file = err_to_str(File::open(path))?;
    let mut contents = String::new();
    err_to_str(file.read_to_string(&mut contents))?;

    let actions = parse(&contents).map_err(|e| e.join("\n"))?;

    if let Some(start) = diff {
        let before = calculate(&actions, None, Some(start))?;
        let after = calculate(&actions, None, until)?;
        println!("The difference is:\n{}", after.diff(&before)?);
    } else {
        let account = calculate(&actions, None, until)?;
        match until {
            Some(until) => println!("The budget as of {}:\n{}", until, account),
            None => println!("The current budget:\n{}", account)
        }
    }
    Ok(())
}

