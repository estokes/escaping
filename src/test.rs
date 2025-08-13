use crate::Escape;

fn use_generic_escape(c: char) -> bool {
    c.is_control()
}

const ESC: Escape<8, 4> = Escape::const_new(
    '\\',
    ['\\', '[', ']', '"', '\0', '\n', '\r', '\t'],
    [('\n', "n"), ('\r', "r"), ('\0', "0"), ('\t', "t")],
    Some(use_generic_escape),
);

#[test]
fn basic_round_trip() {
    assert_eq!(ESC.escape("foo [e] bar\n"), r#"foo \[e\] bar\n"#);
    assert_eq!(ESC.unescape(r#"foo \[e\] bar\n"#), "foo [e] bar\n");
}
