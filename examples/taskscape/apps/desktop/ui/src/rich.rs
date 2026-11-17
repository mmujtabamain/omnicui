//! Notes as the reference stores them: a small HTML subset (paragraphs, line breaks, bold, italic,
//! underline, strikethrough, links, bulleted and numbered lists, attachment mentions), read into plain
//! text with a style per character and a kind per paragraph, and written back the same way.

/// Style bits, one per character.
pub const BOLD: u8 = 1;
pub const ITALIC: u8 = 2;
pub const UNDERLINE: u8 = 4;
pub const STRIKE: u8 = 8;
/// Part of an attachment mention (`<span data-att="name">@name</span>`).
pub const MENTION: u8 = 16;

/// Paragraph kinds.
pub const PARA: u8 = 0;
pub const BULLET: u8 = 1;
pub const NUMBER: u8 = 2;

/// A note: its text (paragraphs joined by '\n'), a style per character, the link each character is in
/// (an index into `hrefs`, 0 for none), a kind per paragraph, and the '\n's that are line breaks.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Rich {
    pub text: String,
    pub styles: Vec<u8>,
    pub links: Vec<u16>,
    pub hrefs: Vec<String>,
    pub kinds: Vec<u8>,
    pub soft: Vec<usize>,
}

/// Whether a stored note is HTML (the reference's) rather than plain text.
pub fn is_html(s: &str) -> bool {
    let t = s.trim_start();
    t.starts_with('<') || s.contains("</") || s.contains("<br")
}

fn unescape(s: &str) -> String {
    s.replace("&nbsp;", "\u{a0}").replace("&lt;", "<").replace("&gt;", ">").replace("&quot;", "\"").replace("&#39;", "'").replace("&amp;", "&")
}

fn escape(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}

fn attr(tag: &str, name: &str) -> Option<String> {
    let at = tag.find(&format!("{name}=\""))? + name.len() + 2;
    let end = tag[at..].find('"')? + at;
    Some(unescape(&tag[at..end]))
}

/// Reads a stored note. Plain text stays as it is, with no styles.
pub fn parse(stored: &str) -> Rich {
    let mut r = Rich { kinds: vec![PARA], hrefs: vec![String::new()], ..Default::default() };
    if !is_html(stored) {
        r.text = stored.to_owned();
        let n = r.text.chars().count();
        r.styles = vec![0; n];
        r.links = vec![0; n];
        r.kinds = vec![PARA; r.text.matches('\n').count() + 1];
        return r;
    }
    let mut style: u8 = 0;
    let mut stack: Vec<(String, u8)> = vec![];
    let mut link: u16 = 0;
    let mut list: Vec<u8> = vec![];
    let mut started = false; // whether the current paragraph has begun (so a block tag ends it)
    let mut len = 0usize;
    let push_char = |r: &mut Rich, c: char, style: u8, link: u16, len: &mut usize| {
        r.text.push(c);
        r.styles.push(style);
        r.links.push(link);
        *len += 1;
    };
    let mut i = 0;
    let b = stored.as_bytes();
    while i < b.len() {
        if b[i] == b'<' {
            let Some(end) = stored[i..].find('>').map(|e| e + i) else { break };
            let tag = &stored[i + 1..end];
            i = end + 1;
            let closing = tag.starts_with('/');
            let name = tag.trim_start_matches('/').split(|c: char| c.is_whitespace() || c == '/').next().unwrap_or("").to_ascii_lowercase();
            let bit = match name.as_str() {
                "b" | "strong" => BOLD,
                "i" | "em" => ITALIC,
                "u" => UNDERLINE,
                "s" | "strike" | "del" => STRIKE,
                _ => 0,
            };
            match (name.as_str(), closing) {
                (_, false) if bit != 0 => {
                    stack.push((name.clone(), style));
                    style |= bit;
                }
                (_, true) if bit != 0 => {
                    if let Some(pos) = stack.iter().rposition(|(n, _)| *n == name) {
                        style = stack[pos].1;
                        stack.truncate(pos);
                    }
                }
                ("span", false) if tag.contains("data-att") => {
                    stack.push(("span".into(), style));
                    style |= MENTION;
                }
                ("span", false) => stack.push(("span".into(), style)),
                ("span", true) => {
                    if let Some(pos) = stack.iter().rposition(|(n, _)| n == "span") {
                        style = stack[pos].1;
                        stack.truncate(pos);
                    }
                }
                ("a", false) => {
                    r.hrefs.push(attr(tag, "href").unwrap_or_default());
                    link = (r.hrefs.len() - 1) as u16;
                }
                ("a", true) => link = 0,
                ("br", _) => {
                    r.soft.push(len);
                    push_char(&mut r, '\n', 0, 0, &mut len);
                }
                ("ul", false) => list.push(BULLET),
                ("ol", false) => list.push(NUMBER),
                ("ul" | "ol", true) => {
                    list.pop();
                }
                ("p" | "div" | "li", false) => {
                    if started {
                        push_char(&mut r, '\n', 0, 0, &mut len);
                        r.kinds.push(PARA);
                    }
                    started = true;
                    if name == "li" {
                        *r.kinds.last_mut().unwrap() = *list.last().unwrap_or(&BULLET);
                    }
                }
                _ => {}
            }
        } else {
            let next = stored[i..].find('<').map_or(b.len(), |n| n + i);
            let chunk = unescape(&stored[i..next]);
            for c in chunk.chars() {
                if c == '\n' || c == '\r' {
                    continue;
                }
                started = true;
                push_char(&mut r, c, style, link, &mut len);
            }
            i = next;
        }
    }
    r
}

/// Writes a note back as the reference stores it.
pub fn to_html(r: &Rich) -> String {
    let chars: Vec<char> = r.text.chars().collect();
    // Paragraphs: the char ranges between hard '\n's, each with its kind.
    let mut paras: Vec<(usize, usize)> = vec![];
    let mut start = 0;
    for (i, c) in chars.iter().enumerate() {
        if *c == '\n' && !r.soft.contains(&i) {
            paras.push((start, i));
            start = i + 1;
        }
    }
    paras.push((start, chars.len()));
    let mut out = String::new();
    let mut open_list: u8 = PARA;
    for (pi, (a, b)) in paras.iter().enumerate() {
        let kind = r.kinds.get(pi).copied().unwrap_or(PARA);
        if kind != open_list {
            match open_list {
                BULLET => out.push_str("</ul>"),
                NUMBER => out.push_str("</ol>"),
                _ => {}
            }
            match kind {
                BULLET => out.push_str("<ul>"),
                NUMBER => out.push_str("<ol>"),
                _ => {}
            }
            open_list = kind;
        }
        out.push_str(if kind == PARA { "<p>" } else { "<li>" });
        out.push_str(&inline(r, &chars, *a, *b));
        out.push_str(if kind == PARA { "</p>" } else { "</li>" });
    }
    match open_list {
        BULLET => out.push_str("</ul>"),
        NUMBER => out.push_str("</ol>"),
        _ => {}
    }
    out
}

/// One paragraph's characters as inline HTML: runs of one style and link, with line breaks.
fn inline(r: &Rich, chars: &[char], a: usize, b: usize) -> String {
    let mut out = String::new();
    let mut i = a;
    while i < b {
        if chars[i] == '\n' {
            out.push_str("<br>");
            i += 1;
            continue;
        }
        let (st, ln) = (r.styles.get(i).copied().unwrap_or(0), r.links.get(i).copied().unwrap_or(0));
        let mut j = i;
        while j < b && chars[j] != '\n' && r.styles.get(j).copied().unwrap_or(0) == st && r.links.get(j).copied().unwrap_or(0) == ln {
            j += 1;
        }
        let text: String = chars[i..j].iter().collect();
        let mut run = escape(&text);
        if st & MENTION != 0 {
            let name = text.trim_start_matches('@');
            run = format!("<span data-att=\"{}\">{run}</span>", escape(name).replace('"', "&quot;"));
        }
        for (bit, tag) in [(STRIKE, "s"), (UNDERLINE, "u"), (ITALIC, "i"), (BOLD, "b")] {
            if st & bit != 0 {
                run = format!("<{tag}>{run}</{tag}>");
            }
        }
        if ln != 0 {
            let href = r.hrefs.get(ln as usize).cloned().unwrap_or_default();
            run = format!("<a href=\"{}\">{run}</a>", escape(&href).replace('"', "&quot;"));
        }
        out.push_str(&run);
        i = j;
    }
    out
}

/// After an edit that kept `p` characters, removed `removed` and inserted `inserted` with the style
/// `typing`: the styles, links and paragraph kinds follow the text.
/// `soft_old` are the line breaks before the edit; `inserted_soft` says the inserted newline is a line break.
pub fn splice(r: &mut Rich, old: &[char], new: &[char], p: usize, removed: usize, inserted: usize, typing: u8, soft_old: &[usize], inserted_soft: bool) {
    let end = (p + removed).min(r.styles.len());
    let start = p.min(r.styles.len());
    r.styles.splice(start..end, std::iter::repeat_n(typing & !MENTION, inserted));
    let lend = (p + removed).min(r.links.len());
    let lstart = p.min(r.links.len());
    let link = if lstart > 0 { r.links.get(lstart - 1).copied().unwrap_or(0) } else { 0 };
    r.links.splice(lstart..lend, std::iter::repeat_n(link, inserted));
    // Paragraph kinds: the newlines removed and added at the edit, in the paragraph it is in.
    let hard = |i: usize, c: &char| *c == '\n' && !soft_old.contains(&i);
    let para = old[..p.min(old.len())].iter().enumerate().filter(|(i, c)| hard(*i, c)).count();
    let gone = old.get(p..p + removed).map_or(0, |s| s.iter().enumerate().filter(|(i, c)| hard(p + i, c)).count());
    let added = if inserted_soft { 0 } else { new.get(p..p + inserted).map_or(0, |s| s.iter().filter(|c| **c == '\n').count()) };
    let kind = r.kinds.get(para).copied().unwrap_or(PARA);
    let from = (para + 1).min(r.kinds.len());
    let to = (para + 1 + gone).min(r.kinds.len());
    r.kinds.splice(from..to, std::iter::repeat_n(kind, added));
}

/// Sets or clears a style bit on a char range: set when any character in it lacks it.
pub fn toggle(r: &mut Rich, a: usize, b: usize, bit: u8) -> bool {
    let b = b.min(r.styles.len());
    let set = (a..b).any(|i| r.styles[i] & bit == 0);
    for s in &mut r.styles[a..b] {
        if set {
            *s |= bit;
        } else {
            *s &= !bit;
        }
    }
    set
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_reference_note_round_trips() {
        let html = "<p>sdfads <b>asdfas</b> <i>sdfa</i></p>";
        let r = parse(html);
        assert_eq!(r.text, "sdfads asdfas sdfa");
        assert_eq!(r.styles[7], BOLD);
        assert_eq!(r.styles[14], ITALIC);
        assert_eq!(to_html(&r), html);
    }

    #[test]
    fn lists_links_breaks_and_mentions_survive() {
        let html = "<p>a<br>b</p><ul><li>one</li><li>two</li></ul><p><a href=\"https://x.y\">site</a> <span data-att=\"hero.png\">@hero.png</span></p>";
        let r = parse(html);
        assert_eq!(r.text, "a\nb\none\ntwo\nsite @hero.png");
        assert_eq!(r.kinds, vec![PARA, BULLET, BULLET, PARA]);
        assert_eq!(to_html(&r), html);
    }

    #[test]
    fn plain_text_stays_plain() {
        let r = parse("just text");
        assert_eq!(r.text, "just text");
        assert_eq!(to_html(&r), "<p>just text</p>");
    }

    #[test]
    fn typing_inside_bold_keeps_the_rest() {
        let mut r = parse("<p>ab <b>cd</b></p>");
        let old: Vec<char> = r.text.chars().collect();
        r.text = "ab Xcd".into();
        let new: Vec<char> = r.text.chars().collect();
        splice(&mut r, &old, &new, 3, 0, 1, BOLD, &[], false);
        assert_eq!(to_html(&r), "<p>ab <b>Xcd</b></p>");
    }
}

#[cfg(test)]
mod demo_note {
    #[test]
    fn the_hero_note_keeps_its_three_bullets() {
        let html = "<p>Lead with the <strong>one-line promise</strong>, then three proof points.</p><ul><li>Fast capture</li><li>Works offline</li><li>Keyboard first</li></ul>";
        let r = super::parse(html);
        assert_eq!(r.text, "Lead with the one-line promise, then three proof points.\nFast capture\nWorks offline\nKeyboard first");
        assert_eq!(super::to_html(&r), html.replace("strong>", "b>"));
    }
}
