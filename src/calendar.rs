use chrono::prelude::*;
use chrono::NaiveDate;

use crate::{Account, ColouredString, pence_to_pound_colour, pence_to_pound_colour_bg, pence_to_pound_transfer, SideBySide, Transaction};

pub const MONTHS: [&str; 12] = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];
pub const DAYS: [&str; 7] = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];

fn truncate(string: String, l:usize) -> String{
    return string.chars().take(l).collect()
}
fn truncate_ellipsis(string: String, l:usize) -> String{
    let ellipsis: String = "..".to_string();
    if string.len() > l {
        let mut s = truncate(string, l-ellipsis.len());
        s.push_str(ellipsis.as_str());
        return s;
    } else {
        return string;
    }
}
fn wrap(string: String, l:usize) -> Vec<String> { // TODO convert other functions to use options rather than results
    let mut strings:Vec<String> = Vec::new();
    let mut curr = String::new();
    for c in string.chars().into_iter() {
        if curr.len() < l {
            curr.push_str(c.to_string().as_str())
        } else {
            strings.push(curr.clone());
            curr = String::from(c);
        }
    }
    strings.push(curr);
    return strings;
}
macro_rules! wrap_string {
    ($sbs:ident, $string:expr, $width:expr) => {
        for s in wrap($string.to_string(), $width){
            $sbs.add_line(ColouredString::from_string(s));
        }
    }
}

pub fn render_calendar(
    day_width:usize,
    weeks:i64,
    max_transactions_per_day:usize,
    today:i64, // index of today in the calendar grid
    signed_with_colour:bool,
    brackets_when_neg:bool,
    transactions:&Vec<Transaction>,
    visible_accounts:&Vec<Account>,
    highlight_index:i64,
    zero_index:i64,
) -> SideBySide
{
    let mut sbs = SideBySide::new(7*(day_width+1)+1);
    let h_sep = '-';
    let corner_sep = '+';
    let v_sep = '|';


    // generate re-usable strings -- could convert to macros for speedup ??
    let mut hdiv = String::from("");
    for _ in 0..day_width { hdiv.push(h_sep); }

    let mut hdiv_row = String::from(corner_sep);
    for _day_index in 0..7 {
        hdiv_row.push_str(hdiv.as_str());
        hdiv_row.push(corner_sep)
    }

    // generate Year and Month display
    {
        let first_date = Utc::now().date().checked_add_signed(
            chrono::Duration::days(-today+zero_index)).unwrap();
        let last_date = Utc::now().date().checked_add_signed(
            chrono::Duration::days(7*weeks-today+zero_index)).unwrap();
        let yhtxt = format!("{} {} - {} {}",
                            MONTHS[first_date.month0() as usize], first_date.year(),
                            MONTHS[last_date.month0() as usize], last_date.year(),
        );
        let mut year_header = String::from("");
        while year_header.len() < ((7*(day_width+1)+1) - yhtxt.len())/2 {
            year_header.push(' ');
        }
        sbs.add_line(ColouredString::from_string(format!("{}{}", year_header, yhtxt)));
    }

    // do the day of the week titles
    {
        let mut hdiv_row_titles = String::from(corner_sep);
        for day_index in 0..7 {
            let before = (day_width - 3) / 2;
            let after = day_width - 3 - before;
            for _ in 0..before { hdiv_row_titles.push(h_sep); }
            hdiv_row_titles.push_str(DAYS[day_index]);
            for _ in 0..after { hdiv_row_titles.push(h_sep); }
            hdiv_row_titles.push(corner_sep)
        }
        sbs.add_line(ColouredString::from_string(hdiv_row_titles));
    }

    // construct the transaction grid
    let mut ts:Vec<Vec<Vec<Transaction>>> = vec!(
        vec!(vec!(),vec!(),vec!(),vec!(),vec!(),vec!(),vec!(),),
        vec!(vec!(),vec!(),vec!(),vec!(),vec!(),vec!(),vec!(),),
        vec!(vec!(),vec!(),vec!(),vec!(),vec!(),vec!(),vec!(),),
        vec!(vec!(),vec!(),vec!(),vec!(),vec!(),vec!(),vec!(),),
    );
    // populate grid with transactions
    let accs = acc_ids_from(&visible_accounts);
    for t in transactions.to_vec().into_iter() {
        let days_diff = (Utc.from_utc_date(&t.date()) - Utc::now().date()).num_days();
        let i = today + days_diff - zero_index;
        if (i >= 0) && (i < 7*weeks as i64) && (accs.contains(&t.account_id_1()) || accs.contains(&t.account_id_2())) {
            // on display range and in the accounts list
            ts[(i/7) as usize][(i%7) as usize].push(t.clone());
        }
    }
    // construct the transaction grid counters
    let mut ts_counter:Vec<Vec<usize>> = vec!(
        vec!(0,0,0,0,0,0,0),
        vec!(0,0,0,0,0,0,0),
        vec!(0,0,0,0,0,0,0),
        vec!(0,0,0,0,0,0,0),
    );
    // construct the transaction grid overflow summations
    let mut ts_overflow:Vec<Vec<i64>> = vec!(
        vec!(0,0,0,0,0,0,0),
        vec!(0,0,0,0,0,0,0),
        vec!(0,0,0,0,0,0,0),
        vec!(0,0,0,0,0,0,0),
    );


    // display calendar
    for week_index in 0..weeks {
        let mut header = ColouredString::from_string(v_sep.to_string());
        let mut t_rows = Vec::new();
        for _ in 0..max_transactions_per_day {
            t_rows.push(ColouredString::from_string(v_sep.to_string()));
        }
        for day_index in 0..7 {
            let today_index = week_index*7 + day_index;
            // assemble date header
            let today_date = Utc::now().date();
            let diff:i64 = today_index - today + zero_index;
            let date = today_date.checked_add_signed(
                chrono::Duration::days(diff)).unwrap();
            // create the header for the day
            let date_string = ColouredString::from_string(format!("{} {}",
                                                                  date.day(),
                                                                  MONTHS[date.month0() as usize]
            ));
            let closing_amt = pence_to_pound_colour_bg(
                get_closing_balance(
                    today_date.checked_add_signed(chrono::Duration::days(diff)).unwrap().naive_utc(),
                    transactions,
                    visible_accounts
                ),
                signed_with_colour,
                brackets_when_neg);
            // add date_string
            if today_index - highlight_index + zero_index == 0{ // special formatting for highlighted day
                if diff == 0 { // special formatting for today
                    header.push_coloured_string(date_string.bold().italic().black().on_cyan());
                } else {
                    header.push_coloured_string(date_string.black().on_cyan());
                }
            } else {
                if diff == 0 { // special formatting for today
                    header.push_coloured_string(date_string.bold().italic());
                } else {
                    header.push_coloured_string(date_string);
                }
            }
            // fill in the gap (leaving space for the closing amount
            while header.len()+closing_amt.len() < ((day_index as usize)+1)*(day_width+1) {
                header.push_str(" ");
            }
            // add closing amount to header
            header.push_coloured_string(closing_amt);
            header.push_string(v_sep.to_string());
            // add transactions
            for t in ts[week_index as usize][day_index as usize].to_vec().into_iter() {
                let x = ts_counter[week_index as usize][day_index as usize];
                if (x >= max_transactions_per_day-1) && (ts[week_index as usize][day_index as usize].len() > max_transactions_per_day) { // add to extra total
                    if accs.contains(&t.account_id_1()) {
                        ts_overflow[week_index as usize][day_index as usize] += t.amount();
                    }
                    if accs.contains(&t.account_id_2()) {
                        ts_overflow[week_index as usize][day_index as usize] -= t.amount();
                    }
                } else { // show transaction as a whole
                    let mut s;
                    let p = match t.is_transfer() {
                        true => {
                            pence_to_pound_transfer(
                                t.amount(),
                                signed_with_colour,
                                brackets_when_neg)
                        },
                        false => {
                            pence_to_pound_colour(
                                t.amount(),
                                signed_with_colour,
                                brackets_when_neg)
                        },
                    };
                    s = ColouredString::from_string(truncate_ellipsis(
                        if t.reference().len() > 0 { t.reference() } else { "(unnamed)".to_string() },
                        day_width-p.len()-1));
                    s.push_str(" ");
                    s.push_coloured_string(p);
                    t_rows[x].push_coloured_string(s);
                    ts_counter[week_index as usize][day_index as usize] += 1;
                }
            }
            // show overflow sums
            if ts[week_index as usize][day_index as usize].len()
                > max_transactions_per_day {
                let mut s;
                s = ColouredString::from_str("Others ");
                s.push_coloured_string(pence_to_pound_colour(
                    ts_overflow[week_index as usize][day_index as usize],
                    signed_with_colour,
                    brackets_when_neg));
                s = s.italic();
                t_rows[max_transactions_per_day-1].push_coloured_string(s);
            }
            // fill in rest of day for each of the transaction rows
            for x in 0..max_transactions_per_day {
                while t_rows[x].len() < ((day_index as usize)+1)*(day_width+1) {
                    t_rows[x].push_str(" ");
                }
                t_rows[x].push_string(v_sep.to_string());
            }
        }
        // draw those created lines
        sbs.add_line( header);
        for i in 0..max_transactions_per_day {
            sbs.add_line(t_rows[i].clone());
        }
        sbs.add_line(ColouredString::from_string(hdiv_row.clone()));
    }
    return sbs;
}

pub fn render_day_submenu(
    highlight_index:isize,
    cal_highlight_index:i64,
    cal_this_index:i64,
    signed_with_colour:bool,
    brackets_when_neg:bool,
    transactions: &Vec<Transaction>,
    visible_accounts: &Vec<Account>,
    accounts: &Vec<Account>,
    width:usize,
) -> (SideBySide, Option<Transaction>, isize) // return copy of the highlighted transaction too, as well as the largest index for bounds calculations
{
    let highlighted_date = Utc::now().date().checked_add_signed(
        chrono::Duration::days(-cal_this_index+cal_highlight_index)).unwrap().naive_utc();

    let accs = acc_ids_from(&visible_accounts);
    let mut highlighted_date_transactions = Vec::new();
    for t in transactions.to_vec().into_iter() {
        if t.date() == highlighted_date && (accs.contains(&t.account_id_1()) || accs.contains(&t.account_id_2())) { // on display range
            highlighted_date_transactions.push(t.clone());
        }
    }
    let num_hi_trans = highlighted_date_transactions.len() as isize;

    let mut sbs = SideBySide::new(32);
    sbs.add_line(ColouredString::new()); // blank line at the top!
    wrap_string!(sbs, "Detailed Transactions for", width);
    for s in wrap(format!("{} {} {}",
                          highlighted_date.day(),
                          MONTHS[highlighted_date.month0() as usize],
                          highlighted_date.year()), width) {
        sbs.add_line(ColouredString::from_string(s).bold().black().on_white());
    }
    wrap_string!(sbs, "Closing balance:", width);
    sbs.add_line(pence_to_pound_colour_bg(get_closing_balance( // TODO deal with wrapping / truncating numbers later
                            highlighted_date, transactions, visible_accounts
                        ),signed_with_colour, brackets_when_neg));
    for s in wrap("Transactions:".to_string(), width) {
        sbs.add_line(ColouredString::from_string(s).italic());
    }

    let mut highlighted_transaction = None;

    for (i, t) in highlighted_date_transactions.into_iter().enumerate() {
        sbs.add_line(ColouredString::from_string(truncate("----------------".to_string(), width)));
        let ss = wrap(
            match t.reference().as_str() {
                "" => "(unnamed)",
                r => r
            }.to_string(), width);
        for (index, s) in ss.clone().into_iter().enumerate() {
            let amount = match t.is_transfer() {
                true => {
                    pence_to_pound_transfer(
                        t.amount(),
                        signed_with_colour,
                        brackets_when_neg)
                },
                false => {
                    pence_to_pound_colour(
                        t.amount(),
                        signed_with_colour,
                        brackets_when_neg)
                }
            }.bold();
            if index == ss.len()-1 { // last one so now deal with the amount (seperately because seperate styling)
                if s.len() + 1 + amount.len() <= width {
                    // draw at end of current line
                    let mut temp;
                    if i as isize == highlight_index {
                        temp = ColouredString::from_string(s).black().on_cyan().bold();
                        highlighted_transaction = Some(t.clone());
                    } else {
                        temp = ColouredString::from_string(s).bold();
                    };
                    temp.push_str(" ");
                    temp.push_coloured_string(amount);
                    sbs.add_line(temp);
                } else {
                    // draw on new line
                    if i as isize == highlight_index {
                        sbs.add_line(ColouredString::from_string(s).black().on_cyan().bold());
                        highlighted_transaction = Some(t.clone());
                    } else {
                        sbs.add_line(ColouredString::from_string(s).bold());
                    };
                    sbs.add_line(amount); // TODO handle when this is too long (truncate amount)
                }
            } else {
                // don't worry about any of that
                if i as isize == highlight_index {
                    sbs.add_line(ColouredString::from_string(s).black().on_cyan().bold());
                    highlighted_transaction = Some(t.clone());
                } else {
                    sbs.add_line(ColouredString::from_string(s).bold());
                };
            }
        }
        // assemble a couple of re-usable parts before the match to draw the transaction's accounts / account+category
        let mut ac = match t.account_1(&accounts) {
            Ok(acc) => acc.name,
            Err(_) => "Unknown Account".to_string()
        };
        match t.account_2(&accounts){
            Ok(acc) => match acc.category {
                true => { // Display as a category
                    wrap_string!(sbs, "Account:", width);
                    for s in wrap(ac, width) {
                        sbs.add_line(ColouredString::from_string(s).purple());
                    }
                    wrap_string!(sbs, format!("Category: {}",acc.name), width);
                },
                false => { // Display as another account
                    wrap_string!(sbs, "From Account:", width);
                    for s in wrap(ac, width) {
                        sbs.add_line(ColouredString::from_string(s).purple());
                    }
                    wrap_string!(sbs, "To Account:", width);
                    for s in wrap(acc.name, width) {
                        sbs.add_line(ColouredString::from_string(s).purple());
                    }
                }
            },
            Err(_) => {
                sbs.add_line(ColouredString::from_str("<Category/Account name not found>"));
            }
        }
        wrap_string!(sbs, format!("Date: {}",t.date()), width);
        wrap_string!(sbs, "Notes:", width);
        wrap_string!(sbs, t.notes(), width);
    }
    return (sbs,highlighted_transaction,num_hi_trans);
}

fn get_closing_balance(
    date:NaiveDate,
    transactions: &Vec<Transaction>,
    visible_accounts: &Vec<Account>
) -> i64
{
    // calculate the closing balance summed over all the <accounts> on >date>
    let accs = acc_ids_from(&visible_accounts);
    let mut sum:i64 = 0;
    for transaction in transactions.to_vec().into_iter() {
        // If the main transaction account, add amount
        if transaction.date() <= date && accs.contains(&transaction.account_id_1()) {
            sum += transaction.amount();
        }
        // If the destination / secondary account visible too, then subtract to offset it!
        // (or if this is the only one visible then it counts negatively)
        if transaction.date() <= date && accs.contains(&transaction.account_id_2()) {
            sum -= transaction.amount();
        }
    }
    // TODO this is where to account for prediction values ??
    return sum;
}

fn acc_ids_from(accounts: &Vec<Account>) -> Vec<u32> {
    let mut v = Vec::new();
    for a in accounts.to_vec().into_iter() {
        v.push(a.id);
    }
    return v;
}