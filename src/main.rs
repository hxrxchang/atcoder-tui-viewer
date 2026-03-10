mod convert;
mod extract;
mod fetch;
mod tui;

use std::panic::{AssertUnwindSafe, catch_unwind};

use anyhow::{Context, Result};
use clap::{Parser, ValueEnum};

#[derive(Debug, Clone, Copy, ValueEnum)]
enum Lang {
    Ja,
    En,
}

#[derive(Debug, Parser)]
#[command(name = "atv", version, about = "AtCoder problem viewer in terminal")]
struct Cli {
    /// AtCoder task URL.
    url: String,
    /// Language to extract.
    #[arg(long, value_enum, default_value_t = Lang::Ja)]
    lang: Lang,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    run_view(&cli.url, cli.lang)?;
    Ok(())
}

fn run_view(url: &str, lang: Lang) -> Result<()> {
    let html = fetch::fetch_html(url).with_context(|| format!("failed to fetch URL: {url}"))?;
    let task = extract::extract_task(&html, lang).context("failed to extract task statement")?;
    let body_markdown =
        convert::to_pretty_markdown(&task.statement_html).context("failed to convert HTML")?;
    let markdown = assemble_markdown(&task.title, task.limits.as_deref(), &body_markdown);

    let rendered = catch_unwind(AssertUnwindSafe(|| tui::run(&markdown)));
    match rendered {
        Ok(result) => result,
        Err(_) => anyhow::bail!("application panicked while rendering TUI"),
    }
}

fn assemble_markdown(title: &str, limits: Option<&str>, body: &str) -> String {
    let mut out = String::new();
    out.push_str("# ");
    out.push_str(title);
    out.push_str("\n\n");
    if let Some(l) = limits {
        out.push_str(l);
        out.push_str("\n\n");
    }
    out.push_str(body.trim());
    out
}
