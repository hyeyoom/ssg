use anyhow::Result;
use std::path::Path;
use tera::{Context, Tera};

pub struct Renderer {
    tera: Tera,
}

impl Renderer {
    pub fn new(templates_dir: &Path) -> Result<Self> {
        let pattern = format!("{}/**/*.html", templates_dir.display());
        let tera = Tera::new(&pattern)?;
        Ok(Self { tera })
    }

    pub fn render(&self, name: &str, ctx: &Context) -> Result<String> {
        Ok(self.tera.render(name, ctx)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn loads_and_renders_template() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("hello.html"), "<p>Hi {{ name }}</p>").unwrap();
        let r = Renderer::new(dir.path()).unwrap();
        let mut ctx = tera::Context::new();
        ctx.insert("name", "world");
        let out = r.render("hello.html", &ctx).unwrap();
        assert_eq!(out, "<p>Hi world</p>");
    }
}
