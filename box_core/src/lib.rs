use std::collections::BTreeMap;

mod build_tuple;

use crate::build_tuple::BuildTuple;

mod system_resolver;
use system_resolver::{detect_abi_tag, detect_platform, detect_python_version};

use reqwest::blocking::get;
use std::fs::File;
use std::io::copy;

use flate2::read::GzDecoder;
use tar::Archive;

mod python_builder;
use python_builder::{build_wheel, setup_python_env};

use std::fs;
use std::io;
use std::path::Path;

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

pub fn download_source(
    url: &str,
    path: &str,
) -> Result<(String, String), Box<dyn std::error::Error>> {
    if let Some((_, base)) = url.rsplit_once('/') {
        println!("filename: {}", base);
        // Perform the GET request
        let response = get(url)?;

        let file_path = format!("{}{}", path, base);

        // Create a file to save the tarball
        let mut out = File::create(&file_path)?;

        // Stream the response into the file
        let mut content = response;
        copy(&mut content, &mut out)?;

        println!("Downloaded to {}", file_path);
        return Ok((file_path, base.to_string()));
    }
    Err("couldn't get base comp of url".into())
}

pub fn extract_tar_gz(file_path: &str, output_dir: &str) -> std::io::Result<()> {
    // Open the .tar.gz file
    let file = File::open(file_path)?;
    let decoder = GzDecoder::new(file);

    // Create a tar Archive from the decompressed stream
    let mut archive = Archive::new(decoder);

    // Extract the archive to the specified directory
    archive.unpack(output_dir)?;

    Ok(())
}

pub fn create_venv_and_build(project_path: &str) {
    let _ = setup_python_env(project_path);
    let _ = build_wheel(project_path);
}

pub fn move_wheel(build_dir: &Path, cache_dir: &Path) -> io::Result<()> {
    let dist_dir = build_dir.join("dist");

    // Check if dist/ exists
    if !dist_dir.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "dist/ directory not found",
        ));
    }

    // Find the first .whl file in dist/
    let wheel_file = fs::read_dir(&dist_dir)?
        .filter_map(Result::ok)
        .find(|entry| {
            entry
                .path()
                .extension()
                .map(|ext| ext == "whl")
                .unwrap_or(false)
        })
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "No .whl file found in dist/"))?;

    // Ensure cache dir exists
    fs::create_dir_all(cache_dir)?;

    // Build the destination path
    let dest_path = cache_dir.join(wheel_file.file_name());

    // Move (rename) the file
    fs::rename(wheel_file.path(), dest_path)?;

    Ok(())
}
