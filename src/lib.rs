//! Escaping provides general round trippable string escaping. build an `Escape`
//! with either `new` or `const_new`
use anyhow::{bail, Result};
use compact_str::CompactString;
use std::borrow::Cow;

#[cfg(test)]
mod test;

#[derive(Debug, Clone)]
pub struct Escape {
    escape_char: char,
    escape: Box<[char]>,
    tr: Box<[(char, CompactString)]>,
    generic: Option<fn(char) -> bool>,
}

fn is_sep(esc: &mut bool, escape_char: char, c: char, sep: char) -> bool {
    if c == sep {
        !*esc
    } else {
        *esc = c == escape_char && !*esc;
        false
    }
}

impl Escape {
    /// return the escape char
    pub fn get_escape_char(&self) -> char {
        self.escape_char
    }

    /// return the set of escaped chars
    pub fn get_escaped(&self) -> &[char] {
        &self.escape
    }

    /// return the translations
    pub fn get_tr(&self) -> &[(char, CompactString)] {
        &self.tr
    }

    /// Create a new Escape, return an error if the folowing invariants are violated
    /// - the escape array must contain the escape_char.
    /// - the escape array must contain every first char in tr
    /// - the escape char, and the target tr char must be ascii
    /// - translation key may not be the escape char
    /// - translation targets must be ascii,
    /// - translation targets must be unique
    /// - translation targets may not be empty
    /// - translation targets may not start with u
    /// - translation targets may not contain the escape char
    ///
    /// `escape` is the list of characters that will be escaped when you call `escape`
    ///
    /// `tr` is the set of characters that are translated when escaped. For
    /// example the newline character might translate to \n. The original
    /// character is first followed by the escaped translation. e.g. [('\n',
    /// 'n')] for newline to \n translation.
    ///
    /// `generic`, if specified, will be called for each char, if it returns true,
    /// then the character will be translated to it's unicode escape sequence
    pub fn new(
        escape_char: char,
        escape: &[char],
        tr: &[(char, &str)],
        generic: Option<fn(char) -> bool>,
    ) -> Result<Self> {
        if !escape_char.is_ascii() {
            bail!("the escape char must be ascii")
        }
        if !escape.contains(&escape_char) {
            bail!("the escape slice must contain the escape character")
        }
        for (i, (c, s)) in tr.iter().enumerate() {
            if *c == escape_char {
                bail!("you cannot translate the escape char")
            }
            if s.len() == 0 {
                bail!("translation targets may not be empty")
            }
            if !s.is_ascii() {
                bail!("translation targets must be ascii")
            }
            if s.starts_with("u") {
                bail!("translation targets must not start with u")
            }
            if s.contains(escape_char) {
                bail!("translation targets may not contain the escape char")
            }
            if !escape.contains(&c) {
                bail!("the escape array must contain every translation key")
            }
            for (j, (c1, s1)) in tr.iter().enumerate() {
                if i != j {
                    if c == c1 {
                        bail!("duplicate translation key {c}")
                    }
                    if s == s1 {
                        bail!("duplicate translation target {s}")
                    }
                }
            }
        }
        Ok(Self {
            escape_char,
            escape: Box::from(escape),
            tr: Box::from_iter(tr.iter().map(|(c, s)| (*c, CompactString::new(s)))),
            generic,
        })
    }

    /// Escape the string and place the results into the buffer
    pub fn escape_to<T>(&self, s: &T, buf: &mut String)
    where
        T: AsRef<str> + ?Sized,
    {
        for c in s.as_ref().chars() {
            if self.escape.contains(&c) {
                buf.push(self.escape_char);
                match self
                    .tr
                    .iter()
                    .find_map(|(s, e)| if c == *s { Some(e) } else { None })
                {
                    Some(e) => buf.push_str(e),
                    None => buf.push(c),
                }
            } else if let Some(generic) = &self.generic
                && (generic)(c)
            {
                use std::fmt::Write;
                buf.push(self.escape_char);
                write!(buf, "u{{{:x}}}", c as u32).unwrap();
            } else {
                buf.push(c);
            }
        }
    }

    /// Escape the string, or return it unmodifed if it did not need
    /// to be escaped
    pub fn escape<'a, T>(&self, s: &'a T) -> Cow<'a, str>
    where
        T: AsRef<str> + ?Sized,
    {
        let s = s.as_ref();
        let mut to_escape = 0;
        for c in s.chars() {
            if self.escape.contains(&c)
                || self.generic.as_ref().map(|f| (f)(c)).unwrap_or(false)
            {
                to_escape += 1
            }
        }
        if to_escape == 0 {
            Cow::Borrowed(s.as_ref())
        } else {
            let mut out = String::with_capacity(s.len() + to_escape);
            self.escape_to(s, &mut out);
            Cow::Owned(out)
        }
    }

    /// Unescape the string and place the result in the buffer.
    pub fn unescape_to<T>(&self, s: &T, buf: &mut String)
    where
        T: AsRef<str> + ?Sized,
    {
        fn parse_unicode_escape_seq(s: &str) -> Option<(usize, char)> {
            if !s.starts_with("u{") {
                return None;
            }
            let i = s.find('}')?;
            let n = u32::from_str_radix(&s[2..i], 16).ok()?;
            let c = char::from_u32(n)?;
            Some((i + 1, c))
        }
        let mut escaped = false;
        let mut skip_to = 0;
        let s = s.as_ref();
        buf.extend(s.char_indices().filter_map(|(i, c)| {
            if i < skip_to {
                None
            } else if c == self.escape_char && !escaped {
                escaped = true;
                None
            } else if escaped {
                escaped = false;
                for (v, k) in &self.tr {
                    if s[i..].starts_with(k.as_str()) {
                        skip_to = i + k.len();
                        return Some(*v);
                    }
                }
                if let Some((j, c)) = parse_unicode_escape_seq(&s[i..]) {
                    skip_to = i + j;
                    return Some(c);
                }
                Some(c)
            } else {
                Some(c)
            }
        }))
    }

    /// Unescape the string, or return it unmodified if it did not need to be
    /// unescaped
    pub fn unescape<'a, T>(&self, s: &'a T) -> Cow<'a, str>
    where
        T: AsRef<str> + ?Sized,
    {
        let s = s.as_ref();
        if !s.contains(self.escape_char) {
            Cow::Borrowed(s.as_ref())
        } else {
            let mut res = String::with_capacity(s.len());
            self.unescape_to(s, &mut res);
            Cow::Owned(res)
        }
    }

    /// return true if the char at the `i` is escaped. Return true if `i` is
    /// not a valid char boundary
    pub fn is_escaped<T>(&self, s: &T, i: usize) -> bool
    where
        T: AsRef<str> + ?Sized,
    {
        let s = s.as_ref();
        let b = s.as_bytes();
        !s.is_char_boundary(i) || {
            let mut res = false;
            for j in (0..i).rev() {
                if s.is_char_boundary(j) && b[j] == (self.escape_char as u8) {
                    res = !res;
                } else {
                    break;
                }
            }
            res
        }
    }

    /// split the string into at most `n` parts separated by non escaped
    /// instances of `sep` and return an iterator over the parts
    pub fn splitn<'a, T>(
        &self,
        s: &'a T,
        n: usize,
        sep: char,
    ) -> impl Iterator<Item = &'a str> + use<'a, T>
    where
        T: AsRef<str> + ?Sized,
    {
        s.as_ref().splitn(n, {
            let escape_char = self.escape_char;
            let mut esc = false;
            move |c| is_sep(&mut esc, escape_char, c, sep)
        })
    }

    /// split the string into parts separated by non escaped instances of `sep`
    /// and return an iterator over the parts
    pub fn split<'a, T>(
        &self,
        s: &'a T,
        sep: char,
    ) -> impl Iterator<Item = &'a str> + use<'a, T>
    where
        T: AsRef<str> + ?Sized,
    {
        s.as_ref().split({
            let escape_char = self.escape_char;
            let mut esc = false;
            move |c| is_sep(&mut esc, escape_char, c, sep)
        })
    }
}
