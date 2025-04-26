use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct BuildTuple {
    pub package: String,
    pub package_version: String,
    pub python_version: String,                // e.g. "3.11"
    pub platform: String,                      // e.g. "windows-x86_64", "linux-aarch64"
    pub abi: String,                           // e.g. "cp311", "abi3", "none"
    pub compiler: Option<String>,              // e.g. "msvc", "gcc", "clang"
    pub build_flags: BTreeMap<String, String>, // e.g. "WITH_SSL" => "ON"
}

impl BuildTuple {
    pub fn hash_key(&self) -> String {
        let mut hasher = Sha256::new();
        let mut data = format!(
            "{}@{}|{}|{}|{}|{}|",
            self.package,
            self.package_version,
            self.python_version,
            self.platform,
            self.abi,
            self.compiler.clone().unwrap_or_default()
        );
        for (k, v) in &self.build_flags {
            data.push_str(&format!("{}={};", k, v));
        }

        hasher.update(data);
        format!("{:x}", hasher.finalize())
    }
}
