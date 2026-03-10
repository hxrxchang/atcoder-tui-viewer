use anyhow::{Context, Result, bail};
use scraper::node::Node;
use scraper::{Html, Selector};

use crate::Lang;

pub struct ExtractedTask {
    pub title: String,
    pub limits: Option<String>,
    pub statement_html: String,
}

pub fn extract_task(page_html: &str, lang: Lang) -> Result<ExtractedTask> {
    let doc = Html::parse_document(page_html);
    let task_sel = Selector::parse("#task-statement").expect("valid selector");
    let task = doc
        .select(&task_sel)
        .next()
        .context("`#task-statement` was not found")?;

    let title = extract_title(&doc).unwrap_or_else(|| "AtCoder Task".to_string());
    let limits = extract_limits(&doc, lang);

    let lang_selector = match lang {
        Lang::Ja => "span.lang-ja",
        Lang::En => "span.lang-en",
    };
    let lang_sel = Selector::parse(lang_selector).expect("valid selector");
    if let Some(lang_block) = task.select(&lang_sel).next() {
        return Ok(ExtractedTask {
            title,
            limits,
            statement_html: lang_block.html(),
        });
    }

    let generic_lang_sel = Selector::parse("span.lang").expect("valid selector");
    if let Some(lang_block) = task.select(&generic_lang_sel).next() {
        return Ok(ExtractedTask {
            title,
            limits,
            statement_html: lang_block.html(),
        });
    }

    // Some pages may omit language wrappers. Fallback to full statement area.
    let fallback = task.html();
    if fallback.trim().is_empty() {
        bail!("task statement exists but content is empty");
    }
    Ok(ExtractedTask {
        title,
        limits,
        statement_html: fallback,
    })
}

fn extract_title(doc: &Html) -> Option<String> {
    let title_sel = Selector::parse("span.h2").expect("valid selector");
    let h2 = doc.select(&title_sel).next()?;

    let mut chunks = Vec::new();
    for child in h2.children() {
        if let Node::Text(text) = child.value() {
            let t = normalize_spaces(text);
            if !t.is_empty() {
                chunks.push(t);
            }
        }
    }

    let title = normalize_spaces(&chunks.join(" "));
    if title.is_empty() { None } else { Some(title) }
}

fn extract_limits(doc: &Html, lang: Lang) -> Option<String> {
    let p_sel = Selector::parse("p").expect("valid selector");
    for p in doc.select(&p_sel) {
        let text = normalize_spaces(&p.text().collect::<Vec<_>>().join(" "));
        let is_ja = text.contains("実行時間制限") && text.contains("メモリ制限");
        let is_en = text.contains("Time Limit") && text.contains("Memory Limit");
        if is_ja || is_en {
            return Some(localize_limits(&text, lang));
        }
    }
    None
}

fn localize_limits(text: &str, lang: Lang) -> String {
    match lang {
        Lang::Ja => text
            .replace("Time Limit", "実行時間制限")
            .replace("Memory Limit", "メモリ制限"),
        Lang::En => text
            .replace("実行時間制限", "Time Limit")
            .replace("メモリ制限", "Memory Limit"),
    }
}

fn normalize_spaces(input: &str) -> String {
    input.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURE: &str = r#"
<html><body>
<span class="h2">
  D - 88888888
  <a>解説</a>
</span>
<p>実行時間制限: 2 sec / メモリ制限: 1024 MiB</p>
<div id="task-statement">
  <span class="lang-ja"><section><h3>問題文</h3><p>本文JA</p></section></span>
  <span class="lang-en"><section><h3>Statement</h3><p>BodyEN</p></section></span>
</div>
</body></html>
"#;

    #[test]
    fn extract_ja_block() {
        let out = extract_task(FIXTURE, Lang::Ja).expect("must extract");
        assert_eq!(out.title, "D - 88888888");
        assert_eq!(
            out.limits.as_deref(),
            Some("実行時間制限: 2 sec / メモリ制限: 1024 MiB")
        );
        assert!(out.statement_html.contains("本文JA"));
        assert!(!out.statement_html.contains("BodyEN"));
    }

    #[test]
    fn extract_en_block() {
        let out = extract_task(FIXTURE, Lang::En).expect("must extract");
        assert_eq!(
            out.limits.as_deref(),
            Some("Time Limit: 2 sec / Memory Limit: 1024 MiB")
        );
        assert!(out.statement_html.contains("BodyEN"));
        assert!(!out.statement_html.contains("本文JA"));
    }
}
