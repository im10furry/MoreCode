pub fn estimate_text_tokens(text: &str) -> usize {
    if text.is_empty() {
        return 0;
    }

    let mut ascii_chars = 0usize;
    let mut cjk_chars = 0usize;
    let mut other_chars = 0usize;

    for ch in text.chars() {
        if ch.is_ascii() {
            ascii_chars += 1;
        } else if is_cjk_char(ch) {
            cjk_chars += 1;
        } else {
            other_chars += 1;
        }
    }

    let ascii_tokens = (ascii_chars as f64 / 4.0).ceil() as usize;
    let cjk_tokens = (cjk_chars as f64 / 1.5).ceil() as usize;
    let other_tokens = (other_chars as f64 / 3.0).ceil() as usize;

    ascii_tokens + cjk_tokens + other_tokens
}

fn is_cjk_char(ch: char) -> bool {
    let codepoint = ch as u32;

    (0x4E00..=0x9FFF).contains(&codepoint)
        || (0x3400..=0x4DBF).contains(&codepoint)
        || (0x20000..=0x2A6DF).contains(&codepoint)
        || (0xF900..=0xFAFF).contains(&codepoint)
        || (0x3040..=0x309F).contains(&codepoint)
        || (0x30A0..=0x30FF).contains(&codepoint)
        || (0xAC00..=0xD7AF).contains(&codepoint)
        || (0x2E80..=0x2EFF).contains(&codepoint)
        || (0x2F00..=0x2FDF).contains(&codepoint)
        || (0x3000..=0x303F).contains(&codepoint)
}

#[cfg(test)]
mod tests {
    use super::estimate_text_tokens;

    #[test]
    fn estimates_empty_text() {
        assert_eq!(estimate_text_tokens(""), 0);
    }

    #[test]
    fn estimates_ascii_text() {
        let tokens = estimate_text_tokens("Hello, world!");
        assert!((3..=5).contains(&tokens));
    }

    #[test]
    fn estimates_cjk_text() {
        let tokens = estimate_text_tokens("你好世界");
        assert!((2..=4).contains(&tokens));
    }

    #[test]
    fn estimates_mixed_text() {
        let tokens = estimate_text_tokens("Hello你好World世界");
        assert!(tokens > 0);
    }
}
