use crate::{
    account::{
        Action::{self, *}, AccountType::*, Inflow::{self, *}
    },
    chrono::naive::NaiveDate,
    regex::Regex
};

pub fn parse(data: &str) -> Result<Vec<Action>, Vec<String>> {
    let (actions, errors): (Vec<_>, Vec<_>) = Regex::new("#.*\n").unwrap().replace_all(data, "\n")
        .split("\n")
        .enumerate()
        .map(|(index, line)| (index + 1, line))
        .filter(|(_, line)| line.len() > 0)
        .map(|(num, line)| {
            let tokens = &mut line.split_whitespace();
            match tokens.next() {
                Some("add") => parse_new(num, tokens),
                Some("remove") => parse_remove(num, tokens),
                Some("-") => parse_withdraw(num, tokens),
                Some("+") => parse_deposit(num, tokens),
                Some("transfer") => parse_transfer(num, tokens),
                Some("edit") => parse_edit(num, tokens),
                Some(other) => Err(format!("Failed to parse command at line {}: unexpected command {}", num, other)),
                None => Err(format!("Unexpected EOF, likely an internal error"))
            }
        })
        .partition(Result::is_ok);
    if errors.is_empty() {
        Ok(actions.into_iter().map(Result::unwrap).collect())
    } else {
        Err(errors.into_iter().map(Result::unwrap_err).collect())
    }
}

fn parse_new<'a, 'b>(num: usize, line: &'a mut impl Iterator<Item = &'b str>) -> Result<Action, String> {
    let parent = next_token(num, line)?.to_owned();
    assert_token(">", num, line)?;
    let name = next_token(num, line)?.to_owned();
    let inflow = parse_inflow(num, line)?;
    let data = match line.next() {
        Some("with") => {
            let balance = parse_amount(num, line)?;
            let max = parse_max(num, line)?;
            Leaf { balance, max }
        }
        Some(other) => {
            Err(format!("Unexpected token {} at line {}", other, num))?
        }
        None => Branch { children: Vec::new() }
    };
    Ok(New { name, parent, inflow, data })
}

fn parse_remove<'a, 'b>(num: usize, line: &'a mut impl Iterator<Item = &'b str>) -> Result<Action, String> {
    let name = next_token(num, line)?.to_owned();
    Ok(Remove { name })
}

fn parse_edit<'a, 'b>(num: usize, line: &'a mut impl Iterator<Item = &'b str>) -> Result<Action, String> {
    let name = next_token(num, line)?.to_owned();
    let inflow = parse_inflow(num, line)?;
    let max = parse_max(num, line)?;
    Ok(Edit { name, inflow, max })
}

fn parse_max<'a, 'b>(num: usize, line: &'a mut impl Iterator<Item = &'b str>) -> Result<f64, String> {
    match line.next() {
        Some("max") => Ok(parse_amount(num, line)?),
        Some(other) => Err(format!("Expected either 'max value' or end-of-line, found {} at line {}", other, num)),
        None => Ok(std::f64::INFINITY),
    }
}

fn parse_withdraw<'a, 'b>(num: usize, line: &'a mut impl Iterator<Item = &'b str>) -> Result<Action, String> {
    let amount = parse_amount(num, line)?;
    assert_token("from", num, line)?;
    let account = next_token(num, line)?.to_owned();
    assert_token("on", num, line)?;
    let date = parse_date(num, line)?;
    Ok(Withdraw { account, amount, date })
}

fn parse_deposit<'a, 'b>(num: usize, line: &'a mut impl Iterator<Item = &'b str>) -> Result<Action, String> {
    let amount = parse_amount(num, line)?;
    let (account, date) = match next_token(num, line)? {
        "to" => {
            let account = next_token(num, line)?.to_owned();
            assert_token("on", num, line)?;
            (Some(account), parse_date(num, line)?)
        }
        "on" => {
            (None, parse_date(num, line)?)
        }
        other => return Err(format!("Expected either 'to' or 'on', found {} at line {}", other, num)),
    };
    Ok(Deposit { account, amount, date })
}

fn parse_transfer<'a, 'b>(num: usize, line: &'a mut impl Iterator<Item = &'b str>) -> Result<Action, String> {
    let amount = parse_amount(num, line)?;
    assert_token("from", num, line)?;
    let from = next_token(num, line)?.to_owned();
    let (to, date) = match next_token(num, line)? {
        "to" => {
            let account = next_token(num, line)?.to_owned();
            assert_token("on", num, line)?;
            (Some(account), parse_date(num, line)?)
        }
        "on" => {
            (None, parse_date(num, line)?)
        }
        other => return Err(format!("Expected either 'to' or 'on', found {} at line {}", other, num)),
    };
    Ok(Transfer { from, to, amount, date })
}

fn parse_inflow<'a, 'b>(num: usize, line: &'a mut impl Iterator<Item = &'b str>) -> Result<Inflow, String> {
    let amount = parse_amount(num, line)?;
    let inflow = match next_token(num, line)? {
        "flex" => Ok(Flex(amount)),
        "fixed" => Ok(Fixed(amount)),
        other => Err(format!("Expected either 'flex' or 'fixed', found {} at line {}", other, num))
    };
    Ok(inflow?)
}

fn parse_amount<'a, 'b>(num: usize, line: &'a mut impl Iterator<Item = &'b str>) -> Result<f64, String> {
    let token = next_token(num, line)?;
    token.parse::<f64>().map_err(|_| format!("Expected floating point literal at line {}, found {}", num, token))
}

pub fn parse_date<'a, 'b>(num: usize, line: &'a mut impl Iterator<Item = &'b str>) -> Result<NaiveDate, String> {
    let date = &mut next_token(num, line)?.split("/");
    let month = parse_int(num, date)?;
    let day = parse_int(num, date)?;
    let year = parse_int(num, date)?;
    Ok(NaiveDate::from_ymd(year as i32, month, day))
}

fn parse_int<'a, 'b>(num: usize, line: &'a mut impl Iterator<Item = &'b str>) -> Result<u32, String> {
    next_token(num, line)?
        .parse::<u32>()
        .map_err(|_| format!("Failed to read an integer at line {}", num))
}

fn next_token<'a, 'b>(num: usize, line: &'a mut impl Iterator<Item = &'b str>) -> Result<&'b str, String> {
    line.next().ok_or_else(|| format!("Unexpected end of command at line {}", num))
}

fn assert_token<'a, 'b>(expected: &str, num: usize, line: &'a mut impl Iterator<Item = &'b str>) -> Result<(), String> {
    match next_token(num, line)? {
        x if x == expected => Ok(()),
        other => Err(format!("Expected token {} at line {}, found {}", expected, num, other))
    }
}
