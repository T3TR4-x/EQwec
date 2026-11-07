//! Strict HTTPS package-source URLs.
//!
//! This is a deliberately narrow parser, not a general URL implementation. It
//! accepts only forms that are unambiguous for a signed manifest and rejects
//! userinfo, fragments, IPv6 literals, non-ASCII, dot segments, and malformed
//! percent escapes.

use crate::error::UpdaterError;

/// A validated HTTPS URL.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct HttpsUrl(String);

impl HttpsUrl {
    /// Maximum accepted URL length in bytes.
    pub const MAX_LEN: usize = 2_048;

    /// Parses and validates an HTTPS URL.
    ///
    /// # Errors
    ///
    /// Returns [`UpdaterError::InvalidPackageSource`] with a short reason.
    pub fn parse(value: &str) -> Result<Self, UpdaterError> {
        validate(value)?;
        Ok(Self(value.to_owned()))
    }

    /// Returns the URL as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

fn invalid(reason: &'static str) -> UpdaterError {
    UpdaterError::InvalidPackageSource { reason }
}

fn validate(value: &str) -> Result<(), UpdaterError> {
    if value.len() < 12 || value.len() > HttpsUrl::MAX_LEN {
        return Err(invalid("URL length is outside the accepted range"));
    }
    if !value.is_ascii() {
        return Err(invalid("URL must be ASCII"));
    }
    if value.bytes().any(|byte| !(0x21..=0x7e).contains(&byte)) {
        return Err(invalid("URL must not contain spaces or control characters"));
    }
    if value.contains('\\') {
        return Err(invalid("backslash is not allowed"));
    }
    if value.contains('#') {
        return Err(invalid("fragment is not allowed"));
    }
    if value.contains('@') {
        return Err(invalid("userinfo is not allowed"));
    }

    let rest = value
        .strip_prefix("https://")
        .ok_or_else(|| invalid("scheme must be lowercase https"))?;

    let authority_end = rest.find(['/', '?']).unwrap_or(rest.len());
    let authority = &rest[..authority_end];
    let remainder = &rest[authority_end..];
    if authority.is_empty() {
        return Err(invalid("host is empty"));
    }

    let (host, port) = match authority.rsplit_once(':') {
        Some((host, port)) => {
            if host.contains(':') {
                return Err(invalid("IPv6 literals are not supported"));
            }
            (host, Some(port))
        }
        None => (authority, None),
    };
    validate_host(host)?;
    if let Some(port) = port {
        validate_port(port)?;
    }

    let (path, query) = match remainder.split_once('?') {
        Some((path, query)) => (path, Some(query)),
        None => (remainder, None),
    };
    if !path.is_empty() {
        if !path.starts_with('/') {
            return Err(invalid("path must start with a slash"));
        }
        validate_path(path)?;
    }
    if let Some(query) = query {
        validate_query(query)?;
    }

    Ok(())
}

fn validate_host(host: &str) -> Result<(), UpdaterError> {
    if host.len() > 253 {
        return Err(invalid("host is too long"));
    }
    if host.bytes().any(|byte| byte.is_ascii_uppercase()) {
        return Err(invalid("host must be lowercase"));
    }

    let labels: Vec<&str> = host.split('.').collect();
    if labels.len() < 2 {
        return Err(invalid("host must be a dotted DNS name"));
    }
    if labels
        .iter()
        .all(|label| label.bytes().all(|byte| byte.is_ascii_digit()))
    {
        return Err(invalid("IP address literals are not supported"));
    }

    for label in labels {
        if label.is_empty() || label.len() > 63 {
            return Err(invalid("host label length is invalid"));
        }
        if label.starts_with('-') || label.ends_with('-') {
            return Err(invalid("host label must not start or end with a hyphen"));
        }
        if label
            .bytes()
            .any(|byte| !(byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-'))
        {
            return Err(invalid("host contains an invalid character"));
        }
    }

    Ok(())
}

fn validate_port(port: &str) -> Result<(), UpdaterError> {
    if port.is_empty() || port.len() > 5 {
        return Err(invalid("port is invalid"));
    }
    if port.len() > 1 && port.starts_with('0') {
        return Err(invalid("port must not have a leading zero"));
    }
    if port.bytes().any(|byte| !byte.is_ascii_digit()) {
        return Err(invalid("port must be decimal"));
    }
    let port: u32 = port.parse().map_err(|_| invalid("port is invalid"))?;
    if !(1..=65_535).contains(&port) {
        return Err(invalid("port is outside the accepted range"));
    }
    Ok(())
}

fn validate_path(path: &str) -> Result<(), UpdaterError> {
    if path.contains("//") {
        return Err(invalid("empty path segment is not allowed"));
    }
    for segment in path.split('/').skip(1) {
        if segment == "." || segment == ".." {
            return Err(invalid("dot path segment is not allowed"));
        }
        validate_component(segment, true)?;
    }
    Ok(())
}

fn validate_query(query: &str) -> Result<(), UpdaterError> {
    if query.is_empty() {
        return Err(invalid("empty query is not allowed"));
    }
    validate_component(query, false)
}

fn validate_component(value: &str, path: bool) -> Result<(), UpdaterError> {
    let bytes = value.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        let byte = bytes[index];
        if byte == b'%' {
            if index + 2 >= bytes.len()
                || !bytes[index + 1].is_ascii_hexdigit()
                || !bytes[index + 2].is_ascii_hexdigit()
            {
                return Err(invalid("malformed percent escape"));
            }
            index += 3;
            continue;
        }
        if !is_allowed_component_byte(byte, path) {
            return Err(invalid("character is not allowed in the path or query"));
        }
        index += 1;
    }
    Ok(())
}

fn is_allowed_component_byte(byte: u8, path: bool) -> bool {
    let unreserved = byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~');
    let sub_delim = matches!(
        byte,
        b'!' | b'$' | b'&' | b'\'' | b'(' | b')' | b'*' | b'+' | b',' | b';' | b'='
    );
    let path_specific = path && byte == b'/';
    let query_specific = !path && byte == b'?';
    unreserved || sub_delim || matches!(byte, b':' | b'@') || path_specific || query_specific
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_canonical_https_urls() {
        for value in [
            "https://packages.eqwec.example/hub/0.1.0/pkg.tar.zst",
            "https://cdn.example.com:8443/a/b?x=1&y=two",
            "https://example.com/",
            "https://example.com/a%20b",
        ] {
            assert!(
                HttpsUrl::parse(value).is_ok(),
                "{value:?} should be accepted"
            );
        }
    }

    #[test]
    fn rejects_non_https_schemes() {
        for value in [
            "http://example.com/a",
            "HTTPS://example.com/a",
            "ftp://example.com/a",
            "file:///etc/passwd",
        ] {
            assert!(
                HttpsUrl::parse(value).is_err(),
                "{value:?} should be rejected"
            );
        }
    }

    #[test]
    fn rejects_ambiguous_authorities() {
        for value in [
            "https://user@example.com/a",
            "https://example.com:0/a",
            "https://example.com:65536/a",
            "https://example.com:080/a",
            "https://example.com:/a",
            "https://127.0.0.1/a",
            "https://[::1]/a",
            "https://localhost/a",
            "https://Example.com/a",
            "https://-bad.example/a",
            "https://bad-.example/a",
        ] {
            assert!(
                HttpsUrl::parse(value).is_err(),
                "{value:?} should be rejected"
            );
        }
    }

    #[test]
    fn rejects_fragments_and_dot_segments() {
        for value in [
            "https://example.com/a#fragment",
            "https://example.com/a/../b",
            "https://example.com/a/./b",
            "https://example.com/a//b",
            "https://example.com/a%2",
            "https://example.com/a b",
            "https://example.com/a\\b",
        ] {
            assert!(
                HttpsUrl::parse(value).is_err(),
                "{value:?} should be rejected"
            );
        }
    }

    #[test]
    fn rejects_empty_query() {
        assert!(HttpsUrl::parse("https://example.com/a?").is_err());
    }
}
