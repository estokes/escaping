use crate::Escape;
use proptest::prelude::*;
use std::sync::LazyLock;

fn use_generic_escape(c: char) -> bool {
    c.is_control()
}

static ESC: LazyLock<Escape> = LazyLock::new(|| {
    Escape::new(
        '\\',
        &['\\', '[', ']', '"', '\0', '\n', '\r', '\t'],
        &[('\n', "n"), ('\r', "r"), ('\0', "0"), ('\t', "t")],
        Some(use_generic_escape),
    )
    .unwrap()
});

#[test]
fn basic_round_trip() {
    assert_eq!(ESC.escape("foo [e] bar\n"), r#"foo \[e\] bar\n"#);
    assert_eq!(ESC.unescape(r#"foo \[e\] bar\n"#), "foo [e] bar\n");
}

// Property-based test for round-trip with generated configurations
proptest! {
    #[test]
    fn prop_round_trip(
        escape_char in prop::char::range('\0', '\x7F'),
        escape in prop::array::uniform10(any::<char>()),
        tr_keys in prop::array::uniform5(any::<char>()),
        tr_values in prop::array::uniform5(proptest::string::string_regex("[a-tv-zA-TV-Z0-9][a-zA-Z0-9]{0,4}").unwrap()),
        use_generic_flag in any::<bool>(),
        input in any::<String>(),
    ) {
        let generic = if use_generic_flag { Some(use_generic_escape as fn(char) -> bool) } else { None };
        let tr: [(char, &str); 5] = std::array::from_fn(|i| (tr_keys[i], tr_values[i].as_str()));
        if let Ok(esc) = Escape::new(escape_char, &escape, &tr, generic) {
            let escaped = esc.escape(&input);
            let unescaped = esc.unescape(&escaped);
            assert_eq!(unescaped, input);
        }
    }
}

#[test]
fn test_new_success() {
    let _ = Escape::new(
        '\\',
        &['\\', '[', ']', '"', '\0', '\n', '\r', '\t'],
        &[('\n', "n"), ('\r', "r"), ('\0', "0"), ('\t', "t")],
        Some(use_generic_escape),
    )
    .unwrap();
}

#[test]
fn test_new_fail_missing_escape_char() {
    let res = Escape::new(
        '\\',
        &['[', ']', '"', '\0', '\n', '\r', '\t'],
        &[('\n', "n"), ('\r', "r"), ('\0', "0"), ('\t', "t")],
        None,
    );
    assert!(res.is_err());
}

#[test]
fn test_new_fail_duplicate_tr_key() {
    let res = Escape::new(
        '\\',
        &['\\', '[', ']', '"', '\0', '\n', '\r', '\t'],
        &[('\n', "n"), ('\r', "r"), ('\0', "0"), ('\n', "t")], // duplicate key '\n'
        None,
    );
    assert!(res.is_err());
}

#[test]
fn test_new_fail_non_ascii_escape_char() {
    let res = Escape::new(
        '☃',
        &['\\', '[', ']', '"', '\0', '\n', '\r', '\t'],
        &[('\n', "n"), ('\r', "r"), ('\0', "0"), ('\t', "t")],
        None,
    );
    assert!(res.is_err());
}

#[test]
fn test_new_fail_translate_escape_char() {
    let res = Escape::new(
        '\\',
        &['\\', '[', ']', '"', '\0', '\n', '\r', '\t'],
        &[('\\', "esc"), ('\r', "r"), ('\0', "0"), ('\t', "t")],
        None,
    );
    assert!(res.is_err());
}

#[test]
fn test_new_fail_empty_translation_target() {
    let res = Escape::new(
        '\\',
        &['\\', '[', ']', '"', '\0', '\n', '\r', '\t'],
        &[('\n', ""), ('\r', "r"), ('\0', "0"), ('\t', "t")],
        None,
    );
    assert!(res.is_err());
}

#[test]
fn test_new_fail_non_ascii_translation_target() {
    let res = Escape::new(
        '\\',
        &['\\', '[', ']', '"', '\0', '\n', '\r', '\t'],
        &[('\n', "nñ"), ('\r', "r"), ('\0', "0"), ('\t', "t")],
        None,
    );
    assert!(res.is_err());
}

#[test]
fn test_new_fail_translation_starts_with_u() {
    let res = Escape::new(
        '\\',
        &['\\', '[', ']', '"', '\0', '\n', '\r', '\t'],
        &[('\n', "uabc"), ('\r', "r"), ('\0', "0"), ('\t', "t")],
        None,
    );
    assert!(res.is_err());
}

#[test]
fn test_new_fail_translation_contains_escape() {
    let res = Escape::new(
        '\\',
        &['\\', '[', ']', '"', '\0', '\n', '\r', '\t'],
        &[('\n', "n\\"), ('\r', "r"), ('\0', "0"), ('\t', "t")],
        None,
    );
    assert!(res.is_err());
}

#[test]
fn test_new_fail_key_not_in_escape() {
    let res = Escape::new(
        '\\',
        &['\\', '[', ']', '"', '\0', '\r', '\t', 'x'],
        &[('\n', "n"), ('\r', "r"), ('\0', "0"), ('\t', "t")],
        None,
    );
    assert!(res.is_err());
}

#[test]
fn test_new_fail_duplicate_translation_target() {
    let res = Escape::new(
        '\\',
        &['\\', '[', ']', '"', '\0', '\n', '\r', '\t'],
        &[('\n', "n"), ('\r', "n"), ('\0', "0"), ('\t', "t")],
        None,
    );
    assert!(res.is_err());
}

#[test]
fn test_escape_to() {
    let mut buf = String::new();
    ESC.escape_to("foo [e] bar\n", &mut buf);
    assert_eq!(buf, r#"foo \[e\] bar\n"#);
}

#[test]
fn test_unescape_to() {
    let mut buf = String::new();
    ESC.unescape_to(r#"foo \[e\] bar\n"#, &mut buf);
    assert_eq!(buf, "foo [e] bar\n");
}

#[test]
fn test_escape_no_change() {
    assert_eq!(ESC.escape("foo bar"), "foo bar");
}

#[test]
fn test_unescape_no_change() {
    assert_eq!(ESC.unescape("foo bar"), "foo bar");
}

#[test]
fn test_generic_escape() {
    let input = "control\u{1}";
    let escaped = ESC.escape(&input).to_string();
    assert!(escaped.contains(r#"\u{1}"#));
    let unescaped = ESC.unescape(&escaped);
    assert_eq!(unescaped, input);
}

#[test]
fn test_is_escaped() {
    let s = r#"foo \[e\] bar\n"#;
    // byte indices for chars: assuming ASCII mostly
    // "foo \[e\] bar\n" length 15 chars? Let's count: f o o space \ [ e \ ] space b a r \ n
    // 15 chars
    assert!(ESC.is_escaped(s, 5)); // position of [
    assert!(ESC.is_escaped(s, 8)); // position of ]
    assert!(ESC.is_escaped(s, 14)); // position of n
    assert!(!ESC.is_escaped(s, 4)); // position of \
    assert!(!ESC.is_escaped(s, 0)); // f
    assert!(!ESC.is_escaped(s, 6)); // e
}

#[test]
fn test_split() {
    let s = "a\\,b,c\\,d";
    let parts: Vec<&str> = ESC.split(s, ',').collect();
    assert_eq!(parts, vec!["a\\,b", "c\\,d"]);
}

#[test]
fn test_splitn() {
    let s = "a\\,b,c\\,d";
    let parts: Vec<&str> = ESC.splitn(s, 3, ',').collect();
    assert_eq!(parts, vec!["a\\,b", "c\\,d"]);
}
