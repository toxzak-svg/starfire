//! Read-only, public-web source validation for research.

use anyhow::{bail, Result};
use url::Url;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResearchUrl(Url);

impl ResearchUrl {
    pub fn parse(input: &str) -> Result<Self> {
        let url = Url::parse(input)?;
        if !matches!(url.scheme(), "http" | "https") {
            bail!("research sources must use http or https");
        }
        if !url.username().is_empty() || url.password().is_some() {
            bail!("research sources cannot contain credentials");
        }
        let host = url.host_str().ok_or_else(|| anyhow::anyhow!("research source needs a host"))?;
        if is_private_host(host) {
            bail!("research sources cannot target local or private networks");
        }
        Ok(Self(url))
    }

    pub fn as_str(&self) -> &str { self.0.as_str() }
}

fn is_private_host(host: &str) -> bool {
    let host = host.trim_end_matches('.').to_ascii_lowercase();
    if host == "localhost" || host.ends_with(".localhost") || host.ends_with(".local") { return true; }
    let octets: Vec<u8> = host.split('.').filter_map(|part| part.parse().ok()).collect();
    if octets.len() != 4 { return host == "::1"; }
    matches!(octets.as_slice(), [10, ..] | [127, ..] | [169, 254, ..] | [192, 168, ..] | [0, ..])
        || (octets[0] == 172 && (16..=31).contains(&octets[1]))
}

fn extract_readable_text(html: &str) -> String {
    let mut text = String::with_capacity(html.len());
    let mut in_tag = false;
    let mut skip_until: Option<&str> = None;
    let mut index = 0;

    while index < html.len() {
        let rest = &html[index..];
        if let Some(tag) = skip_until {
            if rest.to_ascii_lowercase().starts_with(tag) {
                skip_until = None;
            }
        }
        if rest.starts_with("<script") || rest.starts_with("<style") {
            skip_until = Some("</");
        }
        let character = rest.chars().next().expect("index stays in bounds");
        match character {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag && skip_until.is_none() => text.push(character),
            _ => {}
        }
        index += character.len_utf8();
    }

    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[cfg(test)]
mod tests {
    use super::extract_readable_text;

    #[test]
    fn extracts_readable_text_without_markup_or_scripts() {
        let text = extract_readable_text(
            "<html><head><script>steal()</script><title>Ignored</title></head><body><h1>Starfire</h1><p>Reads <b>public</b> sources.</p></body></html>",
        );

        assert_eq!(text, "Starfire Reads public sources.");
    }
}
