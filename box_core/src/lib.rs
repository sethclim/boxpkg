use std::collections::BTreeMap;
use std::path::PathBuf;
use std::process::Command;

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

pub fn download_source(url: &str, path: &Path) -> Result<PathBuf, Box<dyn std::error::Error>> {
    if let Some((_, base)) = url.rsplit_once('/') {
        println!("filename: {}", base);
        // Perform the GET request
        let response = get(url)?;

        let file_path = path.join(base);

        // Create a file to save the tarball
        let mut out = File::create(&file_path)?;

        // Stream the response into the file
        let mut content = response;
        copy(&mut content, &mut out)?;

        println!("Downloaded to {}", file_path.display());
        return Ok(file_path.to_path_buf());
    }
    Err("couldn't get base comp of url".into())
}

pub fn extract_tar_gz(file_path: &Path, output_dir: &Path) -> std::io::Result<()> {
    // Open the .tar.gz file
    let file = File::open(file_path)?;
    let decoder = GzDecoder::new(file);

    // Create a tar Archive from the decompressed stream
    let mut archive = Archive::new(decoder);

    // Extract the archive to the specified directory
    archive.unpack(output_dir)?;

    Ok(())
}

pub fn create_venv_and_build(project_path: &Path) {
    let _ = setup_python_env(project_path);
    let _ = build_wheel(project_path);
}

pub fn move_wheel(build_dir: &Path, cache_dir: &Path) -> io::Result<()> {
    let dist_dir = build_dir.join("dist/").canonicalize();

    match dist_dir {
        Ok(dist_dir) => {
            println!("dist_dir {:?} ", dist_dir.display());

            // Check if dist/ exists
            if !dist_dir.exists() {
                println!("! dist_dir exists");
                return Err(io::Error::new(
                    io::ErrorKind::NotFound,
                    "dist/ directory not found",
                ));
            }

            println!("dist_dir.exists ");

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
                .ok_or_else(|| {
                    io::Error::new(io::ErrorKind::NotFound, "No .whl file found in dist/")
                })?;

            println!("wheel_file {:?} ", wheel_file);

            // Ensure cache dir exists
            // fs::create_dir_all(cache_dir)?;

            // Build the destination path
            let dest_path = cache_dir.join(wheel_file.file_name());

            // Move (rename) the file
            fs::rename(wheel_file.path(), &dest_path)?;

            println!(
                "wheel_file {:?} dest_path {:?}",
                wheel_file.path().to_str(),
                &dest_path.to_str()
            );
        }
        Err(e) => {
            eprintln!("Error occurred: {}", e);
        }
    }

    Ok(())
}

pub fn install_wheel(
    venv_path: &Path,
    wheel_path: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    // Path to the virtual env's Python executable
    let mut python_executable = venv_path.join("bin").join("python");
    if cfg!(target_os = "windows") {
        python_executable = venv_path.join("Scripts").join("python.exe"); // Windows
    }
    println!("python_executable {}", python_executable.display());
    println!("wheel_path.to_str() {}", wheel_path.to_str().unwrap());
    // Run: python -m pip install /path/to/wheel.whl
    let status = Command::new(python_executable)
        .args(["-m", "pip", "install", wheel_path.to_str().unwrap()])
        .status()
        .expect("pip install failed");

    println!("status {}", status.code().expect("FAIL"));

    if !status.success() {
        println!("Failed to install wheel {}", wheel_path.display());
        return Err("Failed to install wheel".into());
    } else {
        println!("python_executable {} installed", wheel_path.display());
    }

    Ok(())
}

pub fn create_python_env(project_path: &Path) {
    let _ = setup_python_env(project_path);
}
