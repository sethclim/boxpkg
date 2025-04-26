use std::collections::BTreeMap;

mod build_tuple;

use crate::build_tuple::BuildTuple;

mod system_resolver;
use system_resolver::{detect_abi_tag, detect_platform, detect_python_version};

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct SystemEnvironmentInfo {
    pub platform: String,
    pub python_version: String,
    pub abi_tag: String,
}

pub fn get_system_info() -> SystemEnvironmentInfo {
    let py = detect_python_version();
    let abi = detect_abi_tag().unwrap_or("None".to_string());

    let system_env: SystemEnvironmentInfo = SystemEnvironmentInfo {
        platform: detect_platform().to_string(),
        python_version: py.unwrap_or("".to_string()),
        abi_tag: abi,
    };

    println!(
        "platform {} python_version {} abi_tag {}",
        system_env.platform, system_env.python_version, system_env.abi_tag
    );

    return system_env;
}

pub fn get_build_tuple(name: &str, version: &str, system_env: SystemEnvironmentInfo) -> BuildTuple {
    let mut flags = BTreeMap::new();
    flags.insert("WITH_SSL".to_string(), "ON".to_string());
    flags.insert("ENABLE_THREADS".to_string(), "OFF".to_string());

    let tuple = BuildTuple {
        package: name.to_string(),
        package_version: version.to_string(),
        python_version: system_env.python_version,
        platform: system_env.platform,
        abi: system_env.abi_tag,
        compiler: Some("gcc".to_string()),
        build_flags: flags,
    };

    // println!("Build tuple: {:#?}", tuple);
    // println!("Cache key: {}", tuple.hash_key());

    return tuple;
}
