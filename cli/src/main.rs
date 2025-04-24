use std::path::PathBuf;

use clap::{Parser, Subcommand};

use serde::Serialize;
// use std::fs::File;
// use std::io::Write;

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
        #[arg(short, long)]
        path: bool,
    },
    Install {
        #[arg(short, long)]
        path: bool,
    },
}

fn main() {
    let cli = Cli::parse();

    // You can check the value provided by positional arguments, or option arguments
    if let Some(name) = cli.name.as_deref() {
        println!("Value for name: {name}");
    }

    if let Some(config_path) = cli.config.as_deref() {
        println!("Value for config: {}", config_path.display());
    }

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
                let _ = init();
            } else {
                println!("Not initializing...");
            }
        }
        Some(Commands::Add { path }) => {
            if *path {
                println!("Adding...");
            } else {
                println!("Not adding...");
            }
        }
        Some(Commands::Install { path }) => {
            if *path {
                println!("installing...");
            } else {
                println!("Not installing...");
            }
        }
        None => {}
    }

    // Continued program logic goes here...
    #[derive(Serialize)]
    struct Project {
        name: String,
        version: String,
    }

    #[derive(Serialize)]
    struct Manifest {
        project: Project,
        dependencies: std::collections::HashMap<String, String>,
    }

    // #[derive(Serialize)]
    // struct Config {
    //     name: String,
    //     version: String,
    //     authors: Vec<String>,
    //     debug: bool,
    // }

    fn init() -> Result<(), Box<dyn std::error::Error>> {
        println!("initializing...");

        let project = Project {
            name: "myproject".into(),
            version: "0.1.0".into(),
        };

        let manifest = Manifest {
            project,
            dependencies: std::collections::HashMap::new(),
        };

        // Serialize to a TOML string
        let toml_string = toml::to_string_pretty(&manifest)?;

        // Write to a file
        std::fs::write("./temp/mypkg.toml", toml_string)?;

        println!("mypkg.toml written successfully.");

        std::fs::create_dir("./temp/.box")?;
        std::fs::create_dir_all("./temp/.box/cache")?;

        Ok(())
    }
}
