use crate::ColouredString;


pub fn pence_to_pound(pence:i64, sign:bool, brackets:bool) -> ColouredString {
    let mut temp = ColouredString::from_string(format!("{}£{}{}.{:02}{}",
                   if pence < 0 && sign {"-"} else {""},
                   if pence < 0 && brackets {"("} else {""},
                   (pence/100).abs(),
                   (pence%100).abs(),
                   if pence < 0 && brackets {")"} else {""},
    ));
    temp.bodge_alter_len(1); // I think this is needed because the pound sign messes things up!
    return temp.clone();
}

pub fn pence_to_pound_colour(pence:i64, sign:bool, brackets:bool) -> ColouredString {
    let s = pence_to_pound(pence, sign, brackets);
    if pence >= 0 {
        return s.green();
    } else {
        return s.red();
    }
}

pub fn pence_to_pound_colour_bg(pence:i64, sign:bool, brackets:bool) -> ColouredString {
    let s = pence_to_pound(pence, sign, brackets);
    if pence >= 0 {
        return s.on_green().black();
    } else {
        return s.on_red().black();
    }
}

pub fn pence_to_pound_transfer(pence:i64, sign:bool, brackets:bool) -> ColouredString {
    pence_to_pound(pence, sign, brackets).yellow()
}