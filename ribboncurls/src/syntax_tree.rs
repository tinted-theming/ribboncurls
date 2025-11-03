mod utils;

use regex::Regex;

use self::utils::cleanup_syntax_item_text_newline_and_spacing;

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
        indent: u8,
        is_standalone: bool,
        name: String,
    },
    Comment {
        is_standalone: bool,
    },
    Section {
        name: String,
        is_inverted: bool,
        items: Vec<SyntaxItem>,
        open_is_standalone: bool,
        closed_is_standalone: bool,
    },
}

#[allow(clippy::too_many_lines)]
pub fn create_syntax_tree(
    tokens: &[Token],
    ctx: &SyntaxCtx,
) -> Result<Vec<SyntaxItem>, RibboncurlsError> {
    let mut syntax_tree: Vec<SyntaxItem> = Vec::new();
    let mut section_stack: Vec<SyntaxItem> = vec![];
    let re_before_text = get_regex_for_newline(
        NewlineRegex::EndsWithNewlineFollowedByWhitespace,
        ctx.newline,
    );
    let re_after_text = get_regex_for_newline(
        NewlineRegex::StartsWithNewlineFollowedByWhitespace,
        ctx.newline,
    );
    // If SyntaxItem::Text whitespace matches it must be index == 0 since all text should start
    // with newline str
    let re_whitespace = Regex::new(r"^[ \t]*\z").expect("Unable to get regex");

    for (index, token) in tokens.iter().enumerate() {
        match token {
            Token::Text(content) => {
                let lines: Vec<&str> = content.split(ctx.newline.as_str()).collect();
                if let Some((first_line, rest_of_lines)) = lines.split_first() {
                    // Emtpy if starts with newline
                    if !first_line.is_empty() {
                        push_item(
                            &mut syntax_tree,
                            &mut section_stack,
                            SyntaxItem::Text((*first_line).to_string()),
                        );
                    }

                    for other_line in rest_of_lines {
                        let new_content = format!("{}{other_line}", ctx.newline.as_str());
                        push_item(
                            &mut syntax_tree,
                            &mut section_stack,
                            SyntaxItem::Text(new_content.clone()),
                        );
                    }
                }
            }
            Token::Variable(content) => push_item(
                &mut syntax_tree,
                &mut section_stack,
                SyntaxItem::Variable(content.clone()),
            ),
            Token::EscapedVariable(content) => push_item(
                &mut syntax_tree,
                &mut section_stack,
                SyntaxItem::EscapedVariable(content.clone()),
            ),
            Token::Partial(name) => {
                let is_standalone = get_is_standalone(tokens, index, ctx);
                let indent = get_indent(&syntax_tree, &section_stack, ctx);

                push_item(
                    &mut syntax_tree,
                    &mut section_stack,
                    SyntaxItem::Partial {
                        name: name.clone(),
                        is_standalone,
                        indent,
                    },
                );
            }
            Token::Delimiter => {
                let is_standalone = get_is_standalone(tokens, index, ctx);

                push_item(
                    &mut syntax_tree,
                    &mut section_stack,
                    SyntaxItem::Delimiter { is_standalone },
                );
            }
            Token::Comment => {
                let is_standalone = get_is_standalone(tokens, index, ctx);

                push_item(
                    &mut syntax_tree,
                    &mut section_stack,
                    SyntaxItem::Comment { is_standalone },
                );
            }
            Token::OpenSection(name) => {
                let section = create_syntax_tree_open_section(
                    tokens,
                    name.clone(),
                    index,
                    &re_after_text,
                    &re_before_text,
                    &re_whitespace,
                );

                section_stack.push(section);
            }
            Token::OpenInvertedSection(name) => {
                let mut open_is_standalone = false;

                match (get_prev_item(tokens, index), get_next_item(tokens, index)) {
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

                section_stack.push(SyntaxItem::Section {
                    name: name.clone(),
                    items: Vec::new(),
                    is_inverted: true,
                    open_is_standalone,
                    closed_is_standalone: false,
                });
            }
            Token::CloseSection(_) => {
                if let Some(SyntaxItem::Section {
                    name,
                    is_inverted,
                    items,
                    open_is_standalone,
                    closed_is_standalone: _,
                }) = section_stack.pop()
                {
                    let section = create_syntax_tree_close_section(
                        tokens,
                        items,
                        name,
                        index,
                        is_inverted,
                        open_is_standalone,
                        &re_after_text,
                        &re_before_text,
                    );
                    push_item(&mut syntax_tree, &mut section_stack, section);
                }
            }
        }
    }

    cleanup_syntax_item_text_newline_and_spacing(&mut syntax_tree, ctx)?;

    Ok(syntax_tree)
}

pub fn create_syntax_tree_open_section(
    tokens: &[Token],
    name: String,
    index: usize,
    re_after_text: &Regex,
    re_before_text: &Regex,
    re_whitespace: &Regex,
) -> SyntaxItem {
    // Set standalone if applicable
    let mut open_is_standalone = false;

    match (get_prev_item(tokens, index), get_next_item(tokens, index)) {
        (None, Some(Token::Text(after_text))) => {
            if re_after_text.is_match(after_text) {
                open_is_standalone = true;
            }
        }
        (Some(Token::Text(before_text)), None) => {
            if re_before_text.is_match(before_text) || re_whitespace.is_match(before_text) {
                open_is_standalone = true;
            }
        }
        (Some(Token::Text(before_text)), Some(Token::Text(after_text))) => {
            if (re_before_text.is_match(before_text) || re_whitespace.is_match(before_text))
                && re_after_text.is_match(after_text)
            {
                open_is_standalone = true;
            }
        }
        _ => {}
    }

    SyntaxItem::Section {
        name,
        items: Vec::new(),
        is_inverted: false,
        open_is_standalone,
        closed_is_standalone: false,
    }
}

#[allow(clippy::too_many_arguments)]
pub fn create_syntax_tree_close_section(
    tokens: &[Token],
    items: Vec<SyntaxItem>,
    name: String,
    index: usize,
    is_inverted: bool,
    open_is_standalone: bool,
    re_after_text: &Regex,
    re_before_text: &Regex,
) -> SyntaxItem {
    let mut closed_is_standalone = false;
    match (get_prev_item(tokens, index), get_next_item(tokens, index)) {
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
            if re_before_text.is_match(before_text) && re_after_text.is_match(after_text) {
                closed_is_standalone = true;
            }
        }
        _ => {}
    }

    SyntaxItem::Section {
        name,
        is_inverted,
        items,
        open_is_standalone,
        closed_is_standalone,
    }
}

fn get_indent(syntax_tree: &[SyntaxItem], section_stack: &[SyntaxItem], ctx: &SyntaxCtx) -> u8 {
    let re_starts_with_newline_followed_by_whitespace = get_regex_for_newline(
        NewlineRegex::StartsWithNewlineFollowedByWhitespace,
        ctx.newline,
    );
    let mut indent: u8 = 0;

    if let Some(SyntaxItem::Section { items, .. }) = section_stack.last() {
        let last_text_item = items.iter().rfind(|item| {
            if let SyntaxItem::Text(_) = item {
                return true;
            }
            false
        });

        if let Some(SyntaxItem::Text(text)) = last_text_item {
            let tmp_indent = re_starts_with_newline_followed_by_whitespace
                .find(text)
                .map_or(0, |m| m.as_str().len());

            if let Ok(tmp_indent_u8) = u8::try_from(tmp_indent) {
                if tmp_indent_u8 > 1 {
                    indent = tmp_indent_u8 - 1;
                }
            }
        }
    }

    // Find the last newline and determine indent
    syntax_tree.iter().rfind(|item| {
        if let SyntaxItem::Text(text) = item {
            if text.starts_with('\n') {
                let tmp_indent = re_starts_with_newline_followed_by_whitespace
                    .find(text)
                    .map_or(0, |m| m.as_str().len());

                if let Ok(tmp_indent_u8) = u8::try_from(tmp_indent) {
                    if tmp_indent_u8 > 1 {
                        indent = tmp_indent_u8 - 1;
                    }
                }

                return true;
            }
        }
        false
    });

    indent
}

fn push_item(
    syntax_tree: &mut Vec<SyntaxItem>,
    section_stack: &mut [SyntaxItem],
    item: SyntaxItem,
) {
    if let Some(SyntaxItem::Section { items, .. }) = section_stack.last_mut() {
        items.push(item);
    } else {
        syntax_tree.push(item);
    }
}

fn get_is_standalone(tokens: &[Token], index: usize, ctx: &SyntaxCtx) -> bool {
    let re_before_text = get_regex_for_newline(
        NewlineRegex::EndsWithNewlineFollowedByWhitespace,
        ctx.newline,
    );
    let re_after_text = get_regex_for_newline(
        NewlineRegex::StartsWithNewlineFollowedByWhitespace,
        ctx.newline,
    );
    let re_whitespace = Regex::new(r"^[ \t]*\z").expect("Unable to get regex");

    match (get_prev_item(tokens, index), get_next_item(tokens, index)) {
        (None, Some(Token::Text(after_text))) => {
            if re_after_text.is_match(after_text) {
                return true;
            }
        }
        (Some(Token::Text(before_text)), None) => {
            if re_before_text.is_match(before_text)
                || (index == 1 && ctx.is_root && re_whitespace.is_match(before_text))
            {
                return true;
            }
        }
        (Some(Token::Text(before_text)), Some(Token::Text(after_text))) => {
            if (re_before_text.is_match(before_text)
                || (index == 1 && ctx.is_root && re_whitespace.is_match(before_text)))
                && re_after_text.is_match(after_text)
            {
                return true;
            }
        }
        _ => {}
    }

    false
}
