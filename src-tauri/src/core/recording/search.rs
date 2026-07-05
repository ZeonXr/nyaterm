enum HistoryMatcher {
    Literal {
        needle: String,
        case_sensitive: bool,
        whole_word: bool,
    },
    Regex(regex::Regex),
}

impl HistoryMatcher {
    fn new(query: &str, case_sensitive: bool, regex: bool, whole_word: bool) -> AppResult<Self> {
        if regex {
            let pattern = if whole_word {
                format!(r"\b(?:{query})\b")
            } else {
                query.to_string()
            };
            let compiled = RegexBuilder::new(&pattern)
                .case_insensitive(!case_sensitive)
                .build()
                .map_err(|error| {
                    AppError::Config(format!("Invalid regular expression: {error}"))
                })?;
            return Ok(Self::Regex(compiled));
        }

        let needle = if case_sensitive {
            query.to_string()
        } else {
            query.to_lowercase()
        };

        Ok(Self::Literal {
            needle,
            case_sensitive,
            whole_word,
        })
    }

    fn find(&self, haystack: &str) -> Option<(usize, usize)> {
        match self {
            Self::Literal {
                needle,
                case_sensitive,
                whole_word,
            } => {
                let searchable = if *case_sensitive {
                    haystack.to_string()
                } else {
                    haystack.to_lowercase()
                };
                find_literal_match(&searchable, needle, *whole_word)
            }
            Self::Regex(regex) => regex
                .find(haystack)
                .map(|found| (found.start(), found.end())),
        }
    }
}

fn find_literal_match(haystack: &str, needle: &str, whole_word: bool) -> Option<(usize, usize)> {
    if needle.is_empty() {
        return None;
    }

    let mut offset = 0;
    while offset <= haystack.len() {
        let relative = haystack[offset..].find(needle)?;
        let start = offset + relative;
        let end = start + needle.len();

        if !whole_word || is_word_boundary_match(haystack, start, end) {
            return Some((start, end));
        }

        offset = end;
    }

    None
}

fn is_word_boundary_match(text: &str, start: usize, end: usize) -> bool {
    let before = text[..start].chars().next_back();
    let after = text[end..].chars().next();

    before.is_none_or(|ch| !is_word_char(ch)) && after.is_none_or(|ch| !is_word_char(ch))
}

fn is_word_char(ch: char) -> bool {
    ch.is_alphanumeric() || ch == '_'
}

fn context_records(
    records: &[TranscriptRecord],
    index: usize,
    count: usize,
    before: bool,
) -> Vec<String> {
    if count == 0 {
        return Vec::new();
    }

    if before {
        let start = index.saturating_sub(count);
        return records[start..index]
            .iter()
            .map(|record| record.data.clone())
            .collect();
    }

    let start = index.saturating_add(1);
    let end = start.saturating_add(count).min(records.len());
    records[start..end]
        .iter()
        .map(|record| record.data.clone())
        .collect()
}

fn strip_terminal_control_sequences(text: &str) -> String {
    let bytes = text.as_bytes();
    let mut out = String::with_capacity(text.len());
    let mut i = 0;

    while i < bytes.len() {
        match bytes[i] {
            b'\x1b' => {
                i += 1;
                if i >= bytes.len() {
                    break;
                }
                match bytes[i] {
                    b'[' => {
                        i += 1;
                        while i < bytes.len() {
                            let b = bytes[i];
                            i += 1;
                            if (0x40..=0x7e).contains(&b) {
                                break;
                            }
                        }
                    }
                    b']' => {
                        i += 1;
                        while i < bytes.len() {
                            if bytes[i] == b'\x07' {
                                i += 1;
                                break;
                            }
                            if bytes[i] == b'\x1b' && i + 1 < bytes.len() && bytes[i + 1] == b'\\' {
                                i += 2;
                                break;
                            }
                            i += 1;
                        }
                    }
                    b'P' | b'X' | b'^' | b'_' => {
                        i += 1;
                        while i < bytes.len() {
                            if bytes[i] == b'\x1b' && i + 1 < bytes.len() && bytes[i + 1] == b'\\' {
                                i += 2;
                                break;
                            }
                            i += 1;
                        }
                    }
                    _ => {
                        advance_one_char(text, &mut i);
                    }
                }
            }
            b'\r' => {
                if i + 1 < bytes.len() && bytes[i + 1] == b'\n' {
                    out.push('\n');
                    i += 2;
                } else {
                    i += 1;
                }
            }
            b'\n' | b'\t' => {
                out.push(bytes[i] as char);
                i += 1;
            }
            b if b.is_ascii_control() => {
                i += 1;
            }
            b if b.is_ascii() => {
                out.push(b as char);
                i += 1;
            }
            _ => {
                if !text.is_char_boundary(i) {
                    i += 1;
                    continue;
                }
                let Some(ch) = text[i..].chars().next() else {
                    break;
                };
                out.push(ch);
                i += ch.len_utf8();
            }
        }
    }

    out
}

fn advance_one_char(text: &str, index: &mut usize) {
    if *index >= text.len() {
        return;
    }

    if !text.is_char_boundary(*index) {
        *index += 1;
        return;
    }

    if let Some(ch) = text[*index..].chars().next() {
        *index += ch.len_utf8();
    } else {
        *index = text.len();
    }
}
