#[derive(Debug, Clone)]
pub struct UserAgentInfo {
    pub device_type: Option<String>,
    pub browser: Option<String>,
    pub os: Option<String>,
}

impl UserAgentInfo {
    pub fn parse(user_agent: &Option<String>) -> Self {
        if let Some(ua) = user_agent {
            Self {
                device_type: Self::extract_device_type(ua),
                browser: Self::extract_browser(ua),
                os: Self::extract_os(ua),
            }
        } else {
            Self {
                device_type: None,
                browser: None,
                os: None,
            }
        }
    }
    
    fn extract_device_type(user_agent: &str) -> Option<String> {
        let ua_lower = user_agent.to_lowercase();
        if ua_lower.contains("mobile") || ua_lower.contains("android") || ua_lower.contains("iphone") {
            Some("Mobile".to_string())
        } else if ua_lower.contains("tablet") || ua_lower.contains("ipad") {
            Some("Tablet".to_string())
        } else {
            Some("Desktop".to_string())
        }
    }
    
    fn extract_browser(user_agent: &str) -> Option<String> {
        let ua_lower = user_agent.to_lowercase();
        if ua_lower.contains("chrome") && !ua_lower.contains("edg") {
            Some("Chrome".to_string())
        } else if ua_lower.contains("firefox") {
            Some("Firefox".to_string())
        } else if ua_lower.contains("safari") && !ua_lower.contains("chrome") {
            Some("Safari".to_string())
        } else if ua_lower.contains("edg") {
            Some("Edge".to_string())
        } else if ua_lower.contains("opera") {
            Some("Opera".to_string())
        } else {
            Some("Unknown".to_string())
        }
    }
    
    fn extract_os(user_agent: &str) -> Option<String> {
        let ua_lower = user_agent.to_lowercase();
        if ua_lower.contains("windows") {
            Some("Windows".to_string())
        } else if ua_lower.contains("mac") && !ua_lower.contains("iphone") && !ua_lower.contains("ipad") {
            Some("macOS".to_string())
        } else if ua_lower.contains("android") {
            Some("Android".to_string())
        } else if ua_lower.contains("linux") {
            Some("Linux".to_string())
        } else if ua_lower.contains("iphone") || ua_lower.contains("ipad") {
            Some("iOS".to_string())
        } else {
            Some("Unknown".to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_chrome_desktop() {
        let ua = Some("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36".to_string());
        let info = UserAgentInfo::parse(&ua);
        
        assert_eq!(info.device_type, Some("Desktop".to_string()));
        assert_eq!(info.browser, Some("Chrome".to_string()));
        assert_eq!(info.os, Some("Windows".to_string()));
    }
    
    #[test]
    fn test_parse_iphone_safari() {
        let ua = Some("Mozilla/5.0 (iPhone; CPU iPhone OS 14_6 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/14.1.1 Mobile/15E148 Safari/604.1".to_string());
        let info = UserAgentInfo::parse(&ua);
        
        assert_eq!(info.device_type, Some("Mobile".to_string()));
        assert_eq!(info.browser, Some("Safari".to_string()));
        assert_eq!(info.os, Some("iOS".to_string()));
    }
    
    #[test]
    fn test_parse_android_chrome() {
        let ua = Some("Mozilla/5.0 (Linux; Android 11; SM-G991B) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.120 Mobile Safari/537.36".to_string());
        let info = UserAgentInfo::parse(&ua);
        
        assert_eq!(info.device_type, Some("Mobile".to_string()));
        assert_eq!(info.browser, Some("Chrome".to_string()));
        assert_eq!(info.os, Some("Android".to_string()));
    }
    
    #[test]
    fn test_parse_none_user_agent() {
        let info = UserAgentInfo::parse(&None);
        
        assert_eq!(info.device_type, None);
        assert_eq!(info.browser, None);
        assert_eq!(info.os, None);
    }
}