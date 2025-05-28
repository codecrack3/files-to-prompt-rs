use clap::arg;
use clap::Parser;
use rayon::prelude::*;
use regex::Regex;
use std::fs;
use std::io::{self, BufWriter, Write};
use std::path::{Path, PathBuf};
use tera::{Context, Tera};

use files_to_prompt::model::Config;
use files_to_prompt::traverse::traverse;

use rust_embed::Embed;

#[derive(Embed)]
#[folder = "templates/"]
struct Assets;

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value = ".")]
    dir_path: String,
    #[arg(short = 'e', long, default_value = "txt,py,c,go,rs,java,js,html,ts")]
    file_patterns: String,
    #[arg(short, long, default_value = "")]
    output_file: String,
    #[arg(short, long)]
    lines: Option<i32>,
    #[arg(short = 'c', long)]
    clean_input_enabled: bool,
    #[arg(short, long, value_delimiter = ',')]
    files: Option<Vec<String>>,
    #[arg(long, help = "Ignore files: <file1>,<file2>,..., we will ignore .gitignore, .git by default.")]
    ignore_files: Option<String>,
    #[arg(long, help = "Ignore folders: <folder1>,<folder2>,..., we will ignore .git, .idea, node_modules by default.")]
    ignore_folders: Option<String>,

    #[arg(
        long,
        default_value = "default",
        help = "Template file to render the output."
    )]
    template: String,

    #[arg(long, help = "List all available templates.")]
    list_templates: bool,

    #[arg(short = 't', long, help = "Path to the template file.")]
    path_template: Option<String>,
}

const IGNORED_FILES: &[&str] = &["gitignore", "git"];
const IGNORED_FOLDERS: &[&str] = &[".git", ".idea", "node_modules"];

#[derive(Debug)]
struct AppError(String);

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for AppError {}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError(err.to_string())
    }
}

impl From<tera::Error> for AppError {
    fn from(err: tera::Error) -> Self {
        AppError(err.to_string())
    }
}

impl From<std::str::Utf8Error> for AppError {
    fn from(err: std::str::Utf8Error) -> Self {
        AppError(err.to_string())
    }
}

impl From<regex::Error> for AppError {
    fn from(err: regex::Error) -> Self {
        AppError(err.to_string())
    }
}

impl From<String> for AppError {
    fn from(err: String) -> Self {
        AppError(err)
    }
}

type Result<T> = std::result::Result<T, AppError>;

fn list_all_templates() -> Result<()> {
    println!("Available templates:");
    
    // Embedded templates
    for file in Assets::iter() {
        if let Some(file_name) = file.as_ref().split('.').next() {
            println!("  {}", file_name);
        }
    }

    // Custom templates folder
    let templates_path = Path::new("templates");
    if templates_path.exists() {
        println!("\nCustom templates:");
        for entry in fs::read_dir(templates_path)? {
            let entry = entry?;
            let path = entry.path();
            if let Some(file_name) = path.file_stem().and_then(|s| s.to_str()) {
                println!("  {}", file_name);
            }
        }
    }

    Ok(())
}

fn find_template_file(name_template: &str) -> Result<String> {
    // Check if it's a direct file path
    if Path::new(name_template).exists() {
        return Ok(fs::read_to_string(name_template)?);
    }

    // Check embedded templates
    for file in Assets::iter() {
        if let Some(file_name) = file.as_ref().split('.').next() {
            if file_name == name_template {
                let file_data = Assets::get(file.as_ref())
                    .ok_or_else(|| AppError("Failed to get embedded template".to_string()))?;
                return Ok(std::str::from_utf8(&file_data.data)?.to_string());
            }
        }
    }

    // Check custom templates folder
    let template_path = Path::new("templates")
        .join(name_template)
        .with_extension("f2p");
    
    if template_path.exists() {
        return Ok(fs::read_to_string(template_path)?);
    }

    Err(AppError(format!("Template '{}' not found", name_template)))
}

fn parse_comma_separated(input: &str) -> Vec<String> {
    input
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

fn main() -> Result<()> {
    let args = Args::parse();

    if args.list_templates {
        return list_all_templates();
    }

    let file_patterns = parse_comma_separated(&args.file_patterns);
    
    let mut ignored_files: Vec<String> = IGNORED_FILES.iter().map(|&s| s.to_string()).collect();
    let mut ignored_folders: Vec<String> = IGNORED_FOLDERS.iter().map(|&s| s.to_string()).collect();

    if let Some(ref ignore_files) = args.ignore_files {
        ignored_files.extend(parse_comma_separated(ignore_files));
    }

    if let Some(ref ignore_folders) = args.ignore_folders {
        ignored_folders.extend(parse_comma_separated(ignore_folders));
    }

    let config = Config {
        dir_path: args.dir_path,
        file_patterns,
        output_file: args.output_file,
        lines: args.lines,
        clean_input_enabled: Some(args.clean_input_enabled),
        files: args.files,
        ignored_files,
        ignored_folders,
        template: args.template,
        path_template: args.path_template,
    };

    process_files(&config)
}

fn split_content(content: &str, max_lines: i32) -> String {
    if max_lines <= 0 {
        return content.to_string();
    }
    
    content
        .lines()
        .take(max_lines as usize)
        .collect::<Vec<_>>()
        .join("\n")
}

fn render_template(template_name: &str, content: &str) -> Result<String> {
    let template_content = find_template_file(template_name)?;
    
    let mut tera = Tera::default();
    tera.add_raw_template("template", &template_content)?;

    let mut context = Context::new();
    context.insert("code", content);

    Ok(tera.render("template", &context)?)
}

fn process_files(config: &Config) -> Result<()> {
    let files = if let Some(ref file_list) = config.files {
        file_list.iter().map(PathBuf::from).collect()
    } else {
        traverse(
            PathBuf::from(&config.dir_path),
            config.file_patterns.clone(),
            config.clone(),
        )
    };

    let raw_output = collect_output(files, config)?;
    
    let template_name = config.path_template.as_ref().unwrap_or(&config.template);
    let rendered = render_template(template_name, &raw_output)?;

    // Output to stdout
    io::stdout().write_all(rendered.as_bytes())?;

    // Save to file if specified
    if !config.output_file.is_empty() {
        let mut file = BufWriter::new(fs::File::create(&config.output_file)?);
        file.write_all(rendered.as_bytes())?;
        file.flush()?;
    }

    Ok(())
}

fn collect_output(files: Vec<PathBuf>, config: &Config) -> Result<String> {
    let clean_regex = if config.clean_input_enabled.unwrap_or(false) {
        Some(Regex::new(r"[\r\n\t\s]+")?)
    } else {
        None
    };

    let results: Result<Vec<String>> = files
        .par_iter()
        .map(|file| -> Result<String> {
            let mut result = String::new();
            
            let file_name = file.file_name()
                .and_then(|n| n.to_str())
                .ok_or_else(|| AppError("Invalid file name".to_string()))?;
            
            let parent_path = file.parent()
                .map(|p| p.display().to_string())
                .unwrap_or_else(|| ".".to_string());

            result.push_str(&format!("\n{}/{}\n", parent_path, file_name));
            result.push_str(&format!("{:-<1$}\n", "", 3));

            let content = fs::read_to_string(file)?;
            
            let processed_content = if let Some(max_lines) = config.lines {
                split_content(&content, max_lines)
            } else {
                content
            };

            let final_content = if let Some(ref regex) = clean_regex {
                regex.replace_all(&processed_content, " ").to_string()
            } else {
                processed_content
            };

            result.push_str(&final_content);
            result.push_str(&format!("\n{:-<1$}\n", "", 3));
            
            Ok(result)
        })
        .collect();

    Ok(results?.join(""))
}
