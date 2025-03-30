use std::{
    fs::{self, File},
    io::{self, Write},
    path::{Path, PathBuf},
};
use walkdir::WalkDir;
use clap::{Parser, ArgAction};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Root directory to analyze
    #[arg(default_value = ".")]
    root_path: PathBuf,

    /// Output file path
    #[arg(short, long, default_value = "code_output.txt")]
    output: PathBuf,

    /// Directories to ignore during scanning
    #[arg(short, long, default_value_t = String::from(".git,node_modules,target,.idea,venv,bin,obj,Debug,Release"))]
    ignored_dirs: String,

    /// File extensions to include
    #[arg(short, long, default_value_t = String::from("rs,js,py,cpp,c,java,go,ts,cs,csproj,sln,cshtml,razor,json,xml,config,yml,yaml"))]
    extensions: String,

    /// Verbose output
    #[arg(short, long, action = ArgAction::SetTrue)]
    verbose: bool,
}

struct Config {
    root_path: PathBuf,
    output_file: PathBuf,
    ignored_dirs: Vec<String>,
    allowed_extensions: Vec<String>,
    verbose: bool,
}

impl Config {
    fn new(cli: Cli) -> Self {
        Config {
            root_path: cli.root_path,
            output_file: cli.output,
            ignored_dirs: cli.ignored_dirs.split(',').map(|s| s.to_string()).collect(),
            allowed_extensions: cli.extensions.split(',').map(|s| s.to_string()).collect(),
            verbose: cli.verbose,
        }
    }
}

fn process_directory(config: &Config) -> io::Result<()> {
    println!("Creating output file at: {}", config.output_file.display());
    let mut output = File::create(&config.output_file)?;
    
    println!("Processing directory: {}", config.root_path.display());
    
    writeln!(output, "Directory Tree and Code Contents\n")?;
    writeln!(output, "Root Directory: {}\n", config.root_path.display())?;
    
    println!("Generating directory tree...");
    write_directory_tree(&config.root_path, &mut output, &config)?;
    writeln!(output, "\nCode Contents:\n")?;
    
    println!("Writing file contents...");
    write_file_contents(&config.root_path, &mut output, &config)?;
    
    println!("Processing complete!");
    Ok(())
}

fn write_directory_tree(
    root: &Path,
    output: &mut File,
    config: &Config,
) -> io::Result<()> {
    println!("Starting directory tree generation...");
    for entry in WalkDir::new(root)
        .into_iter()
        .filter_entry(|e| !is_ignored(e, &config.ignored_dirs))
    {
        let entry = entry?;
        let path = entry.path();
        println!("Processing path: {}", path.display());
        
        let depth = entry.depth();
        let prefix = "    ".repeat(depth);
        
        let name = path.file_name()
            .unwrap_or_default()
            .to_string_lossy();
            
        writeln!(output, "{}{}{}", prefix, "├── ", name)?;
    }
    println!("Directory tree generation complete!");
    Ok(())
}

fn write_file_contents(
    root: &Path,
    output: &mut File,
    config: &Config,
) -> io::Result<()> {
    println!("Starting file content writing...");
    for entry in WalkDir::new(root)
        .into_iter()
        .filter_entry(|e| !is_ignored(e, &config.ignored_dirs))
    {
        let entry = entry?;
        if !entry.file_type().is_file() {
            continue;
        }

        let path = entry.path();
        println!("Checking file: {}", path.display());
        
        if let Some(extension) = path.extension() {
            if config.allowed_extensions.contains(&extension.to_string_lossy().to_string()) {
                println!("Processing code file: {}", path.display());
                writeln!(output, "\n=== File: {} ===\n", path.display())?;
                
                match fs::read_to_string(path) {
                    Ok(contents) => {
                        writeln!(output, "{}", contents)?;
                        println!("Successfully wrote contents of: {}", path.display());
                    }
                    Err(e) => {
                        println!("Error reading file {}: {}", path.display(), e);
                        writeln!(output, "Error reading file: {}", e)?;
                    }
                }
            }
        }
    }
    println!("File content writing complete!");
    Ok(())
}

fn is_ignored(entry: &walkdir::DirEntry, ignored_dirs: &[String]) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| ignored_dirs.iter().any(|ignored| s == ignored))
        .unwrap_or(false)
}

fn main() -> io::Result<()> {
    let cli = Cli::parse();
    
    let config = Config::new(cli);
    
    if config.verbose {
        println!("Analyzing directory: {}", config.root_path.display());
        println!("Output will be written to: {}", config.output_file.display());
        println!("Ignored directories: {:?}", config.ignored_dirs);
        println!("Allowed extensions: {:?}", config.allowed_extensions);
    }
    
    match process_directory(&config) {
        Ok(()) => {
            println!("Successfully generated code output at: {}", config.output_file.display());
            Ok(())
        }
        Err(e) => {
            eprintln!("Error processing directory: {}", e);
            Err(e)
        }
    }
}