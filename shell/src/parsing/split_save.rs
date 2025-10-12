use crate::variables::var::*;

pub fn remove_all_quotes(s: &str) -> String {
    s.chars()
        .filter(|&c| c != '\'' && c != '"')
        .collect()
}

pub fn check_quote_error(input: &str) -> Option<String> {
    let mut single_open = false;
    let mut double_open = false;

    for c in input.chars() {
        match c {
            '\'' if !double_open => {
                single_open = !single_open;
            }
            '"' if !single_open => {
                double_open = !double_open;
            }
            _ => {}
        }
    }

    if single_open || double_open {
        Some("Unclosed quote".to_string())
    } else {
        None
    }
}

pub fn flatten_flags(flags: Vec<String>) -> Vec<String> {
    let mut result = Vec::new();

    for flag in flags {
        for c in flag.trim_start_matches('-').chars() {
            result.push(c.to_string());
        }
    }

    result
}


pub fn split_save(input: String) -> Var {
    let input = input.trim();
    if input.is_empty() {
        return Var::new();
    }

    if let Some(err) = check_quote_error(input) {
        println!("Error: {}", err);
        return Var::new();
    }

    let tokens = tokenize(input);
    if tokens.is_empty() {
        return Var::new();
    }

    let command = remove_all_quotes(&tokens[0]);
    let mut flags = Vec::new();
    let mut args = Vec::new();

    for token in tokens.iter().skip(1) {
        if token.starts_with('-') {
            flags.push(remove_all_quotes(token));
        } else {
            args.push(remove_all_quotes(token));
        }
    }

	let flags = flatten_flags(flags);
    Var { command, flags, args }
}

fn tokenize(s: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut chars = s.chars().peekable();
    let mut in_quotes = None;

    while let Some(&c) = chars.peek() {
        match c {
            '"' | '\'' => {
                let quote = c;
                if in_quotes.is_none() {
                    in_quotes = Some(quote);
                    chars.next();
                } else if in_quotes == Some(quote) {
                    in_quotes = None;
                    chars.next();
                } else {
                    current.push(c);
                    chars.next();
                }
            }
            ' ' if in_quotes.is_none() => {
                if !current.is_empty() {
                    tokens.push(current.clone());
                    current.clear();
                }
                chars.next();
            }
            _ => {
                current.push(c);
                chars.next();
            }
        }
    }

    if !current.is_empty() {
        tokens.push(current);
    }

    tokens
}
