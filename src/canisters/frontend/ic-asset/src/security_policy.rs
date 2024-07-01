//! Content-Security Policy

use std::fmt::Display;

use itertools::Itertools;
use serde::{Deserialize, Serialize};

use crate::asset::config::HeadersConfig;

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Copy, Debug)]
#[serde(rename_all = "lowercase")]
/// Asset synchronization will warn if no
pub enum SecurityPolicy {
    /// No CSP provided by asset sync.
    Disabled,
    /// The default CSP that will work for most dapps but could be more secure.
    /// When using this CSP asset sync will still warn that the CSP could be hardened.
    Standard,
    /// Use the default CSP with custom improvements.
    /// Same as `Standard`, but disables the warning that the CSP could be hardened.
    Hardened,
}

#[derive(Debug)]
struct ConcreteSecurityPolicy {
    /// When displaying the policy this will be a preface to the actual headers
    general_comment: &'static str,

    /// (header_name, header_content, header_explanation)
    headers: Vec<(&'static str, &'static str, &'static str)>,
}

impl ConcreteSecurityPolicy {
    fn to_headers(&self) -> HeadersConfig {
        self.headers.iter().map(|(name, content, _explanation)| (name.to_string(), content.to_string())).collect()
    }

    /// Produces the CSP as a String that can be used as valid json5 with explainer comments
    fn to_json5_str(&self) -> String {
        let general_comment = self.general_comment.lines().map(|line| format!("// {line}")).join("\n");
        if self.headers.len() > 0 {
            let headers = self.headers.iter().map(|(name, content, explanation)| {
                let explanation = explanation.lines().map(|line| format!("// {line}")).join("\n");
                let header_line = format!(r#""{name}": "{content}""#);
                format!("{explanation}\n{header_line}")
            }).join(",\n\n");
            format!("{general_comment}\n\n{headers}")
        } else {
            general_comment
        }
    }
}


impl SecurityPolicy {
    fn to_policy(&self) -> ConcreteSecurityPolicy {
        match self {
            SecurityPolicy::Disabled => ConcreteSecurityPolicy {
                general_comment: "No content security policy.",
                headers: vec![],
            },
            SecurityPolicy::Standard | SecurityPolicy::Hardened => ConcreteSecurityPolicy {
                general_comment: r#"Security: The Content Security Policy (CSP) given below aims at working with many apps rather than providing maximal security.
We recommend tightening the CSP for your specific application. Some recommendations are as follows:
- Use the CSP Evaluator (https://csp-evaluator.withgoogle.com/) to validate the CSP you define.
- Follow the “Strict CSP” recommendations (https://csp.withgoogle.com/docs/strict-csp.html). However, note that in the context of the IC,
  nonces cannot be used because the response bodies must be static to work well with HTTP asset certification.
  Thus, we recommend to include script hashes (in combination with strict-dynamic) in the CSP as described
  in https://csp.withgoogle.com/docs/faq.html in section “What if my site is static and I can't add nonces to scripts?”.
  See for example the II CSP (https://github.com/dfinity/internet-identity/blob/main/src/internet_identity/src/http.rs).
- It is recommended to tighten the connect-src directive. With the current CSP configuration the browser can
  make requests to https://*.icp0.io, hence being able to call any canister via https://icp0.io/api/v2/canister/{canister-ID}.
  This could potentially be used in combination with another vulnerability (e.g. XSS) to exfiltrate private data.
  The developer can configure this policy to only allow requests to their specific canisters,
  e.g: connect-src 'self' https://icp-api.io/api/v2/canister/{my-canister-ID}, where {my-canister-ID} has the following format: aaaaa-aaaaa-aaaaa-aaaaa-aaa
- It is recommended to configure style-src, style-src-elem and font-src directives with the resources your canister is going to use
  instead of using the wild card (*) option. Normally this will include 'self' but also other third party styles or fonts resources (e.g: https://fonts.googleapis.com or other CDNs)"#,
                headers: vec![
                    (
                        "Content-Security-Policy",
                        "default-src 'self';script-src 'self';connect-src 'self' http://localhost:* https://icp0.io https://*.icp0.io https://icp-api.io;img-src 'self' data:;style-src * 'unsafe-inline';style-src-elem * 'unsafe-inline';font-src *;object-src 'none';base-uri 'self';frame-ancestors 'none';form-action 'self';upgrade-insecure-requests;",
                        r#"Notes about the CSP below:
- We added img-src data: because data: images are used often.
- frame-ancestors: none mitigates clickjacking attacks. See https://owasp.org/www-community/attacks/Clickjacking."#
                    ),
                    (
                        "Permissions-Policy",
                        "accelerometer=(), ambient-light-sensor=(), autoplay=(), battery=(), camera=(), cross-origin-isolated=(), display-capture=(), document-domain=(), encrypted-media=(), execution-while-not-rendered=(), execution-while-out-of-viewport=(), fullscreen=(), geolocation=(), gyroscope=(), keyboard-map=(), magnetometer=(), microphone=(), midi=(), navigation-override=(), payment=(), picture-in-picture=(), publickey-credentials-get=(), screen-wake-lock=(), sync-xhr=(), usb=(), web-share=(), xr-spatial-tracking=(), clipboard-read=(), clipboard-write=(), gamepad=(), speaker-selection=(), conversion-measurement=(), focus-without-user-activation=(), hid=(), idle-detection=(), interest-cohort=(), serial=(), sync-script=(), trust-token-redemption=(), window-placement=(), vertical-scroll=()",
                        r#"Security: The permissions policy disables all features for security reasons. If your site needs such permissions, activate them.
To configure permissions go here https://www.permissionspolicy.com/"#
                    ),
                    (
                        "X-Frame-Options",
                        "DENY",
                        r#"Security: Mitigates clickjacking attacks.
See: https://owasp.org/www-community/attacks/Clickjacking."#
                    ),
                    (
                        "Referrer-Policy",
                        "same-origin",
                        r#"Security: Avoids forwarding referrer information to other origins.
See: https://owasp.org/www-project-secure-headers/#referrer-policy."#
                    ),
                    (
                        "Strict-Transport-Security",
                        "max-age=31536000; includeSubDomains",
                        r#"Security: Tells the user's browser that it must always use HTTPS with your site.
See: https://owasp.org/www-project-secure-headers/#http-strict-transport-security"#
                    ),
                    (
                        "X-Content-Type-Options",
                        "nosniff",
                        r#"Security: Prevents the browser from interpreting files as a different MIME type to what is specified in the Content-Type header.
See: https://owasp.org/www-project-secure-headers/#x-content-type-options"#
                    ),
                    (
                        "X-XSS-Protection",
                        "1; mode=block",
                        r#"Security: Enables browser features to mitigate some of the XSS attacks. Note that it has to be in mode=block.
See: https://owasp.org/www-community/attacks/xss/"#
                    )  
                ],
            },
        }
    }

    pub(crate) fn to_headers(&self) -> HeadersConfig {
        self.to_policy().to_headers()
    }

    /// Prints the CSP in the format that could be used in `.ic-assets.json5` directly.
    /// Includes explanatory comments.
    pub fn to_json5_str(&self) -> String {
        self.to_policy().to_json5_str()
    }
}

impl Display for SecurityPolicy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SecurityPolicy::Disabled => write!(f, "disabled"),
            SecurityPolicy::Standard => write!(f, "standard"),
            SecurityPolicy::Hardened => write!(f, "hardened"),
        }
    }
}