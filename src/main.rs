use std::fmt::{Display, Formatter};
use chrono::Duration;
use chrono::prelude::*;
use inquire::{Select, Text, MultiSelect};
use clearscreen;

mod account;
use crate::account::*;

mod transaction;
use crate::transaction::*;

mod calendar;
use crate::calendar::*;

mod pence_to_pound_utils;
use crate::pence_to_pound_utils::*;

mod coloured_string;
use crate::coloured_string::*;

mod sidebyside;
mod graph;

mod settings;
use crate::settings::save_settings_to_file;

use crate::sidebyside::*;


#[derive(PartialEq)]
enum MainloopOption {
    Nothing,
    Clear,
    Exit,
    NewTransaction,
    NewTransfer,
    ListAccounts,
    ShowAccount,
    TextCalendar,
    AddAccount,
    AddCategory,
    EditAccountSelection,
    AccountCategoryGraphs,
    ListCategories,
    Settings,
    Save,
}

impl Display for MainloopOption {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        match *self {
            MainloopOption::Exit => write!(f, "Exit"),
            MainloopOption::Clear => write!(f, "Clear Screen"),
            MainloopOption::Nothing => write!(f, "Do Nothing"),
            MainloopOption::NewTransaction => write!(f, "New Transaction"),
            MainloopOption::NewTransfer => write!(f, "New Transfer"),
            MainloopOption::ListAccounts => write!(f, "List Accounts"),
            MainloopOption::ShowAccount => write!(f, "Show Account"),
            MainloopOption::TextCalendar => write!(f, "Text Calendar"),
            MainloopOption::AddAccount => write!(f, "Add Account"),
            MainloopOption::AddCategory => write!(f, "Add Category"),
            MainloopOption::EditAccountSelection => write!(f, "Choose which accounts to view on the calendar"),
            MainloopOption::AccountCategoryGraphs => write!(f, "Bar graph: expenditure per week, for a given account and category(s)"),
            MainloopOption::ListCategories => write!(f, "List Categories"),
            MainloopOption::Settings => write!(f, "Settings"),
            MainloopOption::Save => write!(f, "Save without exiting"),
        }
    }
}

enum CalInteractMethod {
    Calendar,
    Day,
    Transact
}

enum TransactionOptions {
    Nothing,
    Delete,
    Modify,
    CreateNewFromTemplate,
}
impl Display for TransactionOptions {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        match *self {
            TransactionOptions::Nothing => write!(f, "Nothing (deselect i)"),
            TransactionOptions::Delete => write!(f, "Delete it"),
            TransactionOptions::Modify => write!(f, "Modify it"),
            TransactionOptions::CreateNewFromTemplate => write!(f, "Create a New Transaction, using this as a template"),
        }
    }
}

enum Actions {
    Nothing,
    Dec,
    Inc,
}

fn main() {
    const ACCOUNTS_FILEPATH:&str = "data/accounts.json";
    const TRANSACTIONS_FILEPATH:&str = "data/transactions.json";
    const SETTINGS_FILEPATH:&str = "data/settings.csv";
    // load accounts and transactions from file - if unsuccessful then just create empty Vecs
    let mut accounts:Vec<Account> = get_accounts_from_file(ACCOUNTS_FILEPATH); // all accounts
    let mut acc_accounts = Account::acc_accounts(&accounts); // account accounts
    let mut cat_accounts = Account::cat_accounts(&accounts); // category accounts
    // visible accounts in calendar
    let mut visible_accounts = acc_accounts.clone();
    // all transactions
    let mut transactions:Vec<Transaction> = get_transactions_from_file(TRANSACTIONS_FILEPATH);

    // (default) SETTINGS parameters:
    // TODO determine from file and update with setttings page!
    let mut signed_with_colour = false;
    let mut brackets_when_neg = false;
    // text cal params
    let mut weeks = 4;
    let weeks_min = 1;
    let weeks_max = 6;
    let mut max_transactions_per_day = 6;
    let max_transactions_per_day_min = 1;
    let max_transactions_per_day_max = 16;
    let mut day_width = 16;
    let day_width_min = 10;
    let day_width_max = 30;
    let mut info_width = 28;
    let info_width_min = 10;
    let info_width_max = 80;
    // programattically determined
    let mut width = 1+(day_width+1)*7+1+info_width;
    let mut title = ColouredString::from_str("--< moxdtrkr v2.0 >");
    for _ in 19..width { title.push_str("-") }

    // MAINLOOP:
    let mut state = MainloopOption::Nothing;
    while state != MainloopOption::Exit {
        println!("{}", title.yellow());

        // get next state
        match Select::new("Action:", vec!(
            MainloopOption::Clear,
            MainloopOption::TextCalendar,
            MainloopOption::EditAccountSelection,
            MainloopOption::NewTransaction,
            MainloopOption::NewTransfer,
            MainloopOption::AccountCategoryGraphs,
            MainloopOption::ListAccounts,
            MainloopOption::ListCategories,
            MainloopOption::AddCategory,
            MainloopOption::AddAccount,
            MainloopOption::ShowAccount,
            MainloopOption::Save,
            MainloopOption::Settings,
            MainloopOption::Exit,
        )).prompt() {
            Ok(op) => {
                state = op;
            },
            Err(_) => {
                state = MainloopOption::Exit;
            },
        }
        match state{
            MainloopOption::Exit => {
                println!("Saving...");
                save_accounts_to_file(ACCOUNTS_FILEPATH, accounts.to_vec());
                save_transactions_to_file(TRANSACTIONS_FILEPATH, transactions.to_vec());
                println!("Exiting...")
            },
            MainloopOption::Save => {
                println!("Saving...");
                save_accounts_to_file(ACCOUNTS_FILEPATH, accounts.to_vec());
                save_transactions_to_file(TRANSACTIONS_FILEPATH, transactions.to_vec());
            },
            MainloopOption::Clear => clearscreen::clear().expect("failed to clear screen"),
            MainloopOption::NewTransaction => {
                match new_transaction_from_inputs(&acc_accounts, &cat_accounts) { // passing both to save re-calculation
                    Ok(t) => transactions.push(t),
                    Err(_) => {}
                }
            },
            MainloopOption::NewTransfer => {
                match new_transfer_from_inputs(&acc_accounts) {
                    Ok(t) => transactions.push(t),
                    Err(_) => {}
                }
            },
            MainloopOption::ShowAccount => {
                // show account and sample of all transactions under it
                match Select::new("Pick Account:", accounts.to_vec())
                    .prompt() {
                    Ok(account) => {
                        println!("Account Name: {}, id {}", account.name, account.id);
                        let mut total:i64 = 0;
                        let mut recent_transactions:Vec<Transaction> = Vec::new();
                        for t in transactions.to_vec().into_iter() {
                            if t.account_id_1() == account.id {
                                total += t.amount();
                                recent_transactions.push(t);
                            } else if t.account_id_2() == account.id {
                                total -= t.amount();
                                recent_transactions.push(t);
                            }
                        }
                        println!("Contains {}", pence_to_pound_colour_bg(total, true, false));
                        println!("Recent Transactions:");
                        recent_transactions.sort_by_key(|t| t.date());
                        for t in recent_transactions.iter() {
                            match t.account_2(&accounts){
                                Ok(acc) => {
                                    if acc.category {
                                        println!("{}\t{}\tcat: {}\tref: {}",
                                                 t.date(),
                                                 pence_to_pound_colour(t.amount(), true, false),
                                                 acc.name,
                                                 t.reference());

                                    } else {
                                        println!("{}\t{}\t{}\tref: {}",
                                                 t.date(),
                                                 pence_to_pound_transfer(-t.amount(), true, false),
                                                 {
                                                     if acc.id == account.id {
                                                         format!("from: {}",match t.account_1(&accounts){
                                                             Ok(acc1) => acc1.name,
                                                             Err(_) => "<Account name not found>".to_string()
                                                         })
                                                     } else {
                                                         format!("to: {}", acc.name)
                                                     }
                                                 },
                                                 t.reference());

                                    }
                                },
                                Err(_) => {
                                    println!("{}\t{}\tcat: <Category/account name not found>\tref: {}",
                                             t.date(),
                                             pence_to_pound_colour(t.amount(), true, false),
                                             t.reference());
                                }
                            }
                        }
                    },
                    Err(_) => {}
                }
            },
            MainloopOption::ListAccounts => {
                let mut max_len = 0; // find longest string
                for a in acc_accounts.to_vec().into_iter() {
                    let len = format!("{}", a).len();
                    if len > max_len {
                        max_len = len;
                    }
                }
                for a in acc_accounts.to_vec().into_iter() {
                    let mut s = format!("{}", a);
                    let mut total:i64 = 0;
                    for t in transactions.to_vec().into_iter() {
                        if t.account_id_1() == a.id {
                            total += t.amount();
                        }
                        if t.account_id_2() == a.id {
                            total -= t.amount();
                        }
                    }
                    while s.len() < max_len+2 {
                        s.push(' ');
                    }
                    // bodge here:
                    let mut padding = String::new();
                    let target_spacing = 12-pence_to_pound(total, true, false).len();
                    while padding.len() < target_spacing {
                        padding.push(' ');
                    }
                    println!("{s}Total Value: {}{}", padding, pence_to_pound_colour_bg(total, true, false));
                }
            },
            MainloopOption::ListCategories => {
                println!("Categories listed by creation order:");
                for a in cat_accounts.to_vec().into_iter() {
                    println!("- {}", a);
                }
            },
            MainloopOption::AddAccount => {
                match new_account_from_inputs(&accounts) {
                    Ok(a) => {
                        accounts.push(a.clone());
                        // update acc accounts
                        acc_accounts.push(a);
                    },
                    Err(_) => {}
                }
            },
            MainloopOption::AddCategory => {
                match new_category_from_inputs(&accounts) {
                    Ok(a) => {
                        accounts.push(a.clone());
                        // update cat accounts
                        cat_accounts.push(a);
                    },
                    Err(_) => {}
                }
            },
            MainloopOption::EditAccountSelection => {
                match MultiSelect::new("Select the accounts you want to view in the calendar", acc_accounts.to_vec())
                    .prompt() {
                    Ok(va) => visible_accounts = va,
                    Err(_) => {}
                }
            },
            MainloopOption::TextCalendar => {
                // print transaction calendar mini-mainloop

                let mut calendaring = true;
                let ti = 21+ (Utc::now().weekday().num_days_from_monday() as i64); // today index
                let mut hi = ti; // highlight index (position on calendar to highlight)
                let mut zi = 0; // zero index (position for the calendar to start in
                let mut method = CalInteractMethod::Calendar;
                let mut thi:isize = 0; // transaction highlight index
                // let mut highlighted_date_transactions = Vec::new();

                let mut this_transaction = None;

                while calendaring {
                    match method {
                        CalInteractMethod::Calendar => {
                            let cal_pane = render_calendar(
                                day_width,
                                weeks,
                                max_transactions_per_day,
                                ti,
                                signed_with_colour,
                                brackets_when_neg,
                                &transactions,
                                &visible_accounts,
                                hi, // hidden
                                zi
                            );

                            let day_submenu_pane;
                            match render_day_submenu(
                                -1,
                                hi,
                                ti,
                                signed_with_colour,
                                brackets_when_neg,
                                &transactions,
                                &visible_accounts,
                                &accounts,
                                info_width,
                            ) {
                                (sbs, _, _) => {
                                    day_submenu_pane = sbs;
                                }
                            }

                            clearscreen::clear().expect("failed to clear screen");
                            SideBySide::print2(&cal_pane,&day_submenu_pane, " ");

                            println!("{} [Ctrl-C from here will break things - don't do it!]",
                                ColouredString::from_str("[↑↓ → ← to navigate, enter to select, esc to return to main menu]").cyan()
                            );

                            let term = console::Term::stdout();

                            let mut capturing = true;
                            while capturing {
                                match term.read_key() {
                                    Ok(k) => match k {
                                        console::Key::ArrowLeft => {
                                            capturing = false;
                                            hi -= 1;
                                            thi = 0;
                                        },
                                        console::Key::ArrowRight => {
                                            capturing = false;
                                            hi += 1;
                                            thi = 0;
                                        },
                                        console::Key::ArrowUp => {
                                            capturing = false;
                                            hi -= 7;
                                            thi = 0;
                                        },
                                        console::Key::ArrowDown => {
                                            capturing = false;
                                            hi += 7;
                                            thi = 0;
                                        },
                                        console::Key::PageDown => {
                                            capturing = false;
                                            hi += 7*4;
                                            thi = 0;
                                        },
                                        console::Key::PageUp => {
                                            capturing = false;
                                            hi -= 7*4;
                                            thi = 0;
                                        },
                                        console::Key::Enter => {
                                            method = CalInteractMethod::Day;
                                            capturing = false;
                                            thi = 0;
                                        },
                                        console::Key::Escape => {
                                            capturing = false;
                                            calendaring = false;
                                        },
                                        _ => println!("unknown input!")
                                    },
                                    Err(e) => println!("error {}", e),
                                }
                            }
                            while hi < zi {zi -= 7}
                            while hi >= zi+4*7 {zi += 7}
                        },
                        CalInteractMethod::Day => {
                            let cal_pane = render_calendar(
                                day_width,
                                weeks,
                                max_transactions_per_day,
                                ti,
                                signed_with_colour,
                                brackets_when_neg,
                                &transactions,
                                &visible_accounts,
                                zi-1, // hidden
                                zi
                            );

                            let day_submenu_pane;
                            let upper_bound_select;
                            match render_day_submenu(
                                thi,
                                hi,
                                ti,
                                signed_with_colour,
                                brackets_when_neg,
                                &transactions,
                                &visible_accounts,
                                &accounts,
                                info_width,
                            ) {
                                (sbs, opt, ub) => {
                                    day_submenu_pane = sbs;
                                    this_transaction = opt;
                                    upper_bound_select = ub;
                                }
                            }

                            clearscreen::clear().expect("failed to clear screen");
                            SideBySide::print2(&cal_pane,&day_submenu_pane, " ");

                            println!("Use Arrow Keys to navigate, Enter to select, Esc to return to day select. [Ctrl-C from here will break things - don't do it!]");

                            let term = console::Term::stdout();

                            let mut capturing = true;
                            while capturing {
                                match term.read_key() {
                                    Ok(k) => match k {
                                        console::Key::ArrowUp => {
                                            capturing = false;
                                            thi -= 1;
                                            if thi < 0 { // wrap around
                                                thi = upper_bound_select - 1;
                                            }
                                        },
                                        console::Key::ArrowDown => {
                                            capturing = false;
                                            thi += 1;
                                            if thi >= upper_bound_select { // wrap around
                                                thi = 0;
                                            }
                                        },
                                        console::Key::Enter => {
                                            match this_transaction {
                                                Some(_) => {
                                                    capturing = false;
                                                    method = CalInteractMethod::Transact;
                                                },
                                                None => {println!("No transaction highlighted to be selected!")}
                                            }
                                        },
                                        console::Key::Escape => {
                                            capturing = false;
                                            method = CalInteractMethod::Calendar;
                                        },
                                        _ => println!("unknown input!")
                                    },
                                    Err(e) => println!("error {}", e),
                                }
                            }
                        },
                        CalInteractMethod::Transact => {
                            let cal_pane = render_calendar(
                                day_width,
                                weeks,
                                max_transactions_per_day,
                                ti,
                                signed_with_colour,
                                brackets_when_neg,
                                &transactions,
                                &visible_accounts,
                                zi-1, // hidden
                                zi
                            );

                            let day_submenu_pane;
                            match render_day_submenu(
                                thi,
                                hi,
                                ti,
                                signed_with_colour,
                                brackets_when_neg,
                                &transactions,
                                &visible_accounts,
                                &accounts,
                                info_width,
                            ) {
                                (sbs, _, _) => {
                                    day_submenu_pane = sbs
                                }
                            }

                            clearscreen::clear().expect("failed to clear screen");
                            SideBySide::print2(&cal_pane,&day_submenu_pane, " ");

                            match Select::new("What do you want to do with the selected transaction? ", vec!(
                                TransactionOptions::Nothing,
                                TransactionOptions::CreateNewFromTemplate,
                                TransactionOptions::Modify,
                                TransactionOptions::Delete,
                            )).prompt() {
                                Ok(action) => {
                                    match (action, &this_transaction) {
                                        (TransactionOptions::Nothing, Some(_)) => {
                                            method = CalInteractMethod::Day; // go back to day menu
                                        },
                                        (TransactionOptions::CreateNewFromTemplate, Some(t)) => {
                                            match new_transaction_based_on(t) {
                                                Ok(nt) => {
                                                    transactions.push(nt);
                                                    method = CalInteractMethod::Day; // go back to day menu
                                                },
                                                Err(_) => {
                                                    println!("ERROR => New Transaction Based On Current operation ABORTED");
                                                    method = CalInteractMethod::Transact; // return just to the transaction menu
                                                }
                                            }
                                        },
                                        (TransactionOptions::Modify, Some(t)) => {
                                            // Create a new transaction based on the existing one, remove the existing one and add the new one (TODO: atomically?)
                                            match get_modified_transaction(t, &acc_accounts, &cat_accounts) {
                                                Ok(new_transaction) => {
                                                    // remove old transaction & add new one
                                                    transactions.retain(|x| *x != *t);
                                                    transactions.push(new_transaction);
                                                    method = CalInteractMethod::Day; // go back to day menu
                                                    thi = 0; // reset selection when returning to Day
                                                }
                                                Err(_) => {
                                                    println!("ERROR => Modify Transaction operation ABORTED");
                                                    method = CalInteractMethod::Transact; // return just to the transaction menu
                                                }
                                            }
                                        },
                                        (TransactionOptions::Delete, Some(t)) => {
                                            transactions.retain(|x| *x != *t);
                                            method = CalInteractMethod::Day; // go back to day menu
                                            thi = 0; // reset selection when returning to Day
                                        },
                                        (_, None) => panic!("shouldn't be given a non-existent selected transaction")
                                    }
                                }
                                Err(_) => println!("ERROR => Transaction Selection operation ABORTED")
                            }
                        },
                    }
                }
            },
            MainloopOption::AccountCategoryGraphs => {
                // ask for accounts
                match MultiSelect::new("Select the Account(s) to plot for", acc_accounts.to_vec())
                    .prompt() {
                    Ok(aas) => {
                        // calc list of IDs from accounts
                        let mut selected_accounts = Vec::new();
                        for aa in aas.into_iter() {
                            selected_accounts.push(aa.id);
                        }

                        // ask for categories
                        match MultiSelect::new("Select the category(s) to plot for", cat_accounts.to_vec())
                            .prompt() {
                            Ok(cas) => {
                                // calc list of IDs from accounts
                                let mut selected_categories = Vec::new();
                                for ca in cas.into_iter() {
                                    selected_categories.push(ca.id);
                                }

                                // do graphing loop for selection and others etc..
                                // calculates values and populates graph automatically
                                graph::graph_acc_cats(selected_accounts, selected_categories, &transactions);
                            },
                            Err(_) => {} //("ERROR => Select accounts for graphing operation ABORTED")
                        }
                    },
                    Err(_) => {} // inquire produced it's own error messages in place, no need for them here
                }
                println!();
            },
            MainloopOption::Settings => {
                let mut settingsing = true;
                let mut hi = 0; // highlighted index;
                let num_hi = 6; // number of highlightable positions
                let mut temp;
                while settingsing {
                    // draw settings
                    {
                        clearscreen::clear().expect("failed to clear screen");
                        println!("Settings: [default values in square brackets]");
                        println!(" {}", ColouredString::from_str("General:").purple());

                        temp = ColouredString::from_str(match signed_with_colour { true => "yes", false => "no", });
                        println!("  Signed when coloured: {} [no]\n    {}",
                                 match hi {
                                     0 => {temp.black().on_cyan()},
                                     _ => {temp.cyan()},
                                 },
                                 ColouredString::from_str("show a minus sign for negative values when the value happens to be coloured to show sign").blue(),
                        );
                        temp = ColouredString::from_str(match brackets_when_neg { true => "yes", false => "no", });
                        println!("  Brackets: {} [no]\n    {}",
                                 match hi {
                                     1 => {temp.black().on_cyan()},
                                     _ => {temp.cyan()},
                                 },
                                 ColouredString::from_str("represent negative values with brackets").blue(),
                        );

                        println!("  e.g.\n   123p:\n    {}\n    {}\n  -123p:\n    {}\n    {}\n",
                                 pence_to_pound_colour(123, signed_with_colour, brackets_when_neg),
                                 pence_to_pound(123, true, brackets_when_neg),
                                 pence_to_pound_colour(-123, signed_with_colour, brackets_when_neg),
                                 pence_to_pound(-123, true, brackets_when_neg),
                        );

                        println!(" {}", ColouredString::from_str("Calendar:").purple());

                        temp = ColouredString::from_string(format!("{}", weeks));
                        println!("  Weeks visible: {} [4]\n    {}",
                                 match hi {
                                     2 => {temp.black().on_cyan()},
                                     _ => {temp.cyan()},
                                 },
                                 ColouredString::from_str("number of weeks visible at once in the calendar").blue(),
                        );
                        temp = ColouredString::from_string(format!("{}", max_transactions_per_day));
                        println!("  Transactions per day: {} [6]\n    {}",
                                 match hi {
                                     3 => {temp.black().on_cyan()},
                                     _ => {temp.cyan()},
                                 },
                                 ColouredString::from_str("max number of transactions visible in any given day before overflowing").blue(),
                        );

                        println!("  (not demonstrated for sake of space)\n");

                        temp = ColouredString::from_string(format!("{}", day_width));
                        println!("  Day width: {} [16]\n    {}",
                                 match hi {
                                     4 => {temp.black().on_cyan()},
                                     _ => {temp.cyan()},
                                 },
                                 ColouredString::from_str("width of each day in the calendar").blue(),
                        );
                        temp = ColouredString::from_string(format!("{}", info_width));
                        println!("  Info width: {} [28]\n    {}",
                                 match hi {
                                     5 => {temp.black().on_cyan()},
                                     _ => {temp.cyan()},
                                 },
                                 ColouredString::from_str("width of the region to the right of the calendar itself").blue(),
                        );

                        print!("  e.g.\n|");
                        for _ in 0..7 {
                            for _ in 0..day_width { print!("-"); }
                            print!("|")
                        }
                        for _ in 0..info_width { print!("-"); }
                        println!("|\n");

                        println!(" {}", ColouredString::from_str("Automatic: [these are calculated from combinations of other settings]").purple());

                        println!("  Width: {}\n    {}",
                                 width,
                                 ColouredString::from_str("maximum width the program will occupy in the terminal").blue(),
                        );

                        print!("  e.g.\n|");
                        for _ in 2..width { print!("-"); }
                        println!("|\n");

                        println!("\n{} {}",
                                 ColouredString::from_str("[↑↓ to move selection, → ← to change value, esc to save and exit to main menu]").cyan(),
                                 ColouredString::from_str("[Ctrl-C from here will break things - don't do it!]").red(),
                        );
                    }

                    // gather input
                    let term = console::Term::stdout();
                    let mut capturing:bool = true;
                    let mut action = Actions::Nothing;
                    while capturing {
                        match term.read_key() {
                            Ok(k) => match k {
                                console::Key::ArrowUp => {
                                    capturing = false;
                                    hi -= 1;
                                },
                                console::Key::ArrowDown => {
                                    capturing = false;
                                    hi += 1;
                                },
                                console::Key::ArrowLeft => {
                                    capturing = false;
                                    action = Actions::Dec;
                                },
                                console::Key::ArrowRight => {
                                    capturing = false;
                                    action = Actions::Inc;
                                },
                                console::Key::Escape => {
                                    capturing = false;
                                    settingsing = false;
                                },
                                _ => println!("unknown input!")
                            },
                            Err(e) => println!("error {}", e),
                        }
                        if hi >= num_hi { hi = num_hi - 1 }
                        if hi < 0 { hi = 0 }
                    }

                    // update settings:
                    match hi {
                        0 => {match action {
                            Actions::Dec => {signed_with_colour = false;},
                            Actions::Inc => {signed_with_colour = true;},
                            _ => {}
                        }},
                        1 => {match action {
                            Actions::Dec => {brackets_when_neg = false;},
                            Actions::Inc => {brackets_when_neg = true;},
                            _ => {}
                        }},
                        2 => {match action {
                            Actions::Dec => {
                                weeks -= 1;
                                if weeks < weeks_min { weeks = weeks_min; }
                            },
                            Actions::Inc => {
                                weeks += 1;
                                if weeks > weeks_max { weeks = weeks_max; }
                            },
                            _ => {}
                        }},
                        3 => {match action {
                            Actions::Dec => {
                                max_transactions_per_day -= 1;
                                if max_transactions_per_day < max_transactions_per_day_min {
                                    max_transactions_per_day = max_transactions_per_day_min; }
                            },
                            Actions::Inc => {
                                max_transactions_per_day += 1;
                                if max_transactions_per_day > max_transactions_per_day_max {
                                    max_transactions_per_day = max_transactions_per_day_max; }
                            },
                            _ => {}
                        }},
                        4 => {match action {
                            Actions::Dec => {
                                day_width -= 1;
                                if day_width < day_width_min { day_width = day_width_min; }
                            },
                            Actions::Inc => {
                                day_width += 1;
                                if day_width > day_width_max { day_width = day_width_max; }
                            },
                            _ => {}
                        }; width = 1+(day_width+1)*7+1+info_width},
                        5 => {match action {
                            Actions::Dec => {
                                info_width -= 1;
                                if info_width < info_width_min { info_width = info_width_min; }
                            },
                            Actions::Inc => {
                                info_width += 1;
                                if info_width > info_width_max { info_width = info_width_max; }
                            },
                            _ => {}
                        }; width = 1+(day_width+1)*7+1+info_width},
                        _ => {}
                    }

                    // save settings to file
                    save_settings_to_file(SETTINGS_FILEPATH,
                                          signed_with_colour,
                                          brackets_when_neg,
                                          weeks as u64,
                                          max_transactions_per_day as u64,
                                          day_width as u64,
                                          info_width as u64);

                }
            },
            // match anything else with non implemented message
            _ => println!("State {} is not implemented", state),
        }
    }
}