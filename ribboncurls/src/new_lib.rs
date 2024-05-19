#![doc = include_str!("../README.md")]

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
    let syntax_tree = create_syntax_tree(tokens)?;
    println!("{:?}", syntax_tree);

    render_syntax_tree(&syntax_tree, &data)
}

fn render_syntax_tree(
    syntax_tree: &[SyntaxItem],
    data: &HashMap<String, String>,
) -> Result<String, RibboncurlsError> {
    let mut output = String::new();

    for (index, node) in syntax_tree.iter().enumerate() {
        match node {
            SyntaxItem::Text(content) => {
                output.push_str(content.as_str());
            }
            SyntaxItem::EscapedVariable(content) => {
                if let Some(value) = data.get(content.as_str()) {
                    output.push_str(&html_escape::encode_text(value));
                }
            }
            SyntaxItem::Variable(content) => {
                if let Some(value) = data.get(content.as_str()) {
                    output.push_str(value);
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
                        let re = Regex::new(r"\n[ \t]*\z").unwrap();

                        if re.is_match(&output) {
                            output = re.replace_all(&output, "").to_string();
                        }
                    }
                } else {
                    let re = Regex::new(r"[ \t]*\z").unwrap();

                    if re.is_match(&output) {
                        output = re.replace_all(&output, "").to_string();
                    }
                }
            }
            SyntaxItem::Section {
                name,
                items,
                inverted,
            } => match (data.get(name), inverted) {
                (Some(_), false) => {
                    let section_output = render_syntax_tree(items, data)?;

                    output.push_str(&section_output);
                }
                (None, true) => {
                    let section_output = render_syntax_tree(items, data)?;

                    output.push_str(&section_output);
                }
                (Some(_), true) => {}
                (None, false) => {}
            },
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
            Token::Delimiter(_) => {}
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

    set_standalone_syntax_items_mut(&mut syntax_tree);

    Ok(syntax_tree)
}

// Find standalone SyntaxItems and set standalone properties to true
// A standalone item is any non-SyntaxItem::Text item that is surrounded
// by new line chars
fn set_standalone_syntax_items_mut(syntax_tree: &mut [SyntaxItem]) {
    let before_re = Regex::new(r"^\n[ \t]*\z").unwrap();
    let after_re = Regex::new(r"^\n").unwrap();

    let empty_line_syntax_item_text_vec = syntax_tree
        .iter()
        .enumerate()
        .filter_map(|(index, item)| {
            if let SyntaxItem::Text(text) = item {
                if before_re.is_match(text.as_str()) {
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
                    if after_re.is_match(text) {
                        // TODO this should be a match and not just match comment syntax items
                        if let Some(SyntaxItem::Comment {
                            ref mut is_standalone,
                            ..
                        }) = syntax_tree.get_mut(index + 1)
                        {
                            *is_standalone = true;
                        }
                    }
                }
            }
            None => {
                // TODO this should be a match and not just match comment syntax items
                if let Some(SyntaxItem::Comment {
                    ref mut is_standalone,
                    ..
                }) = syntax_tree.get_mut(index + 1)
                {
                    *is_standalone = true;
                }
            }
        }
    }
}

fn tokenize(template: &str, ctx: &mut ParseContext) -> Result<Vec<Token>, RibboncurlsError> {
    let mut tokens = Vec::new();
    let mut i = 0;
    let single_char_left_delimiter = &ctx.left_delimiter[0..1];
    let single_char_right_delimiter = &ctx.right_delimiter[0..1];
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

                    tokens.push(Token::Variable(content.to_string()));

                    i = end + tripple_right_delimiter.len();
                } else {
                    break;
                }
            }

            _ if current_str.starts_with(&ctx.left_delimiter) => {
                // If there is a following end-delimiter
                if let Some(end) = current_str.find(&ctx.right_delimiter) {
                    let end = end + i; // index in `template`
                    let content = &template[i + ctx.left_delimiter.len()..end].trim();

                    if let Ok(token) = parse_tag(content, ctx) {
                        tokens.push(token);
                    }

                    i = end + DEFAULT_RIGHT_DELIMITER.len();
                } else {
                    break;
                }
            }

            _ => {
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
    }

    Ok(tokens)
}

fn parse_tag(content: &str, ctx: &mut ParseContext) -> Result<Token, RibboncurlsError> {
    match content.chars().next() {
        Some('#') => Ok(Token::OpenSection(content[1..].trim().to_string())),
        Some('/') => Ok(Token::CloseSection(content[1..].trim().to_string())),
        Some('^') => Ok(Token::OpenInvertedSection(content[1..].trim().to_string())),
        // '>' => Some(Token::Partial(content[1..].trim().to_string())),
        Some('!') => Ok(Token::Comment(content[1..].trim().to_string())),
        Some('=') => {
            let delimiters: Vec<&str> = content[1..content.len() - 1].split(' ').collect();

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
