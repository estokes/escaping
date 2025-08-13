use escaping::Escape;

const _ESC: Escape<8, 4> = Escape::const_new(
    '\\',
    ['\\', '[', ']', '"', '\0', '\r', '\t', 'x'],
    [('\n', "n"), ('\r', "r"), ('\0', "0"), ('\t', "t")],
    None,
);

fn main() {}
