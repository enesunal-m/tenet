use regex::Regex;

/// Secret patterns used in v0.
pub struct SecretPatterns {
    pub github: Regex,
    pub aws: Regex,
    pub pem: Regex,
}

impl SecretPatterns {
    pub fn compile() -> Self {
        Self {
            github: Regex::new(r"ghp_[A-Za-z0-9]{36}").expect("regex"),
            aws: Regex::new(r"AKIA[0-9A-Z]{16}").expect("regex"),
            pem: Regex::new(r"-----BEGIN .* PRIVATE KEY-----").expect("regex"),
        }
    }
}
