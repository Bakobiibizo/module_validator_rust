//! Utility functions for the Module Validator application.

use reqwest::Url;
use std::error::Error;

/// Parse the URL and provide a base if the URL is relative.
///
/// # Arguments
///
/// * `input` - The input URL as a string or string-like type.
///
/// # Returns
///
/// A Result containing the parsed Url if successful, or an error if parsing fails.
pub fn parse_url(input: impl AsRef<str>) -> Result<Url, Box<dyn Error>> {
    let input = input.as_ref();
    // If the input is a relative URL, provide a base
    if !input.starts_with("http://") && !input.starts_with("https://") {
        let base = Url::parse("https://example.com/")?;
        Ok(base.join(input)?)
    } else {
        Ok(Url::parse(input)?)
    }
}