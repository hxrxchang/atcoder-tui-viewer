use anyhow::{Context, Result};
use reqwest::Url;
use scraper::node::Node;
use scraper::{ElementRef, Html, Selector};

pub fn to_pretty_markdown(statement_html: &str) -> Result<String> {
    let wrapped = format!(r#"<div id="atv-root">{statement_html}</div>"#);
    let doc = Html::parse_document(&wrapped);
    let root_sel = Selector::parse("#atv-root").expect("valid selector");
    let root = doc
        .select(&root_sel)
        .next()
        .context("failed to parse statement fragment")?;

    let mut out = Vec::<String>::new();
    for child in root.children() {
        if let Some(el) = ElementRef::wrap(child) {
            process_element(&el, &mut out);
        }
    }

    let mut rendered = squash_blank_lines(out.join("\n"));
    if rendered.ends_with("\n---\n") {
        rendered.truncate(rendered.len() - 5);
    }
    Ok(rendered.trim().to_string())
}

fn process_element(el: &ElementRef<'_>, out: &mut Vec<String>) {
    match tag_name(el).as_str() {
        "section" => process_section(el, out),
        "h3" => push_heading(text_content(el), out),
        "p" => push_paragraph(render_inline(el), out),
        "ul" => push_unordered_list(el, out),
        "ol" => push_ordered_list(el, out),
        "pre" => push_code_block(pre_text(el), out),
        "img" => push_paragraph(render_img_markdown(el), out),
        "div" | "span" | "article" => {
            for child in el.children() {
                if let Some(child_el) = ElementRef::wrap(child) {
                    process_element(&child_el, out);
                }
            }
        }
        _ => {
            let text = render_inline(el);
            if !text.is_empty() {
                push_paragraph(text, out);
            }
        }
    }
}

fn process_section(section: &ElementRef<'_>, out: &mut Vec<String>) {
    let mut had_content = false;
    for child in section.children() {
        if let Some(child_el) = ElementRef::wrap(child) {
            if tag_name(&child_el) == "h3" {
                push_heading(text_content(&child_el), out);
                had_content = true;
                continue;
            }
            let before = out.len();
            process_element(&child_el, out);
            if out.len() != before {
                had_content = true;
            }
        }
    }
    if had_content {
        out.push("---".to_string());
        out.push(String::new());
    }
}

fn push_heading(raw: String, out: &mut Vec<String>) {
    let heading = cleanup_heading(raw);
    if heading.is_empty() {
        return;
    }
    out.push(format!("## {heading}"));
    out.push(String::new());
}

fn push_paragraph(text: String, out: &mut Vec<String>) {
    if text.is_empty() {
        return;
    }
    out.push(text);
    out.push(String::new());
}

fn push_unordered_list(el: &ElementRef<'_>, out: &mut Vec<String>) {
    for child in el.children() {
        if let Some(li) = ElementRef::wrap(child) {
            if tag_name(&li) == "li" {
                let line = render_inline(&li);
                if !line.is_empty() {
                    out.push(format!("- {line}"));
                }
            }
        }
    }
    out.push(String::new());
}

fn push_ordered_list(el: &ElementRef<'_>, out: &mut Vec<String>) {
    let mut i = 1usize;
    for child in el.children() {
        if let Some(li) = ElementRef::wrap(child) {
            if tag_name(&li) == "li" {
                let line = render_inline(&li);
                if !line.is_empty() {
                    out.push(format!("{i}. {line}"));
                    i += 1;
                }
            }
        }
    }
    out.push(String::new());
}

fn push_code_block(raw: String, out: &mut Vec<String>) {
    let body = raw.trim_end_matches('\n');
    if body.trim().is_empty() {
        return;
    }
    out.push("```text".to_string());
    out.push(body.to_string());
    out.push("```".to_string());
    out.push(String::new());
}

fn tag_name(el: &ElementRef<'_>) -> String {
    el.value().name.local.to_string()
}

fn text_content(el: &ElementRef<'_>) -> String {
    normalize_spaces(&el.text().collect::<Vec<_>>().join(" "))
}

fn pre_text(el: &ElementRef<'_>) -> String {
    el.text().collect::<Vec<_>>().join("")
}

fn render_inline(el: &ElementRef<'_>) -> String {
    let mut chunks = Vec::new();
    for child in el.children() {
        match child.value() {
            Node::Text(t) => chunks.push(t.to_string()),
            Node::Element(_) => {
                if let Some(child_el) = ElementRef::wrap(child) {
                    match tag_name(&child_el).as_str() {
                        "img" => chunks.push(render_img_markdown(&child_el)),
                        "br" => chunks.push("\n".to_string()),
                        _ => chunks.push(render_inline(&child_el)),
                    }
                }
            }
            _ => {}
        }
    }
    normalize_inline_text(&chunks.join(" "))
}

fn render_img_markdown(el: &ElementRef<'_>) -> String {
    let alt = el
        .value()
        .attr("alt")
        .map(normalize_spaces)
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "image".to_string());
    let src = el.value().attr("src").unwrap_or("");
    if src.is_empty() {
        format!("![{alt}](image-src-missing)")
    } else {
        format!("![{alt}]({})", resolve_url(src))
    }
}

fn resolve_url(src: &str) -> String {
    if src.starts_with("http://") || src.starts_with("https://") {
        return src.to_string();
    }
    if src.starts_with("//") {
        return format!("https:{src}");
    }
    if src.starts_with('/') {
        return format!("https://atcoder.jp{src}");
    }
    if let Ok(base) = Url::parse("https://atcoder.jp/") {
        if let Ok(joined) = base.join(src) {
            return joined.to_string();
        }
    }
    src.to_string()
}

fn cleanup_heading(raw: String) -> String {
    let cleaned = normalize_spaces(&raw);
    cleaned
        .strip_suffix(" Copy")
        .map_or(cleaned.clone(), ToString::to_string)
}

fn normalize_spaces(input: &str) -> String {
    let s = input.split_whitespace().collect::<Vec<_>>().join(" ");
    prettify_tex(&s)
}

fn normalize_inline_text(input: &str) -> String {
    let joined = input
        .split('\n')
        .map(normalize_spaces)
        .collect::<Vec<_>>()
        .join("\n");
    prettify_tex(joined.trim())
}

fn squash_blank_lines(input: String) -> String {
    let mut out = Vec::new();
    let mut previous_blank = false;
    for line in input.lines() {
        let is_blank = line.trim().is_empty();
        if is_blank && previous_blank {
            continue;
        }
        out.push(line.to_string());
        previous_blank = is_blank;
    }
    out.join("\n")
}

fn prettify_tex(input: &str) -> String {
    let mut s = input.to_string();
    s = replace_frac(&s);
    s = s
        .replace(r"\(", "")
        .replace(r"\)", "")
        .replace(r"\[", "")
        .replace(r"\]", "")
        .replace(r"\left", "")
        .replace(r"\right", "")
        .replace(r"\leq", " <= ")
        .replace(r"\geq", " >= ")
        .replace(r"\neq", " != ")
        .replace(r"\times", " * ")
        .replace(r"\cdot", " * ")
        .replace(r"\min", "min")
        .replace(r"\max", "max")
        .replace(r"\,", " ");
    s = normalize_plain_spaces(&s);
    s = s
        .replace("<=", "≤")
        .replace(">=", "≥")
        .replace("!=", "≠")
        .replace('*', "×");
    normalize_plain_spaces(&s)
}

fn replace_frac(input: &str) -> String {
    let mut s = input.to_string();
    let token = r"\frac{";
    while let Some(start) = s.find(token) {
        let num_start = start + token.len();
        let Some((num_end, num)) = parse_braced(&s, num_start) else {
            break;
        };
        if num_end + 1 >= s.len() || !s[num_end + 1..].starts_with('{') {
            break;
        }
        let den_start = num_end + 2;
        let Some((den_end, den)) = parse_braced(&s, den_start) else {
            break;
        };
        let replacement = format!("({})/({})", prettify_tex(num), prettify_tex(den));
        s.replace_range(start..=den_end, &replacement);
    }
    s
}

fn parse_braced(s: &str, content_start: usize) -> Option<(usize, &str)> {
    let mut depth = 1usize;
    let mut i = content_start;
    while i < s.len() {
        let ch = s[i..].chars().next()?;
        match ch {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return Some((i, &s[content_start..i]));
                }
            }
            _ => {}
        }
        i += ch.len_utf8();
    }
    None
}

fn normalize_plain_spaces(input: &str) -> String {
    input.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn convert_sections_and_samples() {
        let html = r#"
<span class="lang-ja">
  <section>
    <h3>問題文</h3>
    <p>本文です。</p>
    <ul><li>1 ≤ N</li><li>N は整数</li></ul>
  </section>
  <section>
    <h3>入力例 1 <span>Copy</span></h3>
    <pre>5</pre>
  </section>
</span>
"#;
        let md = to_pretty_markdown(html).expect("convert should succeed");
        assert!(md.contains("## 問題文"));
        assert!(md.contains("- 1 ≤ N"));
        assert!(md.contains("## 入力例 1"));
        assert!(md.contains("```text\n5\n```"));
        assert!(!md.contains("Copy"));
    }

    #[test]
    fn prettify_tex_math_expressions() {
        let html = r#"
<span class="lang-ja">
  <section>
    <h3>制約</h3>
    <ul>
      <li>2\leq N \leq 2\times 10^5</li>
      <li>N-1\leq M \leq \min\left(\frac{N(N-1)}{2}, 2\times 10^5\right)</li>
      <li>i\neq j</li>
    </ul>
  </section>
</span>
"#;
        let md = to_pretty_markdown(html).expect("convert should succeed");
        assert!(md.contains("2 ≤ N ≤ 2 × 10^5"));
        assert!(md.contains("N-1 ≤ M ≤ min((N(N-1))/(2), 2 × 10^5)"));
        assert!(md.contains("i ≠ j"));
    }

    #[test]
    fn keep_image_as_markdown() {
        let html = r#"
<span class="lang-ja">
  <section>
    <h3>図</h3>
    <p>説明 <img src="/img/abc.png" alt="sample fig"></p>
  </section>
</span>
"#;
        let md = to_pretty_markdown(html).expect("convert should succeed");
        assert!(md.contains("![sample fig](https://atcoder.jp/img/abc.png)"));
    }
}
