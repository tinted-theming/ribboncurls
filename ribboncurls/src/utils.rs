pub fn escape_html(input: &str) -> String {
    let mut output = String::with_capacity(input.len() * 2);
    for c in input.chars() {
        match c {
            '&' => output.push_str("&amp;"),
            '<' => output.push_str("&lt;"),
            '>' => output.push_str("&gt;"),
            '"' => output.push_str("&quot;"),
            '\'' => output.push_str("&#x27;"),
            '/' => output.push_str("&#x2F;"),
            _ => output.push(c),
        }
    }

    // Not using shrink_to_fit() on purpose
    output
}
