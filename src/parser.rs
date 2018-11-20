use {
    account::{
        Action::{self, *}, AccountType::*, Inflow::{self, *}
    },
    chrono::naive::NaiveDate,
    regex::Regex
};

pub fn parse(data: &str) -> Vec<Result<Action, String>> {
    Regex::new("#.*\n").unwrap().replace_all(data, "\n")
        .split("\n")
        .enumerate()
        .map(|(index, line)| (index + 1, line))
        .filter(|(_, line)| line.len() > 0)
        .map(|(num, line)| {
            let tokens = &mut line.split_whitespace();
            match tokens.next() {
                Some("add") => parse_new(num, tokens),
                Some("-") => parse_withdraw(num, tokens),
                Some("+") => parse_deposit(num, tokens),
                _ => Err(format!("Failed to parse command at line {}", num))
            }
        })
        .collect()
}

fn parse_new<'a, 'b>(num: usize, line: &'a mut impl Iterator<Item = &'b str>) -> Result<Action, String> {
    let parent = next_token(num, line)?.to_owned();
    assert_token(">", num, line)?;
    let name = next_token(num, line)?.to_owned();
    let (inflow, max) = parse_inflow(num, line)?;
    let data = match line.next() {
        Some("with") => {
            let balance = parse_amount(num, line)?;
            Leaf { balance }
        }
        Some(other) => {
            Err(format!("Unexpected token {} at line {}", other, num))?
        }
        None => Branch { children: Vec::new() }
    };
    Ok(New { name, parent, inflow, max, data })
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

fn parse_inflow<'a, 'b>(num: usize, line: &'a mut impl Iterator<Item = &'b str>) -> Result<(Inflow, Option<f64>), String> {
    let amount = parse_amount(num, line)?;
    let inflow = match next_token(num, line)? {
        "flex" => Ok(Flex(amount)),
        "fixed" => Ok(Fixed(amount)),
        other => Err(format!("Expected either 'flex' or 'fixed', found {} at line {}", other, num))
    };
    let max = match next_token(num, line)? {
        "max" => Ok(Some(parse_amount(num, line)?)),
        "nomax" => Ok(None),
        other => Err(format!("Expected either 'max value' or 'nomax', found {} at line {}", other, num))
    };
    Ok((inflow?, max?))
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
