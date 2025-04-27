use std::env;
use std::path::Path;
use std::process::Command;

pub fn setup_python_env(project_path: &Path) -> Result<(), String> {
    // Define the path to the Python virtual environment
    let venv_dir = project_path.join("venv");

    // Check if the virtual environment already exists
    if !&venv_dir.exists() {
        // Create the virtual environment if it doesn't exist
        println!("Creating virtual environment...");
        let status = Command::new("python")
            .arg("-m")
            .arg("venv")
            .arg(&venv_dir)
            .status()
            .map_err(|e| format!("Failed to create virtual environment: {}", e))?;

        if !status.success() {
            return Err("Failed to create virtual environment".to_string());
        }
    }

    // Activate the virtual environment (platform-specific)
    println!("Activating virtual environment...");
    let activate_script = if cfg!(target_os = "windows") {
        &venv_dir.join("Scripts").join("activate.bat")
    } else {
        &venv_dir.join("bin").join("activate")
    };

    // Use `source` for Unix or `activate.bat` for Windows
    let activate_status = Command::new("sh")
        .arg("-c")
        .arg(format!("source {}", &activate_script.display()))
        .status()
        .map_err(|e| format!("Failed to activate environment: {}", e))?;

    if !activate_status.success() {
        return Err("Failed to activate virtual environment".to_string());
    }

    let mut python_command = Command::new("python");

    if cfg!(target_os = "windows") {
        python_command
            .arg("-m")
            .arg("pip")
            .arg("install")
            .arg("setuptools")
            .arg("wheel")
            .arg("build")
            // Set the environment variable for the virtual environment
            .env("VIRTUAL_ENV", venv_dir.display().to_string()) // This is the path to the venv
            // Update PATH to include the virtual environment's Scripts directory
            .env(
                "PATH",
                format!(
                    "{}/Scripts;{}",
                    &venv_dir.display().to_string(),
                    std::env::var("PATH").expect("string")
                ),
            );
    } else {
        python_command
            .arg("-m")
            .arg("pip")
            .arg("install")
            .arg("setuptools")
            .arg("wheel")
            .arg("build")
            // Set the environment variable for the virtual environment
            .env("VIRTUAL_ENV", &venv_dir.display().to_string()) // This is the path to the venv
            // Update PATH to include the virtual environment's binary directory
            .env(
                "PATH",
                format!(
                    "{}/bin:{}",
                    &venv_dir.display().to_string(),
                    std::env::var("PATH").expect("string")
                ),
            ); // Unix-based systems
    }

    let install_status = python_command
        .status()
        .map_err(|e| format!("Failed to install dependencies: {}", e))?;

    if !install_status.success() {
        return Err("Failed to install dependencies in the virtual environment".to_string());
    }

    Ok(())
}

pub fn build_wheel(project_path: &Path) -> Result<(), String> {
    let dis = env::current_dir();

    match dis {
        Ok(dis) => {
            println!("current_dir! {}", dis.display());
            // Ensure the project path exists
            if !project_path.exists() {
                return Err(format!("Directory not found: {}", project_path.display()));
            }

            // Change to the project directory
            env::set_current_dir(project_path).map_err(|e| e.to_string())?;

            // Run the build command for the Python wheel
            let status = Command::new("python")
                .arg("-m")
                .arg("build")
                .status()
                .map_err(|e| format!("Failed to run wheel build: {}", e))?;

            if !status.success() {
                return Err("Wheel build failed".to_string());
            }

            println!("Wheel build successful, check the 'dist/' directory.");

            // Change to the project directory
            println!("current_dir! {}", dis.display());
            env::set_current_dir(dis).map_err(|e| e.to_string())?;
        }
        Err(e) => {
            println!("Failed to check current_dir: {}", e);
        }
    }

    Ok(())
}
