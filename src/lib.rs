use serde_yaml::Value;

#[non_exhaustive]
#[derive(thiserror::Error, Debug)]
pub enum VMError {
    #[error("missing delimiter")]
    MissingDelimiter,
    #[error("bad tag")]
    BadTag,
    #[error("blah")]
    NotMatching,
    #[error("bad input")]
    YamlParseError(#[from] serde_yaml::Error)
}

struct ParseContext {
    left_delimiter: String,
    right_delimiter: String,
    data: Value,
    partials: Value,
    context_stack: Vec<Value>,
}
const DEFAULT_LEFT_DELIMITER: &str = "{{";
const DEFAULT_RIGHT_DELIMITER: &str = "}}";

enum Tag {
    Comment,
    Interpolation(String),
    SectionStart(String),
    SectionEnd(String),
    InvertedSectionStart(String),
    DelimiterChange(String, String)
}

pub fn render(template: &str, data: &str, partials: Option<&str>) -> Result<String, VMError> {
    let ctx = ParseContext {
        left_delimiter: DEFAULT_LEFT_DELIMITER.to_string(),
        right_delimiter: DEFAULT_RIGHT_DELIMITER.to_string(),
        data: serde_yaml::from_str(data)?,
        partials: serde_yaml::from_str(partials.unwrap_or("null"))?,
        context_stack: Vec::new()
    };
    render_with_context(template, &ctx)
}

fn render_with_context(template: &str, ctx: &ParseContext) -> Result<String, VMError> {
    let mut input = template;
    let mut output = String::new();
    let mut is_at_start_of_line = true;
    let mut i = 0;
    loop {
        i += 1;
        if i > 100 {
            break;
        }
        if input.is_empty() {
            break;
        }

        if is_at_start_of_line {
            if let Ok((remaining_input, tag)) = parse_standalone_tag_line(input, &ctx) {
                input = remaining_input;
                match tag {
                    Tag::Comment => {}
                    Tag::Interpolation(value) => {
                        output.push_str(&value);
                    }
                    _ => todo!()
                }
                continue;
            }
        }
        is_at_start_of_line = false;

        if let Some(after_delimiter) = input.strip_prefix(&ctx.left_delimiter) {
            let (remaining_input, tag) = parse_tag(after_delimiter, &ctx)?;
            input = remaining_input;
            match tag {
                Tag::Comment => {}
                Tag::Interpolation(value) => {
                    output.push_str(&value);
                }
                _ => todo!()
            }
            continue;
        }

        let first_delimiter = input.find(&ctx.left_delimiter);
        let first_newline = input.find('\n');
        match (first_delimiter, first_newline) {
            (Some(delimiter_i), Some(newline_i)) => {
                if delimiter_i < newline_i {
                    let (before_delimiter, remaining_input) = input.split_at(delimiter_i);
                    output.push_str(before_delimiter);
                    input = remaining_input;
                } else {
                    let (line, input_after_line) = input.split_at(newline_i+1);
                    output.push_str(line);
                    input = input_after_line;
                    is_at_start_of_line = true;
                }
            }
            (Some(delimiter_i), None) => {
                let (before_delimiter, remaining_input) = input.split_at(delimiter_i);
                output.push_str(before_delimiter);
                input = remaining_input;
            }
            (None, Some(newline_i)) => {
                let (line, input_after_line) = input.split_at(newline_i+1);
                output.push_str(line);
                input = input_after_line;
                is_at_start_of_line = true;
            }
            (None, None) => {
                output.push_str(input);
                break;
            }
        }
    }
    return Ok(output);
}

fn skip_whitespace(input: &str) -> &str {
    input.trim_start_matches(' ')
}

fn parse_standalone_tag_line<'a>(
    input: &'a str,
    ctx: &ParseContext,
) -> Result<(&'a str, Tag), VMError> {
    let input_without_indent = skip_whitespace(input);
    if input_without_indent.starts_with(&ctx.left_delimiter) {
        let (remaining_input, tag) = parse_tag(&input_without_indent[2..], &ctx)?;
        return if remaining_input.is_empty() {
            Ok((remaining_input, tag))
        } else if remaining_input.starts_with("\r\n") {
            Ok((&remaining_input[2..], tag))
        } else if remaining_input.starts_with('\n') {
            Ok((&remaining_input[1..], tag))
        } else {
            Err(VMError::NotMatching)
        };
    } else {
        return Err(VMError::NotMatching);
    }
}

fn parse_tag<'a>(input: &'a str, ctx: &ParseContext) -> Result<(&'a str, Tag), VMError> {
    if input.starts_with('!') {
        return match input[1..].split_once(&ctx.right_delimiter) {
            Some((_tag_contents, remaining_input)) => Ok((remaining_input, Tag::Comment)),
            None => Err(VMError::MissingDelimiter),
        };
    }
    if input.starts_with('>') {
        return match input[1..].split_once(&ctx.right_delimiter) {
            Some((tag_contents, remaining_input)) => {
                let value = lookup_value(tag_contents.trim(), &ctx.partials);
                let value_as_string = value_to_string(value);
                let child_ctx = ParseContext {
                    left_delimiter: DEFAULT_LEFT_DELIMITER.to_string(),
                    right_delimiter: DEFAULT_RIGHT_DELIMITER.to_string(),
                    data: ctx.data.clone(),
                    partials: ctx.partials.clone(),
                    context_stack: ctx.context_stack.clone(),
                };
                let output = render_with_context(&value_as_string, &child_ctx)?;
                Ok((remaining_input, Tag::Interpolation(output)))
            },
            None => Err(VMError::MissingDelimiter),
        };
    }
    if input.starts_with('#') {
        return match input[1..].split_once(&ctx.right_delimiter) {
            Some((tag_contents, remaining_input)) => {
                Ok((remaining_input, Tag::SectionStart(tag_contents.trim().to_string())))
            },
            None => Err(VMError::MissingDelimiter),
        };
    }
    if input.starts_with('^') {
        return match input[1..].split_once(&ctx.right_delimiter) {
            Some((tag_contents, remaining_input)) => {
                Ok((remaining_input, Tag::InvertedSectionStart(tag_contents.trim().to_string())))
            },
            None => Err(VMError::MissingDelimiter),
        };
    }
    if input.starts_with('/') {
        return match input[1..].split_once(&ctx.right_delimiter) {
            Some((tag_contents, remaining_input)) => {
                Ok((remaining_input, Tag::SectionEnd(tag_contents.trim().to_string())))
            },
            None => Err(VMError::MissingDelimiter),
        };
    }
    if input.starts_with('=') {
        let right_delimiter = format!("={}", ctx.right_delimiter);
        return match input[1..].split_once(&right_delimiter) {
            Some((tag_contents, remaining_input)) => {
                let (left, right) = tag_contents.trim().split_once(' ').ok_or(VMError::BadTag)?;
                Ok((remaining_input, Tag::DelimiterChange(left.trim().to_string(), right.trim().to_string())))
            },
            None => Err(VMError::MissingDelimiter),
        };
    }
    let mut escape = true;
    let mut right_delimiter = ctx.right_delimiter.clone();
    let mut input = input;
    if input.starts_with('{') {
        escape = false;
        right_delimiter = format!("}}{right_delimiter}");
        input = &input[1..];
    } else if input.starts_with('&') {
        escape = false;
        input = &input[1..];
    }
    match input.split_once(&right_delimiter) {
        Some((tag_contents, remaining_input)) => {
            let value = lookup_value(tag_contents.trim(), &ctx.data);
            let unescaped = value_to_string(value);
            let output = if escape {
                html_escape::encode_safe(&unescaped).to_string()
            } else {
                unescaped
            };
            Ok((remaining_input, Tag::Interpolation(output)))
        },
        None => Err(VMError::MissingDelimiter),
    }
}

fn lookup_value<'a>(path: &str, root: &'a Value) -> &'a Value {
    let path_elements = path.split('.');
    let mut current = root;
    for elem in path_elements {
        current = &current[elem]
    }
    return current;
}

fn value_to_string(value: &Value) -> String {
    match value {
        Value::Number(n) => {n.to_string()}
        Value::String(s) => {s.to_owned()}
        _ => "".to_string()
    }
}