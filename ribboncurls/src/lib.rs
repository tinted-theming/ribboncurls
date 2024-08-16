#![doc = include_str!("../README.md")]

mod syntax_tree;
mod token;
mod utils;

use serde_yaml::Value;
use syntax_tree::{create_syntax_tree, SyntaxItem};
use token::{tokenize, Token};
use utils::{escape_html, get_newline_variant, get_regex_for_newline, Newline, NewlineRegex};

const DEFAULT_LEFT_DELIMITER: &str = "{{";
const DEFAULT_RIGHT_DELIMITER: &str = "}}";

#[non_exhaustive]
#[derive(thiserror::Error, Debug)]
pub enum RibboncurlsError {
    #[error("missing delimiter")]
    MissingDelimiter,
    #[error("missing end tag")]
    MissingEndTag,
    #[error("missing data")]
    MissingData,
    #[error("string too large")]
    StringSize,
    #[error("bad tag")]
    BadTag,
    #[error("bad input")]
    YamlParseError(#[from] serde_yaml::Error),
}

struct TokenCtx {
    left_delimiter: String,
    right_delimiter: String,
    section_stack: Vec<Token>,
}

struct SyntaxCtx {
    is_root: bool,
    newline: Newline,
}

#[derive(Debug)]
struct RenderCtx {
    data_stack: Vec<Value>,
    partials: Value,
    section_path: Vec<String>,
    newline: Newline,
    indent: u8,
}

pub fn render(
    template: &str,
    data: &str,
    partials: Option<&str>,
) -> Result<String, RibboncurlsError> {
    let data_stack = vec![serde_yaml::from_str(data).unwrap_or(Value::String(data.to_string()))];
    let mut ctx = TokenCtx {
        left_delimiter: DEFAULT_LEFT_DELIMITER.to_string(),
        right_delimiter: DEFAULT_RIGHT_DELIMITER.to_string(),
        section_stack: Vec::new(),
    };
    let tokens = tokenize(template, &mut ctx)?;
    let mut syntax_ctx = SyntaxCtx {
        is_root: true,
        newline: get_newline_variant(template),
    };
    let syntax_tree = create_syntax_tree(tokens, &mut syntax_ctx)?;
    let mut render_context = RenderCtx {
        data_stack,
        partials: serde_yaml::from_str(partials.unwrap_or("null"))?,
        section_path: vec![],
        newline: get_newline_variant(template),
        indent: 0,
    };
    render_syntax_tree(&syntax_tree, &mut render_context)
}

fn render_syntax_tree(
    syntax_tree: &[SyntaxItem],
    ctx: &mut RenderCtx,
) -> Result<String, RibboncurlsError> {
    let re_starts_with_newline_followed_by_whitespace_until_end = get_regex_for_newline(
        NewlineRegex::StartsWithNewlineFollowedByWhitespaceUntilEnd,
        ctx.newline,
    );
    let mut output = String::new();

    for (index, node) in syntax_tree.iter().enumerate() {
        match node {
            SyntaxItem::Text(content) => {
                // Indent if indent exists (for partial)
                if ctx.indent > 0 {
                    if let Some(updated) = content.strip_prefix('\n') {
                        let mut indent_string = String::from('\n');

                        // The standalone partial removes the following newline and the last item
                        // here replaces that, but it should not contain the indent
                        if index != syntax_tree.len() - 1
                            || !re_starts_with_newline_followed_by_whitespace_until_end
                                .is_match(content)
                        {
                            for _ in 0..ctx.indent {
                                indent_string.push(' ');
                            }
                        }

                        output.push_str(format!("{}{}", indent_string, updated).as_str());
                    } else if index == 0 {
                        let mut indent_string = String::new();
                        for _ in 0..ctx.indent {
                            indent_string.push(' ');
                        }

                        output.push_str(format!("{}{}", indent_string, content).as_str());
                    }
                } else {
                    output.push_str(content);
                }
            }
            SyntaxItem::EscapedVariable(content) => {
                if let Some(value) = get_value_from_context(ctx, content.as_str()) {
                    output.push_str(&escape_html(&serde_yaml_value_to_string(value)));
                }
            }
            SyntaxItem::Variable(content) => {
                if let Some(value) = get_value_from_context(ctx, content.as_str()) {
                    output.push_str(&serde_yaml_value_to_string(value));
                }
            }
            SyntaxItem::Partial {
                name: partial_name,
                indent,
                is_standalone: _,
            } => {
                if let Some(partial_data) = ctx.partials.clone().get(partial_name) {
                    let mut token_ctx = TokenCtx {
                        left_delimiter: DEFAULT_LEFT_DELIMITER.to_string(),
                        right_delimiter: DEFAULT_RIGHT_DELIMITER.to_string(),
                        section_stack: Vec::new(),
                    };
                    let partial_tokens = tokenize(
                        partial_data
                            .as_str()
                            .expect("Unable to extract string from serde_yaml::Value"),
                        &mut token_ctx,
                    )?;
                    let mut syntax_ctx = SyntaxCtx {
                        is_root: false,
                        newline: ctx.newline,
                    };
                    let original_indent = ctx.indent;
                    ctx.indent = *indent;
                    let tree = create_syntax_tree(partial_tokens, &mut syntax_ctx)?;
                    let rendered = render_syntax_tree(&tree, ctx)?;
                    ctx.indent = original_indent;

                    output.push_str(&rendered);
                }
            }
            SyntaxItem::Comment { is_standalone: _ } => {}
            SyntaxItem::Delimiter { is_standalone: _ } => {}
            SyntaxItem::Section {
                name,
                items,
                is_inverted,
                open_is_standalone: _,
                closed_is_standalone: _,
            } => {
                // Sequence of sequences
                // ---------------------
                // A sequence of sequences behaves differently the the rest of the sections, so if
                // it matches, render and continue
                if let Some(sequence_output) =
                    render_sequence_of_sequences(name.to_string(), ctx, items)?
                {
                    output.push_str(&sequence_output);
                } else {
                    // All other sections
                    // ------------------
                    ctx.section_path.push(name.to_string());

                    let mut section_context_option = None;
                    let mut is_mutating_context_stack = false;
                    let mut iterator_option: Option<Value> = None;

                    // Add section context to the ctx.data_stack
                    if let Some(section_context) = get_value_from_context(ctx, name) {
                        section_context_option = Some(section_context.clone());
                        if matches!(section_context, Value::Mapping(_)) {
                            ctx.data_stack.push(section_context.clone());

                            is_mutating_context_stack = true;
                        } else if matches!(section_context, Value::Sequence(_)) {
                            iterator_option = Some(section_context.clone());
                        }
                    }

                    // Iterate and render over the sequence
                    match (iterator_option, is_inverted) {
                        (Some(Value::Sequence(section_context)), false) => {
                            for item in section_context {
                                ctx.data_stack.push(item);

                                match (&section_context_option, is_inverted) {
                                    (Some(value), false) => {
                                        if is_value_truthy(value) {
                                            let section_output = render_syntax_tree(items, ctx)?;

                                            output.push_str(&section_output);
                                        };
                                    }
                                    (None, true) => {
                                        let section_output = render_syntax_tree(items, ctx)?;

                                        output.push_str(&section_output);
                                    }
                                    (Some(value), true) => {
                                        if is_value_falsy(value)
                                            && !matches!(value, Value::Mapping(_))
                                        {
                                            let section_output = render_syntax_tree(items, ctx)?;

                                            output.push_str(&section_output);
                                        }
                                    }
                                    (None, false) => {}
                                }
                                ctx.data_stack.pop();
                            }
                        }
                        // Otherwise render without iteration
                        _ => match (section_context_option, is_inverted) {
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
                                if is_value_falsy(&value) {
                                    let section_output = render_syntax_tree(items, ctx)?;

                                    output.push_str(&section_output);
                                }
                            }
                            (None, false) => {}
                        },
                    };

                    if is_mutating_context_stack {
                        ctx.data_stack.pop();
                    }
                    ctx.section_path.pop();
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

fn get_value_from_context<'a>(ctx: &'a RenderCtx, path: &str) -> Option<&'a Value> {
    let section_path = &ctx.section_path;
    let data_stack_len = &ctx.data_stack.len();
    if path.is_empty() {
        return None;
    }

    // Return context for "." implicit iterator variables
    if path == "." {
        let current_section_option = section_path.last();
        let latest_context_option = ctx.data_stack.last();

        return match (current_section_option, latest_context_option) {
            (Some(current_section), Some(latest_context)) => {
                let value_option = latest_context.get(current_section);
                if value_option.is_some() {
                    return value_option;
                }

                Some(latest_context)
            }
            (None, Some(latest_context)) => {
                if *data_stack_len == 1 && !matches!(latest_context, Value::Mapping(_)) {
                    return Some(latest_context);
                }
                None
            }
            (Some(_), None) | (None, None) => None,
        };
    }

    // if `path`'s `a` in `a.b.c.d` doesn't exist in latest context, search up the context stack.
    // If it doesn't exist anywhere, assume `"a.b"` is the property name and repeat. Once `a.b`
    // property is found, assume `c` in `c.d` is a Mapping, if nothing is found assume `"c.d"` is a
    // property name and search
    if !ctx.data_stack.is_empty() {
        let path_vec = path.split('.');
        let mut possible_path_list = Vec::default();
        let mut path_item_prefix = String::default();

        for path_item in path_vec {
            let new_path_item = if path_item_prefix.is_empty() {
                path_item.to_string()
            } else {
                format!("{}.{}", path_item_prefix, path_item)
            };
            possible_path_list.push(new_path_item.clone());
            path_item_prefix = new_path_item;
        }

        for possible_path in &possible_path_list {
            for context in ctx.data_stack.iter().rev() {
                let value_option = context.get(possible_path);

                if let Some(value) = value_option {
                    if let Some(target_property_name) =
                        &path.strip_prefix(&format!("{}.", possible_path))
                    {
                        let result = get_value(value, target_property_name);

                        if result.is_none() {
                            return value.get(target_property_name);
                        } else {
                            return result;
                        }
                    } else {
                        return get_value(context, path);
                    }
                }
            }
        }

        None
    } else {
        None
    }
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

    // Match if property `a` or `a.b.c.d` exists
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

fn render_sequence_of_sequences(
    name: String,
    ctx: &mut RenderCtx,
    items: &[SyntaxItem],
) -> Result<Option<String>, RibboncurlsError> {
    let mut value = String::new();
    if name == "." {
        if let Some(Value::Sequence(sequence)) = ctx.data_stack.last() {
            let sequence_clone = sequence.clone();
            for item in sequence_clone {
                let meh = item.clone();
                let name = serde_yaml_value_to_string(&item);
                ctx.section_path.push(name);
                ctx.data_stack.push(meh);
                if is_value_truthy(&item) {
                    let section_output = render_syntax_tree(items, ctx)?;

                    value.push_str(&section_output);
                };
                ctx.data_stack.pop();
                ctx.section_path.pop();
            }
        };
    };

    if value.is_empty() {
        Ok(None)
    } else {
        Ok(Some(value))
    }
}
