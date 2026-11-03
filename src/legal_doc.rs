use std::fmt;

use pulldown_cmark::{html, Event, Parser};

use crate::constants::{
    HEADER_DELIMITER, HEADER_KEY_EFFECTIVE_DATE, HEADER_KEY_MATERIAL, HEADER_KEY_STATUS,
    HEADER_KEY_VERSION, STATUS_DRAFT, STATUS_IN_FORCE,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LegalStatus {
    Draft,
    InForce,
}

/// Machine-readable header at the top of the legal document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LegalHeader {
    pub version: String,
    pub status: LegalStatus,
    /// Empty until the terms are approved and in force.
    pub effective_date: String,
    /// Whether this version requires existing users to re-accept (read by `auth-web`).
    pub material: bool,
}

/// The parsed terms: header plus the body rendered to HTML.
#[derive(Debug, Clone)]
pub struct LegalDoc {
    pub header: LegalHeader,
    pub html: String,
}

#[derive(Debug, PartialEq, Eq)]
pub enum LegalError {
    MissingHeader,
    UnclosedHeader,
    MalformedLine(String),
    DuplicateKey(String),
    UnknownKey(String),
    MissingKey(&'static str),
    InvalidValue { key: &'static str, value: String },
}

impl fmt::Display for LegalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingHeader => write!(f, "legal document must start with a `---` header"),
            Self::UnclosedHeader => write!(f, "legal document header has no closing `---` line"),
            Self::MalformedLine(line) => {
                write!(
                    f,
                    "legal document header line is not `key: value`: {line:?}"
                )
            }
            Self::DuplicateKey(key) => write!(f, "legal document header repeats key `{key}`"),
            Self::UnknownKey(key) => write!(f, "legal document header has unknown key `{key}`"),
            Self::MissingKey(key) => write!(f, "legal document header is missing key `{key}`"),
            Self::InvalidValue { key, value } => {
                write!(
                    f,
                    "legal document header key `{key}` has invalid value {value:?}"
                )
            }
        }
    }
}

impl std::error::Error for LegalError {}

impl LegalDoc {
    /// Splits `source` into its header and markdown body and renders the body.
    ///
    /// Raw HTML in the markdown is escaped, not passed through.
    pub fn parse(source: &str) -> Result<Self, LegalError> {
        let (header_text, body) = split_header(source)?;
        Ok(Self {
            header: parse_header(header_text)?,
            html: render_markdown(body),
        })
    }
}

fn split_header(source: &str) -> Result<(&str, &str), LegalError> {
    let rest = source
        .strip_prefix(HEADER_DELIMITER)
        .and_then(|r| r.strip_prefix('\n'))
        .ok_or(LegalError::MissingHeader)?;
    let mut offset = 0;
    for line in rest.split_inclusive('\n') {
        if line.trim_end() == HEADER_DELIMITER {
            return Ok((&rest[..offset], &rest[offset + line.len()..]));
        }
        offset += line.len();
    }
    Err(LegalError::UnclosedHeader)
}

fn parse_header(text: &str) -> Result<LegalHeader, LegalError> {
    let mut version = None;
    let mut status = None;
    let mut effective_date = None;
    let mut material = None;

    for line in text.lines().filter(|l| !l.trim().is_empty()) {
        let (key, raw) = line
            .split_once(':')
            .ok_or_else(|| LegalError::MalformedLine(line.to_string()))?;
        let key = key.trim();
        let value = unquote(raw.trim());
        let slot = match key {
            HEADER_KEY_VERSION => &mut version,
            HEADER_KEY_STATUS => &mut status,
            HEADER_KEY_EFFECTIVE_DATE => &mut effective_date,
            HEADER_KEY_MATERIAL => &mut material,
            _ => return Err(LegalError::UnknownKey(key.to_string())),
        };
        if slot.replace(value).is_some() {
            return Err(LegalError::DuplicateKey(key.to_string()));
        }
    }

    let version = version.ok_or(LegalError::MissingKey(HEADER_KEY_VERSION))?;
    if version.is_empty() {
        return Err(LegalError::InvalidValue {
            key: HEADER_KEY_VERSION,
            value: version,
        });
    }
    Ok(LegalHeader {
        version,
        status: parse_status(status.ok_or(LegalError::MissingKey(HEADER_KEY_STATUS))?)?,
        effective_date: effective_date.ok_or(LegalError::MissingKey(HEADER_KEY_EFFECTIVE_DATE))?,
        material: parse_bool(material.ok_or(LegalError::MissingKey(HEADER_KEY_MATERIAL))?)?,
    })
}

fn unquote(value: &str) -> String {
    value
        .strip_prefix('"')
        .and_then(|v| v.strip_suffix('"'))
        .unwrap_or(value)
        .to_string()
}

fn parse_status(value: String) -> Result<LegalStatus, LegalError> {
    match value.as_str() {
        STATUS_DRAFT => Ok(LegalStatus::Draft),
        STATUS_IN_FORCE => Ok(LegalStatus::InForce),
        _ => Err(LegalError::InvalidValue {
            key: HEADER_KEY_STATUS,
            value,
        }),
    }
}

fn parse_bool(value: String) -> Result<bool, LegalError> {
    match value.as_str() {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => Err(LegalError::InvalidValue {
            key: HEADER_KEY_MATERIAL,
            value,
        }),
    }
}

fn render_markdown(body: &str) -> String {
    let events = Parser::new(body).map(|event| match event {
        Event::Html(raw) | Event::InlineHtml(raw) => Event::Text(raw),
        other => other,
    });
    let mut out = String::new();
    html::push_html(&mut out, events);
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants::{PRIVACY_SOURCE, TERMS_SOURCE};

    const VALID: &str =
        "---\nversion: \"0.1\"\nstatus: draft\neffective_date: \"\"\nmaterial: false\n---\n# Title\n\nBody.\n";

    #[test]
    fn valid_header_parses_and_body_renders() {
        let terms = LegalDoc::parse(VALID).expect("valid terms");
        assert_eq!(
            terms.header,
            LegalHeader {
                version: "0.1".to_string(),
                status: LegalStatus::Draft,
                effective_date: String::new(),
                material: false,
            }
        );
        assert!(terms.html.contains("<h1>Title</h1>"));
    }

    #[test]
    fn in_force_status_with_date_and_material_flag_parses() {
        let source = VALID
            .replace("status: draft", "status: in_force")
            .replace("effective_date: \"\"", "effective_date: \"2026-11-01\"")
            .replace("material: false", "material: true");
        let header = LegalDoc::parse(&source).unwrap().header;
        assert_eq!(header.status, LegalStatus::InForce);
        assert_eq!(header.effective_date, "2026-11-01");
        assert!(header.material);
    }

    #[test]
    fn missing_header_is_rejected() {
        let err = LegalDoc::parse("# Title\n\nBody.\n").unwrap_err();
        assert_eq!(err, LegalError::MissingHeader);
    }

    #[test]
    fn unclosed_header_is_rejected() {
        let err = LegalDoc::parse("---\nversion: \"0.1\"\n").unwrap_err();
        assert_eq!(err, LegalError::UnclosedHeader);
    }

    #[test]
    fn missing_key_is_rejected() {
        let err = LegalDoc::parse(&VALID.replace("status: draft\n", "")).unwrap_err();
        assert_eq!(err, LegalError::MissingKey(HEADER_KEY_STATUS));
    }

    #[test]
    fn malformed_line_is_rejected() {
        let err = LegalDoc::parse(&VALID.replace("status: draft", "status draft")).unwrap_err();
        assert_eq!(err, LegalError::MalformedLine("status draft".to_string()));
    }

    #[test]
    fn unknown_status_is_rejected() {
        let err = LegalDoc::parse(&VALID.replace("draft", "final")).unwrap_err();
        assert!(matches!(
            err,
            LegalError::InvalidValue { key: "status", .. }
        ));
    }

    #[test]
    fn non_boolean_material_is_rejected() {
        let err =
            LegalDoc::parse(&VALID.replace("material: false", "material: maybe")).unwrap_err();
        assert!(matches!(
            err,
            LegalError::InvalidValue {
                key: "material",
                ..
            }
        ));
    }

    #[test]
    fn unknown_and_duplicate_keys_are_rejected() {
        let unknown = LegalDoc::parse(&VALID.replace("material", "materail")).unwrap_err();
        assert_eq!(unknown, LegalError::UnknownKey("materail".to_string()));
        let duplicate = LegalDoc::parse(&VALID.replace("---\n#", "version: \"0.2\"\n---\n#"));
        assert_eq!(
            duplicate.unwrap_err(),
            LegalError::DuplicateKey("version".to_string())
        );
    }

    #[test]
    fn raw_html_in_the_body_is_escaped() {
        let source = VALID.replace("Body.", "<script>alert(1)</script>\n\nInline <b>x</b>.");
        let html = LegalDoc::parse(&source).unwrap().html;
        assert!(!html.contains("<script>"));
        assert!(!html.contains("<b>"));
        assert!(html.contains("&lt;script&gt;"));
    }

    #[test]
    fn embedded_documents_parse() {
        for source in [TERMS_SOURCE, PRIVACY_SOURCE] {
            let doc = LegalDoc::parse(source).expect("embedded document must parse");
            assert!(!doc.header.version.is_empty());
            assert!(doc.html.contains("<h2>"));
        }
    }
}
