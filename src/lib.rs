use serde_yaml::Value;

#[non_exhaustive]
#[derive(thiserror::Error, Debug)]
pub enum VMError {
    #[error("missing delimiter")]
    MissingDelimiter,
    #[error("bad tag")]
    BadTag,
    #[error("missing end tag")]
    MissingEndTag,
    #[error("blah")]
    NotMatching,
    #[error("bad input")]
    YamlParseError(#[from] serde_yaml::Error),
}

struct ParseContext {
    left_delimiter: String,
    right_delimiter: String,
    skipping: bool,
    partials: Value,
    context_stack: Vec<Value>,
    close_tag_stack: Vec<String>,
}
const DEFAULT_LEFT_DELIMITER: &str = "{{";
const DEFAULT_RIGHT_DELIMITER: &str = "}}";

#[derive(Debug)]
enum Tag {
    Comment,
    Interpolation(String),
    SectionStart(String),
    SectionEnd(String),
    InvertedSectionStart(String),
    DelimiterChange(String, String),
}

pub fn render(template: &str, data: &str, partials: Option<&str>) -> Result<String, VMError> {
    let mut ctx = ParseContext {
        left_delimiter: DEFAULT_LEFT_DELIMITER.to_string(),
        right_delimiter: DEFAULT_RIGHT_DELIMITER.to_string(),
        skipping: false,
        partials: serde_yaml::from_str(partials.unwrap_or("null"))?,
        context_stack: vec![serde_yaml::from_str(data)?],
        close_tag_stack: vec![],
    };
    let (_, output) = render_with_context(template, &mut ctx)?;
    Ok(output)
}

fn render_with_context<'a>(
    template: &'a str,
    ctx: &mut ParseContext,
) -> Result<(&'a str, String), VMError> {
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
                    Tag::DelimiterChange(left, right) => {
                        ctx.left_delimiter = left;
                        ctx.right_delimiter = right;
                    }
                    Tag::SectionStart(tag_name) => {
                        let end_tag =
                            format!("{}/{}{}", ctx.left_delimiter, tag_name, ctx.right_delimiter);
                        let value = lookup_value(tag_name.trim(), &ctx.context_stack);
                        let sequence = value_as_sequence(value);
                        ctx.close_tag_stack.push(end_tag);
                        let input_at_section_start = input;
                        if sequence.is_empty() {
                            let old_skipping = ctx.skipping;
                            ctx.skipping = true;
                            let (remaining_input, _) =
                                render_with_context(input_at_section_start, ctx)?;
                            input = remaining_input;
                            ctx.skipping = old_skipping;
                        }
                        for val in sequence {
                            ctx.context_stack.push(val);
                            let (remaining_input, section_output) =
                                render_with_context(input_at_section_start, ctx)?;
                            input = remaining_input;
                            output.push_str(&section_output);
                            ctx.context_stack.pop();
                        }
                        ctx.close_tag_stack.pop();
                    }
                    Tag::InvertedSectionStart(tag_name) => {
                        let end_tag =
                            format!("{}/{}{}", ctx.left_delimiter, tag_name, ctx.right_delimiter);
                        let value = lookup_value(tag_name.trim(), &ctx.context_stack);
                        let sequence = value_as_sequence(value);
                        ctx.close_tag_stack.push(end_tag);
                        let input_at_section_start = input;
                        if sequence.is_empty() {
                            let (remaining_input, section_output) =
                                render_with_context(input_at_section_start, ctx)?;
                            input = remaining_input;
                            output.push_str(&section_output);
                        } else {
                            let old_skipping = ctx.skipping;
                            ctx.skipping = true;
                            let (remaining_input, _) =
                                render_with_context(input_at_section_start, ctx)?;
                            input = remaining_input;
                            ctx.skipping = old_skipping;
                        }
                        ctx.close_tag_stack.pop();
                    }
                    Tag::SectionEnd(_tag_name) => return Ok((remaining_input, output)),
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
                Tag::DelimiterChange(left, right) => {
                    ctx.left_delimiter = left;
                    ctx.right_delimiter = right;
                }
                Tag::SectionStart(tag_name) => {
                    let end_tag =
                        format!("{}/{}{}", ctx.left_delimiter, tag_name, ctx.right_delimiter);
                    let value = lookup_value(tag_name.trim(), &ctx.context_stack);
                    let sequence = value_as_sequence(value);
                    ctx.close_tag_stack.push(end_tag);
                    let input_at_section_start = input;
                    if sequence.is_empty() {
                        let old_skipping = ctx.skipping;
                        ctx.skipping = true;
                        let (remaining_input, _) =
                            render_with_context(input_at_section_start, ctx)?;
                        input = remaining_input;
                        ctx.skipping = old_skipping;
                    }
                    for val in sequence {
                        ctx.context_stack.push(val);
                        let (remaining_input, section_output) =
                            render_with_context(input_at_section_start, ctx)?;
                        input = remaining_input;
                        output.push_str(&section_output);
                        ctx.context_stack.pop();
                    }
                    ctx.close_tag_stack.pop();
                }
                Tag::InvertedSectionStart(tag_name) => {
                    let end_tag =
                        format!("{}/{}{}", ctx.left_delimiter, tag_name, ctx.right_delimiter);
                    let value = lookup_value(tag_name.trim(), &ctx.context_stack);
                    let sequence = value_as_sequence(value);
                    ctx.close_tag_stack.push(end_tag);
                    let input_at_section_start = input;
                    if sequence.is_empty() {
                        let (remaining_input, section_output) =
                            render_with_context(input_at_section_start, ctx)?;
                        input = remaining_input;
                        output.push_str(&section_output);
                    } else {
                        let old_skipping = ctx.skipping;
                        ctx.skipping = true;
                        let (remaining_input, _) =
                            render_with_context(input_at_section_start, ctx)?;
                        input = remaining_input;
                        ctx.skipping = old_skipping;
                    }
                    ctx.close_tag_stack.pop();
                }
                Tag::SectionEnd(_tag_name) => return Ok((remaining_input, output)),
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
                    let (line, input_after_line) = input.split_at(newline_i + 1);
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
                let (line, input_after_line) = input.split_at(newline_i + 1);
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
    return Ok((input, output));
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
        let (remaining_input, tag) =
            parse_tag(&input_without_indent[ctx.left_delimiter.len()..], &ctx)?;
        if let Tag::Interpolation(_) = tag {
            return Err(VMError::NotMatching);
        }
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
                let value = &ctx
                    .partials
                    .get(tag_contents.trim())
                    .unwrap_or(&Value::Null);
                let value_as_string = value_to_string(value);
                let mut child_ctx = ParseContext {
                    left_delimiter: DEFAULT_LEFT_DELIMITER.to_string(),
                    right_delimiter: DEFAULT_RIGHT_DELIMITER.to_string(),
                    partials: ctx.partials.clone(),
                    skipping: ctx.skipping,
                    context_stack: ctx.context_stack.clone(),
                    close_tag_stack: ctx.close_tag_stack.clone(),
                };
                if ctx.skipping {
                    Ok((remaining_input, Tag::Interpolation("".to_string())))
                } else {
                    let (_, output) = render_with_context(&value_as_string, &mut child_ctx)?;
                    Ok((remaining_input, Tag::Interpolation(output)))
                }
            }
            None => Err(VMError::MissingDelimiter),
        };
    }
    if input.starts_with('#') {
        return match input[1..].split_once(&ctx.right_delimiter) {
            Some((tag_contents, remaining_input)) => {
                Ok((remaining_input, Tag::SectionStart(tag_contents.to_string())))
            }
            None => Err(VMError::MissingDelimiter),
        };
    }
    if input.starts_with('^') {
        return match input[1..].split_once(&ctx.right_delimiter) {
            Some((tag_contents, remaining_input)) => Ok((
                remaining_input,
                Tag::InvertedSectionStart(tag_contents.to_string()),
            )),
            None => Err(VMError::MissingDelimiter),
        };
    }
    if input.starts_with('/') {
        return match input[1..].split_once(&ctx.right_delimiter) {
            Some((tag_contents, remaining_input)) => {
                Ok((remaining_input, Tag::SectionEnd(tag_contents.to_string())))
            }
            None => Err(VMError::MissingDelimiter),
        };
    }
    if input.starts_with('=') {
        let right_delimiter = format!("={}", ctx.right_delimiter);
        return match input[1..].split_once(&right_delimiter) {
            Some((tag_contents, remaining_input)) => {
                let (left, right) = tag_contents.trim().split_once(' ').ok_or(VMError::BadTag)?;
                Ok((
                    remaining_input,
                    Tag::DelimiterChange(left.trim().to_string(), right.trim().to_string()),
                ))
            }
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
            let value = lookup_value(tag_contents.trim(), &ctx.context_stack);
            let unescaped = value_to_string(value);
            let output = if escape {
                html_escape::encode_safe(&unescaped).to_string()
            } else {
                unescaped
            };
            Ok((remaining_input, Tag::Interpolation(output)))
        }
        None => Err(VMError::MissingDelimiter),
    }
}

fn lookup_value<'a>(path: &str, context_stack: &'a Vec<Value>) -> &'a Value {
    if path == "." {
        return &context_stack[&context_stack.len() - 1];
    }

    // Description from spec:
    //     1) Split the name on periods; the first part is the name to resolve, any
    //     remaining parts should be retained.
    //     2) Walk the context stack from top to bottom, finding the first context
    //     that is a) a hash containing the name as a key OR b) an object responding
    //     to a method with the given name.
    //     3) If the context is a hash, the data is the value associated with the
    //     name.
    //     4) If the context is an object, the data is the value returned by the
    //     method with the given name.
    //     5) If any name parts were retained in step 1, each should be resolved
    //     against a context stack containing only the result from the former
    //     resolution.  If any part fails resolution, the result should be considered
    //     falsey, and should interpolate as the empty string.

    let first_and_rest = path.split_once('.');
    let (first, rest_path) = match first_and_rest {
        None => (path, None),
        Some((f, r)) => (f, Some(r)),
    };
    for value in context_stack.iter().rev() {
        if let Some(res) = value.get(first) {
            return match rest_path {
                Some(rest) => {
                    let mut current = res;
                    for elem in rest.split('.') {
                        current = &current[elem]
                    }
                    current
                }
                None => res,
            };
        }
    }

    // Fallback, will be Value::Null
    &context_stack[0][first]
}

fn value_to_string(value: &Value) -> String {
    match value {
        Value::Number(n) => n.to_string(),
        Value::String(s) => s.to_owned(),
        _ => "".to_string(),
    }
}

fn value_as_sequence(value: &Value) -> Vec<Value> {
    match value {
        Value::Null => vec![],
        Value::Bool(b) => {
            if *b {
                vec![value.clone()]
            } else {
                vec![]
            }
        }
        Value::Number(_) => vec![value.clone()],
        Value::String(_) => vec![value.clone()],
        Value::Sequence(s) => s.clone(),
        Value::Mapping(_) => vec![value.clone()],
        Value::Tagged(_) => vec![value.clone()],
    }
}
