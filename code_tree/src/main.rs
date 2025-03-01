use std::{
    fs::{self, File},
    io::{self, Write},
    path::{Path, PathBuf},
};
use walkdir::WalkDir;

struct Config {
    root_path: PathBuf,
    output_file: PathBuf,
    ignored_dirs: Vec<String>,
    allowed_extensions: Vec<String>,
}

impl Config {
    fn new(root_path: PathBuf) -> Self {
        Config {
            root_path,
            output_file: PathBuf::from("code_output.txt"),
            ignored_dirs: vec![
                ".git".to_string(),
                "node_modules".to_string(),
                "target".to_string(),
                ".idea".to_string(),
                "venv".to_string(),
                "bin".to_string(),
                "obj".to_string(),      // .NET build output
                "Debug".to_string(),    // .NET build output
                "Release".to_string(),  // .NET build output
            ],
            allowed_extensions: vec![
                // Existing extensions
                "rs".to_string(),
                "js".to_string(),
                "py".to_string(),
                "cpp".to_string(),
                "c".to_string(),
                "java".to_string(),
                "go".to_string(),
                "ts".to_string(),
                // .NET related extensions
                "cs".to_string(),       // C# source files
                "csproj".to_string(),   // Project files
                "sln".to_string(),      // Solution files
                "cshtml".to_string(),   // Razor views
                "razor".to_string(),    // Razor components
                "json".to_string(),     // Configuration files
                "xml".to_string(),      // Configuration files
                "config".to_string(),   // Configuration files
                "yml".to_string(),      // YAML config files
                "yaml".to_string(),     // YAML config files
            ],
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
    let args: Vec<String> = std::env::args().collect();
    
    println!("Arguments received: {:?}", args);
    
    if args.len() != 2 {
        println!("Usage: {} <directory_path>", args[0]);
        println!("Example: {} .", args[0]);
        std::process::exit(1);
    }

    let config = Config::new(PathBuf::from(&args[1]));
    println!("Analyzing directory: {}", config.root_path.display());
    println!("Output will be written to: {}", config.output_file.display());
    
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