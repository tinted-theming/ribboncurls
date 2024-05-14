#![doc = include_str!("../README.md")]

use serde_yaml::Error;

const DEFAULT_LEFT_DELIMITER: &str = "{{";
const DEFAULT_RIGHT_DELIMITER: &str = "}}";

#[derive(Debug)]
pub enum Token {
    Text(String),
    Variable(String),
    EscapedVariable(String),
    OpenSection(String),
    CloseSection(String),
    OpenInvertedSection(String),
    Partial(String),
    Comment(String),
}

pub fn render(template: &str) -> Result<Vec<Token>, Error> {
    let output = tokenize(template);

    Ok(output)
}

fn tokenize(template: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut i = 0;
    let single_char_left_delimiter = &DEFAULT_LEFT_DELIMITER[0..1];
    let single_char_right_delimiter = &DEFAULT_RIGHT_DELIMITER[0..1];
    let tripple_left_delimiter = format!(
        "{single_char_left_delimiter}{single_char_left_delimiter}{single_char_left_delimiter}"
    );
    let tripple_right_delimiter = format!(
        "{single_char_right_delimiter}{single_char_right_delimiter}{single_char_right_delimiter}"
    );

    while i < template.len() {
        let current_str = &template[i..];

        match current_str {
            _ if current_str.starts_with(&tripple_left_delimiter) => {
                if let Some(end) = current_str.find(&tripple_right_delimiter) {
                    let end = end + i; // index in `template`
                    let content = &template[i + tripple_left_delimiter.len()..end].trim();

                    tokens.push(Token::Variable(content.trim().to_string()));

                    i = end + tripple_right_delimiter.len();
                } else {
                    break;
                }
            }

            _ if current_str.starts_with(DEFAULT_LEFT_DELIMITER) => {
                // If there is a following end-delimiter
                if let Some(end) = current_str.find(DEFAULT_RIGHT_DELIMITER) {
                    let end = end + i; // index in `template`
                    let content = &template[i + DEFAULT_LEFT_DELIMITER.len()..end].trim();

                    if let Some(token) = parse_tag(content) {
                        tokens.push(token);
                    }

                    i = end + DEFAULT_RIGHT_DELIMITER.len();
                } else {
                    break;
                }
            }

            _ => {
                // Find the start of the next tag or end of the template
                if let Some(next_tag_start) = current_str.find(DEFAULT_LEFT_DELIMITER) {
                    let text = &template[i..i + next_tag_start];
                    if !text.is_empty() {
                        tokens.push(Token::Text(text.to_string()));
                    }
                    i += next_tag_start;
                } else {
                    break;
                }
            }
        }
    }

    tokens
}

fn parse_tag(content: &str) -> Option<Token> {
    match content.chars().next()? {
        '#' => Some(Token::OpenSection(content[1..].trim().to_string())),
        '/' => Some(Token::CloseSection(content[1..].trim().to_string())),
        '^' => Some(Token::OpenInvertedSection(content[1..].trim().to_string())),
        '>' => Some(Token::Partial(content[1..].trim().to_string())),
        '!' => Some(Token::Comment(content[1..].trim().to_string())),
        _ => Some(Token::EscapedVariable(content.trim().to_string())),
    }
}

