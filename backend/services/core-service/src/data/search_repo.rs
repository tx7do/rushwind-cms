//! The search repository — the SQL ILIKE fallback over posts
//! (search_repo.go), with the summary/html helpers it rides on.

use sea_orm::{
    ConnectionTrait, DatabaseConnection,
};

use crate::state::StatusResult;


/// The search fallback: (post_id, language, title) hits.
pub async fn search_hits(
    db: &DatabaseConnection,
    keyword: &str,
    language: &str,
    limit: i64,
    offset: i64,
) -> StatusResult<Vec<(i64, String, String)>> {
    let rows = db
        .query_all_raw(sea_orm::Statement::from_sql_and_values(
            sea_orm::DatabaseBackend::Postgres,
            r#"SELECT p.id AS post_id, t.language_code AS lang, t.title AS title
               FROM posts p
               JOIN post_translations t ON t.post_id = p.id
               WHERE p.status = 'POST_STATUS_PUBLISHED'
                 AND (t.title ILIKE $1 OR t.content ILIKE $1 OR t.summary ILIKE $1)
                 AND ($2 = '' OR t.language_code = $2)
               ORDER BY p.created_at DESC
               LIMIT $3 OFFSET $4"#,
            [
                keyword.to_string().into(),
                language.to_string().into(),
                limit.into(),
                offset.into(),
            ],
        ))
        .await
        .map_err(crate::db_status)?;
    Ok(rows
        .into_iter()
        .map(|r| {
            (
                r.try_get::<i64>("", "post_id").unwrap_or(0),
                r.try_get::<String>("", "lang").unwrap_or_default(),
                r.try_get::<String>("", "title").unwrap_or_default(),
            )
        })
        .collect())
}

/// Strip HTML tags exactly like the `<[^>]+>` regex (a lone `<` without
/// a closing `>` stays literal).
fn strip_html(html: &str) -> String {
    let mut out = String::with_capacity(html.len());
    let mut rest = html;
    while let Some(start) = rest.find('<') {
        out.push_str(&rest[..start]);
        match rest[start..].find('>') {
            Some(end) if end > 1 => rest = &rest[start + end + 1..],
            _ => {
                out.push('<');
                rest = &rest[start + 1..];
            }
        }
    }
    out.push_str(rest);
    out
}

/// The rule-based excerpt (GenerateSummaryByRule with maxLength=100,
/// bySentence=true): single-line plain text, cut at the last
/// sentence-end punctuation within the first 100 chars.
pub(crate) fn generate_summary(content: &str) -> String {
    let plain = collapse_spaces(&strip_html(content));
    let plain = plain.trim();
    if plain.is_empty() {
        return "暂无摘要".to_string();
    }
    let runes: Vec<char> = plain.chars().collect();
    let mut summary: String = if runes.len() <= 100 {
        plain.to_string()
    } else {
        let truncated = &runes[..100];
        let last_end = truncated
            .iter()
            .rposition(|c| matches!(c, '。' | '！' | '？' | '；' | '.' | '!' | '?' | ';'))
            .map_or(100, |i| i + 1);
        truncated[..last_end].iter().collect()
    };
    if summary.chars().count() < runes.len() {
        summary.push_str("...");
    }
    summary
}

/// `\s+` (Go's whitespace class) collapses to a single space.
fn collapse_spaces(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut in_space = false;
    for c in s.chars() {
        if matches!(c, ' ' | '\t' | '\n' | '\r' | '\u{0C}') {
            if !in_space {
                out.push(' ');
                in_space = true;
            }
        } else {
            out.push(c);
            in_space = false;
        }
    }
    out
}

/// RawChars: the rune count of the tag-stripped content.
pub(crate) fn raw_chars(content: &str) -> i64 {
    strip_html(content).chars().count() as i64
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── strip_html ──────────────────────────────────────────────────
    // The `<[^>]+>` port: leftmost matches, a lone `<` without a later
    // `>` stays literal. Mirrors the Go summary package's stripping.

    #[test]
    fn strip_html_removes_simple_and_nested_tags() {
        assert_eq!(strip_html("<p>Hello World</p>"), "Hello World");
        assert_eq!(
            strip_html("<div><p>Test</p><span>Content</span></div>"),
            "TestContent"
        );
        assert_eq!(
            strip_html("<div><p>This is <strong>bold</strong> text</p></div>"),
            "This is bold text"
        );
    }

    #[test]
    fn strip_html_removes_tags_with_attributes_and_self_closing() {
        assert_eq!(
            strip_html(r#"<p class="content" id="main">Hello</p>"#),
            "Hello"
        );
        assert_eq!(
            strip_html("Text<br/>More text<hr/>Final"),
            "TextMore textFinal"
        );
    }

    #[test]
    fn strip_html_keeps_lone_lt_literal() {
        // `<` without a closing `>` never forms a match (the regex
        // needs `[^>]+>`).
        assert_eq!(strip_html("a < b"), "a < b");
        assert_eq!(strip_html("a<"), "a<");
        // `<>` is empty between the brackets — no match, kept verbatim.
        assert_eq!(strip_html("<>"), "<>");
    }

    #[test]
    fn strip_html_matches_leftmost_span() {
        // `< b and 3<4` is one `[^>]+` run — the whole span vanishes.
        assert_eq!(strip_html("a < b and 3<4>5"), "a 5");
        assert_eq!(strip_html("<a<b>"), "");
        assert_eq!(strip_html("3<4>5"), "35");
        assert_eq!(strip_html(""), "");
    }

    // ── collapse_spaces ─────────────────────────────────────────────

    #[test]
    fn collapse_spaces_runs_whitespace_to_single_space() {
        assert_eq!(
            collapse_spaces("Hello    World    Test"),
            "Hello World Test"
        );
        assert_eq!(
            collapse_spaces("Hello\n\nWorld\t\tTest"),
            "Hello World Test"
        );
        assert_eq!(collapse_spaces(" \t\n\r\u{0C} mix "), " mix ");
        assert_eq!(collapse_spaces("nochange"), "nochange");
        assert_eq!(collapse_spaces(""), "");
    }

    // ── generate_summary ────────────────────────────────────────────
    // GenerateSummaryByRule(content, 100, true): the fixed 100-char
    // sentence-cut form. Mirrors the Go summary_test.go cases whose
    // expectations survive the fixed parameters.

    #[test]
    fn generate_summary_strips_and_collapses() {
        assert_eq!(generate_summary("<p>Hello World</p>"), "Hello World");
        assert_eq!(
            generate_summary("<div><p>Test</p><span>Content</span></div>"),
            "TestContent"
        );
        assert_eq!(
            generate_summary(r#"<p class="content" id="main">Hello</p>"#),
            "Hello"
        );
        assert_eq!(
            generate_summary("Text<br/>More text<hr/>Final"),
            "TextMore textFinal"
        );
        assert_eq!(
            generate_summary("Hello    World    Test"),
            "Hello World Test"
        );
        assert_eq!(
            generate_summary("Hello\n\nWorld\t\tTest"),
            "Hello World Test"
        );
        assert_eq!(generate_summary("   Hello World   "), "Hello World");
        assert_eq!(
            generate_summary("<p>  Hello   \n  World  </p>"),
            "Hello World"
        );
    }

    #[test]
    fn generate_summary_empty_falls_back_to_placeholder() {
        assert_eq!(generate_summary(""), "暂无摘要");
        assert_eq!(generate_summary("   \n\t  "), "暂无摘要");
        assert_eq!(
            generate_summary("<p></p><div></div><span></span>"),
            "暂无摘要"
        );
    }

    #[test]
    fn generate_summary_short_text_passes_through_without_ellipsis() {
        assert_eq!(
            generate_summary("This is a sentence."),
            "This is a sentence."
        );
        assert_eq!(
            generate_summary("<p>Vue3 暗黑模式教程是前端开发的重要知识点。</p>"),
            "Vue3 暗黑模式教程是前端开发的重要知识点。"
        );
        assert_eq!(
            generate_summary("<p>Vue3 <strong>暗黑模式</strong>教程。通过 CSS 变量实现。</p>"),
            "Vue3 暗黑模式教程。通过 CSS 变量实现。"
        );
        assert_eq!(
            generate_summary("一二三四五，六七八九十。"),
            "一二三四五，六七八九十。"
        );
    }

    #[test]
    fn generate_summary_exact_100_chars_gets_no_ellipsis() {
        let text: String = "a".repeat(100);
        assert_eq!(generate_summary(&text), text);
    }

    #[test]
    fn generate_summary_over_100_without_punctuation_hard_cuts() {
        // Mirrors the reference's "very long content" case at the fixed
        // 100-char window.
        let text = format!("a{}", "b".repeat(100));
        let want = format!("a{}", "b".repeat(99)) + "...";
        assert_eq!(generate_summary(&text), want);
    }

    #[test]
    fn generate_summary_cuts_at_last_sentence_end_within_window() {
        // 162 runes: the 100-char window keeps both 。, so the cut is
        // 62 runes + ellipsis.
        let text = format!(
            "{}。{}。{}",
            "一".repeat(30),
            "二".repeat(30),
            "三".repeat(100)
        );
        let want = format!("{}。{}。...", "一".repeat(30), "二".repeat(30));
        assert_eq!(generate_summary(&text), want);

        // The window's last 。 lands mid-text; the tail after it drops.
        let text = format!("{}。{}", "一".repeat(60), "x".repeat(60));
        let want = format!("{}。...", "一".repeat(60));
        assert_eq!(generate_summary(&text), want);
    }

    #[test]
    fn generate_summary_sentence_end_past_window_is_ignored() {
        // The only 。 sits beyond the 100-char window — hard cut.
        let text = format!("{}。{}", "x".repeat(100), "。尾");
        let want = format!("{}...", "x".repeat(100));
        assert_eq!(generate_summary(&text), want);
    }

    #[test]
    fn generate_summary_counts_ascii_punctuation_as_sentence_end() {
        // '.' within the window wins as the last sentence end.
        let text = format!("{} {} {}", "a".repeat(50), "b. c", "d".repeat(60));
        assert_eq!(
            generate_summary(&text),
            format!("{} b.", "a".repeat(50)) + "..."
        );
    }

    // ── raw_chars ───────────────────────────────────────────────────

    #[test]
    fn raw_chars_counts_stripped_runes() {
        assert_eq!(raw_chars(""), 0);
        assert_eq!(raw_chars("<p></p>"), 0);
        assert_eq!(raw_chars("<p>Hello</p>"), 5);
        assert_eq!(raw_chars("<div><p>Test</p><span>Content</span></div>"), 11);
        // Astral planes count per char, not per byte.
        assert_eq!(raw_chars("Hello 👋"), 7);
        assert_eq!(raw_chars("你好"), 2);
    }
}
