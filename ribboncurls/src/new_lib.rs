#![doc = include_str!("../README.md")]

use serde_yaml::Value;
use regex::Regex;

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
    // Partial(String),
    Comment(String),
    Delimiter((String, String)),
}

#[derive(Debug)]
pub enum SyntaxItem {
    Text(String),
    Variable(String),
    EscapedVariable(String),
    Delimiter {
        is_standalone: bool,
    },
    // Partial(String),
    Comment {
        text: String,
        is_standalone: bool,
    },
    Section {
        name: String,
        inverted: bool,
        items: Vec<SyntaxItem>,
    },
}

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

pub fn rndr(
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
    println!();
    println!("tokens: {:?}", &tokens);
    let syntax_tree = create_syntax_tree(tokens)?;
    println!();
    println!("syntaxtree: {:?}", syntax_tree);

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

fn push_item(syntax_tree: &mut Vec<SyntaxItem>, stack: &mut [SyntaxItem], item: SyntaxItem) {
    if let Some(SyntaxItem::Section { items, .. }) = stack.last_mut() {
        items.push(item);
    } else {
        syntax_tree.push(item);
    }
}

fn create_syntax_tree(tokens: Vec<Token>) -> Result<Vec<SyntaxItem>, RibboncurlsError> {
    let mut syntax_tree: Vec<SyntaxItem> = Vec::new();
    let mut stack: Vec<SyntaxItem> = Vec::new();

    for token in tokens {
        match token {
            Token::Text(content) => {
                let lines: Vec<&str> = content.split('\n').collect();
                if let Some((first_line, rest_of_lines)) = lines.split_first() {
                    // Emtpy if the first char is \n
                    if !first_line.is_empty() {
                        push_item(
                            &mut syntax_tree,
                            &mut stack,
                            SyntaxItem::Text(first_line.to_string()),
                        );
                    }

                    for other_line in rest_of_lines {
                        let new_content = format!("\n{other_line}");
                        push_item(&mut syntax_tree, &mut stack, SyntaxItem::Text(new_content))
                    }
                }
            }
            Token::Variable(content) => {
                push_item(&mut syntax_tree, &mut stack, SyntaxItem::Variable(content))
            }
            Token::EscapedVariable(content) => push_item(
                &mut syntax_tree,
                &mut stack,
                SyntaxItem::EscapedVariable(content),
            ),
            // Token::Partial(content) => push_item(&mut syntax_tree, &mut stack, SyntaxItem::Partial(content)),
            Token::Delimiter(_) => {
                syntax_tree.push(SyntaxItem::Delimiter {
                    is_standalone: false,
                });
            }
            Token::Comment(content) => {
                push_item(
                    &mut syntax_tree,
                    &mut stack,
                    SyntaxItem::Comment {
                        text: content,
                        is_standalone: false,
                    },
                );
            }
            Token::OpenSection(name) => {
                stack.push(SyntaxItem::Section {
                    name,
                    items: Vec::new(),
                    inverted: false,
                });
            }
            Token::OpenInvertedSection(name) => {
                stack.push(SyntaxItem::Section {
                    name,
                    items: Vec::new(),
                    inverted: true,
                });
            }
            Token::CloseSection(_) => {
                if let Some(finished_section) = stack.pop() {
                    push_item(&mut syntax_tree, &mut stack, finished_section);
                }
            }
        }
    }

    set_standalone_to_syntax_items_mut(&mut syntax_tree);
    clean_up_syntax_item_spaces(&mut syntax_tree);

    Ok(syntax_tree)
}

fn clean_up_syntax_item_spaces(syntax_tree: &mut [SyntaxItem]) {
    for syntax_item in syntax_tree.iter_mut() {
        if let SyntaxItem::Section {
            name: _,
            inverted: _,
            items,
        } = syntax_item
        {
            let first_item = items.first_mut();

            if let Some(SyntaxItem::Text(text)) = first_item {
                if text.starts_with('\n') {
                    *text = text.trim_start_matches('\n').to_string()
                }
            }

            let last_item = items.last_mut();

            if let Some(SyntaxItem::Text(text)) = last_item {
                if text.starts_with('\n') {
                    *text = text.trim_start_matches('\n').to_string()
                }
            }
        }
    }
}

// Find standalone SyntaxItems and set standalone properties to true
// A standalone item is any non-SyntaxItem::Text item that is surrounded
// by new line chars
fn set_standalone_to_syntax_items_mut(syntax_tree: &mut [SyntaxItem]) {
    let re_before = Regex::new(r"^\n[ \t]*\z").unwrap();
    let re_after = Regex::new(r"^\n").unwrap();

    let empty_line_syntax_item_text_vec = syntax_tree
        .iter()
        .enumerate()
        .filter_map(|(index, item)| {
            if let SyntaxItem::Text(text) = item {
                if re_before.is_match(text.as_str()) {
                    Some(index)
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect::<Vec<usize>>();

    for index in empty_line_syntax_item_text_vec {
        match syntax_tree.get_mut(index + 2) {
            Some(syntax_item) => {
                if let SyntaxItem::Text(text) = syntax_item {
                    if re_after.is_match(text) {
                        set_standalone(syntax_tree, index + 1);
                    }
                }
            }
            None => {
                set_standalone(syntax_tree, index + 1);
            }
        }
    }

    if let Some(SyntaxItem::Text(text)) = syntax_tree.first() {
        let re = Regex::new(r"^[ \t]*\z").unwrap();
        if re.is_match(text) {
            set_standalone(syntax_tree, 1);
        }
    }

    fn set_standalone(syntax_tree: &mut [SyntaxItem], index: usize) {
        match syntax_tree.get_mut(index) {
            Some(SyntaxItem::Comment {
                ref mut is_standalone,
                ..
            }) => {
                *is_standalone = true;
            }
            Some(SyntaxItem::Delimiter {
                ref mut is_standalone,
                ..
            }) => {
                *is_standalone = true;
            }
            _ => {}
        }
    }
}

fn tokenize(template: &str, ctx: &mut ParseContext) -> Result<Vec<Token>, RibboncurlsError> {
    let mut tokens = Vec::new();
    let mut i = 0;

    while i < template.len() {
        let current_str = &template[i..];
        let single_char_left_delimiter = &ctx.left_delimiter[0..1];
        let left_delimiter_escape =
            format!("{}{}", &ctx.left_delimiter, single_char_left_delimiter);

        if current_str.starts_with(&left_delimiter_escape) {
            let single_char_right_delimiter = &ctx.right_delimiter[0..1];
            let right_delimiter_escape =
                format!("{}{}", &ctx.right_delimiter, single_char_right_delimiter);

            if let Some(end) = current_str.find(&right_delimiter_escape) {
                let end = end + i; // index in `template`
                let content = &template[i + left_delimiter_escape.len()..end].trim();

                tokens.push(Token::Variable(content.to_string()));

                i = end + right_delimiter_escape.len();
            } else {
                break;
            }
        } else if current_str.starts_with(&ctx.left_delimiter) {
            // If there is a following end-delimiter
            if let Some(end) = current_str[ctx.left_delimiter.len()..].find(&ctx.right_delimiter) {
                let end = end + i + ctx.left_delimiter.len(); // index in `template`
                let start_index = i + ctx.left_delimiter.len();
                let right_delimiter_len = ctx.right_delimiter.len();

                if start_index < template.len() && end < template.len() {
                    let content = &template[start_index..end];

                    if let Ok(token) = parse_tag(content, ctx) {
                        tokens.push(token);
                    }
                }

                i = end + right_delimiter_len;
            } else {
                break;
            }
        } else {
            // Find the start of the next tag or end of the template
            if let Some(next_tag_start) = current_str.find(&ctx.left_delimiter) {
                let text = &template[i..i + next_tag_start];
                if !text.is_empty() {
                    tokens.push(Token::Text(text.to_string()));
                }
                i += next_tag_start;
            // Otherwise add the remaining text
            } else {
                let text = &template[i..];
                tokens.push(Token::Text(text.to_string()));
                break;
            }
        }
    }

    Ok(tokens)
}

fn parse_tag(content: &str, ctx: &mut ParseContext) -> Result<Token, RibboncurlsError> {
    match content.chars().next() {
        Some('&') => Ok(Token::Variable(content[1..].trim().to_string())),
        Some('#') => Ok(Token::OpenSection(content[1..].trim().to_string())),
        Some('/') => Ok(Token::CloseSection(content[1..].trim().to_string())),
        Some('^') => Ok(Token::OpenInvertedSection(content[1..].trim().to_string())),
        // '>' => Some(Token::Partial(content[1..].trim().to_string())),
        Some('!') => Ok(Token::Comment(content[1..].trim().to_string())),
        Some('=') => {
            let delimiters: Vec<&str> = content[1..content.len() - 1].trim().split(' ').collect();

            match (delimiters.first(), delimiters.last()) {
                (Some(left_delimiter), Some(right_delimiter)) => {
                    ctx.left_delimiter = left_delimiter.to_string();
                    ctx.right_delimiter = right_delimiter.to_string();

                    Ok(Token::Delimiter((
                        left_delimiter.to_string(),
                        right_delimiter.to_string(),
                    )))
                }
                _ => Err(RibboncurlsError::MissingDelimiter),
            }
        }
        _ => Ok(Token::EscapedVariable(content.trim().to_string())),
    }
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
