mod utils;

use regex::Regex;

use self::utils::{
    clean_up_partial_item_spaces_mut, clean_up_section_item_spaces_mut,
    set_standalone_to_syntax_items_mut, strip_newline_from_standalone_items,
};

use super::RibboncurlsError;
use crate::{
    token::Token,
    utils::{get_next_item, get_prev_item, get_regex_for_newline, NewlineRegex},
    SyntaxCtx,
};

#[derive(Clone, Debug)]
pub enum SyntaxItem {
    Text(String),
    Variable(String),
    EscapedVariable(String),
    Delimiter {
        is_standalone: bool,
    },
    Partial {
        name: String,
        is_standalone: bool,
    },
    Comment {
        text: String,
        is_standalone: bool,
    },
    Section {
        name: String,
        inverted: bool,
        items: Vec<SyntaxItem>,
        open_is_standalone: bool,
        closed_is_standalone: bool,
    },
}

pub fn create_syntax_tree(
    tokens: Vec<Token>,
    ctx: SyntaxCtx,
) -> Result<Vec<SyntaxItem>, RibboncurlsError> {
    let mut syntax_tree: Vec<SyntaxItem> = Vec::new();
    let mut stack: Vec<SyntaxItem> = vec![];
    let re_before_text = get_regex_for_newline(
        NewlineRegex::EndsWtihNewlineFollowedByWhitespace,
        ctx.newline,
    );
    let re_after_text = get_regex_for_newline(
        NewlineRegex::StartsWithNewlineFollowedByWhitespace,
        ctx.newline,
    );
    // If SyntaxItem::Text whitespace matches it must be index == 0 since all text should start
    // with newline str
    let re_whitespace = Regex::new(r"^[ \t]*\z").unwrap();

    for (index, token) in tokens.iter().enumerate() {
        match token {
            Token::Text(content) => {
                let lines: Vec<&str> = content.split(ctx.newline.as_str()).collect();
                if let Some((first_line, rest_of_lines)) = lines.split_first() {
                    // Emtpy if starts with newline
                    if !first_line.is_empty() {
                        push_item(
                            &mut syntax_tree,
                            &mut stack,
                            SyntaxItem::Text(first_line.to_string()),
                        );
                    }

                    for other_line in rest_of_lines {
                        let new_content = format!("{}{other_line}", ctx.newline.as_str());
                        push_item(
                            &mut syntax_tree,
                            &mut stack,
                            SyntaxItem::Text(new_content.clone()),
                        );
                    }
                }
            }
            Token::Variable(content) => push_item(
                &mut syntax_tree,
                &mut stack,
                SyntaxItem::Variable(content.to_string()),
            ),
            Token::EscapedVariable(content) => push_item(
                &mut syntax_tree,
                &mut stack,
                SyntaxItem::EscapedVariable(content.to_string()),
            ),
            Token::Partial(name) => push_item(
                &mut syntax_tree,
                &mut stack,
                SyntaxItem::Partial {
                    name: name.to_string(),
                    is_standalone: false,
                },
            ),
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
                        text: content.to_string(),
                        is_standalone: false,
                    },
                );
            }
            Token::OpenSection(name) => {
                // Set standalone if applicable
                let mut open_is_standalone = false;

                match (get_prev_item(&tokens, index), get_next_item(&tokens, index)) {
                    (None, Some(Token::Text(after_text))) => {
                        if re_after_text.is_match(after_text) {
                            open_is_standalone = true;
                        }
                    }
                    (Some(Token::Text(before_text)), None) => {
                        if re_before_text.is_match(before_text)
                            || re_whitespace.is_match(before_text)
                        {
                            open_is_standalone = true;
                        }
                    }
                    (Some(Token::Text(before_text)), Some(Token::Text(after_text))) => {
                        if (re_before_text.is_match(before_text)
                            || re_whitespace.is_match(before_text))
                            && re_after_text.is_match(after_text)
                        {
                            open_is_standalone = true;
                        }
                    }
                    _ => {}
                }

                stack.push(SyntaxItem::Section {
                    name: name.to_string(),
                    items: Vec::new(),
                    inverted: false,
                    open_is_standalone,
                    closed_is_standalone: false,
                });
            }
            Token::OpenInvertedSection(name) => {
                // Set standalone if applicable
                let mut open_is_standalone = false;

                match (get_prev_item(&tokens, index), get_next_item(&tokens, index)) {
                    (None, Some(Token::Text(after_text))) => {
                        if re_after_text.is_match(after_text) {
                            open_is_standalone = true;
                        }
                    }
                    (Some(Token::Text(before_text)), None) => {
                        if re_before_text.is_match(before_text)
                            || re_whitespace.is_match(before_text)
                        {
                            open_is_standalone = true;
                        }
                    }
                    (Some(Token::Text(before_text)), Some(Token::Text(after_text))) => {
                        if (re_before_text.is_match(before_text)
                            || re_whitespace.is_match(before_text))
                            && re_after_text.is_match(after_text)
                        {
                            open_is_standalone = true;
                        }
                    }
                    _ => {}
                }

                stack.push(SyntaxItem::Section {
                    name: name.to_string(),
                    items: Vec::new(),
                    inverted: true,
                    open_is_standalone,
                    closed_is_standalone: false,
                });
            }
            Token::CloseSection(_) => {
                if let Some(SyntaxItem::Section {
                    name,
                    inverted,
                    items,
                    open_is_standalone,
                    mut closed_is_standalone,
                }) = stack.pop()
                {
                    // Set standalone if applicable
                    match (get_prev_item(&tokens, index), get_next_item(&tokens, index)) {
                        (None, Some(Token::Text(after_text))) => {
                            if re_after_text.is_match(after_text) {
                                closed_is_standalone = true;
                            }
                        }
                        (Some(Token::Text(before_text)), None) => {
                            if re_before_text.is_match(before_text) {
                                closed_is_standalone = true;
                            }
                        }
                        (Some(Token::Text(before_text)), Some(Token::Text(after_text))) => {
                            if re_before_text.is_match(before_text)
                                && re_after_text.is_match(after_text)
                            {
                                closed_is_standalone = true;
                            }
                        }
                        _ => {}
                    }

                    push_item(
                        &mut syntax_tree,
                        &mut stack,
                        SyntaxItem::Section {
                            name,
                            inverted,
                            items,
                            open_is_standalone,
                            closed_is_standalone,
                        },
                    );
                }
            }
        }
    }

    set_standalone_to_syntax_items_mut(&mut syntax_tree, &ctx);
    clean_up_section_item_spaces_mut(&mut syntax_tree, &ctx);
    clean_up_partial_item_spaces_mut(&mut syntax_tree, &ctx);
    strip_newline_from_standalone_items(&mut syntax_tree, &ctx);

    Ok(syntax_tree)
}

fn push_item(syntax_tree: &mut Vec<SyntaxItem>, stack: &mut [SyntaxItem], item: SyntaxItem) {
    if let Some(SyntaxItem::Section { items, .. }) = stack.last_mut() {
        items.push(item);
    } else {
        syntax_tree.push(item);
    }
}
