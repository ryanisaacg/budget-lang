use {
    chrono::naive::NaiveDate,
    self::{AccountType::*, Action::*, Inflow::*},
    std::fmt,
};

#[derive(Clone, Debug)]
pub struct Account {
    name: String,
    data: AccountType
}

#[derive(Clone, Debug)]
pub struct BranchEntry {
    account: Account,
    inflow: Inflow,
}

#[derive(Clone, Debug)]
pub enum AccountType {
    Leaf { balance: f64, max: f64 },
    Branch { children: Vec<BranchEntry> }
}

#[derive(Clone, Debug)]
pub enum Inflow  {
    Fixed(f64),
    Flex(f64)
}

#[derive(Clone, Debug)]
pub enum Action {
    New { name: String, inflow: Inflow, parent: String, data: AccountType },
    Remove { name: String },
    Edit { name: String, inflow: Inflow, max: f64 },
    Withdraw { account: String, amount: f64, date: NaiveDate },
    Deposit { account: Option<String>, amount: f64, date: NaiveDate },
    Transfer { from: String, to: Option<String>, amount: f64, date: NaiveDate }
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
            New { name, inflow, parent, data } => {
                let parent = self.find_child(&parent)
                    .ok_or(format!("Could not find parent account {} to create account {}", parent, name))?;
                let account = Account { name, data };
                parent.add_child(account, inflow)
            }
            Remove { name } => {
                self.remove(&name)?;
                Ok(())
            }
            Edit { name, inflow, max } => {
                let mut data = self.find_child(&name)
                    .ok_or(format!("Could not find account {} to remove", name))?
                    .data.clone();
                if let Leaf { max: node_max, .. } = &mut data {
                    *node_max = max;
                }
                let parent = self.remove(&name)?;
                self.apply(New { name, inflow, parent: parent.clone(), data })
            }
            Withdraw { account, amount, .. } => {
                let parent = self.find_child(&account)
                    .ok_or(format!("Could not find parent account {} to withdraw from", account))?;
                parent.withdraw(amount)?;
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
            Transfer { from, to, amount, date } => {
                self.apply(Action::Withdraw { account: from, amount, date })?;
                self.apply(Action::Deposit { account: to, amount, date })
            }
        }
    }

    pub fn balance(&self) -> f64 {
        match self.data {
            Leaf { balance, .. } => balance,
            Branch { ref children } => children
                .iter()
                .map(|BranchEntry { account, .. }| account.balance())
                .sum()
        }
    }

    fn remove(&mut self, name: &str) -> Result<String, String> {
        match &mut self.data {
            Leaf { .. } => Err(format!("{} not found to remove", name)),
            Branch { ref mut children } => {
                let len = children.len();
                children.retain(|child| child.account.name != name);
                if len != children.len() {
                    return Ok(self.name.clone());
                }
                for child in children.iter_mut() {
                    if let Ok(name) = child.account.remove(name.clone()) {
                        return Ok(name);
                    }
                }
                Err(format!("{} not found to remove", name))
            }
        }
    }

    pub fn deposit(&mut self, amount: f64) {
        match self.data {
            Leaf { ref mut balance, .. } => *balance += amount,
            Branch { ref mut children } => {
                // Make fixed deposits
                let mut amount = children.iter_mut()
                    .fold(amount, |amount, child| child.make_fixed_deposit(amount));
                // Make flex deposits
                let mut total_flex: f64 = children.iter().map(BranchEntry::get_flex).sum();
                while total_flex != 0.0 && amount > 0.01 {
                    let per_flex = amount / total_flex;
                    amount = children.iter_mut()
                        .fold(amount, |amount, child| child.make_flex_deposit(amount, per_flex));
                    total_flex = children.iter().map(BranchEntry::get_flex).sum();
                }
                // Give up and redistribute
                let remaining = amount / children.len() as f64;
                if remaining > 0.01 {
                    children.iter_mut().for_each(|child| child.account.deposit(remaining));
                }
            }
        }
    }

    pub fn withdraw(&mut self, amount: f64) -> Result<(), String> {
        match self.data {
            Leaf { ref mut balance, .. } => Ok(*balance -= amount),
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

    pub fn add_child(&mut self, account: Account, inflow: Inflow) -> Result<(), String> {
        match &mut self.data {
            Leaf { .. } => Err("Cannot add a child to a leaf account".to_owned()),
            Branch { children } => {
                children.push(BranchEntry { account, inflow });
                Ok(())
            }
        }
    }

    fn print_level(&self, f: &mut fmt::Formatter, level: u32, inflow: Inflow) -> fmt::Result {
        print!("{:?}:\t", inflow);
        for _ in 0..level {
            print!("  ");
        }
        println!("{}: {:.2}", self.name, self.balance());
        match &self.data {
            Leaf {..}  => Ok(()),
            Branch { children } => {
                for child in children {
                    child.account.print_level(f, level + 1, child.inflow.clone())?
                }
                Ok(())
            }
        }
    }

    pub fn diff(&self, other: &Account) -> Result<Account, String> {
        let data = match (&self.data, &other.data) {
            (Leaf { balance: end, max }, Leaf { balance: start, .. }) => {
                Leaf {
                    balance: end - start,
                    max: *max
                }
            }
            (Branch { children: start }, Branch { children: end }) => {
                let mut children = Vec::new();
                for start_child in start {
                    for end_child in end {
                        let end_child = &end_child.account;
                        let start_child = &start_child.account;
                        if end_child.name == start_child.name {
                            children.push(BranchEntry {
                                account: end_child.diff(start_child)?,
                                inflow: Inflow::Fixed(0.0)
                            });
                        }
                    }
                }
                for end_child in end {
                    let end_child = &end_child.account;
                    if children.iter().filter(|child| child.account.name == end_child.name).count() == 0 {
                            children.push(BranchEntry {
                                account: end_child.clone(),
                                inflow: Inflow::Fixed(0.0)
                            });
                    }
                }

                Branch { children }
            }
            (_, _) => return Err("Tried to diff accounts of different types".to_owned())
        };

        Ok(Account {
            name: self.name.clone(),
            data
        })
    }
}

impl fmt::Display for Account {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.print_level(f, 0, Inflow::Flex(1.0))
    }
}

impl BranchEntry {
    fn max(&self) -> f64 {
        match &self.account.data {
            Leaf { max, .. } => *max,
            Branch { children } => children.iter().map(BranchEntry::max).sum()
        }
    }

    fn until_max(&self) -> f64 {
        self.max() - self.account.balance()
    }

    fn at_max(&self) -> bool {
        self.until_max() <= 0.0
    }

    fn get_flex(&self) -> f64 {
        match self.inflow {
            Fixed(_) => 0.0,
            Flex(_) if self.at_max() => 0.0,
            Flex(x) => x
        }
    }

    fn make_fixed_deposit(&mut self, available: f64) -> f64 {
        match self.inflow {
            Fixed(take) => {
                let take = take.min(self.until_max()).min(available);
                self.account.deposit(take);
                available - take
            }
            Flex(_) => available
        }
    }

    fn make_flex_deposit(&mut self, available: f64, per_flex: f64) -> f64 {
        match self.inflow {
            Flex(flex) => {
                let take = (per_flex * flex).min(available).min(self.until_max());
                self.account.deposit(take);
                available - take
            },
            _ => available
        }
    }
}
