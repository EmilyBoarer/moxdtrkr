use std::collections::HashMap;
use chrono::prelude::*;
use chrono::Duration;
use clearscreen;
use crate::{ColouredString, sidebyside, Transaction};

fn render_bar_util(mut pence:i64, scale:i64) -> ColouredString { // scale is pence per character
    let green = pence >= 0;
    if pence < 0 {
        pence = -pence;
    }
    let mut bar = ColouredString::new();
    let txt = format!("£{}.{:02}", // TODO update this so that it is using pence per pound and the standard settings for which sign and brackets!
                   (pence/100).abs(),
                   (pence%100).abs());
    for character in txt.chars() {
        if pence > 0 {
            if green{
                bar.push_coloured_string(ColouredString::from_string(character.to_string()).black().on_green());
            } else {
                bar.push_coloured_string(ColouredString::from_string(character.to_string()).black().on_red());
            }
            pence -= scale;
        } else {
            bar.push_coloured_string(ColouredString::from_string(character.to_string()));
        }
    }
    while pence > 0 {
        if green{
            bar.push_coloured_string(ColouredString::from_str(" ").black().on_green());
        } else {
            bar.push_coloured_string(ColouredString::from_str(" ").black().on_red());
        }
        pence -= scale;
    }
    return bar;
}

pub fn draw_graph(labels: &Vec<ColouredString>, values: &Vec<i64>, selection_index: i64) {
    let y_width = 20;
    let width = 60;

    let mut max_value = 1;
    for v in values.to_vec().into_iter() {
        if v > max_value { max_value = v }
        if -v > max_value { max_value = -v } // account for magnitude only!
    }

    let mut scale = max_value/width;

    let mut y_annotations = sidebyside::SideBySide::new(y_width);
    let mut y_values = sidebyside::SideBySide::new(1);

    let mut counter = 0;
    for cs in labels.to_vec().into_iter() {
        if counter == selection_index {
            y_annotations.add_line(cs.black().on_cyan());
        } else {
            y_annotations.add_line(cs);
        }
        counter += 1;
    }

    for v in values.to_vec().into_iter(){
        y_values.add_line(render_bar_util(v, scale));
    }

    sidebyside::SideBySide::print2(&y_annotations, &y_values, " | ");
    // for _ in 0..(y_width+1) { print!(" "); }
    // print!("+");
    // for _ in 0..(width+1) { print!("-"); }
    // println!(">");
}

enum SubGraphCounter {
    CountAmount(i64, i64)
}

pub fn graph_acc_cats(accounts: Vec<u32>, categories: Vec<u32>, transactions: &Vec<Transaction>) {
    let mut selecting = true;
    let num_weeks = 10;

    let mut labels = Vec::new();
    let mut values = Vec::new();

    // calc end of the current week = next monday
    let now = Utc::now().date();
    let diff = match now.weekday(){
        Weekday::Mon => {7}
        Weekday::Tue => {6}
        Weekday::Wed => {5}
        Weekday::Thu => {4}
        Weekday::Fri => {3}
        Weekday::Sat => {2}
        Weekday::Sun => {1}
    };

    let mut ub = NaiveDate::from_ymd_opt(now.year(), now.month(), now.day()).unwrap()
        .checked_add_signed(Duration::days(diff)).unwrap(); // TODO remove options

    let mut lb = ub.checked_sub_signed(Duration::days(7)).unwrap(); // TODO handle option

    for _ in 0..num_weeks {
        labels.push(ColouredString::from_string(format!("Week {}", lb)));
        // sum all values in range
        let mut working_value = 0;
        for transaction in transactions.to_vec().into_iter() {
            if accounts.contains(&transaction.account_id_1()) &&
                categories.contains(&transaction.account_id_2()) &&
                (transaction.date() >= lb) &&
                (transaction.date() < ub) {
                working_value += transaction.amount();
            }
        }
        values.push(working_value);

        // calc range bounds for next iter
        ub = lb;
        lb = ub.checked_sub_signed(Duration::days(7)).unwrap(); // TODO handle option
    }

    // reverse the vecs so newest at bottom
    labels.reverse();
    values.reverse();

    let mut hi = num_weeks-1; // highlighted index

    // draw graph on that data
    while selecting {
        // calc sub-graph
        // calc bounds
        ub = NaiveDate::from_ymd_opt(now.year(), now.month(), now.day()).unwrap()
            .checked_add_signed(Duration::days(diff)).unwrap();
        ub = ub.checked_sub_signed(Duration::days(7*(num_weeks-1-hi))).unwrap(); // TODO handle option
        lb = ub.checked_sub_signed(Duration::days(7)).unwrap(); // TODO handle option

        // calc values into hash map
        let mut sub_graph:HashMap<String, SubGraphCounter> = HashMap::new();
        for transaction in transactions.to_vec().into_iter() {
            if accounts.contains(&transaction.account_id_1()) &&
                categories.contains(&transaction.account_id_2()) &&
                (transaction.date() >= lb) &&
                (transaction.date() < ub) {
                match sub_graph.get(transaction.reference().as_str()) {
                    Some(SubGraphCounter::CountAmount(count, amount)) => {sub_graph.insert(
                        transaction.reference().to_string(),
                        SubGraphCounter::CountAmount(
                            count + 1,
                            transaction.amount() + amount
                        )
                    );}
                    None => {sub_graph.insert(
                        transaction.reference().to_string(),
                        SubGraphCounter::CountAmount(
                            1,
                            transaction.amount()
                        )
                    );}
                }
            }
        }
        // convert sub_graph to vecs for use in draw_graph
        let mut sub_graph_labels = Vec::new();
        let mut sub_graph_values = Vec::new();
        for (k, SubGraphCounter::CountAmount(count, amount))
            in sub_graph.keys().zip(sub_graph.values()) {
            sub_graph_labels.push(ColouredString::from_string(format!("{} ({})",k,count.clone())));
            sub_graph_values.push(amount.clone());
        }

        // TODO rank sub-graph by size

        // draw
        clearscreen::clear().expect("failed to clear screen");
        println!("Expenditure across selected categories and accounts in each week beginning:\n");
        draw_graph(&labels, &values, hi);
        println!("\nExpenditure breakdown for selected week\n");
        draw_graph(&sub_graph_labels, &sub_graph_values, -1);
        println!("\n{} {}",
             ColouredString::from_str("[↑↓ to move selection, esc to stop sub-graphing]").cyan(),
             ColouredString::from_str("[Ctrl-C from here will break things - don't do it!]").red(),
        );

        let term = console::Term::stdout();

        let mut capturing:bool = true;
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
                    console::Key::Escape => {
                        capturing = false;
                        selecting = false;
                    },
                    _ => println!("unknown input!")
                },
                Err(e) => println!("error {}", e),
            }
            if hi > num_weeks-1 { hi = num_weeks - 1 }
            if hi < 0 { hi = 0 }
        }
    }
}