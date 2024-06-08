use std::cmp::Ordering;

use regex::Regex;

pub fn escape_html(input: &str) -> String {
    let mut output = String::with_capacity(input.len() * 2);
    for c in input.chars() {
        match c {
            '&' => output.push_str("&amp;"),
            '<' => output.push_str("&lt;"),
            '>' => output.push_str("&gt;"),
            '"' => output.push_str("&quot;"),
            '/' => output.push_str("&#x2F;"),
            _ => output.push(c),
        }
    }

    // Not using shrink_to_fit() on purpose
    output
}

pub fn get_prev_item<T>(data: &[T], index: usize) -> Option<&T> {
    if index > 0 {
        data.get(index - 1)
    } else {
        None
    }
}

pub fn get_next_item<T>(data: &[T], index: usize) -> Option<&T> {
    if index < data.len() - 1 {
        data.get(index + 1)
    } else {
        None
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Newline {
    Crlf,
    Lf,
}

impl Newline {
    pub fn as_str(self) -> &'static str {
        match self {
            Newline::Lf => "\n",
            Newline::Crlf => "\r\n",
        }
    }

    pub fn to_regex(self) -> Regex {
        match self {
            Newline::Lf => Regex::new(r"\n").expect("Unable to get regex for '\\n'"),
            Newline::Crlf => Regex::new(r"\r\n").expect("Unable to get regex for '\\r\\n'"),
        }
    }
}

pub enum NewlineRegex {
    StartsWithNewline,
    EndsWithNewlineFollowedByWhitespace,
    StartsWithNewlineFollowedByWhitespace,
    StartsWithNewlineFollowedByWhitespaceUntilEnd,
}

pub fn get_newline_variant(text: &str) -> Newline {
    let re_crlf = Newline::Crlf.to_regex();
    let re_lf = Newline::Lf.to_regex();

    let count_crlf = re_crlf.find_iter(text).count();
    let count_lf = re_lf.find_iter(text).count();

    match count_crlf.cmp(&count_lf) {
        Ordering::Equal | Ordering::Greater => Newline::Crlf,
        Ordering::Less => Newline::Lf,
    }
}

pub fn get_regex_for_newline(newline_regex: NewlineRegex, newline: Newline) -> Regex {
    match (newline_regex, newline) {
        (NewlineRegex::StartsWithNewline, Newline::Lf) => {
            Regex::new(r"^\n").expect("Unable to get regex")
        }
        (NewlineRegex::EndsWithNewlineFollowedByWhitespace, Newline::Lf) => {
            Regex::new(r"\n[ \t]*\z").expect("Unable to get regex")
        }
        (NewlineRegex::StartsWithNewlineFollowedByWhitespace, Newline::Lf) => {
            Regex::new(r"^\n[ \t]*").expect("Unable to get regex")
        }
        (NewlineRegex::StartsWithNewlineFollowedByWhitespaceUntilEnd, Newline::Lf) => {
            Regex::new(r"^\n[ \t]*\z").expect("Unable to get regex")
        }

        (NewlineRegex::StartsWithNewline, Newline::Crlf) => {
            Regex::new(r"^\r\n").expect("Unable to get regex")
        }
        (NewlineRegex::EndsWithNewlineFollowedByWhitespace, Newline::Crlf) => {
            Regex::new(r"\r\n[ \t]*\z").expect("Unable to get regex")
        }
        (NewlineRegex::StartsWithNewlineFollowedByWhitespace, Newline::Crlf) => {
            Regex::new(r"^\r\n[ \t]*").expect("Unable to get regex")
        }
        (NewlineRegex::StartsWithNewlineFollowedByWhitespaceUntilEnd, Newline::Crlf) => {
            Regex::new(r"^\r\n[ \t]*\z").expect("Unable to get regex")
        }
    }
}
