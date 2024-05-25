#![doc = include_str!("../README.md")]

mod syntax_tree;
mod token;
mod utils;

use regex::Regex;
use serde_yaml::Value;
use syntax_tree::{create_syntax_tree, SyntaxItem};
use token::tokenize;

use crate::utils::escape_html;

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
    let mut context_stack =
        vec![serde_yaml::from_str(data).unwrap_or(Value::String(data.to_string()))];

    let mut ctx = ParseContext {
        left_delimiter: DEFAULT_LEFT_DELIMITER.to_string(),
        right_delimiter: DEFAULT_RIGHT_DELIMITER.to_string(),
    };
    let tokens = tokenize(template, &mut ctx)?;
    let syntax_tree = create_syntax_tree(tokens)?;

    render_syntax_tree(&syntax_tree, &mut context_stack, true)
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
    context_stack: &mut Vec<Value>,
    is_root: bool,
) -> Result<String, RibboncurlsError> {
    let mut output = String::new();

    for (index, node) in syntax_tree.iter().enumerate() {
        match node {
            SyntaxItem::Text(content) => {
                if is_root && (index == 1 || index == 2) {
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
                if let Some(value) = get_context_value(context_stack, content.as_str()) {
                    output.push_str(&escape_html(&serde_yaml_value_to_string(value)));
                }
            }
            SyntaxItem::Variable(content) => {
                if let Some(value) = get_context_value(context_stack, content.as_str()) {
                    output.push_str(&serde_yaml_value_to_string(value));
                }
            }
            SyntaxItem::Comment {
                text: _,
                is_standalone,
            } => {
                if index == 1 && is_root {
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
                let section_value_option = find_section_value(context_stack, name);
                let mut is_mutating_context_stack = false;

                if let Some(section_value) = &section_value_option {
                    if matches!(section_value, Value::Mapping(_)) {
                        context_stack.push(section_value.clone());

                        is_mutating_context_stack = true;
                    }
                }

                match (section_value_option, inverted) {
                    (Some(value), false) => {
                        if is_value_truthy(&value) {
                            let section_output = render_syntax_tree(items, context_stack, false)?;

                            output.push_str(&section_output);
                        };
                    }
                    (None, true) => {
                        let section_output = render_syntax_tree(items, context_stack, false)?;

                        output.push_str(&section_output);
                    }
                    (Some(value), true) => {
                        if is_value_falsy(&value) && !matches!(value, Value::Mapping(_)) {
                            let section_output = render_syntax_tree(items, context_stack, false)?;

                            output.push_str(&section_output);
                        }
                    }
                    (None, false) => {}
                }

                if is_mutating_context_stack {
                    context_stack.pop();
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

fn find_section_value(context_stack: &[Value], section_name: &str) -> Option<Value> {
    let parts = section_name.split('.').collect::<Vec<&str>>();
    let mut current_option: Option<Value> = None;

    for context in context_stack.iter().rev() {
        current_option = Some(context.clone());
        for (index, part) in parts.clone().iter().enumerate() {
            if let Some(current) = current_option {
                match current.get(part) {
                    Some(Value::Mapping(map)) => {
                        current_option = Some(Value::Mapping(map.clone()));
                    }
                    Some(value) => {
                        if index == parts.len() - 1 {
                            current_option = Some(value.clone());
                        } else {
                            current_option = None;
                        }
                    }
                    None => {
                        current_option = None;

                        continue;
                    }
                }
            }
        }
    }

    current_option
}

fn get_context_value<'a>(context_stack: &'a [Value], path: &str) -> Option<&'a Value> {
    if path.is_empty() || path.is_empty() {
        return None;
    }

    if path == "." {
        if let Some(root_context) = context_stack.first() {
            return Some(root_context);
        }
    }

    let parts = path.split('.').collect::<Vec<&str>>();

    for context in context_stack.iter().rev() {
        if parts.len() == 1 {
            return match context {
                Value::Mapping(map) => map.get(path),
                _ => None,
            };
        } else if let Value::Mapping(map) = context {
            let first_part = parts.first()?;

            if let Some(map) = map.get(first_part) {
                return get_value(map, &parts[1..].join("."));
            } else {
                return None;
            };
        }
    }

    None
}

fn get_value<'a>(data: &'a Value, path: &str) -> Option<&'a Value> {
    if path.is_empty() || path.is_empty() {
        return None;
    }

    if path == "." {
        return Some(data);
    }

    let parts = path.split('.').collect::<Vec<&str>>();

    if parts.len() == 1 {
        return data.get(path);
    }

    match data {
        Value::Mapping(map) => {
            let first_part = parts.first()?;

            if let Some(map) = map.get(first_part) {
                get_value(map, &parts[1..].join("."))
            } else {
                None
            }
        }
        _ => None,
    }
}

fn is_value_falsy(value: &Value) -> bool {
    match value {
        Value::Null => true,
        Value::Bool(b) => !*b,
        Value::String(s) => s.is_empty(),
        Value::Sequence(seq) => seq.is_empty(),
        Value::Mapping(map) => map.is_empty(),
        _ => false, // For other types, consider them as non-falsy
    }
}

fn is_value_truthy(value: &Value) -> bool {
    !is_value_falsy(value)
}
