use regex::Regex;

use super::RibboncurlsError;
use crate::token::Token;

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

pub fn create_syntax_tree(tokens: Vec<Token>) -> Result<Vec<SyntaxItem>, RibboncurlsError> {
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

fn push_item(syntax_tree: &mut Vec<SyntaxItem>, stack: &mut [SyntaxItem], item: SyntaxItem) {
    if let Some(SyntaxItem::Section { items, .. }) = stack.last_mut() {
        items.push(item);
    } else {
        syntax_tree.push(item);
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
