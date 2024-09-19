use super::RibboncurlsError;
use super::TokenCtx;

#[derive(Clone, Debug)]
pub(crate) enum Token {
    Text(String),
    Variable(String),
    EscapedVariable(String),
    OpenSection(String),
    CloseSection(String),
    OpenInvertedSection(String),
    Partial(String),
    Comment,
    Delimiter,
}

pub(crate) fn tokenize(template: &str, ctx: &mut TokenCtx) -> Result<Vec<Token>, RibboncurlsError> {
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
                return Err(RibboncurlsError::MissingEndTag);
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
                        if let Token::OpenSection(_) | Token::OpenInvertedSection(_) = token {
                            ctx.section_stack.push(token.clone());
                        }
                        if let Token::CloseSection(ref close_section_name) = token {
                            if let Some(Token::OpenSection(open_section_name)) =
                                ctx.section_stack.last()
                            {
                                if close_section_name == open_section_name {
                                    ctx.section_stack.pop();
                                } else {
                                    return Err(RibboncurlsError::MissingEndTag);
                                }
                            }
                        }

                        tokens.push(token);
                    }
                }

                i = end + right_delimiter_len;
            } else {
                return Err(RibboncurlsError::MissingEndTag);
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

fn parse_tag(content: &str, ctx: &mut TokenCtx) -> Result<Token, RibboncurlsError> {
    match content.chars().next() {
        Some('&') => Ok(Token::Variable(content[1..].trim().to_string())),
        Some('#') => Ok(Token::OpenSection(content[1..].trim().to_string())),
        Some('/') => Ok(Token::CloseSection(content[1..].trim().to_string())),
        Some('^') => Ok(Token::OpenInvertedSection(content[1..].trim().to_string())),
        Some('>') => Ok(Token::Partial(content[1..].trim().to_string())),
        Some('!') => Ok(Token::Comment),
        Some('=') => {
            let delimiters: Vec<&str> = content[1..content.len() - 1].trim().split(' ').collect();

            match (delimiters.first(), delimiters.last()) {
                (Some(left_delimiter), Some(right_delimiter)) => {
                    ctx.left_delimiter = left_delimiter.to_string();
                    ctx.right_delimiter = right_delimiter.to_string();

                    Ok(Token::Delimiter)
                }
                _ => Err(RibboncurlsError::MissingDelimiter),
            }
        }
        _ => Ok(Token::EscapedVariable(content.trim().to_string())),
    }
}
