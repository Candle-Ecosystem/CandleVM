pub mod callstack;
pub mod constants;
pub mod error;
pub mod function;
pub mod heap;
pub mod loader;
pub mod module;
pub mod plugin;
pub mod value;
pub mod globals;
pub mod opcode;
pub mod vm;

mod gc;
mod handlers;

use crate::loader::Loader;
use crate::vm::Vm;
use clap::Parser;
use std::path::PathBuf;
use std::process::exit;
use std::{env, fs};

#[derive(Parser)]
#[command(
    name = "CandleVM",
    bin_name = "candlevm",
    about = concat!("Candle Virtual Machine v", env!("CARGO_PKG_VERSION")),
    version = env!("CARGO_PKG_VERSION"),
)]
struct Cli {
    /// Path to program
    file: String,

    /// Load plugin
    #[arg(short = 'p', long = "plugin", value_name = "PATH")]
    plugins: Vec<PathBuf>,

    /// Debug mode
    #[arg(short = 'd', long = "debug")]
    debug: bool,
}

fn main() {
    let mut cli = Cli::parse();

    load_local_plugins(&mut cli.plugins);

    let path = &cli.file;

    if cli.debug {
        println!("DEBUG: Reading file");
    }

    let data = match fs::read(path) {
        Ok(data) => data,
        Err(err) => {
            eprintln!("Could not read program:\n> {}", err);
            exit(1);
        }
    };

    if cli.debug {
        println!("DEBUG: Read {} bytes!", data.len());
    }

    let loader = Loader::new(&data, cli.plugins, cli.debug);

    if cli.debug {
        println!("DEBUG: Loading program");
    }

    match loader.load() {
        Ok(program) => {
            if cli.debug {
                println!("DEBUG: Program loaded!");
            }

            let mut vm = Vm::new(
                program.bytecode,
                program.modules,
                program.resources,
                cli.debug,
            );

            if cli.debug {
                println!("DEBUG: Starting Virtual Machine");
            }

            let code = vm.run();
            if cli.debug {
                println!("DEBUG: Exiting with code {}", code);
            }
            exit(code as i32);
        }

        Err(err) => {
            eprintln!(
                "Failed to load the program. Please make sure that the program file is not damaged, and that all provided plugins are valid, not damaged, and compatible with your environment (Candle version, operating system, etc.).\n> {}",
                err
            );
            if cli.debug {
                println!("DEBUG: Exiting with code 1");
            }
            exit(1);
        }
    }
}

fn load_local_plugins(plugins: &mut Vec<PathBuf>) {
    let Ok(exe_path) = env::current_exe() else {
        return;
    };

    let Ok(exe_path) = fs::canonicalize(exe_path) else {
        return;
    };

    let Some(exe_dir) = exe_path.parent() else {
        return;
    };

    let plugin_dir = exe_dir.join("plugins");

    let Ok(entries) = fs::read_dir(plugin_dir) else {
        return;
    };

    #[cfg(target_os = "windows")]
    const EXTENSION: &str = "dll";

    #[cfg(target_os = "macos")]
    const EXTENSION: &str = "dylib";

    #[cfg(all(unix, not(target_os = "macos")))]
    const EXTENSION: &str = "so";

    for entry in entries.flatten() {
        let path = entry.path();

        if path.is_file()
            && path.extension().and_then(|ext| ext.to_str()) == Some(EXTENSION)
            && !plugins.contains(&path)
        {
            plugins.push(path);
        }
    }
}
