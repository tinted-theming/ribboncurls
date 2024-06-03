use regex::Regex;

use super::RibboncurlsError;
use crate::{
    token::Token,
    utils::{get_next_item, get_prev_item, get_regex_for_newline, NewlineRegex},
    SyntaxCtx,
};

#[derive(Debug)]
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
fn set_standalone_to_syntax_items_mut(syntax_tree: &mut [SyntaxItem], ctx: &SyntaxCtx) {
    let re_before = get_regex_for_newline(
        NewlineRegex::StartsWithNewlineFollowedByWhitespaceUntilEnd,
        ctx.newline,
    );
    let re_after = get_regex_for_newline(NewlineRegex::StartsWithNewline, ctx.newline);

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
            Some(SyntaxItem::Partial {
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

// When a Section open or close tag is on a newline, the tag itself should not take up space, so
// remove the starting and ending newlines and whitespaces accociated with that
//
// This strips away the content within the section since that's easier to deal with, even though
// technically the leading-space following a newline str, section tag and then following newline
// char is what constitutes a standalone tag. To deal with more easily within ribboncurls, a check
// is done to determine if the previous char matches `[[\r\n|\n][ \t]*\z`. If that's the case it
// strips away the internal newline and end newline instead of mutating the items around the
// section.
fn clean_up_section_item_spaces_mut(syntax_tree: &mut Vec<SyntaxItem>, ctx: &SyntaxCtx) {
    let re_before_text = get_regex_for_newline(
        NewlineRegex::StartsWithNewlineFollowedByWhitespaceUntilEnd,
        ctx.newline,
    );
    let re_before_text_last_syntax_item = Regex::new(r"[ \t]*\z").unwrap();
    let re_after_text = get_regex_for_newline(
        NewlineRegex::StartsWithNewlineFollowedByWhitespace,
        ctx.newline,
    );

    // If the first item is only white-space and the second item is standalone, remove the white
    // space
    if let Some(SyntaxItem::Section {
        open_is_standalone: true,
        ..
    }) = syntax_tree.get(1)
    {
        if let Some(SyntaxItem::Text(text)) = syntax_tree.first() {
            let re_before_text_white_space = Regex::new(r"[ \t]*\z").unwrap();
            if re_before_text_white_space.is_match(text) {
                syntax_tree.remove(0);
            }
        }
    }

    // Iterate with indices so we can access previous items
    let syntax_tree_len = syntax_tree.len();
    for i in 0..syntax_tree.len() {
        if let SyntaxItem::Section {
            name: _,
            inverted: _,
            items,
            open_is_standalone,
            closed_is_standalone,
        } = &mut syntax_tree[i]
        {
            clean_up_section_item_spaces_mut(items, ctx);

            // Strip the last SyntaxItem::Section.items item if it begins
            // with a newline and only contains spaces afterwards
            if *closed_is_standalone {
                if let Some(SyntaxItem::Text(text)) = items.last_mut() {
                    if i == syntax_tree_len - 1 {
                        if re_before_text_last_syntax_item.is_match(text) {
                            *text = re_before_text_last_syntax_item
                                .replace_all(text, "")
                                .to_string();
                        }
                    } else if re_before_text.is_match(text) {
                        *text = re_before_text.replace_all(text, "").to_string();
                    }
                }
            }

            if *open_is_standalone {
                match i {
                    // When the first SyntaxItem is a section, strip the leading newline and spaces
                    // within the SyntaxItem::Section.items
                    0 => {
                        if let Some(SyntaxItem::Text(text)) = items.first_mut() {
                            if re_after_text.is_match(text) {
                                *text = re_after_text.replace_all(text, "").to_string();
                            }
                        }
                    }
                    // Strip the previous SyntaxItem::Text newline and spaces
                    _ => {
                        if let SyntaxItem::Text(text) = &mut syntax_tree[i - 1] {
                            if re_after_text.is_match(text) {
                                *text = re_after_text.replace_all(text, "").to_string();
                            }
                        };
                    }
                };
            };
        }
    }
}

fn clean_up_partial_item_spaces_mut(syntax_tree: &mut [SyntaxItem], ctx: &SyntaxCtx) {
    let re_after_text = get_regex_for_newline(
        NewlineRegex::StartsWithNewlineFollowedByWhitespace,
        ctx.newline,
    );
    for i in 0..syntax_tree.len() {
        if let SyntaxItem::Partial {
            name: _,
            is_standalone,
        } = &mut syntax_tree[i]
        {
            if *is_standalone && i < syntax_tree.len() - 1 {
                // Strip the next SyntaxItem::Text newline and spaces
                if let SyntaxItem::Text(text) = &mut syntax_tree[i + 1] {
                    if re_after_text.is_match(text) {
                        *text = re_after_text.replace_all(text, "").to_string();
                    }
                };
            };
        }
    }
}
