use regex::Regex;

use super::SyntaxItem;
use crate::{
    utils::{get_regex_for_newline, NewlineRegex},
    SyntaxCtx,
};

/// Modifies the `is_standalone` property of non-Text `SyntaxItem` objects in the syntax tree
/// based on their surrounding text items. A standalone item is typically a non-Text item
/// that is surrounded by text containing only newline characters and optional whitespace.
///
/// # Parameters
/// - `syntax_tree`: A mutable slice of `SyntaxItem`, representing structured document components.
/// - `ctx`: Context containing configurations like newline characters.
///
/// # Examples
/// ```
/// let mut items = vec![
///     SyntaxItem::Text("\n    ".to_string()), // This should trigger the following Comment to be standalone
///     SyntaxItem::Comment { text: "This is a comment".to_string(), is_standalone: false },
///     SyntaxItem::Text("Normal text".to_string()),
/// ];
/// let ctx = SyntaxCtx { newline: "\n".to_string() };
/// set_standalone_to_syntax_items_mut(&mut items, &ctx);
///
/// assert_eq!(if let SyntaxItem::Comment { is_standalone, .. } = items[1] {
///     is_standalone
/// } else { false }, true);
/// ```
pub fn set_standalone_to_syntax_items_mut(syntax_tree: &mut [SyntaxItem], ctx: &SyntaxCtx) {
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

/// Cleans up whitespace associated with section open and close tags in the syntax tree,
/// ensuring that section tags on their own lines do not leave behind unwanted whitespace.
///
/// The function iterates through the syntax tree and modifies text elements based on
/// the properties of adjacent section items, particularly focusing on the handling of
/// standalone sections.
///
/// # Parameters
/// - `syntax_tree`: A mutable vector of `SyntaxItem` representing structured document components.
/// - `ctx`: Context containing configurations like the newline characters.
///
/// # Examples
/// ```
/// let mut items = vec![
///     SyntaxItem::Text("\n    ".to_string()),
///     SyntaxItem::Section {
///         name: "Intro".to_string(),
///         items: vec![SyntaxItem::Text("\nHello\n".to_string())],
///         inverted: false,
///         open_is_standalone: true,
///         closed_is_standalone: true,
///     },
///     SyntaxItem::Text("More text".to_string()),
/// ];
/// let ctx = SyntaxCtx { newline: "\n".to_string() };
/// clean_up_section_item_spaces_mut(&mut items, &ctx);
///
/// assert_eq!(if let SyntaxItem::Text(ref content) = items[0] { content } else { "" }, "");
/// assert_eq!(if let SyntaxItem::Section { items, .. } = &items[1] {
///     if let SyntaxItem::Text(ref content) = items[0] { content } else { "" }
/// } else { "" }, "Hello");
/// ```
pub fn clean_up_section_item_spaces_mut(syntax_tree: &mut Vec<SyntaxItem>, ctx: &SyntaxCtx) {
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

/// Cleans up spaces following partial items in a syntax tree if they are marked as standalone.
/// This function looks for standalone `Partial` items and, if followed by a `Text` item,
/// removes newline characters and whitespace that immediately follow the partial.
///
/// # Parameters
/// - `syntax_tree`: A mutable slice of `SyntaxItem`, representing parts of a parsed template or document.
/// - `ctx`: Context containing configurations like the newline characters.
///
/// # Examples
/// ```
/// let mut items = vec![
///     SyntaxItem::Partial { name: "header".to_string(), is_standalone: true },
///     SyntaxItem::Text("\n    Welcome to the site!".to_string()),
///     SyntaxItem::Partial { name: "footer".to_string(), is_standalone: false },
///     SyntaxItem::Text("\n    Goodbye!".to_string()),
/// ];
/// let ctx = SyntaxCtx { newline: "\n".to_string() };
/// clean_up_partial_item_spaces_mut(&mut items, &ctx);
///
/// assert_eq!(if let SyntaxItem::Text(ref content) = items[1] { content } else { "" }, "Welcome to the site!");
/// assert_eq!(if let SyntaxItem::Text(ref content) = items[3] { content } else { "" }, "\n    Goodbye!");
/// ```
pub fn clean_up_partial_item_spaces_mut(syntax_tree: &mut [SyntaxItem], ctx: &SyntaxCtx) {
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

/// Strips leading newlines from `Text` items in the syntax tree that are identified as standalone.
/// This function is specifically used when certain conditions are met based on the item's position
/// and the surrounding context.
///
/// # Parameters
/// - `syntax_tree`: A mutable slice of `SyntaxItem` references, allowing the function to modify the content of items.
/// - `ctx`: A reference to the `SyntaxCtx` structure providing context such as the newline string and root status.
///
/// # Behavior
/// The function iterates over the syntax tree. For each `Text` node at specific positions (index 1 or 2),
/// if the preceding item is a standalone `Delimiter` or `Comment`, and the `Text` content starts with a newline
/// specified in `ctx`, that newline is removed. This operation only occurs if the syntax tree's root flag
/// in the context (`ctx.is_root`) is true, ensuring that transformations apply only in specific scenarios.
///
/// This adjustment helps clean up text nodes that are formatted to appear as standalone due to preceding
/// formatting syntax but should not begin with a newline in the rendered output.
///
/// # Examples
/// ```
/// let mut items = vec![
///     SyntaxItem::Delimiter { is_standalone: true },
///     SyntaxItem::Text("\nHello".to_string()),
///     SyntaxItem::Text("\nWorld".to_string()),
/// ];
/// let ctx = SyntaxCtx {
///     newline: '\n'.to_string(),
///     is_root: true,
/// };
///
/// strip_newline_from_standalone_items(&mut items, &ctx);
///
/// assert_eq!(if let SyntaxItem::Text(ref content) = items[1] { content } else { "" }, "Hello");
/// assert_eq!(if let SyntaxItem::Text(ref content) = items[2] { content } else { "" }, "World");
/// ```
pub fn strip_newline_from_standalone_items(syntax_tree: &mut [SyntaxItem], ctx: &SyntaxCtx) {
    let syntax_tree_clone = syntax_tree.to_vec();

    for (index, node) in syntax_tree.iter_mut().enumerate() {
        if let SyntaxItem::Text(content) = node {
            if ctx.is_root && (index == 1 || index == 2) {
                match syntax_tree_clone.get(index - 1) {
                    Some(SyntaxItem::Delimiter { is_standalone })
                    | Some(SyntaxItem::Comment {
                        text: _,
                        is_standalone,
                    }) => {
                        if *is_standalone && content.starts_with(ctx.newline.as_str()) {
                            if let Some(updated_content) =
                                content.strip_prefix(ctx.newline.as_str())
                            {
                                *content = updated_content.to_string();
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}
