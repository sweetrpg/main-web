//! Crate-wide constants. Add new ones here rather than next to their first use.

/// Public path of the terms of service page.
pub const TERMS_PATH: &str = "/terms";

/// Public path of the privacy policy page.
pub const PRIVACY_PATH: &str = "/privacy";

/// Public path of the support page.
pub const SUPPORT_PATH: &str = "/support";

/// Environment variable holding the support contact (an email address or an http(s) URL).
pub const SUPPORT_CONTACT_ENV: &str = "SUPPORT_CONTACT";

/// Canonical legal documents, embedded at compile time. Each front-matter header carries the
/// version, status, effective date and materiality flag (see `legal_doc.rs`).
pub const TERMS_SOURCE: &str = include_str!("../content/legal/terms-of-service.md");
pub const PRIVACY_SOURCE: &str = include_str!("../content/legal/privacy-policy.md");

/// Front-matter delimiter line in the terms file.
pub const HEADER_DELIMITER: &str = "---";

pub const HEADER_KEY_VERSION: &str = "version";
pub const HEADER_KEY_STATUS: &str = "status";
pub const HEADER_KEY_EFFECTIVE_DATE: &str = "effective_date";
pub const HEADER_KEY_MATERIAL: &str = "material";

pub const STATUS_DRAFT: &str = "draft";
pub const STATUS_IN_FORCE: &str = "in_force";
