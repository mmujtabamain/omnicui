//! Generated from omnicui/contracts/signin.contract.yaml by the stub generator. Do not edit: change the contract instead.

/// The props of `app.signin`, with the contract's defaults.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(default)]
pub struct SigninProps {
    pub mode: String,
    pub dataset: String,
}

impl Default for SigninProps {
    fn default() -> Self {
        Self {
            mode: "signin".to_owned(),
            dataset: "standard".to_owned(),
        }
    }
}

/// The events `app.signin` reports.
#[derive(Debug, Clone)]
pub enum SigninEvent {
    SignedIn { email: String },
}

impl SigninEvent {
    /// The event's contract name and fields, as omnicui receives them.
    pub fn to_json(&self) -> (&'static str, serde_json::Value) {
        match self {
            Self::SignedIn { email } => ("signedIn", serde_json::json!({ "email": email })),
        }
    }
}

/// The part ids, as every target names them.
pub mod parts {
    pub const CARD: &str = "app.signin.card";
    pub const APPNAME: &str = "app.signin.appName";
    pub const NAMEFIELD: &str = "app.signin.nameField";
    pub const EMAILFIELD: &str = "app.signin.emailField";
    pub const PASSWORDFIELD: &str = "app.signin.passwordField";
    pub const SUBMITBUTTON: &str = "app.signin.submitButton";
    pub const MODELINK: &str = "app.signin.modeLink";
    pub const FORMERROR: &str = "app.signin.formError";
    pub const NAMEERROR: &str = "app.signin.nameError";
    pub const EMAILERROR: &str = "app.signin.emailError";
    pub const PASSWORDERROR: &str = "app.signin.passwordError";
}
