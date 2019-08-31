use std::path::Path;

use path_absolutize::Absolutize;

pub fn canonicalize_path(path: &str) -> String {
    if path.is_empty() || path == "/" {
        return String::from("");
    }

    match Path::new(path).absolutize() {
        Err(_) => String::from(""),
        Ok(result) => String::from(result.to_str().unwrap())
    }
}

pub fn is_sanitized(needle_path: &str, catalog_path: &str) -> bool {
    needle_path.len() == catalog_path.len() ||
        (needle_path.len() > catalog_path.len() &&
        needle_path.chars().last().unwrap() == '/')
}

#[cfg(test)]
mod tests {
    use crate::operations::path::{canonicalize_path, is_sanitized};

    #[test]
    fn test_canonicalize_path() {
        assert_eq!("/1/2/3", canonicalize_path("/1/2/3/4/.."));
        assert_eq!("/1/2/3/4", canonicalize_path("/1/2/3/4/."));
        assert_eq!("", canonicalize_path("/"));
    }

    #[test]
    fn test_is_sanitized() {
        assert!(is_sanitized("/1/2/3/", "/1/2/3"));
    }
}
