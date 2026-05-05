use crate::config::Config;
use crate::content::PostKind;
use crate::{feed, render, scanner, sitemap};
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

pub fn run(project_root: &Path) -> Result<()> {
    println!("loading config.toml");
    let config = Config::load(&project_root.join("config.toml"))
        .with_context(|| "loading config.toml")?;
    let templates_dir = project_root.join("templates");
    let content_dir = project_root.join("content");
    let static_dir = project_root.join("static");
    let public_dir = project_root.join("public");

    if public_dir.exists() {
        println!("clearing public/");
        fs::remove_dir_all(&public_dir)?;
    }
    fs::create_dir_all(&public_dir)?;

    println!("loading templates");
    let renderer = render::Renderer::new(&templates_dir)?;
    println!("scanning content/");
    let posts = scanner::scan(&content_dir)?;
    let site = render::Site { config: &config };

    let mut articles: Vec<&_> = posts
        .iter()
        .filter(|p| p.kind == PostKind::Article)
        .collect();
    articles.sort_by(|a, b| b.frontmatter.date.cmp(&a.frontmatter.date));
    println!(
        "  found {} article(s), {} page(s)",
        articles.len(),
        posts.len() - articles.len()
    );

    println!("rendering index");
    let index_html = render::render_index(&renderer, &site, &articles)?;
    fs::write(public_dir.join("index.html"), index_html)?;

    for post in &articles {
        println!("rendering post: {}", post.slug);
        let dir = public_dir.join("posts").join(&post.slug);
        fs::create_dir_all(&dir)?;
        let html = render::render_post(&renderer, &site, post)?;
        fs::write(dir.join("index.html"), html)?;
    }

    if let Some(about) = posts
        .iter()
        .find(|p| p.kind == PostKind::Page && p.slug == "about")
    {
        println!("rendering about");
        let dir = public_dir.join("about");
        fs::create_dir_all(&dir)?;
        let html = render::render_about(&renderer, &site, about)?;
        fs::write(dir.join("index.html"), html)?;
    }

    let base = config.base_url.trim_end_matches('/');
    let mut urls = vec![sitemap::SitemapUrl {
        loc: format!("{}/", base),
        lastmod: None,
    }];
    for p in &articles {
        urls.push(sitemap::SitemapUrl {
            loc: format!("{}/posts/{}/", base, p.slug),
            lastmod: Some(p.frontmatter.date.clone()),
        });
    }
    if posts
        .iter()
        .any(|p| p.kind == PostKind::Page && p.slug == "about")
    {
        urls.push(sitemap::SitemapUrl {
            loc: format!("{}/about/", base),
            lastmod: None,
        });
    }
    println!("writing sitemap.xml ({} urls)", urls.len());
    fs::write(public_dir.join("sitemap.xml"), sitemap::build_sitemap(&urls))?;
    println!("writing robots.txt");
    fs::write(
        public_dir.join("robots.txt"),
        sitemap::build_robots(&config.base_url),
    )?;
    println!("writing rss.xml ({} items)", articles.len().min(20));
    fs::write(public_dir.join("rss.xml"), feed::build_rss(&config, &articles))?;

    if static_dir.exists() {
        println!("copying static/");
        copy_dir_recursive(&static_dir, &public_dir)?;
    }

    Ok(())
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<()> {
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let path = entry.path();
        let dest = dst.join(entry.file_name());
        if path.is_dir() {
            fs::create_dir_all(&dest)?;
            copy_dir_recursive(&path, &dest)?;
        } else {
            fs::copy(&path, &dest)?;
        }
    }
    Ok(())
}
