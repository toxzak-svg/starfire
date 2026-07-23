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
        let host = url
            .host_str()
            .ok_or_else(|| anyhow::anyhow!("research source needs a host"))?;
        if is_private_host(host) {
            bail!("research sources cannot target local or private networks");
        }
        Ok(Self(url))
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

fn is_private_host(host: &str) -> bool {
    let host = host.trim_end_matches('.').to_ascii_lowercase();
    if host == "localhost" || host.ends_with(".localhost") || host.ends_with(".local") {
        return true;
    }
    let octets: Vec<u8> = host
        .split('.')
        .filter_map(|part| part.parse().ok())
        .collect();
    if octets.len() != 4 {
        return host == "::1";
    }
    matches!(
        octets.as_slice(),
        [10, ..] | [127, ..] | [169, 254, ..] | [192, 168, ..] | [0, ..]
    ) || (octets[0] == 172 && (16..=31).contains(&octets[1]))
}

#[cfg(test)]
fn extract_readable_text(html: &str) -> String {
    let mut output = String::with_capacity(html.len());
    let mut index = 0;
    let mut skipped_tag: Option<String> = None;

    while index < html.len() {
        if html[index..].starts_with('<') {
            let Some(end_offset) = html[index..].find('>') else {
                break;
            };
            let tag = html[index + 1..index + end_offset].trim();
            let closing = tag.starts_with('/');
            let tag_name = tag
                .trim_start_matches('/')
                .split_whitespace()
                .next()
                .unwrap_or("")
                .trim_end_matches('/')
                .to_ascii_lowercase();

            if closing && skipped_tag.as_deref() == Some(tag_name.as_str()) {
                skipped_tag = None;
            } else if !closing && matches!(tag_name.as_str(), "script" | "style" | "title") {
                skipped_tag = Some(tag_name);
            }

            if skipped_tag.is_none()
                && output
                    .chars()
                    .last()
                    .is_some_and(|character| !character.is_whitespace())
            {
                output.push(' ');
            }
            index += end_offset + 1;
            continue;
        }

        let character = html[index..]
            .chars()
            .next()
            .expect("index stays on a character boundary");
        if skipped_tag.is_none() {
            output.push(character);
        }
        index += character.len_utf8();
    }

    output.split_whitespace().collect::<Vec<_>>().join(" ")
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
