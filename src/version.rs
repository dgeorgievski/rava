use std::str::FromStr;
use semver::Version;

#[derive(serde::Serialize, Debug, Clone)]
pub struct HCVersion {
    pub major: u64,
    pub minor: u64,
    pub patch: u64,
    pub pre: String,
}

pub fn get_version() -> HCVersion {
    let ver = env!("CARGO_PKG_VERSION");

    match Version::from_str(ver) {
        Ok(v) => {
            HCVersion {
                major: v.major,
                minor: v.minor,
                patch: v.patch,
                pre: v.pre.to_string(),
            }
        },
        Err(why) => {
            HCVersion {
                major: 0,
                minor: 0,
                patch: 0,
                pre: why.to_string(),
            }
        }
    }

    
} 