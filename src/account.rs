use num_rational::Rational;

#[derive(Debug, Deserialize, Serialize)]
pub struct Account {
    name: String,
    data: AccountData
}

#[derive(Debug, Deserialize, Serialize)]
struct BranchEntry {
    account: Account,
    inflow: Inflow,
    max: Option<Rational>
}

#[derive(Debug, Deserialize, Serialize)]
enum AccountData {
    Leaf { balance: Rational },
    Branch { children: Vec<BranchEntry> }
}

#[derive(Debug, Deserialize, Serialize)]
pub enum Inflow  {
    Fixed(Rational),
    Flex(Rational)
}

use self::AccountData::*;
use self::Inflow::*;

impl Account {
    pub fn new_root() -> Account {
        Account::new_branch("root".to_owned())
    }

    pub fn new_branch(name: String) -> Account {
        Account {
            name,
            data: Branch { children: Vec::new() }
        }
    }

    pub fn new_leaf(name: String, balance: Rational) -> Account {
        Account {
            name,
            data: Leaf { balance }
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
                let amount = children.iter_mut()
                    .fold(amount, |amount, child| child.make_fixed_deposit(amount));
                let total_flex: Rational = children.iter().map(BranchEntry::get_flex).sum();
                let per_flex = amount / total_flex;
                let amount = children.iter_mut()
                    .fold(amount, |amount, child| child.make_flex_deposit(amount, per_flex));
                let remaining = amount / Rational::from_integer(children.len() as isize);
                children.iter_mut().for_each(|child| child.account.deposit(remaining));
            }
        }
    }

    pub fn withdraw(&mut self, amount: Rational) {
        match self.data {
            Leaf { ref mut balance } => *balance -= amount,
            _ => panic!("Can't withdraw from a non-Leaf account")
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

    pub fn add_child(&mut self, account: Account, inflow: Inflow, max: Option<Rational>) {
        match &mut self.data {
            Leaf { .. } => panic!("Leaf accounts can't have children"),
            Branch { children } => {
                children.push(BranchEntry { account, inflow, max });
            }
        }
    }

    fn print_level(&self, level: u32, inflow: &Inflow) {
        for _ in 0..level {
            print!("\t");
        }
        let (string, amount) = match inflow {
            Fixed(x) => ("Fixed", x),
            Flex(x) => ("Flex", x)
        };
        let balance = (self.balance() * Rational::from_integer(100)).to_integer() as f32 / 100.0;
        println!("{}:\t{}\t{}({})", self.name, balance, string, amount);
        match &self.data {
            Leaf {..}  => {}
            Branch { children } => {
                for child in children {
                    child.account.print_level(level + 1, &child.inflow)
                }
            }
        }
    }

    pub fn print(&self) {
        self.print_level(0, &Flex(Rational::from_integer(1)));
    }
}

impl BranchEntry {
    fn get_flex(&self) -> Rational {
        match self.inflow {
            Fixed(_) => Rational::from_integer(0),
            Flex(x) => x
        }
    }

    fn make_fixed_deposit(&mut self, available: Rational) -> Rational {
        match self.inflow {
            Fixed(take) => {
                match self.max {
                    Some(max) if self.account.balance() >= max => available,
                    _ => {
                        let take = if take > available { available } else { take };
                        self.account.deposit(take);
                        available - take
                    }
                }
            }
            Flex(_) => available
        }
    }

    fn make_flex_deposit(&mut self, available: Rational, per_flex: Rational) -> Rational {
        match (&self.inflow, self.max) {
            (_, Some(max)) if self.account.balance() >= max => available,
            (Flex(flex), _) => {
                let amount = per_flex * flex;
                self.account.deposit(amount);
                available - amount
            },
            _ => available
        }
    }
}
