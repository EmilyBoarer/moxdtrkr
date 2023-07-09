use std::fmt::{Display, Formatter};
use serde::{Serialize, Deserialize};
use std::io::Read;
use std::io::prelude::*;
use inquire::Text;
use colored::Colorize;

use crate::Transaction;

// ACCOUNT -----------------------------------------------------------------------------------------
#[derive(Clone, Serialize, Deserialize)]
pub struct Account {
    pub id: u32,
    pub name: String,
    pub category: bool // true if cat account, false if acc account
}

impl Display for Account {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}", self.name)
    }
}

impl Account {
    pub fn get_total_amount(all_transactions: &Vec<Transaction>, account: &Account) -> i64 {
        let mut total:i64 = 0;
        for transaction in all_transactions.into_iter() {
            if transaction.account_id_1() == account.id {
                total += transaction.amount();
            }
        }
        return total;
    }
}

// TRANSACTION FILE I/O ----------------------------------------------------------------------------

macro_rules! oafe {
    ($e:ident) => {
        println!("{}: {}\nStarting with fresh Account Database. Inconsistency likely if Transaction Database is not also fresh.","Error opening accounts file".to_string().red().bold(),$e)
    }
}

pub fn get_accounts_from_file(filepath:&str) -> Vec<Account> { // TODO return results rather than handling errors here??
    // open file
    match std::fs::File::open(filepath) {
        Ok(f) => {
            let mut file = f;
            let mut s = String::new();
            match file.read_to_string(&mut s) {
                Ok(_) => {
                    // Deserialize JSON
                    match serde_json::from_str(&mut s) {
                        Ok(v) => return v,
                        Err(e) => { oafe!(e); return Vec::new()}
                    }
                },
                Err(e) => {oafe!(e); return Vec::new()}
            }
        },
        Err(e) => {oafe!(e); return Vec::new()}
        // TODO should create an empty file to see that will be allowed to, to save running the program for ages before realising that you can't save anything??
    }
}

macro_rules! safe {
    ($e:ident) => {
        println!("Error saving accounts to file: {}", $e)
    }
}

pub fn save_accounts_to_file(filepath:&str, accounts:Vec<Account>) {
    // Serialize to JSON
    match serde_json::to_string(&accounts) {
        Ok(s) => {
            // save to file
            match std::fs::File::create(filepath){
                Ok(f) => {
                    let mut file = f;
                    match file.write_all(s.as_bytes()) {
                        Ok(_) => {}, // success & exit function without complaining
                        Err(e) => safe!(e)
                    }
                },
                Err(e) => safe!(e)
            }
        },
        Err(e) => safe!(e)
    }
}

// ACCOUNT FUNCTIONS -------------------------------------------------------------------------------
impl Account {
    pub fn cat_accounts(accounts: &Vec<Account>) -> Vec<Account> {
        let mut cat_accounts = Vec::new();
        for account in accounts.to_vec().into_iter() {
            if account.category { cat_accounts.push(account) }
        }
        return cat_accounts;
    }
    pub fn acc_accounts(accounts: &Vec<Account>) -> Vec<Account> {
        let mut acc_accounts = Vec::new();
        for account in accounts.to_vec().into_iter() {
            if !account.category { acc_accounts.push(account) }
        }
        return acc_accounts;
    }
}

pub fn new_account_from_inputs(existing_accounts: &Vec<Account>) -> Result<Account, inquire::InquireError> {
    // find new largest id
    let mut id = 0;
    for a in existing_accounts.to_vec().into_iter() {
        if a.id >= id {
            id = a.id + 1;
        }
    }

    // get reference note
    let n = Text::new("Account Name: ").prompt();
    let mut name = String::from(format!("UNNAMED ACCOUNT {}", id));
    match n {
        Ok(s) => {
            name = s;
        },
        Err(error) => return Err(error),
    }

    return Ok(Account{
        id,
        name,
        category: false,
    });
}

pub fn new_category_from_inputs(existing_accounts: &Vec<Account>) -> Result<Account, inquire::InquireError> {
    // find new largest id
    let mut id = 0;
    for a in existing_accounts.to_vec().into_iter() {
        if a.id >= id {
            id = a.id + 1;
        }
    }

    // get reference note
    let n = Text::new("Category Name: ").prompt();
    let mut name = String::from(format!("UNNAMED CATEGORY {}", id));
    match n {
        Ok(s) => {
            name = s;
        },
        Err(error) => return Err(error),
    }

    return Ok(Account{
        id,
        name,
        category: true,
    });
}