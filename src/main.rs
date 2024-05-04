use std::fs;
use std::io::{self, Write};
use std::path::Path;
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use rayon::prelude::*;



#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value = ".")]
    dir_path: String,
    #[arg(short = 'e', long, default_value = "txt,py,c,go,rs,java,js,html,ts")]
    file_patterns: Vec<String>,
    #[arg(short, long, default_value = "")]
    output_file: String,
    #[arg(short, long, default_value = "stdout")]
    mode: String,
    #[arg(short, long, default_value = "-1")]
    lines: Option<i32>
}
#[derive(Debug)]
struct Config {
    dir_path: String,
    file_patterns: Vec<String>,
    output_file: String,
    mode: String,
    lines: Option<i32>
}

fn main () -> Result<(), std::io::Error> {
    let args = Args::parse();
    let patterns: Vec<String> = args.file_patterns.iter().map(|x| x.split(',').collect::<Vec<_>>()).flatten().map(|x| x.trim().to_string()).collect();
    let config = Config {
        dir_path: args.dir_path,
        file_patterns: patterns,
        output_file: args.output_file,
        mode: args.mode,
        lines: args.lines
    };
    
    
    process_files(&config)?;

    Ok(())
}

fn split_content(content: &str, lines: i32) -> String {
    let mut result = String::new();
    let mut count = 0;
    for line in content.lines() {
        if lines != -1 && count >= lines {
            break;
        }
        result.push_str(line);
        result.push_str("\n");
        count += 1;
    }
    result
}
fn traverse(dir: PathBuf, files_extension: Vec<String>) -> Vec<PathBuf> {
    let mut files = Vec::new();
    let mut dirs = vec![dir];

    while !dirs.is_empty() {
        let (new_dirs, new_files): (Vec<_>, Vec<_>) = dirs.par_iter().map(|dir| {
            let mut f = Vec::new();
            let mut d = Vec::new();
            let entries = fs::read_dir(dir).unwrap();
            for entry in entries {
                let entry = entry.unwrap();
                let path = entry.path();
                if path.is_dir() {
                    d.push(path);
                } else {
                    if let Some(extension) = path.extension() {
                        if files_extension.contains(&extension.to_str().unwrap().to_string()) {
                            f.push(path);
                        }
                    }
                }
            }
            (d, f)
        }).unzip();
        files.extend(new_files.into_iter().flatten());
        dirs = new_dirs.into_iter().flatten().collect();
    }
    files
}

fn process_files(config: &Config) -> Result<(), std::io::Error> {
    // ignore the output file if mode is stdout
    let mut output: Box<dyn Write> = Box::new(io::stdout());

    if !config.output_file.is_empty() {
        output = Box::new(io::BufWriter::new(fs::File::create(&config.output_file)?));
    }

    // traversal the directory and get all files using threading
    let files = traverse(Path::new(&config.dir_path).to_path_buf(), config.file_patterns.clone());
    for file in files {
        let file_name = file.file_name().unwrap().to_str().unwrap();
        let file_name_ext = file_name.split('.').last().unwrap();
        
        if config.file_patterns.contains(&file_name_ext.to_string()) {
            if config.mode == "stdout" {
                println!("{}/{}", file.parent().unwrap().display(), file_name);
                println!("==========");
                let content = fs::read(&file)?;
                let mut content_str = String::from_utf8_lossy(&content);
                if config.lines.unwrap() != -1{
                    content_str = split_content(&content_str, config.lines.unwrap()).into();
                }
                println!("{}", content_str);
                println!("---");
            } else
            {
                writeln!(output, "{}/{}", file.parent().unwrap().display(), file_name)?;
                writeln!(output, "==========")?;
                let content = fs::read(&file)?;
                let mut content_str = String::from_utf8_lossy(&content);
                if config.lines.unwrap() != -1{
                    content_str = split_content(&content_str, config.lines.unwrap()).into();
                }
                writeln!(output, "{}", content_str)?;
                writeln!(output, "---")?;
            }
        }
    }

    Ok(())
}