// SPDX-License-Identifier: MIT OR Apache-2.0
//! Just enough of Rust's lexical grammar to find calls: identifiers, string literals and punctuation, each with
//! its line. Comments (line and block), char literals and the insides of raw strings produce no tokens.

#[derive(Debug, PartialEq, Clone)]
pub enum Tok {
    Ident(String),
    Str(String),
    Punct(char),
}

/// A token and the line it starts on.
pub type Spot = (Tok, usize);

pub fn tokens(text: &str) -> Vec<Spot> {
    let c: Vec<char> = text.chars().collect();
    let (mut i, mut line, mut out) = (0, 1, vec![]);
    while i < c.len() {
        let ch = c[i];
        if ch == '\n' {
            line += 1;
            i += 1;
        } else if ch.is_whitespace() {
            i += 1;
        } else if ch == '/' && c.get(i + 1) == Some(&'/') {
            while i < c.len() && c[i] != '\n' {
                i += 1;
            }
        } else if ch == '/' && c.get(i + 1) == Some(&'*') {
            i = skip_block_comment(&c, i, &mut line);
        } else if let Some((s, next)) = raw_string(&c, i) {
            line += s.matches('\n').count();
            out.push((Tok::Str(s), line));
            i = next;
        } else if ch == '"' {
            let (s, next) = string(&c, i + 1);
            let start = line;
            line += s.matches('\n').count();
            out.push((Tok::Str(s), start));
            i = next;
        } else if ch == '\'' {
            i = char_or_lifetime(&c, i);
        } else if ch.is_alphabetic() || ch == '_' {
            let start = i;
            while i < c.len() && (c[i].is_alphanumeric() || c[i] == '_') {
                i += 1;
            }
            out.push((Tok::Ident(c[start..i].iter().collect()), line));
        } else {
            out.push((Tok::Punct(ch), line));
            i += 1;
        }
    }
    out
}

fn skip_block_comment(c: &[char], mut i: usize, line: &mut usize) -> usize {
    let mut depth = 0;
    while i < c.len() {
        if c[i] == '/' && c.get(i + 1) == Some(&'*') {
            depth += 1;
            i += 2;
        } else if c[i] == '*' && c.get(i + 1) == Some(&'/') {
            depth -= 1;
            i += 2;
            if depth == 0 {
                break;
            }
        } else {
            if c[i] == '\n' {
                *line += 1;
            }
            i += 1;
        }
    }
    i
}

/// A `"` string from just after its opening quote: its text (escapes kept as written) and the index after it.
fn string(c: &[char], mut i: usize) -> (String, usize) {
    let mut s = String::new();
    while i < c.len() && c[i] != '"' {
        if c[i] == '\\' && i + 1 < c.len() {
            s.push(c[i]);
            i += 1;
        }
        s.push(c[i]);
        i += 1;
    }
    (s, i + 1)
}

/// `r"..."`, `r#"..."#`, `br#"..."#` at `i`: the text and the index after it.
fn raw_string(c: &[char], i: usize) -> Option<(String, usize)> {
    let mut j = i;
    if c.get(j) == Some(&'b') {
        j += 1;
    }
    if c.get(j) != Some(&'r') {
        return None;
    }
    j += 1;
    let hashes = c[j..].iter().take_while(|&&x| x == '#').count();
    j += hashes;
    if c.get(j) != Some(&'"') {
        return None;
    }
    j += 1;
    let close: Vec<char> = std::iter::once('"')
        .chain(std::iter::repeat_n('#', hashes))
        .collect();
    let start = j;
    while j + close.len() <= c.len() && c[j..j + close.len()] != close[..] {
        j += 1;
    }
    Some((c[start..j.min(c.len())].iter().collect(), j + close.len()))
}

/// A char literal (`'a'`, `'"'`, `'\n'`) or a lifetime (`'a`) at `i`: the index after it.
fn char_or_lifetime(c: &[char], i: usize) -> usize {
    if c.get(i + 1) == Some(&'\\') {
        let mut j = i + 2;
        while j < c.len() && c[j] != '\'' {
            j += 1;
        }
        return j + 1;
    }
    if c.get(i + 2) == Some(&'\'') {
        return i + 3;
    }
    i + 1
}
