#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineEnding {
    Lf,
    Crlf,
}

impl LineEnding {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Lf => "lf",
            Self::Crlf => "crlf",
        }
    }

    pub fn sequence(self) -> &'static str {
        match self {
            Self::Lf => "\n",
            Self::Crlf => "\r\n",
        }
    }

    pub fn system_default() -> Self {
        if cfg!(windows) {
            Self::Crlf
        } else {
            Self::Lf
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EolStats {
    pub is_binary: bool,
    pub lf_count: usize,
    pub crlf_count: usize,
    pub mixed: bool,
    pub has_final_newline: bool,
}

pub fn is_probably_binary(bytes: &[u8]) -> bool {
    if bytes.is_empty() {
        return false;
    }
    if bytes.iter().any(|b| *b == 0) {
        return true;
    }

    let mut non_printable = 0usize;
    for &b in bytes {
        if b == b'\n' || b == b'\r' || b == b'\t' {
            continue;
        }
        if b < 0x20 || b == 0x7f {
            non_printable += 1;
        }
    }

    non_printable * 10 >= bytes.len() * 3
}

pub fn detect_line_endings(bytes: &[u8]) -> EolStats {
    let is_binary = is_probably_binary(bytes);
    if is_binary {
        return EolStats {
            is_binary: true,
            lf_count: 0,
            crlf_count: 0,
            mixed: false,
            has_final_newline: false,
        };
    }

    let mut lf_count = 0usize;
    let mut crlf_count = 0usize;
    let mut i = 0usize;
    while i < bytes.len() {
        match bytes[i] {
            b'\r' => {
                if i + 1 < bytes.len() && bytes[i + 1] == b'\n' {
                    crlf_count += 1;
                    i += 2;
                    continue;
                }
                i += 1;
            }
            b'\n' => {
                lf_count += 1;
                i += 1;
            }
            _ => i += 1,
        }
    }

    let has_final_newline = bytes.ends_with(b"\n") || bytes.ends_with(b"\r");
    EolStats {
        is_binary: false,
        lf_count,
        crlf_count,
        mixed: lf_count > 0 && crlf_count > 0,
        has_final_newline,
    }
}

pub fn normalize_line_endings(input: &str, target: LineEnding) -> String {
    let had_final_newline = input.ends_with('\n') || input.ends_with('\r');

    let mut canonical = input.replace("\r\n", "\n");
    if canonical.contains('\r') {
        canonical = canonical.replace('\r', "\n");
    }

    let body = if had_final_newline && canonical.ends_with('\n') {
        &canonical[..canonical.len().saturating_sub(1)]
    } else {
        canonical.as_str()
    };

    let mut out = if body.is_empty() {
        String::new()
    } else {
        let eol = target.sequence();
        let mut it = body.split('\n');
        let first = it.next().unwrap_or("");
        let mut s = String::from(first);
        for part in it {
            s.push_str(eol);
            s.push_str(part);
        }
        s
    };

    if had_final_newline {
        out.push_str(target.sequence());
    }
    out
}

pub fn canonicalize_newlines(input: &str) -> String {
    let mut s = input.replace("\r\n", "\n");
    if s.contains('\r') {
        s = s.replace('\r', "\n");
    }
    s
}

pub fn is_safe_newline_rewrite(before: &str, after: &str) -> bool {
    canonicalize_newlines(before) == canonicalize_newlines(after)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_lf_crlf_and_mixed() {
        let lf = b"a\nb\n";
        let st = detect_line_endings(lf);
        assert!(!st.is_binary);
        assert_eq!(st.lf_count, 2);
        assert_eq!(st.crlf_count, 0);
        assert!(!st.mixed);

        let crlf = b"a\r\nb\r\n";
        let st = detect_line_endings(crlf);
        assert_eq!(st.lf_count, 0);
        assert_eq!(st.crlf_count, 2);
        assert!(!st.mixed);

        let mixed = b"a\r\nb\nc\r\n";
        let st = detect_line_endings(mixed);
        assert_eq!(st.lf_count, 1);
        assert_eq!(st.crlf_count, 2);
        assert!(st.mixed);
    }

    #[test]
    fn normalizes_preserving_final_newline() {
        let input = "a\r\nb\n";
        let out = normalize_line_endings(input, LineEnding::Lf);
        assert_eq!(out, "a\nb\n");

        let out = normalize_line_endings("a\r\nb", LineEnding::Crlf);
        assert_eq!(out, "a\r\nb");

        let out = normalize_line_endings("a\n\n", LineEnding::Crlf);
        assert_eq!(out, "a\r\n\r\n");
    }

    #[test]
    fn safe_rewrite_only_changes_newlines() {
        let before = "a\r\nb\n";
        let after = "a\nb\n";
        assert!(is_safe_newline_rewrite(before, after));

        let after_bad = "a\nbb\n";
        assert!(!is_safe_newline_rewrite(before, after_bad));
    }
}
