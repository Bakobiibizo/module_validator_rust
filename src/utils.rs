use reqwest::Url;
use std::error::Error;

// Parse the URL and provide a base if the URL is relative
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