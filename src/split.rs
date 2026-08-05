/// Split a directory name into its parts.
///
/// ```
/// # use fuzzy_dir::split;
/// assert_eq!(split("helloWorld"), vec!["hello", "World"]);
/// assert_eq!(split("hello_world"), vec!["hello", "world"]);
/// assert_eq!(split("hello-world"), vec!["hello", "world"]);
/// assert_eq!(split("hello world"), vec!["hello", "world"]);
/// assert_eq!(split("hello\\ world"), vec!["hello", "world"]);
/// assert_eq!(split("helloMy world"), vec!["hello", "My", "world"]);
/// ```
pub fn split(name: &str) -> Vec<&str> {
    let mut res = Vec::new();
    let mut flush = |start: usize, end: Option<usize>| {
        if end.is_none_or(|end| end > start) {
            let slice = end.map_or(&name[start..], |end| &name[start..end]);
            res.push(slice);
        }
    };

    let mut start = 0;
    let mut prev_lower = false;
    for (idx, char) in name.char_indices() {
        if matches!(
            name.as_bytes().get(idx..),
            Some([b'_', ..] | [b'-', ..] | [b' ', ..] | [b'\\', b' ', ..] | [b'~', ..])
        ) {
            flush(start, Some(idx));
            start = idx + 1;
        } else if prev_lower && char.is_uppercase() {
            flush(start, Some(idx));
            start = idx;
        }
        prev_lower = char.is_lowercase();
    }
    flush(start, None);

    res
}

#[cfg(test)]
mod tests {
    use super::split;

    #[test]
    fn test_basic_cases() {
        assert_eq!(split("helloWorld"), vec!["hello", "World"]);
        assert_eq!(split("hello_world"), vec!["hello", "world"]);
        assert_eq!(split("hello-world"), vec!["hello", "world"]);
        assert_eq!(split("hello world"), vec!["hello", "world"]);
        assert_eq!(split("hello\\ world"), vec!["hello", "world"]);
        assert_eq!(split("helloMy world"), vec!["hello", "My", "world"]);
    }

    #[test]
    fn test_advanced_cases() {
        assert_eq!(split("helloWWorld"), vec!["hello", "WWorld"]);
        assert_eq!(split("helloW_World"), vec!["hello", "W", "World"]);
        assert_eq!(split("aA___a"), vec!["a", "A", "a"]);
    }

    // TODO: Add more edge cases
}
