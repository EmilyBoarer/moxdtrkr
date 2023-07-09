use std::io::Write;

macro_rules! sfe {
    ($e:ident) => {
        println!("Error saving settings to file: {}", $e)
    }
}

pub fn save_settings_to_file(filepath:&str,
                             signed_with_colour:bool,
                             brackets_when_neg:bool,
                             weeks:u64,
                             max_transactions_per_day:u64,
                             day_width:u64,
                             info_width:u64,
) {
    // Serialize
    let s = format!("{},{},{},{},{},{}",
                    signed_with_colour,
                    brackets_when_neg,
                    weeks,
                    max_transactions_per_day,
                    day_width,
                    info_width
    );

    // Save file
    // save to file
    match std::fs::File::create(filepath) {
        Ok(f) => {
            let mut file = f;
            match file.write_all(s.as_bytes()) {
                Ok(_) => {}, // success & exit function without complaining
                Err(e) => sfe!(e)
            }
        },
        Err(e) => sfe!(e)
    }
}