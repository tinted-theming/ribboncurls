#![doc = include_str!("../README.md")]

mod syntax_tree;
mod token;
mod utils;

use serde_yaml::Value;
use syntax_tree::{create_syntax_tree, SyntaxItem};
use token::tokenize;
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
    #[error("bad input")]
    YamlParseError(#[from] serde_yaml::Error),
}

struct TokenCtx {
    left_delimiter: String,
    right_delimiter: String,
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
                if let Some(value) = get_context_value(ctx, content.as_str()) {
                    output.push_str(&escape_html(&serde_yaml_value_to_string(value)));
                }
            }
            SyntaxItem::Variable(content) => {
                if let Some(value) = get_context_value(ctx, content.as_str()) {
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
                    };
                    let partial_tokens =
                        tokenize(partial_data.as_str().unwrap(), &mut token_ctx).expect("waaa");
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
            SyntaxItem::Comment {
                text: _,
                is_standalone: _,
            } => {}
            SyntaxItem::Delimiter { is_standalone: _ } => {}
            SyntaxItem::Section {
                name,
                items,
                is_inverted,
                open_is_standalone: _,
                closed_is_standalone: _,
            } => {
                ctx.section_path.push(name.to_string());

                let mut section_value_option = None;
                let mut is_mutating_context_stack = false;
                let mut iterator_option: Option<Value> = None;

                // Add section context to the ctx.data_stack
                if let Some(section_value) = get_context_value(ctx, &ctx.section_path.join(".")) {
                    section_value_option = Some(section_value.clone());
                    if matches!(section_value, Value::Mapping(_)) {
                        ctx.data_stack.push(section_value.clone());

                        is_mutating_context_stack = true;
                    } else if matches!(section_value, Value::Sequence(_)) {
                        iterator_option = Some(section_value.clone());
                    }
                }

                // Iterate and render over the sequence
                match (iterator_option, is_inverted) {
                    (Some(Value::Sequence(section_value)), false) => {
                        for item in section_value {
                            ctx.data_stack.push(item);

                            match (&section_value_option, is_inverted) {
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
                                    if is_value_falsy(value) && !matches!(value, Value::Mapping(_))
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
                    _ => match (section_value_option, is_inverted) {
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

fn get_context_value<'a>(ctx: &'a RenderCtx, path: &str) -> Option<&'a Value> {
    let context_stack = &ctx.data_stack;

    if path.is_empty() || ctx.data_stack.is_empty() {
        return None;
    }
    // Return context for "." variables
    if path == "." {
        return match (ctx.section_path.last(), context_stack.last()) {
            (Some(current_section), Some(context)) => {
                let value_option = context.get(current_section);
                if value_option.is_some() {
                    return value_option;
                }

                Some(context)
            }
            (None, Some(context)) => Some(context),
            _ => {
                None
                // throw
            }
        };
    }

    let parts: Vec<&str> = if path == "." {
        ctx.section_path.iter().map(|s| s.as_str()).collect()
    } else {
        path.split('.').collect::<Vec<&str>>()
    };

    // context_stack index at which the root path value begins
    let mut context_stack_start_index: usize = 0;

    // Does root path exist?
    for (index, context) in context_stack.iter().enumerate().rev() {
        if let Value::Mapping(_) = context {
            let value_option = get_value(context, &parts.join("."));

            match (parts.first(), value_option) {
                (Some(first_part), Some(_)) => {
                    if get_value(context, first_part).is_some() {
                        context_stack_start_index = index;
                        break;
                    }
                }
                (None, Some(_)) | (None, None) => {
                    // Throw
                    return None;
                }
                (Some(_), None) => {
                    continue;
                }
            }
        }
    }

    // Check for partial path matches on property names
    for context in context_stack[context_stack_start_index..].iter().rev() {
        if context.get(path).is_some() {
            return context.get(path);
        } else if let Some(value) = get_value(context, &parts.join(".")) {
            return Some(value);
        } else {
            // Search for values from the furthest section back to the root
            for index_outer in (0..parts.len()).rev() {
                if let Some(Value::Mapping(_)) = get_value(context, parts[index_outer]) {
                    return get_value(context, parts[index_outer]);
                } else {
                    for index_inner in (index_outer + 1..parts.len()).rev() {
                        if let Some(value) = context.get(&parts[index_inner..].join(".")) {
                            return Some(value);
                        }
                    }
                }
            }
        }
    }

    None
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
        _ => Some(&Value::Null),
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
