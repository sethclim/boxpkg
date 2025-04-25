use std::process::Command;

pub fn detect_platform() -> String {
    let info = os_info::get();
    let arch = info.architecture().unwrap_or("unknown");
    println!("Type: {}", info.os_type());
    // println!("Architecture: {}", info.architecture());

    let key = format!("{}-{}", info.os_type(), arch);
    return key;
}

pub fn detect_python_version() -> Option<String> {
    let output = Command::new("python3")
        .arg("--version")
        .output()
        .or_else(|_| Command::new("python").arg("--version").output())
        .ok()?;

    if output.status.success() {
        let version_str = String::from_utf8_lossy(&output.stdout);
        let trimmed = version_str.trim().replace("Python ", "");
        Some(trimmed)
    } else {
        None
    }
}

pub fn detect_abi_tag() -> Option<String> {
    let try_gcc = Command::new("gcc").arg("-dumpfullversion").output().ok();
    if let Some(output) = try_gcc {
        if output.status.success() {
            let ver = String::from_utf8_lossy(&output.stdout).trim().to_string();
            return Some(format!("gcc{}", ver.replace('.', "_")));
        }
    }

    let try_clang = Command::new("clang").arg("--version").output().ok();
    if let Some(output) = try_clang {
        if output.status.success() {
            let version_line = String::from_utf8_lossy(&output.stdout);
            if let Some(ver) = version_line.split_whitespace().nth(2) {
                return Some(format!("clang{}", ver.replace('.', "_")));
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        let try_cl = Command::new("cl").arg("/Bv").output().ok();
        if let Some(output) = try_cl {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if stdout.contains("Microsoft (R) C/C++") {
                return Some("msvc14".to_string()); // crude version, improve as needed
            }
        }
    }

    None
}
