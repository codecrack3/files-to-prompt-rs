use clap::arg;
use clap::Parser;
use rayon::prelude::*;
use regex::Regex;
use std::fs;
use std::fs::File;
use std::io::{self, Read, Write};
use std::path::Path;
use std::path::PathBuf;
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
    file_patterns: Vec<String>,
    #[arg(short, long, default_value = "")]
    output_file: String,
    #[arg(short, long, default_value = "-1")]
    lines: Option<i32>,
    #[arg(short = 'c', long, default_value = "true")]
    clean_input_enabled: Option<bool>,
    #[arg(short, long, default_value = None)]
    files: Option<Vec<String>>,
    #[arg(long, default_value = None, help = "Ignore files: <file1>,<file2>,..., we will ignore .gitignore, .git by default.")]
    ignore_files: Option<String>,
    #[arg(long, default_value = None, help = "Ignore folders: <folder1>,<folder2>,..., we will ignore .git, .idea, node_modules by default.")]
    ignore_folders: Option<String>,

    #[arg(
        long,
        default_value = "default",
        help = "Template file to render the output."
    )]
    template: String,

    #[arg(long, help = "List all available templates.")]
    list_templates: bool,

    #[arg(short = 't', long, default_value = None, help = "Path to the template file.")]
    path_template: Option<String>,
}

const IGNORED_FILES: [&str; 2] = ["gitignore", "git"];
const IGNORED_FOLDERS: [&str; 3] = [".git", ".idea", "node_modules"];

fn list_all_templates() -> Result<(), std::io::Error> {
    // combine templates folder with the templates from the rust_embed
    // custom folder templates
    // only show name of the files in templates folder and remove the extension
    for file in Assets::iter() {
        // without the extension
        let file_name = file.as_ref().split('.').next().unwrap();
        println!("{}", file_name);
    }

    // templates folder
    let path = Path::new("templates");
    if path.exists() {
        println!("\n[+] Templates in custom templates folder:");

        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();
            let file_name = path.file_name().unwrap().to_str().unwrap();
            // without the extension
            let file_name = file_name.split('.').next().unwrap();
            println!("{}", file_name);
        }
    }

    Ok(())
}

fn find_template_file(name_template: &str) -> Result<String, std::io::Error> {
    // find name in templates in Assets or in the templates folder
    let mut template_content = String::new();
    let mut found = false;

    // if template is path to the file

    if Path::new(name_template).exists() {
        let mut file = File::open(name_template)?;
        file.read_to_string(&mut template_content)?;
        return Ok(template_content);
    }

    for file in Assets::iter() {
        let file_name = file.as_ref().split('.').next().unwrap();
        if file_name == name_template {
            let mut file = Assets::get(file.as_ref()).unwrap();

            template_content = std::str::from_utf8(file.data.as_ref()).unwrap().to_string();

            found = true;
            break;
        }
    }

    if !found {
        let path = Path::new("templates")
            .join(name_template)
            .with_extension("f2p");
        let mut file = File::open(path)?;
        file.read_to_string(&mut template_content)?;
    }

    Ok(template_content)
}

fn main() -> Result<(), std::io::Error> {
    let args = Args::parse();
    let patterns: Vec<String> = args
        .file_patterns
        .iter()
        .map(|x| x.split(',').collect::<Vec<_>>())
        .flatten()
        .map(|x| x.trim().to_string())
        .collect();

    // merge the ignore files and ignore folders with const ignored files and folders

    let mut ignored_files_list: Vec<String> = IGNORED_FILES.iter().map(|x| x.to_string()).collect();
    let mut ignored_folders_list: Vec<String> =
        IGNORED_FOLDERS.iter().map(|x| x.to_string()).collect();

    if args.ignore_files != None && args.ignore_files.is_some() {
        ignored_files_list.extend(
            args.ignore_files
                .as_ref()
                .unwrap()
                .split(',')
                .map(|x| x.trim().to_string())
                .collect::<Vec<_>>(),
        );
    }

    if args.ignore_folders != None && args.ignore_folders.is_some() {
        ignored_folders_list.extend(
            args.ignore_folders
                .as_ref()
                .unwrap()
                .split(',')
                .map(|x| x.trim().to_string())
                .collect::<Vec<_>>(),
        );
    }


    if args.list_templates {
        return list_all_templates();
    }

    let config = Config {
        dir_path: args.dir_path,
        file_patterns: patterns,
        output_file: args.output_file,
        lines: args.lines,
        clean_input_enabled: args.clean_input_enabled,
        files: args.files,
        ignored_files: ignored_files_list,
        ignored_folders: ignored_folders_list,
        template: args.template,
        path_template: args.path_template,
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

fn render_template(template_name: &str, content: &str) -> Result<String, tera::Error> {
    // Find the template file
    let template_content = find_template_file(template_name)?;

    // print!("{}", template_content);

    // Initialize Tera
    let mut tera = Tera::default();
    tera.add_raw_template("template", &template_content)?;

    // Create a context and add variables
    let mut context = Context::new();
    context.insert("code", content);

    // Render the template
    let rendered = tera.render("template", &context)?;
    // println!("{}", rendered);
    Ok(rendered)
}

fn process_files(config: &Config) -> Result<(), std::io::Error> {
    // println!("{:?}", config);

    // ignore the output file if mode is stdout
    let mut output: Box<dyn Write> = Box::new(io::stdout());

    if !config.output_file.is_empty() {
        output = Box::new(io::BufWriter::new(fs::File::create(&config.output_file)?));
    }

    // deprecated but i will keep it for now
    // if config.files != None && config.files.is_some() {
    //     for file in config.files.as_ref().unwrap() {
    //         let path = Path::new(file);
    //         write_output(&mut output, &path.to_path_buf(), &config)?;
    //     }
    //     return Ok(());
    // }

    if config.files != None && config.files.is_some() {
        let files = config.files.as_ref().unwrap();
        let raw_text_output =
            collect_output(files.iter().map(|x| PathBuf::from(x)).collect(), &config)?;
        let rendered = render_template(&config.template, raw_text_output.as_str());
        println!("{}", rendered.unwrap());
        return Ok(());
    }

    // traversal the directory and get all files using threading
    let files = traverse(
        Path::new(&config.dir_path).to_path_buf(),
        config.file_patterns.clone(),
        config.clone(),
    );

    // deprecated but i will keep it for now
    // for file in files {
    //     let file_name = file.file_name().unwrap().to_str().unwrap();
    //     let file_name_ext = file_name.split('.').last().unwrap();

    //     if config.file_patterns.contains(&file_name_ext.to_string()) {
    //         write_output(&mut output, &file, &config)?;
    //     }
    // }

    let raw_text_output = collect_output(files, &config)?;

    let rendered: Result<String, tera::Error>;
    // check if using specific path template
    if !config.path_template.is_none() {
        rendered = render_template(&config.path_template.as_ref().unwrap(), raw_text_output.as_str());
    }
    else {
        rendered = render_template(&config.template, raw_text_output.as_str());
    }

    let rendered_clone = rendered.unwrap().clone(); // Clone the value of rendered

    io::stdout().write_all(rendered_clone.as_bytes())?; // Print the cloned value of rendered

    // save output to file
    if !config.output_file.is_empty() {
        fs::write(&config.output_file, rendered_clone)?; // Use the cloned value of rendered
    }

    Ok(())
}

#[warn(dead_code)]
fn write_output(
    output: &mut dyn Write,
    file: &PathBuf,
    config: &Config,
) -> Result<(), std::io::Error> {
    let file_name = file.file_name().unwrap().to_str().unwrap();
    writeln!(
        output,
        "\n{}/{}",
        file.parent().unwrap().display(),
        file_name
    )?;
    // write '=' 16 times
    // writeln!(output, "=")?;
    writeln!(output, "{:`<1$}", "", 3)?;

    let content = fs::read(&file)?;
    let mut content_str = String::from_utf8_lossy(&content);
    if config.lines.unwrap() != -1 {
        content_str = split_content(&content_str, config.lines.unwrap()).into();
    }
    if config.clean_input_enabled.unwrap() {
        let pattern = Regex::new(r"[\r\n\t\s]+").unwrap();
        let replaced_content_str = pattern.replace_all(&content_str, " ");
        writeln!(output, "{}", replaced_content_str)?;
    } else {
        writeln!(output, "{}", content_str)?;
    }
    writeln!(output, "{:`<1$}", "", 3)?;
    Ok(())
}

fn collect_output(files: Vec<PathBuf>, config: &Config) -> Result<String, std::io::Error> {
    let mut output = String::new();
    for file in files {
        let file_name = file.file_name().unwrap().to_str().unwrap();
        output.push_str(&format!(
            "\n{}/{}\n",
            file.parent().unwrap().display(),
            file_name
        ));
        output.push_str(&format!("{:`<1$}\n", "", 3));

        let content = fs::read(&file)?;
        let mut content_str = String::from_utf8_lossy(&content);
        if config.lines.unwrap() != -1 {
            content_str = split_content(&content_str, config.lines.unwrap()).into();
        }
        if config.clean_input_enabled.unwrap() {
            let pattern = Regex::new(r"[\r\n\t\s]+").unwrap();
            let replaced_content_str = pattern.replace_all(&content_str, " ");
            output.push_str(&format!("{}", replaced_content_str));
        } else {
            output.push_str(&format!("{}", content_str));
        }
        output.push_str(&format!("\n{:`<1$}", "", 3));
    }
    Ok(output)
}
