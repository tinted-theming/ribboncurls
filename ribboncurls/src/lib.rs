#![doc = include_str!("../README.md")]

mod syntax_tree;
mod token;

use regex::Regex;
use serde_yaml::Value;
use syntax_tree::{create_syntax_tree, SyntaxItem};
use token::tokenize;

const DEFAULT_LEFT_DELIMITER: &str = "{{";
const DEFAULT_RIGHT_DELIMITER: &str = "}}";

#[non_exhaustive]
#[derive(thiserror::Error, Debug)]
pub enum RibboncurlsError {
    #[error("missing delimiter")]
    MissingDelimiter,
    #[error("missing end tag")]
    MissingEndTag,
    #[error("bad input")]
    YamlParseError(#[from] serde_yaml::Error),
}

struct ParseContext {
    left_delimiter: String,
    right_delimiter: String,
}

pub fn render(
    template: &str,
    data: &str,
    partials: Option<&str>,
) -> Result<String, RibboncurlsError> {
    let data = serde_yaml::from_str(data)?;
    let mut ctx = ParseContext {
        left_delimiter: DEFAULT_LEFT_DELIMITER.to_string(),
        right_delimiter: DEFAULT_RIGHT_DELIMITER.to_string(),
    };
    let tokens = tokenize(template, &mut ctx)?;
    let syntax_tree = create_syntax_tree(tokens)?;

    render_syntax_tree(&syntax_tree, &data)
}

fn remove_leading_space(output: &mut String) {
    let re = Regex::new(r"[ \t]*\z").unwrap();

    if re.is_match(output) {
        *output = re.replace_all(output, "").to_string();
    }
}

fn remove_leading_line_and_space(output: &mut String) {
    let re = Regex::new(r"\n[ \t]*\z").unwrap();

    if re.is_match(output) {
        *output = re.replace_all(output, "").to_string();
    }
}

fn render_syntax_tree(
    syntax_tree: &[SyntaxItem],
    data: &Value,
) -> Result<String, RibboncurlsError> {
    let mut output = String::new();

    for (index, node) in syntax_tree.iter().enumerate() {
        match node {
            SyntaxItem::Text(content) => {
                if index == 1 || index == 2 {
                    match syntax_tree.get(index - 1) {
                        Some(SyntaxItem::Delimiter { is_standalone })
                        | Some(SyntaxItem::Comment {
                            text: _,
                            is_standalone,
                        }) => {
                            if *is_standalone && content.starts_with('\n') {
                                if let Some(updated_content) = content.strip_prefix('\n') {
                                    output.push_str(updated_content);
                                }
                            } else {
                                output.push_str(content.as_str());
                            }
                        }
                        _ => {
                            output.push_str(content.as_str());
                        }
                    }
                } else {
                    output.push_str(content.as_str());
                }
            }
            SyntaxItem::EscapedVariable(content) => {
                if let Some(value) = data.get(content.as_str()) {
                    output.push_str(&html_escape::encode_text(&serde_yaml_value_to_string(
                        value,
                    )));
                }
            }
            SyntaxItem::Variable(content) => {
                if let Some(value) = data.get(content.as_str()) {
                    output.push_str(&serde_yaml_value_to_string(value));
                }
            }
            SyntaxItem::Comment {
                text: _,
                is_standalone,
            } => {
                if index == 1 {
                    if let Some(SyntaxItem::Text(text_content)) = syntax_tree.first() {
                        let re = Regex::new(r"^[ \t]*\z").unwrap();

                        if re.is_match(text_content) {
                            output = String::new();
                        }
                    }
                } else if let Some(SyntaxItem::Text(_)) = syntax_tree.get(index + 1) {
                    if *is_standalone {
                        remove_leading_line_and_space(&mut output);
                    }
                } else {
                    remove_leading_space(&mut output);
                }
            }
            SyntaxItem::Delimiter { is_standalone } => {
                if *is_standalone {
                    if syntax_tree.len() == index + 1 || index == 1 {
                        remove_leading_space(&mut output);
                    } else {
                        remove_leading_line_and_space(&mut output);
                    }
                }
            }
            SyntaxItem::Section {
                name,
                items,
                inverted,
            } => {
                remove_leading_space(&mut output);

                match (data.get(name), inverted) {
                    (Some(value), false) => {
                        if !serde_yaml_value_to_string(value).is_empty() {
                            let section_output = render_syntax_tree(items, data)?;

                            output.push_str(&section_output);
                        }
                    }
                    (None, true) => {
                        let section_output = render_syntax_tree(items, data)?;

                        output.push_str(&section_output);
                    }
                    (Some(value), true) => {
                        if serde_yaml_value_to_string(value).is_empty() {
                            let section_output = render_syntax_tree(items, data)?;

                            output.push_str(&section_output);
                        }
                    }
                    (None, false) => {}
                }
            }
        };
    }

    Ok(output)
}

fn serde_yaml_value_to_string(value: &Value) -> String {
    match value {
        Value::Number(n) => n.to_string(),
        Value::String(s) => s.to_owned(),
        Value::Bool(b) => {
            if *b {
                b.to_string()
            } else {
                "".to_string()
            }
        }

        _ => "".to_string(),
    }
}
