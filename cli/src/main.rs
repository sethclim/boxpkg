use std::path::Path;
use std::path::PathBuf;

use box_core::create_python_env;
use box_core::install_wheel;
use clap::{Parser, Subcommand};

use serde::Deserialize;
use serde::Serialize;
use std::env;
use toml::de::from_str;

use box_core::{
    create_venv_and_build, download_source, extract_tar_gz, get_build_tuple, get_system_info,
    move_wheel,
};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Optional name to operate on
    name: Option<String>,

    /// Sets a custom config file
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,

    /// Turn debugging information on
    #[arg(short, long, action = clap::ArgAction::Count)]
    debug: u8,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Init {
        #[arg(short, long)]
        path: bool,
    },
    Add {
        name: Option<String>,
    },
    Install {
        #[arg(short, long)]
        path: bool,
    },
}

fn main() {
    let cli = Cli::parse();
    let dis = env::current_dir();

    match dis {
        Ok(dis) => {
            println!("current_dir! {}", dis.display());
        }
        Err(e) => {
            println!("Failed to check current_dir: {}", e);
        }
    }

    // You can check the value provided by positional arguments, or option arguments
    if let Some(name) = cli.name.as_deref() {
        println!("Value for name: {name}");
    }

    if let Some(config_path) = cli.config.as_deref() {
        println!("Value for config: {}", config_path.display());
    }

    let project = Project {
        name: "myproject".into(),
        version: "0.1.0".into(),
    };

    let manifest: Manifest = Manifest {
        project,
        dependencies: std::collections::HashMap::new(),
    };

    let system_info = get_system_info();

    let python: PythonConfig = PythonConfig {
        python_version: system_info.python_version.clone(),
    };

    let lockfile: Lockfile = Lockfile {
        python,
        dependencies: std::collections::HashMap::new(),
    };

    // You can see how many times a particular flag or argument occurred
    // Note, only flags can have multiple occurrences
    // match cli.debug {
    //     0 => println!("Debug mode is off"),
    //     1 => println!("Debug mode is kind of on"),
    //     2 => println!("Debug mode is on"),
    //     _ => println!("Don't be crazy"),
    // }

    // You can check for the existence of subcommands, and if found use their
    // matches just as you would the top level cmd
    match &cli.command {
        Some(Commands::Init { path }) => {
            if *path {
                let _ = init(manifest, lockfile);
            } else {
                println!("Not initializing...");
            }
        }
        Some(Commands::Add { name }) => {
            let _ = add(name, manifest, system_info, lockfile);
        }
        Some(Commands::Install { path }) => {
            if *path {
                install();
            } else {
                println!("Not installing...");
            }
        }
        None => {}
    }

    // Continued program logic goes here...
    #[derive(Serialize, Deserialize, Debug)]
    struct Project {
        name: String,
        version: String,
    }

    #[derive(Serialize, Deserialize, Debug)]
    struct Manifest {
        project: Project,
        dependencies: std::collections::HashMap<String, String>,
    }

    #[derive(Serialize, Deserialize, Debug)]
    struct PythonConfig {
        python_version: String,
    }

    #[derive(Serialize, Deserialize, Debug)]
    struct LockfileDependency {
        version: String,
        path: String,
        hash: String,
    }

    #[derive(Serialize, Deserialize, Debug)]
    struct Lockfile {
        python: PythonConfig,
        dependencies: std::collections::HashMap<String, LockfileDependency>,
    }

    // #[derive(Serialize)]
    // struct Config {
    //     name: String,
    //     version: String,
    //     authors: Vec<String>,
    //     debug: bool,
    // }

    fn init(manifest: Manifest, lockfile: Lockfile) -> Result<(), Box<dyn std::error::Error>> {
        println!("initializing...");

        // Serialize to a TOML string
        let toml_string = toml::to_string_pretty(&manifest)?;
        let lockfile_string = toml::to_string_pretty(&lockfile)?;

        // Write to a file
        std::fs::write("./temp/mypkg.toml", toml_string)?;
        println!("mypkg.toml written successfully.");

        std::fs::write("./temp/box.lock", lockfile_string)?;
        println!("box.lock written successfully.");

        std::fs::create_dir("./temp/.box")?;
        std::fs::create_dir_all("./temp/.box/cache")?;

        Ok(())
    }

    fn add(
        name: &Option<String>,
        mut manifest: Manifest,
        system_info: box_core::SystemEnvironmentInfo,
        mut lockfile: Lockfile,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(name) = name {
            if !matches!(name.as_str(), "lz4" | "lz4-python") {
                return Err("Invalid name".into());
            }

            println!("Adding... {}", name);

            let build_folder = std::path::Path::new("./temp/.box/build/");
            let pkg_build_folder = build_folder.join(name);
            println!("pkg_build_folder: {}", pkg_build_folder.display());

            std::fs::create_dir_all(&pkg_build_folder)?;

            let dl_res = download_source(
                "https://files.pythonhosted.org/packages/source/l/lz4/lz4-4.3.2.tar.gz",
                &build_folder,
            );

            match dl_res {
                Ok(dl_file_path) => {
                    println!(
                        "Download succeeded, file saved at: {}",
                        dl_file_path.display()
                    );
                    let _ = extract_tar_gz(&dl_file_path, &pkg_build_folder.as_path());
                    let pkg_tuple = get_build_tuple(name, "v1.0.0", system_info);

                    println!("Cache key: {}", pkg_tuple.hash_key());

                    let unzipped_folder = dl_file_path
                        .file_name()
                        .and_then(|f| f.to_str())
                        .map(|s| s.replace(".tar.gz", ""));

                    let project_source_folder = if let Some(folder) = unzipped_folder {
                        pkg_build_folder.join(folder)
                    } else {
                        // Handle missing unzipped_folder appropriately
                        panic!("unzipped_folder was None!");
                    };

                    let package_final_path =
                        Path::new("./temp/.box/cache/").join(&pkg_tuple.hash_key());

                    std::fs::create_dir_all(&package_final_path)?;

                    create_venv_and_build(&project_source_folder.as_path());

                    println!("built successfully.");

                    let move_res = move_wheel(
                        project_source_folder.as_path(),
                        package_final_path.as_path(),
                    );

                    match move_res {
                        Ok(path) => {
                            println!("Success! Path is: {}", path.display());
                            manifest
                                .dependencies
                                .insert(name.to_string(), "v1.0.0".to_string());

                            let toml_string = toml::to_string_pretty(&manifest)?;

                            // Write to a file
                            std::fs::write("./temp/mypkg.toml", toml_string)?;

                            let new_dep = LockfileDependency {
                                version: "v1.0.0".into(),
                                path: path.display().to_string(),
                                hash: pkg_tuple.hash_key().to_string(),
                            };

                            lockfile.dependencies.insert(name.to_string(), new_dep);

                            let lockfile_string = toml::to_string_pretty(&lockfile)?;

                            std::fs::write("./temp/box.lock", lockfile_string)?;
                            println!("box.lock updated successfully.");
                        }
                        Err(e) => {
                            println!("Error occurred: {}", e);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error occurred: {}", e);
                }
            }
        }
        Ok(())
    }

    fn install() {
        println!("installing...");

        // Read the TOML file content
        let toml_str = std::fs::read_to_string("./temp/box.lock").expect("Failed to read the file");

        // Parse the TOML string into a Config struct
        let lockfile: Lockfile = from_str(&toml_str).expect("Failed to parse TOML");

        let project_box_path = Path::new("./temp/.box/");
        let _ = create_python_env(&project_box_path);
        let project_box_path_venv = Path::new("./temp/.box/venv/");
        println!("create_python_env finished!");

        for (dep, info) in &lockfile.dependencies {
            println!("Dependency: {} Version: {}", dep, info.version);
            let _ = install_wheel(&project_box_path_venv, Path::new(Path::new(&info.path)));
        }
    }
}
