use chrono::{NaiveDate, NaiveDateTime};
use comrak::{markdown_to_html, ComrakOptions};
use frontmatter_gen::{extract, Frontmatter, Value};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process;
use tera::{Context, Tera};
use walkdir::WalkDir;

fn main() -> io::Result<()> {
    // Argument Parsing
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: program <folder>");
        process::exit(1);
    }
    let folder = PathBuf::from(&args[1]);

    // Initialize site data
    let marmite = match fs::read_to_string("marmite.yaml") {
        Ok(data) => data,
        Err(e) => {
            eprintln!("Unable to read marmite.yaml: {}", e);
            process::exit(1);
        }
    };
    let site: Marmite = match serde_yaml::from_str(&marmite) {
        Ok(site) => site,
        Err(e) => {
            eprintln!("Failed to parse YAML: {}", e);
            process::exit(1);
        }
    };
    let mut site_data = SiteData::new(&site);

    // Walk through the content directory
    for entry in WalkDir::new(folder.join(site_data.site.content_path)) {
        match entry {
            Ok(entry) => {
                let path = entry.path();
                if path.is_file() && path.extension().and_then(|e| e.to_str()) == Some("md") {
                    if let Err(e) = process_file(path, &mut site_data) {
                        eprintln!("Failed to process file {}: {}", path.display(), e);
                    }
                }
            }
            Err(e) => eprintln!("Error reading entry: {}", e),
        }
    }

    // Sort posts by date (newest first)
    site_data.posts.sort_by(|a, b| b.date.cmp(&a.date));
    // Sort pages on title
    site_data.pages.sort_by(|a, b| b.title.cmp(&a.title));

    // Create the output directory
    let output_path = folder.join(&site_data.site.site_path);
    if let Err(e) = fs::create_dir_all(&output_path) {
        eprintln!("Unable to create output directory: {}", e);
        process::exit(1);
    }

    // Initialize Tera templates
    let tera = match Tera::new(format!("{}/**/*", site_data.site.templates_path).as_str()) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("Parsing error(s): {}", e);
            process::exit(1);
        }
    };
    // Render templates
    if let Err(e) = render_templates(&site_data, &tera, &folder.join(&site_data.site.site_path)) {
        eprintln!("Failed to render templates: {}", e);
        process::exit(1);
    }

    // Copy static folder if present
    let static_source = Path::new(site_data.site.static_path);
    let static_destiny = output_path.join(Path::new("static"));
    if static_source.exists() {
        if let Err(e) = fs::create_dir_all(&static_destiny) {
            eprintln!("Unable to create static directory: {}", e);
            process::exit(1);
        }
        for entry in WalkDir::new(&static_source) {
            match entry {
                Ok(entry) => {
                    let static_filename = match entry.path().strip_prefix(&static_source) {
                        Ok(filename) => filename,
                        Err(e) => {
                            eprintln!("Error building static_filename: {}", e);
                            process::exit(1);
                        }
                    };
                    let target_path = static_destiny.join(&static_filename);
                    if entry.file_type().is_dir() {
                        fs::create_dir_all(&target_path)?;
                    } else {
                        fs::copy(entry.path(), target_path)?;
                    }
                }
                Err(e) => eprintln!("Error copying static file: {}", e),
            }
        }
    }
    // TODO: ^same above for content/media folder to site/media

    println!("Site generated at: {}/", site_data.site.site_path);

    Ok(())
}

#[derive(Debug, Deserialize, Clone, Serialize)]
#[serde(rename_all = "lowercase")]
struct Content {
    title: String,
    slug: String,
    html: String,
    tags: Vec<String>,
    date: Option<NaiveDateTime>,
}

#[derive(Serialize)]
struct SiteData<'a> {
    site: &'a Marmite<'a>,
    posts: Vec<Content>,
    pages: Vec<Content>,
}

impl<'a> SiteData<'a> {
    fn new(site: &'a Marmite) -> Self {
        SiteData {
            site,
            posts: Vec::new(),
            pages: Vec::new(),
        }
    }
}

fn parse_front_matter(content: &str) -> Result<(Frontmatter, &str), String> {
    if content.starts_with("---") {
        extract(&content).map_err(|e| e.to_string())
    } else {
        Ok((Frontmatter::new(), content))
    }
}

fn process_file(path: &Path, site_data: &mut SiteData) -> Result<(), String> {
    let file_content = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let (frontmatter, markdown) = parse_front_matter(&file_content)?;
    // TODO: Trim empty first and trailing lines of markdown

    let mut options = ComrakOptions::default();
    options.render.unsafe_ = true; // Allow raw html
    let html = markdown_to_html(markdown, &options);

    let title = get_title(&frontmatter, markdown);
    let tags = get_tags(&frontmatter);
    let slug = get_slug(&frontmatter, &path);
    let date = get_date(&frontmatter, &path);

    let content = Content {
        title,
        slug,
        tags,
        html,
        date,
    };

    if date.is_some() {
        site_data.posts.push(content);
    } else {
        site_data.pages.push(content);
    }
    Ok(())
}

fn get_date(frontmatter: &Frontmatter, path: &Path) -> Option<NaiveDateTime> {
    if let Some(input) = frontmatter.get("date") {
        if let Ok(date) =
            NaiveDateTime::parse_from_str(&input.as_str().unwrap(), "%Y-%m-%d %H:%M:%S")
        {
            return Some(date);
        } else if let Ok(date) =
            NaiveDateTime::parse_from_str(&input.as_str().unwrap(), "%Y-%m-%d %H:%M")
        {
            return Some(date);
        } else if let Ok(date) = NaiveDate::parse_from_str(&input.as_str().unwrap(), "%Y-%m-%d") {
            // Add a default time (00:00:00)
            return date.and_hms_opt(0, 0, 0);
        } else {
            eprintln!(
                "ERROR: Invalid date format {} when parsing {}",
                input.to_string_representation(),
                path.display()
            );
            process::exit(1);
        }
    }
    None
}

fn get_slug<'a>(frontmatter: &'a Frontmatter, path: &'a Path) -> String {
    match frontmatter.get("slug") {
        Some(Value::String(slug)) => slug.to_string(),
        _ => path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap()
            .to_string(),
    }
}

fn get_title<'a>(frontmatter: &'a Frontmatter, html: &'a str) -> String {
    match frontmatter.get("title") {
        Some(Value::String(t)) => t.to_string(),
        _ => html
            .lines()
            .next()
            .unwrap_or("")
            .trim_start_matches("#")
            .trim()
            .to_string(),
    }
}

fn get_tags(frontmatter: &Frontmatter) -> Vec<String> {
    let tags: Vec<String> = match frontmatter.get("tags") {
        Some(Value::Array(tags)) => tags
            .iter()
            .map(Value::to_string)
            .map(|t| t.trim_matches('"').to_string())
            .collect(),
        Some(Value::String(tags)) => tags
            .split(",")
            .map(|t| t.trim())
            .map(String::from)
            .collect(),
        _ => Vec::new(),
    };
    tags
}

fn render_templates(site_data: &SiteData, tera: &Tera, output_dir: &Path) -> Result<(), String> {
    // Build the context of variable that are global on every template
    let mut global_context = Context::new();
    global_context.insert("global", &site_data);
    global_context.insert("site", &site_data.site);
    global_context.insert("menu", &site_data.site.menu);
    global_context.insert("data", &site_data.site.data);

    // Render index.html from list.html template
    let mut list_context = global_context.clone();
    list_context.insert("title", site_data.site.list_title);
    list_context.insert("content_list", &site_data.posts);
    generate_html("list.html", "index.html", tera, &list_context, output_dir)?;

    // Render pages.html from list.html template
    let mut list_context = global_context.clone();
    list_context.insert("title", site_data.site.pages_title);
    list_context.insert("content_list", &site_data.pages);
    generate_html("list.html", "pages.html", tera, &list_context, output_dir)?;

    // TODO: Render tags/index.html to list all tags (from group.html)
    // archive/index.html to list all dates grouped by year/month (from group.html)
    // render each individual /tags/tag.html and /archive/year|month.html (from list.html)

    // Render individual content-slug.html from content.html template
    for content in site_data.posts.iter().chain(&site_data.pages) {
        let mut content_context = global_context.clone();
        content_context.insert("title", &content.title);
        content_context.insert("content", &content);
        generate_html(
            "content.html",
            &format!("{}.html", &content.slug),
            tera,
            &content_context,
            output_dir,
        )?;
    }

    Ok(())
}

fn generate_html(
    template: &str,
    filename: &str,
    tera: &Tera,
    context: &Context,
    output_dir: &Path,
) -> Result<(), String> {
    let rendered = tera.render(template, &context).map_err(|e| e.to_string())?;
    fs::write(output_dir.join(filename), rendered).map_err(|e| e.to_string())?;
    println!("Generated {filename}");
    Ok(())
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
struct Marmite<'a> {
    #[serde(default = "default_name")]
    name: &'a str,
    #[serde(default = "default_tagline")]
    tagline: &'a str,
    #[serde(default = "default_url")]
    url: &'a str,
    #[serde(default = "default_footer")]
    footer: &'a str,
    #[serde(default = "default_pagination")]
    pagination: u32,

    #[serde(default = "default_list_title")]
    list_title: &'a str,
    #[serde(default = "default_pages_title")]
    pages_title: &'a str,
    #[serde(default = "default_tags_title")]
    tags_title: &'a str,
    #[serde(default = "default_archives_title")]
    archives_title: &'a str,

    // To be removed
    #[serde(default = "default_content_path")]
    content_path: &'a str,
    #[serde(default = "default_site_path")]
    site_path: &'a str,
    // to be removed
    #[serde(default = "default_templates_path")]
    templates_path: &'a str,
    #[serde(default = "default_static_path")]
    static_path: &'a str,
    #[serde(default = "default_media_path")]
    media_path: &'a str,

    #[serde(default = "default_card_image")]
    card_image: &'a str,
    #[serde(default = "default_logo_image")]
    logo_image: &'a str,

    #[serde(default = "default_menu")]
    menu: Option<Vec<(String, String)>>,

    #[serde(default = "default_data")]
    data: Option<HashMap<String, String>>,
}

fn default_name() -> &'static str {
    "Home"
}

fn default_tagline() -> &'static str {
    "Site generated from markdown content"
}

fn default_url() -> &'static str {
    "https://example.com"
}

fn default_footer() -> &'static str {
    r#"<a href="https://creativecommons.org/licenses/by-nc-sa/4.0/">CC-BY_NC-SA</a> | Site generated with <a href="https://github.com/rochacbruno/marmite">Marmite</a>"#
}

fn default_pagination() -> u32 {
    10
}

fn default_list_title() -> &'static str {
    "Posts"
}

fn default_tags_title() -> &'static str {
    "Tags"
}

fn default_pages_title() -> &'static str {
    "Pages"
}

fn default_archives_title() -> &'static str {
    "Archive"
}

fn default_site_path() -> &'static str {
    "site"
}

fn default_content_path() -> &'static str {
    "content"
}

fn default_templates_path() -> &'static str {
    "templates"
}

fn default_static_path() -> &'static str {
    "static"
}

fn default_media_path() -> &'static str {
    "content/media"
}

fn default_card_image() -> &'static str {
    ""
}

fn default_logo_image() -> &'static str {
    ""
}

fn default_menu() -> Option<Vec<(String, String)>> {
    vec![
        ("Pages".to_string(), "/pages.html".to_string()),
        // ("Tags".to_string(), "/tags.html".to_string()),
        // ("Archive".to_string(), "/archive.html".to_string()),
    ]
    .into()
}

fn default_data() -> Option<HashMap<String, String>> {
    None
}