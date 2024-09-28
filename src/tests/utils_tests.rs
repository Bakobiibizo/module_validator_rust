use crate::utils::parse_url;

#[test]
fn test_parse_url_absolute() {
    let url = "https://example.com/path";
    let parsed = parse_url(url).unwrap();
    assert_eq!(parsed.as_str(), url);
}

#[test]
fn test_parse_url_relative() {
    let url = "relative/path";
    let parsed = parse_url(url).unwrap();
    assert_eq!(parsed.as_str(), "https://example.com/relative/path");
}