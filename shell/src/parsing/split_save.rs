use std::io::{self, Write};
use crate::variables::var::*;

pub fn remove_all_quotes(s: &str) -> String {
    s.chars()
        .filter(|&c| c != '\'' && c != '"')
        .collect()
}

#[derive(PartialEq, Copy, Clone)]
enum QuoteState {
    None,
    Single,
    Double,
}

fn quote_state(s: &str) -> QuoteState {
    let mut state = QuoteState::None;
    let mut escape = false;

    for c in s.chars() {
        if escape {
            escape = false;
            continue;
        }

        if c == '\\' {
            escape = true;
            continue;
        }

        match (c, state) {
            ('\'', QuoteState::None) => state = QuoteState::Single,
            ('\'', QuoteState::Single) => state = QuoteState::None,
            ('"', QuoteState::None) => state = QuoteState::Double,
            ('"', QuoteState::Double) => state = QuoteState::None,
            _ => {}
        }
    }

    state
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

pub fn split_save(mut input: String) -> Var {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    input = input.trim_end().to_string();
    let mut state = quote_state(&input);

    while state != QuoteState::None {
        match state {
            QuoteState::Single => print!("quote> "),
            QuoteState::Double => print!("dquote> "),
            QuoteState::None => break,
        }
        stdout.flush().unwrap();

        let mut next_line = String::new();
        if stdin.read_line(&mut next_line).is_err() {
            break;
        }
        input.push('\n');
        input.push_str(next_line.trim_end());

        state = quote_state(&input);
    }

    if input.is_empty() {
        return Var::new();
    }

    let tokens = tokenize(&input);
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
    let mut escape = false;

    while let Some(&c) = chars.peek() {
        match c {
            '\\' if !escape => {
                escape = true;
                chars.next();
            }
            '"' | '\'' if !escape => {
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
            _ => {
                if escape {
                    current.push(c);
                    escape = false;
                    chars.next();
                } else if c == ' ' && in_quotes.is_none() {
                    if !current.is_empty() {
                        tokens.push(current.clone());
                        current.clear();
                    }
                    chars.next();
                } else {
                    current.push(c);
                    chars.next();
                }
            }
        }
    }

    if !current.is_empty() {
        tokens.push(current);
    }

    tokens
}
