use regex::Regex;

use super::SyntaxItem;
use crate::{
    utils::{get_next_item, get_prev_item, get_regex_for_newline, NewlineRegex},
    SyntaxCtx,
};

pub fn cleanup_syntax_item_text_newline_and_spacing(
    syntax_tree: &mut Vec<SyntaxItem>,
    ctx: &SyntaxCtx,
) {
    let syntax_tree_clone = syntax_tree.clone();
    let mut syntax_text_items_to_remove: Vec<usize> = vec![];
    let mut syntax_item_newlines_to_remove: Vec<usize> = vec![];
    let mut syntax_items_remove_ending_whitespace: Vec<usize> = vec![];
    let re_before_text_last_syntax_item = Regex::new(r"[ \t]*\z").expect("Unable to get regex");
    let re_newline = get_regex_for_newline(NewlineRegex::StartsWithNewline, ctx.newline);
    let re_empty_line = Regex::new(r"^\s*\z").expect("Unable to get regex");
    let re_ending_whitespace = Regex::new(r"[ \t]*\z").expect("Unable to get regex");

    for (index, node) in syntax_tree.iter_mut().enumerate() {
        match node {
            SyntaxItem::Delimiter { is_standalone }
            | SyntaxItem::Comment {
                text: _,
                is_standalone,
            } => {
                if *is_standalone {
                    if let Some(SyntaxItem::Text(text)) = get_prev_item(&syntax_tree_clone, index) {
                        if re_ending_whitespace.is_match(text) {
                            syntax_items_remove_ending_whitespace.push(index - 1);
                        }
                    }

                    if let Some(SyntaxItem::Text(text)) = get_next_item(&syntax_tree_clone, index) {
                        if re_before_text_last_syntax_item.is_match(text) {
                            syntax_item_newlines_to_remove.push(index + 1);
                        }
                    }
                }
            }
            SyntaxItem::Partial {
                indent: _,
                is_standalone,
                name: _,
            } => {
                if *is_standalone {
                    if let Some(SyntaxItem::Text(text)) = get_prev_item(&syntax_tree_clone, index) {
                        if re_ending_whitespace.is_match(text) {
                            syntax_items_remove_ending_whitespace.push(index - 1);
                        }
                    }

                    if let Some(SyntaxItem::Text(text)) = get_next_item(&syntax_tree_clone, index) {
                        if re_before_text_last_syntax_item.is_match(text) {
                            syntax_item_newlines_to_remove.push(index + 1);
                        }
                    }
                }
            }

            SyntaxItem::Section {
                name: _,
                is_inverted: _,
                items,
                open_is_standalone,
                closed_is_standalone,
            } => {
                let section_ctx = SyntaxCtx {
                    newline: ctx.newline,
                    is_root: false,
                };
                cleanup_syntax_item_text_newline_and_spacing(items, &section_ctx);

                // Strip the last SyntaxItem::Section.items item if it begins
                // with a newline and only contains spaces afterwards
                if *closed_is_standalone {
                    if let Some(SyntaxItem::Text(text)) = get_next_item(&syntax_tree_clone, index) {
                        if re_before_text_last_syntax_item.is_match(text) {
                            syntax_item_newlines_to_remove.push(index + 1);
                        }
                    }

                    if let Some(SyntaxItem::Text(text)) = items.last_mut() {
                        if re_ending_whitespace.is_match(text) {
                            *text = re_ending_whitespace.replace_all(text, "").to_string();
                        }
                    }
                }

                // remove following newline and remove space before

                if *open_is_standalone {
                    // When the first SyntaxItem is a section, strip the leading newline and spaces
                    // within the SyntaxItem::Section.items
                    if ctx.is_root && index == 1 {
                        if let Some(SyntaxItem::Text(text)) =
                            get_prev_item(&syntax_tree_clone, index)
                        {
                            if re_empty_line.is_match(text) {
                                syntax_text_items_to_remove.push(index - 1);
                            }
                        };
                    } else if let Some(SyntaxItem::Text(text)) =
                        get_prev_item(&syntax_tree_clone, index)
                    {
                        if re_ending_whitespace.is_match(text) {
                            syntax_items_remove_ending_whitespace.push(index - 1);
                        }
                    };

                    if let Some(SyntaxItem::Text(text)) = items.first_mut() {
                        if re_newline.is_match(text) {
                            *text = re_newline.replace_all(text, "").to_string();
                        }
                    };
                };
            }
            _ => {}
        }
    }

    for index in syntax_item_newlines_to_remove {
        if let Some(SyntaxItem::Text(text)) = syntax_tree.get_mut(index) {
            *text = re_newline.replace_all(text, "").to_string();
        }
    }

    for index in syntax_items_remove_ending_whitespace {
        if let Some(SyntaxItem::Text(text)) = syntax_tree.get_mut(index) {
            *text = re_ending_whitespace.replace_all(text, "").to_string();
        };
    }

    syntax_text_items_to_remove.sort();
    for index in syntax_text_items_to_remove.iter().rev() {
        if syntax_tree.get(*index).is_some() {
            syntax_tree.remove(*index);
        }
    }
}
