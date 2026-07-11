//! Pure NetworkManager Wi-Fi connect planning (nmcli-style argv).
//!
//! No process execution here — host unit tests exercise validation and plan
//! construction only. Linux session code may run the plan later.

/// Request to connect to a Wi-Fi network via NetworkManager.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NmConnectRequest {
    pub ssid: String,
    pub password_optional: Option<String>,
}

impl NmConnectRequest {
    pub fn new(ssid: impl Into<String>) -> Self {
        Self {
            ssid: ssid.into(),
            password_optional: None,
        }
    }

    pub fn with_password(mut self, password: impl Into<String>) -> Self {
        self.password_optional = Some(password.into());
        self
    }
}

/// Validate an NM connect request (pure).
///
/// Rules:
/// - SSID non-empty
/// - SSID max 32 bytes (IEEE 802.11)
/// - SSID must not contain NUL
/// - Optional password must not contain NUL
pub fn validate_nm_connect_request(req: &NmConnectRequest) -> Result<(), String> {
    if req.ssid.is_empty() {
        return Err("ssid must be non-empty".to_string());
    }
    if req.ssid.len() > 32 {
        return Err("ssid exceeds 32 bytes".to_string());
    }
    if req.ssid.contains('\0') {
        return Err("ssid contains null byte".to_string());
    }
    if let Some(pw) = &req.password_optional {
        if pw.contains('\0') {
            return Err("password contains null byte".to_string());
        }
    }
    Ok(())
}

/// Ordered nmcli-style argv plan for connecting (pure, not executed).
///
/// Open network:
/// `["nmcli", "dev", "wifi", "connect", <ssid>]`
///
/// With password:
/// `["nmcli", "dev", "wifi", "connect", <ssid>, "password", <password>]`
///
/// Empty password string is treated as no password (open network argv).
pub fn nm_connect_plan(req: &NmConnectRequest) -> Vec<String> {
    let mut plan = vec![
        "nmcli".to_string(),
        "dev".to_string(),
        "wifi".to_string(),
        "connect".to_string(),
        req.ssid.clone(),
    ];
    if let Some(pw) = &req.password_optional {
        if !pw.is_empty() {
            plan.push("password".to_string());
            plan.push(pw.clone());
        }
    }
    plan
}

/// Validate then build a connect plan. Returns validation errors unchanged.
pub fn nm_connect_plan_validated(req: &NmConnectRequest) -> Result<Vec<String>, String> {
    validate_nm_connect_request(req)?;
    Ok(nm_connect_plan(req))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_accepts_normal_ssid() {
        let req = NmConnectRequest::new("HomeWiFi");
        assert!(validate_nm_connect_request(&req).is_ok());
    }

    #[test]
    fn validate_rejects_empty_ssid() {
        let req = NmConnectRequest::new("");
        let err = validate_nm_connect_request(&req).unwrap_err();
        assert!(err.contains("non-empty"));
    }

    #[test]
    fn validate_rejects_ssid_over_32_bytes() {
        let req = NmConnectRequest::new("a".repeat(33));
        let err = validate_nm_connect_request(&req).unwrap_err();
        assert!(err.contains("32"));
    }

    #[test]
    fn validate_accepts_ssid_exactly_32_bytes() {
        let req = NmConnectRequest::new("a".repeat(32));
        assert!(validate_nm_connect_request(&req).is_ok());
    }

    #[test]
    fn validate_rejects_null_in_ssid() {
        let req = NmConnectRequest {
            ssid: "bad\0ssid".into(),
            password_optional: None,
        };
        let err = validate_nm_connect_request(&req).unwrap_err();
        assert!(err.contains("null"));
    }

    #[test]
    fn validate_rejects_null_in_password() {
        let req = NmConnectRequest {
            ssid: "ok".into(),
            password_optional: Some("pw\0x".into()),
        };
        let err = validate_nm_connect_request(&req).unwrap_err();
        assert!(err.contains("null"));
    }

    #[test]
    fn nm_connect_plan_open_network() {
        let req = NmConnectRequest::new("Cafe");
        assert_eq!(
            nm_connect_plan(&req),
            vec!["nmcli", "dev", "wifi", "connect", "Cafe"]
        );
    }

    #[test]
    fn nm_connect_plan_with_password() {
        let req = NmConnectRequest::new("Home").with_password("s3cret");
        assert_eq!(
            nm_connect_plan(&req),
            vec![
                "nmcli", "dev", "wifi", "connect", "Home", "password", "s3cret"
            ]
        );
    }

    #[test]
    fn nm_connect_plan_empty_password_is_open() {
        let req = NmConnectRequest {
            ssid: "Open".into(),
            password_optional: Some(String::new()),
        };
        assert_eq!(
            nm_connect_plan(&req),
            vec!["nmcli", "dev", "wifi", "connect", "Open"]
        );
    }

    #[test]
    fn nm_connect_plan_validated_fails_on_bad_ssid() {
        let req = NmConnectRequest::new("");
        assert!(nm_connect_plan_validated(&req).is_err());
    }

    #[test]
    fn nm_connect_plan_validated_ok() {
        let req = NmConnectRequest::new("Net").with_password("pw");
        let plan = nm_connect_plan_validated(&req).unwrap();
        assert_eq!(plan[0], "nmcli");
        assert!(plan.contains(&"password".to_string()));
    }
}
