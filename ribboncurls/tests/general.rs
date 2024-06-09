#[test]
fn test_output_unicode() {
    let template = r#"Url is: {{{url}}}!"#;
    let data = r#"{"url": "https://github.com"}"#;
    let result = ribboncurls::render(template, data, None).unwrap();

    assert_eq!(result, "Url is: https://github.com!");
}

#[test]
fn test_output_escape() {
    let template = r#"Url is: {{url}}!"#;
    let data = r#"{"url": "https://github.com"}"#;
    let result = ribboncurls::render(template, data, None).unwrap();

    assert_eq!(result, "Url is: https:&#x2F;&#x2F;github.com!");
}

#[test]
fn test_complex_template() {
    let template = r#"
    Name: {{name}} {{surname}}
    Admin: {{#admin}}YES{{/admin}}{{^admin}}NO{{/admin}}
    Interests:
    {{#interests}}
        - {{name}} ({{description}})
    {{/interests}}
    Favorite Animals: 
        {{#favorite_animals}}
        - {{.}}
        {{/favorite_animals}}

    {{#contact}}
    Contact Info:
        Phone: {{phone}}
        Email: {{email}}
    {{/contact}}

    Bio: {{bio}}
    Raw bio: {{{bio}}}
    Special chars: {{{special_chars}}}
    Escaped chars: {{special_chars}}
"#
    .trim();

    let data = r#"
name: Tinted
surname: Theming
admin: true
favorite_animals:
  - Dog
  - Cat
  - Mouse
interests:
  - name: "Rust Programming"
    description: "Systems programming"
  - name: "Music"
    description: "Playing guitar"
contact:
  phone: "123-456-7890"
  email: "some@email.com"
bio: |
  Line1
  Line2
  Line3
special_chars: "<script>alert('xss');</script>"
"#;
    let expected = r#"
    Name: Tinted Theming
    Admin: YES
    Interests:
        - Rust Programming (Systems programming)
        - Music (Playing guitar)
    Favorite Animals: 
        - Dog
        - Cat
        - Mouse

    Contact Info:
        Phone: 123-456-7890
        Email: some@email.com

    Bio: Line1
Line2
Line3

    Raw bio: Line1
Line2
Line3

    Special chars: <script>alert('xss');</script>
    Escaped chars: &lt;script&gt;alert('xss');&lt;&#x2F;script&gt;
"#
    .trim();

    let result = ribboncurls::render(template, data, None).unwrap();

    assert_eq!(result, expected);
}
