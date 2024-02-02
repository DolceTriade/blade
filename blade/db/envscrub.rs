#[allow(dead_code)]
pub(crate) fn scrub(s: &str) -> String {
    let Some(first) = s[0..].find('=') else {
        return s.to_string();
    };
    if first + 1 >= s.len() {
        return s.to_string();
    }
    let Some(second) = s[first + 1..].find('=') else {
        return s.to_string();
    };
    if first + second + 2 >= s.len() {
        return s.to_string();
    }
    format!("{}=<SCRUBBED>", &s[..first + second + 1])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scrub() {
        assert_eq!(scrub("foo=bar"), "foo=bar");
        assert_eq!(scrub("foo=bar=baz=qux"), "foo=bar=<SCRUBBED>");
        assert_eq!(scrub(""), "");
        assert_eq!(scrub("yo"), "yo");
        assert_eq!(scrub("foo="), "foo=");
        assert_eq!(scrub("foo=bar="), "foo=bar=");
    }
}
