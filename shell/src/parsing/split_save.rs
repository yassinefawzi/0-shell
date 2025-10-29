use std::io::{ self, Write };
use crate::variables::var::*;

pub fn remove_all_quotes(s: &str) -> String {
    s.chars()
        .filter(|&c| c != '\'' && c != '"')
        .collect()
}

#[derive(PartialEq)]
enum QuoteState {
    None,
    Single,
    Double,
}

fn quote_state(s: &str) -> QuoteState {
    let mut state = QuoteState::None;
    for c in s.chars() {
        match c {
            '\'' if state != QuoteState::Double => {
                state = if state == QuoteState::Single {
                    QuoteState::None
                } else {
                    QuoteState::Single
                };
            }
            '"' if state != QuoteState::Single => {
                state = if state == QuoteState::Double {
                    QuoteState::None
                } else {
                    QuoteState::Double
                };
            }
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
            QuoteState::None => {
                break;
            }
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
