//! Scripture citation parsing for the `@book chapter:verse` inline syntax
//! in idea text (see `ui::idea_row`'s autocomplete and `ui::editor`'s
//! auto-tagging). Three independent uses of the same book dictionary:
//! - `citation_query_at_cursor`: while typing, find the in-progress `@word`
//!   at the cursor so the autocomplete popover can filter book names.
//! - `find_citations`: scan finished text for complete `@Book C:V[-V2]`
//!   spans, for auto-tagging (`editor.rs`) and inline display substitution
//!   (`ui::preaching_view`/`ui::preaching_print`).
//! - `sort_key`: canonical Bible book order, for the bibliography section —
//!   parses a *display*-form tag ("John 3:16"), not the `@`-prefixed input
//!   syntax.

/// (canonical name, lowercase alphanumeric aliases) in canonical Bible
/// order — order doubles as the bibliography sort key. Every alias is
/// matched case-insensitively with spaces/periods stripped, so "1 Cor.",
/// "1cor", and "ICOR" all resolve the same way; the canonical name itself
/// (also normalized) is always an implicit match and need not be repeated
/// in its own alias list.
const BOOKS: &[(&str, &[&str])] = &[
    ("Genesis", &["gen", "ge", "gn"]),
    ("Exodus", &["exo", "ex", "exod"]),
    ("Leviticus", &["lev", "le", "lv"]),
    ("Numbers", &["num", "nu", "nm", "nb"]),
    ("Deuteronomy", &["deut", "dt", "deu"]),
    ("Joshua", &["josh", "jos", "jsh"]),
    ("Judges", &["judg", "jdg", "jg", "jdgs"]),
    ("Ruth", &["rth", "ru"]),
    ("1 Samuel", &["1sam", "1sa", "1s", "isam"]),
    ("2 Samuel", &["2sam", "2sa", "2s", "iisam"]),
    ("1 Kings", &["1kgs", "1ki", "1k"]),
    ("2 Kings", &["2kgs", "2ki", "2k"]),
    ("1 Chronicles", &["1chr", "1ch"]),
    ("2 Chronicles", &["2chr", "2ch"]),
    ("Ezra", &["ezr", "ez"]),
    ("Nehemiah", &["neh", "ne"]),
    ("Esther", &["esth", "est", "es"]),
    ("Job", &["jb"]),
    ("Psalms", &["ps", "psa", "psalm", "pslm", "psalms"]),
    ("Proverbs", &["prov", "pro", "prv", "pr"]),
    ("Ecclesiastes", &["eccl", "ecc", "ec", "qoh"]),
    ("Song of Solomon", &["song", "sos", "sng", "canticles", "songofsongs"]),
    ("Isaiah", &["isa", "is"]),
    ("Jeremiah", &["jer", "je", "jr"]),
    ("Lamentations", &["lam", "la"]),
    ("Ezekiel", &["ezek", "eze", "ezk"]),
    ("Daniel", &["dan", "da", "dn"]),
    ("Hosea", &["hos", "ho"]),
    ("Joel", &["jl"]),
    ("Amos", &["am"]),
    ("Obadiah", &["obad", "ob"]),
    ("Jonah", &["jnh", "jon"]),
    ("Micah", &["mic", "mi"]),
    ("Nahum", &["nah", "na"]),
    ("Habakkuk", &["hab", "hb"]),
    ("Zephaniah", &["zeph", "zep", "zp"]),
    ("Haggai", &["hag", "hg"]),
    ("Zechariah", &["zech", "zec", "zc"]),
    ("Malachi", &["mal", "ml"]),
    ("Matthew", &["matt", "mt"]),
    ("Mark", &["mrk", "mk", "mr"]),
    ("Luke", &["luk", "lk"]),
    ("John", &["jn", "jhn", "joh"]),
    ("Acts", &["act", "ac"]),
    ("Romans", &["rom", "ro", "rm"]),
    ("1 Corinthians", &["1cor", "1co", "icor"]),
    ("2 Corinthians", &["2cor", "2co", "iicor"]),
    ("Galatians", &["gal", "ga"]),
    ("Ephesians", &["eph", "ephes"]),
    ("Philippians", &["phil", "php", "pp"]),
    ("Colossians", &["col", "co"]),
    ("1 Thessalonians", &["1thess", "1th", "ithess"]),
    ("2 Thessalonians", &["2thess", "2th", "iithess"]),
    ("1 Timothy", &["1tim", "1ti", "itim"]),
    ("2 Timothy", &["2tim", "2ti", "iitim"]),
    ("Titus", &["tit", "ti"]),
    ("Philemon", &["phlm", "phm", "pm"]),
    ("Hebrews", &["heb"]),
    ("James", &["jas", "jm"]),
    ("1 Peter", &["1pet", "1pe", "1pt", "ipet"]),
    ("2 Peter", &["2pet", "2pe", "2pt", "iipet"]),
    ("1 John", &["1john", "1jn", "1jo", "ijohn"]),
    ("2 John", &["2john", "2jn", "2jo", "iijohn"]),
    ("3 John", &["3john", "3jn", "3jo", "iiijohn"]),
    ("Jude", &["jud"]),
    ("Revelation", &["rev", "re", "revelations"]),
];

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Citation {
    pub book: String,
    pub chapter: u32,
    pub verse_start: u32,
    pub verse_end: Option<u32>,
}

impl Citation {
    pub fn display(&self) -> String {
        match self.verse_end {
            Some(end) if end != self.verse_start => {
                format!("{} {}:{}-{}", self.book, self.chapter, self.verse_start, end)
            }
            _ => format!("{} {}:{}", self.book, self.chapter, self.verse_start),
        }
    }
}

fn normalize(s: &str) -> String {
    s.chars()
        .filter(|c| c.is_alphanumeric())
        .flat_map(|c| c.to_lowercase())
        .collect()
}

/// Matches the longest known book alias at the start of `normalized`
/// (already-lowercased, alphanumeric-only text with no `@`), returning the
/// canonical name and how many *characters of the original slice* the match
/// consumed. Longest-match-first so "1 John" isn't shadowed by "1" + "John"
/// ambiguity, and so aliases that are prefixes of others (unlikely here,
/// but cheap to guard against) resolve to the more specific one.
fn match_book(text: &str) -> Option<(&'static str, usize)> {
    let mut best: Option<(&'static str, usize)> = None;
    for (name, aliases) in BOOKS {
        let norm_name = normalize(name);
        let candidates = std::iter::once(norm_name.as_str()).chain(aliases.iter().copied());
        for alias in candidates {
            if let Some(rest) = text_starts_with_normalized(text, alias) {
                let consumed = text.len() - rest.len();
                if best.is_none_or(|(_, len)| consumed > len) {
                    best = Some((name, consumed));
                }
            }
        }
    }
    best
}

/// If the alphanumeric-normalized prefix of `text` equals `alias`, returns
/// the remainder of `text` after that prefix (in `text`'s own original
/// indexing, not the normalized one) — lets callers recover a byte offset
/// into the un-normalized source without re-normalizing substrings.
fn text_starts_with_normalized<'a>(text: &'a str, alias: &str) -> Option<&'a str> {
    let mut ai = alias.chars().peekable();
    let mut byte = 0;
    for c in text.chars() {
        if ai.peek().is_none() {
            break;
        }
        byte += c.len_utf8();
        if !c.is_alphanumeric() {
            continue;
        }
        let want = ai.next().unwrap();
        if c.to_lowercase().next() != Some(want) {
            return None;
        }
    }
    if ai.peek().is_some() {
        return None;
    }
    Some(&text[byte..])
}

/// One complete `@Book C:V[-V2]` citation found in `text`, as a byte-range
/// span (covering the leading `@`) plus its parsed value.
pub struct Found {
    pub start: usize,
    pub end: usize,
    pub citation: Citation,
}

/// Scans `text` for complete citations. A book match not followed by a
/// valid `chapter:verse[-verse2]` is not a match at all (not even a partial
/// one) — this function only ever reports citations ready to tag/display,
/// never in-progress typing (see `citation_query_at_cursor` for that).
pub fn find_citations(text: &str) -> Vec<Found> {
    let mut out = Vec::new();
    let bytes = text.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] != b'@' {
            i += 1;
            continue;
        }
        let after_at = &text[i + 1..];
        if let Some((book, consumed)) = match_book(after_at) {
            let mut pos = i + 1 + consumed;
            // Optional single space between book and chapter.
            if text[pos..].starts_with(' ') {
                pos += 1;
            }
            if let Some((chapter, len)) = take_digits(&text[pos..]) {
                let after_chapter = pos + len;
                if text[after_chapter..].starts_with(':') {
                    let after_colon = after_chapter + 1;
                    if let Some((verse, vlen)) = take_digits(&text[after_colon..]) {
                        let mut end = after_colon + vlen;
                        let mut verse_end = None;
                        if text[end..].starts_with('-') {
                            if let Some((v2, v2len)) = take_digits(&text[end + 1..]) {
                                verse_end = Some(v2);
                                end = end + 1 + v2len;
                            }
                        }
                        out.push(Found {
                            start: i,
                            end,
                            citation: Citation {
                                book: book.to_string(),
                                chapter,
                                verse_start: verse,
                                verse_end,
                            },
                        });
                        i = end;
                        continue;
                    }
                }
            }
        }
        i += 1;
    }
    out
}

fn take_digits(s: &str) -> Option<(u32, usize)> {
    let digits: String = s.chars().take_while(|c| c.is_ascii_digit()).collect();
    if digits.is_empty() {
        return None;
    }
    let len = digits.len();
    digits.parse().ok().map(|n| (n, len))
}

/// Replaces every complete citation in `text` with its display form —
/// `render_display("Read @john3:16 aloud")` → `"Read John 3:16 aloud"` —
/// for Preaching View / Print, which show the reference, not the typed
/// shorthand.
pub fn render_display(text: &str) -> String {
    let found = find_citations(text);
    if found.is_empty() {
        return text.to_string();
    }
    let mut out = String::with_capacity(text.len());
    let mut last = 0;
    for f in &found {
        out.push_str(&text[last..f.start]);
        out.push_str(&f.citation.display());
        last = f.end;
    }
    out.push_str(&text[last..]);
    out
}

/// The `@`-prefixed, not-yet-complete token at `cursor` (a byte offset into
/// `text`), if any — the book-name portion only (stops as soon as a digit
/// appears, since at that point the user has moved on to typing
/// chapter:verse and autocomplete no longer applies). Returns the token's
/// byte span (including the leading `@`) and its text after `@`, for the
/// caller to filter book names against and to know what to replace on
/// selection.
pub fn citation_query_at_cursor(text: &str, cursor: usize) -> Option<(usize, usize, String)> {
    let cursor = cursor.min(text.len());
    let before = &text[..cursor];
    let at_pos = before.rfind('@')?;
    let query = &before[at_pos + 1..];
    if query.chars().any(|c| c.is_whitespace() || c.is_ascii_digit() || c == ':') {
        return None;
    }
    Some((at_pos, cursor, query.to_string()))
}

/// Book full names whose normalized form or any alias starts with the
/// normalized `query` — for the autocomplete popover's live filter. Empty
/// query matches nothing (no point offering all 66 books before the user
/// has typed anything to narrow it down).
pub fn search_books(query: &str) -> Vec<&'static str> {
    if query.is_empty() {
        return Vec::new();
    }
    let q = normalize(query);
    let mut out = Vec::new();
    for (name, aliases) in BOOKS {
        let norm_name = normalize(name);
        if norm_name.starts_with(&q) || aliases.iter().any(|a| a.starts_with(&q)) {
            out.push(*name);
        }
    }
    out
}

/// Canonical-order sort key for the bibliography: parses `tag` as a
/// *display*-form citation ("John 3:16", "Isaiah 55:1-13" — the form
/// already stored in `Sermon::s_tags`, not the `@`-prefixed input syntax),
/// falling back to `(usize::MAX, 0, 0, tag)` for anything that doesn't
/// parse (a manually-typed tag that isn't a citation at all) so those sort
/// alphabetically after every recognized reference.
pub fn sort_key(tag: &str) -> (usize, u32, u32, String) {
    for (book_idx, (name, _)) in BOOKS.iter().enumerate() {
        if let Some(rest) = tag.strip_prefix(name) {
            let rest = rest.trim_start();
            if let Some((chapter, len)) = take_digits(rest) {
                if let Some(after_colon) = rest[len..].strip_prefix(':') {
                    if let Some((verse, _)) = take_digits(after_colon) {
                        return (book_idx, chapter, verse, tag.to_string());
                    }
                }
            }
        }
    }
    (usize::MAX, 0, 0, tag.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_simple_citation() {
        let found = find_citations("Read @john3:16 aloud");
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].citation.display(), "John 3:16");
        assert_eq!(&"Read @john3:16 aloud"[found[0].start..found[0].end], "@john3:16");
    }

    #[test]
    fn parses_with_space_and_verse_range() {
        let found = find_citations("@1 Cor 13:4-7");
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].citation.display(), "1 Corinthians 13:4-7");
    }

    #[test]
    fn parses_numbered_book_without_space() {
        let found = find_citations("@1cor13:4");
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].citation.book, "1 Corinthians");
    }

    #[test]
    fn ignores_incomplete_citation() {
        assert!(find_citations("@john").is_empty());
        assert!(find_citations("@john3").is_empty());
        assert!(find_citations("@john3:").is_empty());
        assert!(find_citations("@notabook3:16").is_empty());
    }

    #[test]
    fn finds_multiple_citations() {
        let found = find_citations("@john3:16 and also @rom8:28");
        assert_eq!(found.len(), 2);
        assert_eq!(found[0].citation.display(), "John 3:16");
        assert_eq!(found[1].citation.display(), "Romans 8:28");
    }

    #[test]
    fn render_display_substitutes_in_place() {
        assert_eq!(render_display("See @john3:16 for it."), "See John 3:16 for it.");
        assert_eq!(render_display("no citation here"), "no citation here");
    }

    #[test]
    fn citation_query_finds_in_progress_book_name() {
        let text = "Read @jo";
        let cursor = text.len();
        let (start, end, query) = citation_query_at_cursor(text, cursor).unwrap();
        assert_eq!(&text[start..end], "@jo");
        assert_eq!(query, "jo");
    }

    #[test]
    fn citation_query_none_once_digit_typed() {
        assert!(citation_query_at_cursor("Read @john3", 11).is_none());
    }

    #[test]
    fn citation_query_none_without_at() {
        assert!(citation_query_at_cursor("no at sign here", 5).is_none());
    }

    #[test]
    fn search_books_matches_prefix_and_alias() {
        assert!(search_books("jo").contains(&"John"));
        assert!(search_books("jo").contains(&"Job"));
        assert!(search_books("1co").contains(&"1 Corinthians"));
        assert!(search_books("xyz").is_empty());
    }

    #[test]
    fn sort_key_orders_canonically() {
        let mut tags = vec!["Romans 8:28".to_string(), "Genesis 1:1".to_string(), "John 3:16".to_string()];
        tags.sort_by_key(|t| sort_key(t));
        assert_eq!(tags, vec!["Genesis 1:1", "John 3:16", "Romans 8:28"]);
    }

    #[test]
    fn sort_key_handles_verse_ranges_and_unrecognized_tags() {
        let mut tags = vec!["repentance".to_string(), "Isaiah 55:1-13".to_string()];
        tags.sort_by_key(|t| sort_key(t));
        assert_eq!(tags, vec!["Isaiah 55:1-13", "repentance"]);
    }
}
