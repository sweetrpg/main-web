//! Crate-wide constants. Add new ones here rather than next to their first use.

/// Public path of the terms of service page.
pub const TERMS_PATH: &str = "/terms";

/// Public path of the privacy policy page.
pub const PRIVACY_PATH: &str = "/privacy";

/// Public path of the support page.
pub const SUPPORT_PATH: &str = "/support";

/// Public path of the about page (third-party credits).
pub const ABOUT_PATH: &str = "/about";

pub const FONT_AWESOME_URL: &str = "https://fontawesome.com";
pub const CC_BY_4_URL: &str = "https://creativecommons.org/licenses/by/4.0/";
pub const FEATHER_URL: &str = "https://github.com/feathericons/feather";

/// Feather's license notice. MIT requires it verbatim, so it is not localized.
pub const FEATHER_LICENSE_NOTICE: &str = "The MIT License (MIT)

Copyright (c) 2013-2023 Cole Bemis

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the \"Software\"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED \"AS IS\", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.";

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
