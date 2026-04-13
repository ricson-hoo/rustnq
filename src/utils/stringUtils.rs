use std::char::ToUppercase;


pub(crate) fn to_camel_case(input: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = false;

    for c in input.chars() {
        if c.is_alphanumeric() {
            if capitalize_next {
                result.push(c.to_ascii_uppercase());
                capitalize_next = false;
            } else {
                result.push(c.to_ascii_lowercase());
            }
        } else {
            capitalize_next = true;
        }
    }

    result
}

pub(crate) fn begin_with_upper_case(input: &str) -> String {
    match input.len(){
        0 => String::new(),
        1 => input.to_uppercase(),
        _ => input.chars().take(1).collect::<String>().to_string().to_uppercase() + &input[1..]
    }
}