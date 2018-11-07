use {
    chrono::naive::NaiveDate,
    num_rational::Rational,
    self::{AccountType::*, Action::*, Inflow::*},
    std::fmt,
};

#[derive(Debug)]
pub struct Account {
    name: String,
    data: AccountType
}

#[derive(Debug)]
pub struct BranchEntry {
    account: Account,
    inflow: Inflow,
    max: Option<Rational>
}

#[derive(Debug)]
pub enum AccountType {
    Leaf { balance: Rational },
    Branch { children: Vec<BranchEntry> }
}

#[derive(Debug)]
pub enum Inflow  {
    Fixed(Rational),
    Flex(Rational)
}

pub enum Action {
    New { name: String, inflow: Inflow, max: Option<Rational>, parent: String, data: AccountType },
    Withdraw { account: String, amount: Rational, date: NaiveDate },
    Deposit { account: Option<String>, amount: Rational, date: NaiveDate }
}

impl Account {
    pub fn new_root() -> Account {
        Account {
            name: "root".to_owned(),
            data: Branch { children: Vec::new() }
        }
    }

    pub fn apply(&mut self, action: Action) -> Result<(), String> {
        match action {
            New { name, inflow, max, parent, data } => {
                let parent = self.find_child(&parent)
                    .ok_or(format!("Could not find parent account {} to create account {}", parent, name))?;
                let account = Account { name, data };
                parent.add_child(account, inflow, max)
            }
            Withdraw { account, amount, .. } => {
                let parent = self.find_child(&account)
                    .ok_or(format!("Could not find parent account {} to withdraw from", account))?;
                parent.withdraw(amount);
                Ok(())
            }
            Deposit { account, amount, .. } => {
                let account = match account {
                    Some(parent) => self.find_child(&parent)
                        .ok_or(format!("Could not find parent account {} to deposit to", parent))?,
                    None => self
                };
                account.deposit(amount);
                Ok(())
            }
        }
    }

    pub fn balance(&self) -> Rational {
        match self.data {
            Leaf { balance } => balance,
            Branch { ref children } => children
                .iter()
                .map(|BranchEntry { account, .. }| account.balance())
                .sum()
        }
    }

    pub fn deposit(&mut self, amount: Rational) {
        match self.data {
            Leaf { ref mut balance } => *balance += amount,
            Branch { ref mut children } => {
                let mut amount = children.iter_mut()
                    .fold(amount, |amount, child| child.make_fixed_deposit(amount));
                let total_flex: Rational = children.iter().map(BranchEntry::get_flex).sum();
                if total_flex != Rational::from_integer(0) {
                    let per_flex = amount / total_flex;
                    amount = children.iter_mut()
                        .fold(amount, |amount, child| child.make_flex_deposit(amount, per_flex));
                }
                let remaining = amount / Rational::from_integer(children.len() as isize);
                children.iter_mut().for_each(|child| child.account.deposit(remaining));
            }
        }
    }

    pub fn withdraw(&mut self, amount: Rational) -> Result<(), String> {
        match self.data {
            Leaf { ref mut balance } => Ok(*balance -= amount),
            _ => Err("Cannot withdraw from a branch node".to_owned())
        }
    }

    pub fn find_child(&mut self, name: &str) -> Option<&mut Account> {
        if self.name == name {
            return Some(self);
        }
        match &mut self.data {
            Leaf { .. } => None,
            Branch { children } =>  {
                for child in children.iter_mut() {
                    if child.account.name == name {
                        return Some(&mut child.account)
                    } else {
                        if let Some(account) = child.account.find_child(name) {
                            return Some(account);
                        }
                    }
                }
                None
            }
        }
    }

    pub fn add_child(&mut self, account: Account, inflow: Inflow, max: Option<Rational>) -> Result<(), String> {
        match &mut self.data {
            Leaf { .. } => Err("Cannot add a child to a leaf account".to_owned()),
            Branch { children } => {
                children.push(BranchEntry { account, inflow, max });
                Ok(())
            }
        }
    }

    fn print_level(&self, f: &mut fmt::Formatter, level: u32) -> fmt::Result {
        for _ in 0..level {
            print!("  ");
        }
        let balance = (self.balance() * Rational::from_integer(100)).to_integer() as f32 / 100.0;
        println!("{}: {}", self.name, balance);
        match &self.data {
            Leaf {..}  => Ok(()),
            Branch { children } => {
                for child in children {
                    child.account.print_level(f, level + 1)?
                }
                Ok(())
            }
        }
    }
}

impl fmt::Display for Account {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.print_level(f, 0)
    }
}

impl BranchEntry {
    fn get_flex(&self) -> Rational {
        match self.inflow {
            Fixed(_) => Rational::from_integer(0),
            Flex(_) if self.at_max() => Rational::from_integer(0),
            Flex(x) => x
        }
    }

    fn at_max(&self) -> bool {
        match self.max {
            Some(max) => self.account.balance() >= max,
            None => {
                match &self.account.data {
                    Leaf { .. } => false,
                    Branch { children } => children.iter().all(BranchEntry::at_max)
                }
            }
        }
    }

    fn make_fixed_deposit(&mut self, available: Rational) -> Rational {
        match self.inflow {
            Fixed(take) => {
                if self.at_max() {
                    available
                } else {
                    let take = if take > available { available } else { take };
                    self.account.deposit(take);
                    available - take
                }
            }
            Flex(_) => available
        }
    }

    fn make_flex_deposit(&mut self, available: Rational, per_flex: Rational) -> Rational {
        match (&self.inflow, self.max) {
            (Flex(flex), _) => {
                if self.at_max() {
                    available
                } else {
                    let amount = per_flex * flex;
                    self.account.deposit(amount);
                    available - amount
                }
            },
            _ => available
        }
    }
}
