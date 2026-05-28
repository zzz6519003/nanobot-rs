/// Truncates a UTF-8 string to `max_bytes` without breaking char boundaries,
/// returning the truncated prefix as a string slice.
pub fn truncate_utf8_prefix(s: &str, max_bytes: usize) -> &str {
    if s.len() <= max_bytes {
        return s;
    }
    let mut boundary = max_bytes;
    while boundary > 0 && !s.is_char_boundary(boundary) {
        boundary -= 1;
    }
    &s[..boundary]
}

/// Truncates text to a maximum character count, appending an ellipsis (`…`)
/// when the text exceeds the limit.
pub fn truncate_text(text: &str, max_chars: usize) -> String {
    if text.chars().count() <= max_chars {
        return text.to_string();
    }
    let truncated: String = text.chars().take(max_chars).collect();
    format!("{}\u{2026}", truncated)
}

/// In-place UTF-8 safe truncation of a `String` to `max_len` bytes.
///
/// Returns the number of bytes that were removed, or `0` if no truncation occurred.
pub fn truncate_utf8_in_place(value: &mut String, max_len: usize) -> usize {
    if value.len() <= max_len {
        return 0;
    }

    let mut boundary = max_len.min(value.len());
    while boundary > 0 && !value.is_char_boundary(boundary) {
        boundary -= 1;
    }
    let remaining = value.len() - boundary;
    value.truncate(boundary);
    remaining
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncate_utf8_prefix_noop_when_within_limit() {
        let s = "hello";
        assert_eq!(truncate_utf8_prefix(s, 10), "hello");
    }

    #[test]
    fn truncate_utf8_prefix_respects_byte_limit() {
        let s = "hello world";
        assert_eq!(truncate_utf8_prefix(s, 5), "hello");
    }

    #[test]
    fn truncate_utf8_prefix_respects_char_boundaries() {
        let s = "你好世界";
        assert_eq!(truncate_utf8_prefix(s, 3), "你");
        assert_eq!(truncate_utf8_prefix(s, 6), "你好");
    }

    #[test]
    fn truncate_utf8_prefix_empty_string() {
        assert_eq!(truncate_utf8_prefix("", 10), "");
    }

    #[test]
    fn truncate_text_noop_when_within_limit() {
        assert_eq!(truncate_text("hello", 10), "hello");
    }

    #[test]
    fn truncate_text_adds_ellipsis() {
        let result = truncate_text("hello world", 5);
        assert!(result.starts_with("hello"));
        assert!(result.contains('\u{2026}'));
    }

    #[test]
    fn truncate_text_unicode_chars() {
        let result = truncate_text("你好世界", 2);
        assert!(result.starts_with("你好"));
        assert!(result.contains('\u{2026}'));
    }

    #[test]
    fn truncate_utf8_in_place_noop_when_within_limit() {
        let mut s = "hello".to_string();
        assert_eq!(truncate_utf8_in_place(&mut s, 10), 0);
        assert_eq!(s, "hello");
    }

    #[test]
    fn truncate_utf8_in_place_respects_char_boundaries() {
        let mut s = "你好世界".to_string();
        let remaining = truncate_utf8_in_place(&mut s, 6);
        assert_eq!(s, "你好");
        assert!(remaining > 0);
    }

    #[test]
    fn truncate_utf8_in_place_returns_remaining_count() {
        let mut s = "hello world".to_string();
        let remaining = truncate_utf8_in_place(&mut s, 5);
        assert_eq!(s, "hello");
        assert_eq!(remaining, 6);
    }
}
