#[test]
fn test_output_unicode() {
    let template = r#"Url is: {{url}}!"#;
    let data = r#"{"url": "https://github.com"}"#;
    let result = ribboncurls::render(template, data, None).unwrap();

    assert_eq!(result, "Url is: https://github.com!");
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
        Escaped chars: &lt;script&gt;alert(&#39;xss&#39;);&lt;/script&gt;
    "#
    .trim();

    let result = ribboncurls::render(template, data, None).unwrap();

    assert_eq!(result, expected);
}

#[test]
fn comments() {
    let template = "<div>{{ ! some # comment }}{{text}}</div><p>An {{ fruit }}{{!-- A comment with symbols |}# --}} is a good fruit.</p>";
    let data = r#"
        text: Some content
        fruit: orange"#;
    let rendered = ribboncurls::render(template, data, None).unwrap();

    assert_eq!(
        &rendered,
        "<div>Some content</div><p>An orange is a good fruit.</p>"
    );
}

#[test]
fn escaped_and_unescaped_vars() {
    let data = r#"html: <html><a href="">'content'</a></html>"#;
    let template = "Escaped: This is some {{html}}\nUnescaped: This is some {{& html}}";
    let rendered = ribboncurls::render(template, data, None).unwrap();

    assert_eq!(
        rendered,
        r#"Escaped: This is some &lt;html&gt;&lt;a href=&quot;&quot;&gt;&#39;content&#39;&lt;/a&gt;&lt;/html&gt;
Unescaped: This is some <html><a href="">'content'</a></html>"#
    );
}

#[test]
fn renders_with_empty_data() {
    let template = "This is pretty silly, but why not?";
    let rendered = ribboncurls::render(template, "", None).unwrap();

    assert_eq!(rendered, "This is pretty silly, but why not?");
}

#[test]
fn sections_from_bool() {
    let template = "Hello!{{#secret}} This is a secret!{{/secret}}";

    let data_show = "secret: true";
    let data_hide = "secret: false";
    let template_show = ribboncurls::render(template, data_show, None).unwrap();
    let template_hide = ribboncurls::render(template, data_hide, None).unwrap();

    assert_eq!(template_show, "Hello! This is a secret!");
    assert_eq!(template_hide, "Hello!");
}

#[test]
fn inverse_sections() {
    let template = "Hello!{{^secret}} This is NOT a secret!{{/secret}}";
    let data_show = "secret: true";
    let data_hide = "secret: false";
    let template_show = ribboncurls::render(template, data_show, None).unwrap();
    let template_hide = ribboncurls::render(template, data_hide, None).unwrap();

    assert_eq!(template_show, "Hello!");
    assert_eq!(template_hide, "Hello! This is NOT a secret!");
}

#[test]
fn can_render_inverse_sections_for_empty_strs() {
    let template = "Hello {{name}}{{^name}}Anonymous{{/name}}!";
    let data_named = "name: Maciej";
    let data_unnamed = r#"name: """#;
    let template_named = ribboncurls::render(template, data_named, None).unwrap();
    let template_unnamed = ribboncurls::render(template, data_unnamed, None).unwrap();

    assert_eq!(template_named, "Hello Maciej!");
    assert_eq!(template_unnamed, "Hello Anonymous!");
}

#[test]
fn contextual_data_in_sequence() {
    let template = "{{name}}:{{#fruits}} {{name}}{{/fruits}}";
    let data = r#"
        name: Some fruits are
        fruits:
            - name: orange
            - name: banana
        "#;
    let template = ribboncurls::render(template, data, None).unwrap();

    assert_eq!(template, "Some fruits are: orange banana");
}

#[test]
fn contextual_data_from_parents() {
    let template = "{{#parent}}{{child.name}}'s parent is {{parent}}.{{/parent}}";
    let data = r#"
        parent: Theming
        child: 
          name: Tinted
    "#;
    let template = ribboncurls::render(template, data, None).unwrap();

    assert_eq!(template, "Tinted's parent is Theming.");
}

#[test]
fn data_with_nested_sections() {
    let template = "{{#a}}{{#b}}{{#c}}{{.}}{{/c}}{{/b}}{{/a}}";
    let data = r#"
        a: true
        b: 0
        c: Tinted Theming!
    "#;
    let template = ribboncurls::render(template, data, None).unwrap();

    assert_eq!(template, "Tinted Theming!");
}
