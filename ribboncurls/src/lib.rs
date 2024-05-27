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

struct TokenCtx {
    left_delimiter: String,
    right_delimiter: String,
}

struct RenderCtx {
    section_path: Vec<String>,
    data_stack: Vec<Value>,
}

pub fn render(
    template: &str,
    data: &str,
    partials: Option<&str>,
) -> Result<String, RibboncurlsError> {
    let data_stack = vec![serde_yaml::from_str(data).unwrap_or(Value::String(data.to_string()))];

    let mut render_context = RenderCtx {
        section_path: vec![],
        data_stack,
    };

    let mut ctx = TokenCtx {
        left_delimiter: DEFAULT_LEFT_DELIMITER.to_string(),
        right_delimiter: DEFAULT_RIGHT_DELIMITER.to_string(),
    };
    let tokens = tokenize(template, &mut ctx)?;
    let syntax_tree = create_syntax_tree(tokens)?;

    render_syntax_tree(&syntax_tree, &mut render_context)
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
    ctx: &mut RenderCtx,
) -> Result<String, RibboncurlsError> {
    let mut output = String::new();

    for (index, node) in syntax_tree.iter().enumerate() {
        match node {
            SyntaxItem::Text(content) => {
                let is_root = ctx.section_path.is_empty();

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
                if let Some(value) = get_context_value(ctx, content.as_str()) {
                    output.push_str(&escape_html(&serde_yaml_value_to_string(value)));
                }
            }
            SyntaxItem::Variable(content) => {
                if let Some(value) = get_context_value(ctx, content.as_str()) {
                    output.push_str(&serde_yaml_value_to_string(value));
                }
            }
            SyntaxItem::Comment {
                text: _,
                is_standalone,
            } => {
                let is_root = ctx.section_path.is_empty();

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
                open_is_standalone: _,
                closed_is_standalone: _,
            } => {
                ctx.section_path.push(name.to_string());
                let mut section_value_option = None;
                let mut is_mutating_context_stack = false;

                if let Some(section_value) = get_context_value(ctx, &ctx.section_path.join(".")) {
                    section_value_option = Some(section_value.clone());
                    if matches!(section_value, Value::Mapping(_)) {
                        ctx.data_stack.push(section_value.clone());

                        is_mutating_context_stack = true;
                    }
                }

                match (section_value_option, inverted) {
                    (Some(value), false) => {
                        if is_value_truthy(&value) {
                            let section_output = render_syntax_tree(items, ctx)?;

                            output.push_str(&section_output);
                        };
                    }
                    (None, true) => {
                        let section_output = render_syntax_tree(items, ctx)?;

                        output.push_str(&section_output);
                    }
                    (Some(value), true) => {
                        if is_value_falsy(&value) && !matches!(value, Value::Mapping(_)) {
                            let section_output = render_syntax_tree(items, ctx)?;

                            output.push_str(&section_output);
                        }
                    }
                    (None, false) => {}
                }

                if is_mutating_context_stack {
                    ctx.data_stack.pop();
                }
                ctx.section_path.pop();
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

fn find_section_value(ctx: &RenderCtx, section_name: &str) -> Option<Value> {
    let context_stack = &ctx.data_stack;
    let parts = section_name.split('.').collect::<Vec<&str>>();
    let mut current_option: Option<Value> = None;

    'outer: for context in context_stack.iter().rev() {
        current_option = Some(context.clone());
        for (index, part) in parts.clone().iter().enumerate() {
            if let Some(current) = current_option {
                match current.get(part) {
                    Some(Value::Mapping(map)) => {
                        current_option = Some(Value::Mapping(map.clone()));
                        break 'outer;
                    }
                    Some(value) => {
                        if index == parts.len() - 1 {
                            current_option = Some(value.clone());
                        } else {
                            current_option = None;
                        }

                        break 'outer;
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
fn get_context_value<'a>(ctx: &'a RenderCtx, path: &str) -> Option<&'a Value> {
    let context_stack = &ctx.data_stack;
    if path.is_empty() || path.is_empty() {
        return None;
    }

    let parts = path.split('.').collect::<Vec<&str>>();

    // Check for full path matches on property names
    for context in context_stack.iter().rev() {
        if context.get(path).is_some() {
            return context.get(path);
        }

        if let Value::Mapping(_) = context {
            return get_value(context, &parts.join("."));
        }
    }

    // Check for partial path matches on property names
    for context in context_stack.iter().rev() {
        // Return context data
        if path == "." {
            // Root path should return root context
            if ctx.section_path.is_empty() {
                return Some(context);
            // Otherwise attempt to match on context
            } else if let Some(current_section) = ctx.section_path.last() {
                if context.get(current_section).is_some() {
                    return Some(context);
                };
            }
        // Perform a root level search on the context
        } else if !path.contains('.') {
            if let Value::Mapping(map) = context {
                if map.get(path).is_some() {
                    return map.get(path);
                }
            };
        // Only run on paths that match on ctx.section_path order
        } else if contains_vec_item_order(ctx.section_path.clone(), &parts) {
            if let Value::Mapping(map) = context {

                let first_part = parts.first()?;

                for part_index in 0..parts.len() {
                    let index = parts.len() - (part_index + 1);

                    if let Some(data) = map.get(first_part) {
                        return get_value(data, &parts[index..].join("."));
                    }
                }
            }
        }
    }

    None
}

/// Finds the index of a target string within a slice of strings.
///
/// # Arguments
/// * `vec` - A slice of string slices to search.
/// * `target` - The string slice to find.
///
/// # Returns
/// Returns `Option<usize>` with the index of the first occurrence of `target` if found, otherwise `None`.
///
/// # Examples
/// ```
/// let vec = ["hello", "world", "rust", "code"];
/// assert_eq!(find_vec_index(&vec, "rust"), Some(2));
/// assert_eq!(find_vec_index(&vec, "hello"), Some(0));
/// assert_eq!(find_vec_index(&vec, "none"), None);
/// ```
fn find_vec_index(vec: &[String], target: &str) -> Option<usize> {
    vec.iter().position(|item| item == target)
}

/// Checks if all elements of `vec2` appear in the same order within `vec1`.
///
/// # Arguments
/// * `vec1` - A slice of string slices to search within.
/// * `vec2` - A slice of string slices whose order of appearance in `vec1` is being checked.
///
/// # Returns
/// Returns `true` if all elements of `vec2` appear in the same order within `vec1`, otherwise `false`.
///
/// # Examples
/// ```
/// let vec1 = ["this", "is", "some", "full", "sentence"];
/// let vec2 = ["is", "full"];
/// let vec3 = ["some", "is"];
/// assert_eq!(contains_vec_item_order(&vec1, &vec2), true);
/// assert_eq!(contains_vec_item_order(&vec1, &vec3), false);
/// ```
fn contains_vec_item_order(vec1: Vec<String>, vec2: &[&str]) -> bool {
    let mut match_index = 0;

    if vec1.len() < vec2.len() || vec1.is_empty() || vec2.is_empty() {
        return false;
    };

    // If the vectors match on partially equal return true
    if vec1 != vec2 {
        for item in vec2 {
            if let Some(index_match) = find_vec_index(&vec1[match_index + 1..], item) {
                match_index = index_match;
            } else {
                return false;
            }
        }
    }

    true
}

fn get_value<'a>(data: &'a Value, path: &str) -> Option<&'a Value> {
    if path.is_empty() || path.is_empty() {
        return None;
    }

    let parts = path.split('.').collect::<Vec<&str>>();

    // Match last item in recursion
    if parts.len() == 1 {
        return data.get(path);
    }

    // Match if property `a.b.c.d` exists
    if let Some(data) = data.get(path) {
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
