//! Version information for PM4Py
//!
//! This module provides version constants for the PM4Py library.

/// Current version of PM4Py
pub const VERSION: &str = "0.3.0";

/// Major version number
pub const VERSION_MAJOR: u32 = 0;

/// Minor version number
pub const VERSION_MINOR: u32 = 3;

/// Patch version number
pub const VERSION_PATCH: u32 = 0;

/// Version string with additional info
pub fn version_string() -> String {
    format!("PM4Py v{}", VERSION)
}

/// Get full version information
pub fn version_info() -> VersionInfo {
    VersionInfo {
        version: VERSION.to_string(),
        major: VERSION_MAJOR,
        minor: VERSION_MINOR,
        patch: VERSION_PATCH,
    }
}

/// Information about the current version
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VersionInfo {
    pub version: String,
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl std::fmt::Display for VersionInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_constants() {
        assert_eq!(VERSION, "0.3.0");
        assert_eq!(VERSION_MAJOR, 0);
        assert_eq!(VERSION_MINOR, 3);
        assert_eq!(VERSION_PATCH, 0);
    }

    #[test]
    fn test_version_string() {
        assert_eq!(version_string(), "PM4Py v0.3.0");
    }

    #[test]
    fn test_version_info() {
        let info = version_info();
        assert_eq!(info.version, "0.3.0");
        assert_eq!(info.major, 0);
        assert_eq!(info.minor, 3);
        assert_eq!(info.patch, 0);
        assert_eq!(info.to_string(), "0.3.0");
    }
}
