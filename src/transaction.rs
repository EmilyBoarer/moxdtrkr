use serde::{Serialize, Deserialize};
use chrono::prelude::*;
use chrono::NaiveDate;
use std::io::Read;
use std::io::prelude::*;
use inquire::{DateSelect, validator::Validation, Select, CustomType, Confirm, Text, InquireError};
use colored::Colorize;
use crate::{Account, pence_to_pound};


// TRANSACTION -------------------------------------------------------------------------------------
#[derive(Clone, Serialize, Deserialize, PartialEq)]
pub struct Transaction {
    account_id_1: u32, // account
    account_id_2: u32, // other account if transfer, category account if not
    transfer: bool, // true when transfer between two account accounts, false then from one account to a category account
    amount: i64,    // will store any sensible amount of money,
    // i32 would probably work but not be future proof
    reference: String,
    #[serde(with="test_date_format")]
    date: NaiveDate,
    notes: String,
}
impl Transaction {
    pub fn new_transaction(
        account_id: u32,
        category_account_id: u32,
        amount: i64,
        reference: String,
        date: NaiveDate,
        notes: String
    ) -> Transaction {
        return Transaction{
            account_id_1: account_id,
            account_id_2: category_account_id,
            transfer: false,
            amount,
            reference,
            date,
            notes
        }
    }
    pub fn new_transfer(
        src_account_id: u32,
        dest_account_id: u32,
        amount: u64,
        reference: String,
        date: NaiveDate,
        notes: String
    ) -> Transaction {
        return Transaction{
            account_id_1: src_account_id,
            account_id_2: dest_account_id,
            transfer: true,
            amount: -(amount as i64),
            reference,
            date,
            notes
        }
    }
    pub fn account_id_1(&self) -> u32 { return self.account_id_1 }
    pub fn account_id_2(&self) -> u32 { return self.account_id_2 }
    pub fn account_1(&self, all_accounts: &Vec<Account>) -> Result<Account, String> {
        for a in all_accounts.to_vec().into_iter() {
            if a.id == self.account_id_1 { return Ok(a) }
        }
        return Err("Account not found".to_string())
    }
    pub fn account_2(&self, all_accounts: &Vec<Account>) -> Result<Account, String> {
        for a in all_accounts.to_vec().into_iter() {
            if a.id == self.account_id_2 { return Ok(a) }
        }
        return Err("Account not found".to_string())
    }
    pub fn is_transfer(&self) -> bool { return self.transfer }
    pub fn amount(&self) -> i64 { return self.amount }
    pub fn reference(&self) -> String { return self.reference.clone() }
    pub fn date(&self) -> NaiveDate { return self.date }
    pub fn notes(&self) -> String { return self.notes.clone() }
}


// TRANSACTION FILE I/O ----------------------------------------------------------------------------
macro_rules! otfe {
    ($e:ident) => {
        println!("{}: {}\nStarting with fresh Transaction Database.","Error opening transactions file".to_string().red().bold(), $e)
    }
}
pub fn get_transactions_from_file(filepath:&str) -> Vec<Transaction> {
    match std::fs::File::open(filepath){
        Ok(f) => {
            let mut file = f;
            let mut s = String::new();
            match file.read_to_string(&mut s) {
                Ok(_) => {
                    // Deserialize JSON
                    match serde_json::from_str(&mut s) {
                        Ok(v) => return v,
                        Err(e) => {otfe!(e); println!("HERE"); Vec::new()}
                    }
                },
                Err(e) => {otfe!(e); Vec::new()}
            }
        },
        Err(e) => {otfe!(e); Vec::new()}
    }
}

macro_rules! stfe {
    ($e:ident) => {
        println!("Error saving transactions to file: {}", $e)
    }
}
pub fn save_transactions_to_file(filepath:&str, transactions:Vec<Transaction>) {
    // Serialize to JSON
    match serde_json::to_string(&transactions) {
        Ok(s) => {
            // save to file
            match std::fs::File::create(filepath) {
                Ok(f) => {
                    let mut file = f;
                    match file.write_all(s.as_bytes()) {
                        Ok(_) => {}, // success & exit function without complaining
                        Err(e) => stfe!(e)
                    }
                },
                Err(e) => stfe!(e)
            }
        },
        Err(e) => stfe!(e)
    }
}


// DATE FORMAT -------------------------------------------------------------------------------------
mod test_date_format {
    use chrono::{NaiveDate};
    use serde::{Deserializer, Serializer, Deserialize};

    // https://serde.rs/custom-date-format.html
    const FORMAT: &'static str = "%F";

    pub fn serialize<S>(
        date: &NaiveDate,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
        where S: Serializer {
        let s = format!("{}", date.format(FORMAT));
        serializer.serialize_str(&s)
    }

    pub fn deserialize<'de, D>(
        deserializer: D,
    ) -> Result<NaiveDate, D::Error>
        where D: Deserializer<'de> {
        let s = String::deserialize(deserializer)?;
        NaiveDate::parse_from_str(&s, FORMAT).map_err(serde::de::Error::custom)
    }
}


// TRANSACTION FUNCTIONS ---------------------------------------------------------------------------

macro_rules! query_account {
    ($acc_id:ident, $prompt:expr, $accounts:ident) => {
        match Select::new($prompt, $accounts.to_vec()).prompt() {
            Ok(choice) => {
                $acc_id = choice.id;
            },
            Err(error) => return Err(error),
        }
    }
}
macro_rules! query_date {
    ($date:ident, $prompt:expr) => {
        match DateSelect::new($prompt)
        .with_week_start(Weekday::Mon)
        .with_validator(|d: NaiveDate| {
            let now = Utc::now().date_naive();
            if d.gt(&now) {
                Ok(Validation::Invalid("Date must be in the past or today".into()))
            } else {
                Ok(Validation::Valid)
            }
        })
        .prompt() {
            Ok(d) => {
                $date = d;
            },
            Err(error) => return Err(error),
        }
    }
}

macro_rules! query_category {
    ($cat:ident, $prompt:expr, $cats:ident) => {
        match Select::new($prompt, $cats).prompt() {
            Ok(choice) => {
                $cat = choice.id;
            },
            Err(error) => return Err(error),
        }
    }
}

macro_rules! query_amount {
    ($amount:ident, $prompt:expr) => {
        match CustomType::<f64>::new($prompt)
            .with_formatter(&|i| format!("£{:.2}", i))
            .with_error_message("Please type a valid number")
            .with_help_message("Type the amount in GBP using a decimal point as a separator. Positive values are money in, negative is money spent")
            .prompt() {
            Ok(am) => {
                $amount = (am*100.0).round() as i64; // convert to pence
            },
            Err(error) => return Err(error),
        }
    }
}

macro_rules! query_amount_positive {
    ($amount:ident, $prompt:expr) => {
        match CustomType::<f64>::new($prompt)
                .with_formatter(&|i| format!("£{:.2}", i))
                // TODO validate
                // .with_validator(|float: f64| {
                //     if float < 0.01 {
                //         Ok(Validation::Invalid("Transfer must be of strictly positive value".into()))
                //     } else {
                //         Ok(Validation::Valid)
                //     }
                // })
                .with_error_message("Please type a valid number")
                .with_help_message("Type the amount in GBP using a decimal point as a separator. Strictly positive values only")
                .prompt() {
            Ok(am) => {
                $amount = (am*100.0).round() as u64; // convert to pence
            },
                Err(error) => return Err(error),
            }
    }
}

macro_rules! query_reference {
    ($reference:ident, $prompt:expr) => {
        match Text::new($prompt).prompt() {
            Ok(r) => {
                $reference = r;
            },
            Err(error) => return Err(error),
        }
    }
}

macro_rules! query_notes {
    ($notes:ident, $prompt:expr) => {
        match Text::new($prompt).prompt() {
            Ok(r) => {
                $notes = r;
            },
            Err(error) => return Err(error),
        }
    }
}

macro_rules! should_modify {
    ($msg:expr, $modify_help_message:ident) => {
        match Confirm::new(
            $msg.as_str())
            .with_default(false)
            .with_help_message($modify_help_message)
            .prompt() {
            Ok(b) => b,
            Err(e) => return Err(e)
        }
    }
}

pub fn new_transaction_based_on(transaction: &Transaction)
                                -> Result<Transaction, InquireError> {
    return new_transaction_from_inputs_with_presets(
        transaction.account_id_1(),
        transaction.account_id_2(),
        transaction.reference(),
        transaction.is_transfer() // pass through if is a transfer or not - the logic is the same either wya
    )
}

fn new_transaction_from_inputs_with_presets(
    acc_id: u32,
    cat_id: u32,
    refr: String,
    transfer: bool,
) -> Result<Transaction, InquireError>
{
    let amnt;
    let dat;
    let notes;

    // Date
    query_date!(dat, "Transaction Date:");

    // Amount
    query_amount!(amnt, "Transaction Amount:");

    // Notes
    query_notes!(notes, "Notes:");

    return Ok(Transaction{
        account_id_1: acc_id,
        account_id_2: cat_id,
        amount: amnt,
        reference: refr,
        date: dat,
        notes,
        transfer
    });
}

fn get_acc_from_id(accounts: &Vec<Account>, id: u32) -> Result<Account, bool> {
    for a in accounts.to_vec().into_iter() {
        if a.id == id {
            return Ok(a);
        }
    }
    return Err(true);
}

fn get_cat_accounts(accounts: &Vec<Account>) -> Vec<Account> {
    let mut cat_accs = Vec::new();
    for acc in accounts.to_vec().into_iter() {
        if acc.category {
            cat_accs.push(acc)
        }
    }
    return cat_accs
}

pub fn get_modified_transaction(
    transaction: &Transaction,
    acc_accounts: &Vec<Account>,
    cat_accounts: &Vec<Account>)
    -> Result<Transaction, InquireError>
{
    if transaction.is_transfer() {
        return get_modified_transaction_transfer(transaction, acc_accounts);
    } else {
        return get_modified_transaction_transaction(transaction, acc_accounts, cat_accounts);
    }
}

fn get_modified_transaction_transaction(
    transaction: &Transaction,
    acc_accounts: &Vec<Account>,
    cat_accounts: &Vec<Account>)
    -> Result<Transaction, InquireError>
{
    let mut user_wants_to_modify;
    let modify_help_message = "Press Enter to skip, type \"y\" then press Enter to change";

    // validate enough accounts of each type
    if acc_accounts.len() < 1 { return Err(InquireError::InvalidConfiguration(
        "Not enough accounts to choose from".to_string()))}
    if cat_accounts.len() < 1 { return Err(InquireError::InvalidConfiguration(
        "Not enough categories to choose from".to_string()))}


    // Account
    let mut acc_id_1 = transaction.account_id_1(); // un-modified value
    let acc = match get_acc_from_id(acc_accounts, acc_id_1) {
        Ok(a) => a,
        Err(_) => return Err(InquireError::InvalidConfiguration(
            "Account under preset account id does not exist".to_string()))
    };
    user_wants_to_modify = should_modify!(
        format!("Modify Account (Currently \"{}\")", acc.name), modify_help_message);
    if user_wants_to_modify { query_account!(acc_id_1, "Pick Account:", acc_accounts); }

    // Category
    let mut acc_id_2 = transaction.account_id_2(); // un-modified value
    let acc2 = match get_acc_from_id(cat_accounts, acc_id_2) {
        Ok(a) => a,
        Err(_) => return Err(InquireError::InvalidConfiguration(
            "Account under preset account id does not exist".to_string()))
    };
    user_wants_to_modify = should_modify!(
        format!("Modify Category (Currently \"{}\")", acc2.name), modify_help_message);
    if user_wants_to_modify {
        let cats = cat_accounts.clone();
        query_category!(acc_id_2, "Transaction Category:", cats)
    }

    // Amount
    let mut amnt = transaction.amount();
    user_wants_to_modify = should_modify!(format!("Modify Amount (Currently \"{}\")",
    pence_to_pound(amnt, true, false)), modify_help_message);
    if user_wants_to_modify { query_amount!(amnt, "Transaction Amount:"); }

    // Date
    let mut dat = transaction.date();
    user_wants_to_modify = should_modify!(
    format!("Modify Date (Currently \"{}\")", dat), modify_help_message);
    if user_wants_to_modify { query_date!(dat, "Transaction Date:"); }

    // Reference
    let mut refr = transaction.reference();
    user_wants_to_modify = should_modify!(
    format!("Modify Reference (Currently \"{}\")", refr), modify_help_message);
    if user_wants_to_modify { query_reference!(refr, "Reference:"); }

    // Notes
    let mut notes = transaction.notes();
    user_wants_to_modify = should_modify!(
    format!("Modify Notes (Currently \"{}\")", notes), modify_help_message);
    if user_wants_to_modify { query_notes!(notes, "Notes:"); }

    // Return
    return Ok(Transaction::new_transaction(
        acc_id_1,
        acc_id_2,
        amnt,
        refr,
        dat,
        notes,
    ));
}

fn get_modified_transaction_transfer(
    transaction: &Transaction,
    acc_accounts: &Vec<Account>)
    -> Result<Transaction, InquireError>
{
    let mut user_wants_to_modify;
    let modify_help_message = "Press Enter to skip, type \"y\" then press Enter to change";

    // validate enough accounts of each type
    if acc_accounts.len() < 2 { return Err(InquireError::InvalidConfiguration(
        "Not enough accounts to choose from".to_string()))}

    // Source Account
    let mut acc_id_1 = transaction.account_id_1(); // un-modified value
    let acc = match get_acc_from_id(acc_accounts, acc_id_1) {
        Ok(a) => a,
        Err(_) => return Err(InquireError::InvalidConfiguration(
            "Account under preset account id does not exist".to_string()))
    };
    user_wants_to_modify = should_modify!(
        format!("Modify Source Account (Currently \"{}\")", acc.name), modify_help_message);
    if user_wants_to_modify { query_account!(acc_id_1, "Pick Source Account:", acc_accounts); }

    // Destination Account
    let mut acc_id_2 = transaction.account_id_2(); // un-modified value
    let acc2 = match get_acc_from_id(acc_accounts, acc_id_2) {
        Ok(a) => a,
        Err(_) => return Err(InquireError::InvalidConfiguration(
            "Account under preset account id does not exist".to_string()))
    };
    let mut dest_accounts = acc_accounts.clone();
    dest_accounts.retain(|a| (*a).id != acc_id_1); // can't be same as already selected accounts
    user_wants_to_modify =  if acc_id_1 == acc_id_2{
                                true // force to change to avoid conflicts
                            } else {
                                should_modify!(
                                format!("Modify Destination Account (Currently \"{}\")", acc2.name),
                                    modify_help_message)
                            };
    if user_wants_to_modify { query_account!(acc_id_2, "Pick Destination Account:", dest_accounts); }

    // Amount
    let mut amnt = (-transaction.amount()) as u64; // TODO what happens for all these "as othertype" when it can't go ?? presumable it panics, but this shouldn't be too much of an issue with the size of the numbers here
    user_wants_to_modify = match Confirm::new(
        format!("Modify Amount (Currently \"{}\")",
                pence_to_pound(amnt as i64, true, false)).as_str())
            .with_default(false)
            .with_help_message(modify_help_message)
            .prompt() {
        Ok(b) => b,
        Err(e) => return Err(e)
    };
    if user_wants_to_modify { query_amount_positive!(amnt, "Transfer Amount: "); }

    // Date
    let mut dat = transaction.date();
    user_wants_to_modify = should_modify!(
    format!("Modify Date (Currently \"{}\")", dat), modify_help_message);
    if user_wants_to_modify { query_date!(dat, "Transaction Date:"); }


    // Reference
    let mut refr = transaction.reference();
    user_wants_to_modify = should_modify!(
    format!("Modify Reference (Currently \"{}\")", refr), modify_help_message);
    if user_wants_to_modify { query_reference!(refr, "Reference:"); }

    // Notes
    let mut notes = transaction.notes();
    user_wants_to_modify = should_modify!(
    format!("Modify Notes (Currently \"{}\")", notes), modify_help_message);
    if user_wants_to_modify { query_notes!(notes, "Notes:"); }

    // Return
    return Ok(Transaction::new_transfer(
        acc_id_1,
        acc_id_2,
        amnt,
        refr,
        dat,
        notes,
    ));
}

pub fn new_transaction_from_inputs(acc_accounts: &Vec<Account>, cat_accounts: &Vec<Account>)
                                   -> Result<Transaction, InquireError>
{
    let acc_id;
    let amnt;
    let refr;
    let dat;
    let cat;
    let notes;

    // Check accounts & categories before continuing
    let accs = acc_accounts.clone(); // clone it so that it's not lost!
    if accs.len() < 1 { return Err(InquireError::InvalidConfiguration(
        "No accounts to choose from".to_string()))}

    let cats = cat_accounts.clone(); // clone it so that it's not lost!
    if cats.len() < 1 { return Err(InquireError::InvalidConfiguration(
        "No categories to choose from".to_string()))}

    // Account
    query_account!(acc_id, "Pick Account:", accs);

    // Category
    query_category!(cat, "Transaction Category:", cats);

    // Amount
    query_amount!(amnt, "Transaction Amount:");

    // Date
    query_date!(dat, "Transaction Date:");

    // Reference
    query_reference!(refr, "Reference:");

    // Notes
    query_notes!(notes, "Notes:");

    // Return
    return Ok(Transaction::new_transaction(
        acc_id,
        cat,
        amnt,
        refr,
        dat,
        notes,
    ));
}

pub fn new_transfer_from_inputs(acc_accounts: &Vec<Account>)
                                   -> Result<Transaction, InquireError>
{
    let src_acc_id;
    let dest_acc_id;
    let amnt;
    let refr;
    let dat;
    let notes;

    // Check accounts & categories before continuing
    let mut accs = acc_accounts.clone(); // clone it so that it's not lost!
    if accs.len() < 2 { return Err(InquireError::InvalidConfiguration(
        "Not enough accounts to choose from".to_string()))}

    // Source Account
    query_account!(src_acc_id, "Pick Source Account:", accs);

    // remove src account from selection list for dest accounts
    accs.retain(|a| (*a).id != src_acc_id);

    // Destination Account
    query_account!(dest_acc_id, "Pick Destination Account:", accs);

    // Amount // strictly positive value
    query_amount_positive!(amnt, "Transfer Amount: ");

    // Date
    query_date!(dat, "Transaction Date:");

    // Reference
    query_reference!(refr, "Reference:");

    // Notes
    query_notes!(notes, "Notes:");

    // Return
    return Ok(Transaction::new_transfer(
        src_acc_id,
        dest_acc_id,
        amnt,
        refr,
        dat,
        notes,
    ));
}
