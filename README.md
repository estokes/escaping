# Escaping

Escaping is a general purpose escaping library that is const compatible (for the
configuration). You can configure an escape character, a set of characters you
want to escape, and a set of translations you want to use for escaped characters
(for example to handle non printable characters). It provides,

- bi directional escape and unescape methods based on your configuration
- escaping aware split, splitn, and rsplit methods
- configurable translations of escaped characters to ascii sequences and back
- generic escaping and unescaping of arbitrary characters to and from \u{HHHH} format
- options to avoid allocation by supplying the target buffer

Say you are writing a compiler and you want to implement interpolation of
exressions surrounded by [] in string literals, as well as C like escapes, and
escaping of any remaining control characters to generic \u{HHHH} format.

```rust
use escaping::Escape;

fn use_generic_escape(c: char) -> bool {
    c.is_control()
}

const ESC: Escape<8, 4> = Escape::const_new(
    '\\',
    ['\\', '[', ']', '"', '\0', '\n', '\r', '\t'],
    [('\n', "n"), ('\r', "r"), ('\0', "0"), ('\t', "t")],
    Some(use_generic_escape),
);

fn main() {
    assert_eq!(ESC.escape("foo [e] bar\n"), r#"foo \[e\] bar\n"#);
    assert_eq!(ESC.unescape(r#"foo \[e\] bar\n"#), "foo [e] bar\n");
}
```
