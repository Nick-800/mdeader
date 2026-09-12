use pulldown_cmark::{Alignment as CmarkAlignment, CodeBlockKind, Event, HeadingLevel, Options, Parser, Tag, TagEnd};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeadingItem {
    pub level: u8,
    pub title: String,
    pub anchor_id: String,
    pub index: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Alignment {
    None,
    Left,
    Center,
    Right,
}

impl From<CmarkAlignment> for Alignment {
    fn from(a: CmarkAlignment) -> Self {
        match a {
            CmarkAlignment::None => Alignment::None,
            CmarkAlignment::Left => Alignment::Left,
            CmarkAlignment::Center => Alignment::Center,
            CmarkAlignment::Right => Alignment::Right,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum InlineSpan {
    Text(String),
    Bold(Vec<InlineSpan>),
    Italic(Vec<InlineSpan>),
    Strikethrough(Vec<InlineSpan>),
    Code(String),
    Link {
        url: String,
        title: String,
        text: Vec<InlineSpan>,
    },
    Image {
        url: String,
        title: String,
        alt: String,
    },
    SoftBreak,
    HardBreak,
}

impl InlineSpan {
    pub fn plain_text(&self) -> String {
        match self {
            InlineSpan::Text(s) => s.clone(),
            InlineSpan::Bold(children)
            | InlineSpan::Italic(children)
            | InlineSpan::Strikethrough(children) => {
                children.iter().map(|c| c.plain_text()).collect::<Vec<_>>().join("")
            }
            InlineSpan::Code(s) => s.clone(),
            InlineSpan::Link { text, .. } => {
                text.iter().map(|c| c.plain_text()).collect::<Vec<_>>().join("")
            }
            InlineSpan::Image { alt, .. } => alt.clone(),
            InlineSpan::SoftBreak => " ".to_string(),
            InlineSpan::HardBreak => "\n".to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TableCell {
    pub spans: Vec<InlineSpan>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TableRow {
    pub cells: Vec<TableCell>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ListItem {
    pub checkbox: Option<bool>,
    pub children: Vec<DocNode>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlertKind {
    Note,
    Tip,
    Important,
    Warning,
    Caution,
}

impl AlertKind {
    #[allow(dead_code)]
    pub fn title(&self) -> &'static str {
        match self {
            AlertKind::Note => "Note",
            AlertKind::Tip => "Tip",
            AlertKind::Important => "Important",
            AlertKind::Warning => "Warning",
            AlertKind::Caution => "Caution",
        }
    }

    pub fn tag_label(&self) -> &'static str {
        match self {
            AlertKind::Note => "NOTE",
            AlertKind::Tip => "TIP",
            AlertKind::Important => "IMPORTANT",
            AlertKind::Warning => "WARNING",
            AlertKind::Caution => "CAUTION",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum DocNode {
    Heading {
        level: u8,
        id: String,
        text: String,
        spans: Vec<InlineSpan>,
        heading_index: usize,
    },
    Paragraph(Vec<InlineSpan>),
    CodeBlock {
        lang: String,
        code: String,
    },
    BlockQuote(Vec<DocNode>),
    Alert {
        kind: AlertKind,
        children: Vec<DocNode>,
    },
    List {
        ordered: bool,
        start_num: u64,
        items: Vec<ListItem>,
    },
    Table {
        alignments: Vec<Alignment>,
        header: TableRow,
        rows: Vec<TableRow>,
    },
    Rule,
    Html(String),
}

#[derive(Debug, Clone, Default)]
pub struct DocStats {
    pub word_count: usize,
    pub char_count: usize,
    pub line_count: usize,
    pub reading_time_mins: usize,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct SearchMatch {
    pub line_index: usize,
    pub char_offset: usize,
    pub length: usize,
    pub preview: String,
}

#[derive(Debug, Clone)]
pub struct Document {
    pub raw: String,
    pub nodes: Vec<DocNode>,
    pub headings: Vec<HeadingItem>,
    pub stats: DocStats,
}

impl Document {
    pub fn parse(raw: &str) -> Self {
        let stats = Self::compute_stats(raw);
        let mut headings = Vec::new();
        let mut nodes = Vec::new();

        let mut options = Options::empty();
        options.insert(Options::ENABLE_TABLES);
        options.insert(Options::ENABLE_TASKLISTS);
        options.insert(Options::ENABLE_STRIKETHROUGH);
        options.insert(Options::ENABLE_HEADING_ATTRIBUTES);
        options.insert(Options::ENABLE_SMART_PUNCTUATION);

        let parser = Parser::new_ext(raw, options);
        let events: Vec<Event> = parser.collect();

        let mut idx = 0;
        let mut heading_counter = 0;

        while idx < events.len() {
            let (node, new_idx) = Self::parse_block(&events, idx, &mut headings, &mut heading_counter);
            if let Some(n) = node {
                nodes.push(n);
            }
            if new_idx <= idx {
                idx += 1;
            } else {
                idx = new_idx;
            }
        }

        Self {
            raw: raw.to_string(),
            nodes,
            headings,
            stats,
        }
    }

    fn compute_stats(raw: &str) -> DocStats {
        let char_count = raw.chars().count();
        let line_count = raw.lines().count();
        let word_count = raw.split_whitespace().count();
        let reading_time_mins = if word_count == 0 {
            0
        } else {
            (word_count + 199) / 200
        };

        DocStats {
            word_count,
            char_count,
            line_count,
            reading_time_mins,
        }
    }

    fn parse_block<'a>(
        events: &'a [Event<'a>],
        mut idx: usize,
        headings: &mut Vec<HeadingItem>,
        heading_counter: &mut usize,
    ) -> (Option<DocNode>, usize) {
        if idx >= events.len() {
            return (None, idx);
        }

        match &events[idx] {
            Event::Start(Tag::Heading { level, id, .. }) => {
                let lvl = match level {
                    HeadingLevel::H1 => 1,
                    HeadingLevel::H2 => 2,
                    HeadingLevel::H3 => 3,
                    HeadingLevel::H4 => 4,
                    HeadingLevel::H5 => 5,
                    HeadingLevel::H6 => 6,
                };
                let custom_id = id.as_ref().map(|s| s.to_string());
                idx += 1;

                let mut spans = Vec::new();
                while idx < events.len() {
                    match &events[idx] {
                        Event::End(TagEnd::Heading(_)) => {
                            idx += 1;
                            break;
                        }
                        _ => {
                            let (span, next_idx) = Self::parse_inline(events, idx);
                            if let Some(s) = span {
                                spans.push(s);
                            }
                            if next_idx <= idx {
                                idx += 1;
                            } else {
                                idx = next_idx;
                            }
                        }
                    }
                }

                let plain_text = spans.iter().map(|s| s.plain_text()).collect::<Vec<_>>().join("");
                let h_idx = *heading_counter;
                *heading_counter += 1;

                let anchor_id = custom_id.unwrap_or_else(|| {
                    let slug = plain_text
                        .to_lowercase()
                        .chars()
                        .map(|c| if c.is_alphanumeric() || c == '-' { c } else { '-' })
                        .collect::<String>();
                    format!("heading-{}-{}", h_idx, slug.trim_matches('-'))
                });

                headings.push(HeadingItem {
                    level: lvl,
                    title: plain_text.clone(),
                    anchor_id: anchor_id.clone(),
                    index: h_idx,
                });

                (
                    Some(DocNode::Heading {
                        level: lvl,
                        id: anchor_id,
                        text: plain_text,
                        spans,
                        heading_index: h_idx,
                    }),
                    idx,
                )
            }

            Event::Start(Tag::Paragraph) => {
                idx += 1;
                let mut spans = Vec::new();
                while idx < events.len() {
                    match &events[idx] {
                        Event::End(TagEnd::Paragraph) => {
                            idx += 1;
                            break;
                        }
                        _ => {
                            let (span, next_idx) = Self::parse_inline(events, idx);
                            if let Some(s) = span {
                                spans.push(s);
                            }
                            if next_idx <= idx {
                                idx += 1;
                            } else {
                                idx = next_idx;
                            }
                        }
                    }
                }
                (Some(DocNode::Paragraph(spans)), idx)
            }

            Event::Start(Tag::CodeBlock(kind)) => {
                let lang = match kind {
                    CodeBlockKind::Fenced(l) => l.to_string(),
                    CodeBlockKind::Indented => String::new(),
                };
                idx += 1;
                let mut code = String::new();
                while idx < events.len() {
                    match &events[idx] {
                        Event::End(TagEnd::CodeBlock) => {
                            idx += 1;
                            break;
                        }
                        Event::Text(t) => {
                            code.push_str(t);
                            idx += 1;
                        }
                        _ => {
                            idx += 1;
                        }
                    }
                }
                (Some(DocNode::CodeBlock { lang, code }), idx)
            }

            Event::Start(Tag::BlockQuote(_)) => {
                idx += 1;
                let mut children = Vec::new();
                while idx < events.len() {
                    match &events[idx] {
                        Event::End(TagEnd::BlockQuote) => {
                            idx += 1;
                            break;
                        }
                        _ => {
                            let (child, next_idx) = Self::parse_block(events, idx, headings, heading_counter);
                            if let Some(c) = child {
                                children.push(c);
                            }
                            if next_idx <= idx {
                                idx += 1;
                            } else {
                                idx = next_idx;
                            }
                        }
                    }
                }
                if let Some(alert) = Self::try_parse_alert(&mut children) {
                    (Some(alert), idx)
                } else {
                    (Some(DocNode::BlockQuote(children)), idx)
                }
            }

            Event::Start(Tag::List(first_num)) => {
                let ordered = first_num.is_some();
                let start_num = first_num.unwrap_or(1);
                idx += 1;
                let mut items = Vec::new();

                while idx < events.len() {
                    match &events[idx] {
                        Event::End(TagEnd::List(_)) => {
                            idx += 1;
                            break;
                        }
                        Event::Start(Tag::Item) => {
                            idx += 1;
                            let mut checkbox = None;
                            let mut item_children = Vec::new();

                            while idx < events.len() {
                                match &events[idx] {
                                    Event::End(TagEnd::Item) => {
                                        idx += 1;
                                        break;
                                    }
                                    Event::TaskListMarker(checked) => {
                                        checkbox = Some(*checked);
                                        idx += 1;
                                    }
                                    _ => {
                                        let (child, next_idx) =
                                            Self::parse_block(events, idx, headings, heading_counter);
                                        if let Some(c) = child {
                                            item_children.push(c);
                                        }
                                        if next_idx <= idx {
                                            idx += 1;
                                        } else {
                                            idx = next_idx;
                                        }
                                    }
                                }
                            }

                            items.push(ListItem {
                                checkbox,
                                children: item_children,
                            });
                        }
                        _ => {
                            idx += 1;
                        }
                    }
                }

                (Some(DocNode::List { ordered, start_num, items }), idx)
            }

            Event::Start(Tag::Table(alignments)) => {
                let table_alignments: Vec<Alignment> =
                    alignments.iter().map(|a| Alignment::from(*a)).collect();
                idx += 1;

                let mut header = TableRow { cells: Vec::new() };
                let mut rows = Vec::new();

                while idx < events.len() {
                    match &events[idx] {
                        Event::End(TagEnd::Table) => {
                            idx += 1;
                            break;
                        }
                        Event::Start(Tag::TableHead) => {
                            idx += 1;
                            let mut cells = Vec::new();
                            while idx < events.len() {
                                match &events[idx] {
                                    Event::End(TagEnd::TableHead) => {
                                        idx += 1;
                                        break;
                                    }
                                    Event::Start(Tag::TableCell) => {
                                        idx += 1;
                                        let mut spans = Vec::new();
                                        while idx < events.len() {
                                            match &events[idx] {
                                                Event::End(TagEnd::TableCell) => {
                                                    idx += 1;
                                                    break;
                                                }
                                                _ => {
                                                    let (s, next) = Self::parse_inline(events, idx);
                                                    if let Some(sp) = s {
                                                        spans.push(sp);
                                                    }
                                                    if next <= idx {
                                                        idx += 1;
                                                    } else {
                                                        idx = next;
                                                    }
                                                }
                                            }
                                        }
                                        cells.push(TableCell { spans });
                                    }
                                    _ => idx += 1,
                                }
                            }
                            header = TableRow { cells };
                        }
                        Event::Start(Tag::TableRow) => {
                            idx += 1;
                            let mut cells = Vec::new();
                            while idx < events.len() {
                                match &events[idx] {
                                    Event::End(TagEnd::TableRow) => {
                                        idx += 1;
                                        break;
                                    }
                                    Event::Start(Tag::TableCell) => {
                                        idx += 1;
                                        let mut spans = Vec::new();
                                        while idx < events.len() {
                                            match &events[idx] {
                                                Event::End(TagEnd::TableCell) => {
                                                    idx += 1;
                                                    break;
                                                }
                                                _ => {
                                                    let (s, next) = Self::parse_inline(events, idx);
                                                    if let Some(sp) = s {
                                                        spans.push(sp);
                                                    }
                                                    if next <= idx {
                                                        idx += 1;
                                                    } else {
                                                        idx = next;
                                                    }
                                                }
                                            }
                                        }
                                        cells.push(TableCell { spans });
                                    }
                                    _ => idx += 1,
                                }
                            }
                            rows.push(TableRow { cells });
                        }
                        _ => idx += 1,
                    }
                }

                (
                    Some(DocNode::Table {
                        alignments: table_alignments,
                        header,
                        rows,
                    }),
                    idx,
                )
            }

            Event::Rule => (Some(DocNode::Rule), idx + 1),

            Event::Html(h) => (Some(DocNode::Html(h.to_string())), idx + 1),

            _ => {
                // Inline event at root level -> wrap in paragraph
                let (span, next_idx) = Self::parse_inline(events, idx);
                if let Some(s) = span {
                    (Some(DocNode::Paragraph(vec![s])), next_idx)
                } else {
                    (None, idx + 1)
                }
            }
        }
    }

    fn parse_inline<'a>(events: &'a [Event<'a>], mut idx: usize) -> (Option<InlineSpan>, usize) {
        if idx >= events.len() {
            return (None, idx);
        }

        match &events[idx] {
            Event::Text(t) => (Some(InlineSpan::Text(t.to_string())), idx + 1),
            Event::Code(c) => (Some(InlineSpan::Code(c.to_string())), idx + 1),
            Event::SoftBreak => (Some(InlineSpan::SoftBreak), idx + 1),
            Event::HardBreak => (Some(InlineSpan::HardBreak), idx + 1),

            Event::Start(Tag::Emphasis) => {
                idx += 1;
                let mut inner = Vec::new();
                while idx < events.len() {
                    match &events[idx] {
                        Event::End(TagEnd::Emphasis) => {
                            idx += 1;
                            break;
                        }
                        _ => {
                            let (span, next) = Self::parse_inline(events, idx);
                            if let Some(s) = span {
                                inner.push(s);
                            }
                            if next <= idx {
                                idx += 1;
                            } else {
                                idx = next;
                            }
                        }
                    }
                }
                (Some(InlineSpan::Italic(inner)), idx)
            }

            Event::Start(Tag::Strong) => {
                idx += 1;
                let mut inner = Vec::new();
                while idx < events.len() {
                    match &events[idx] {
                        Event::End(TagEnd::Strong) => {
                            idx += 1;
                            break;
                        }
                        _ => {
                            let (span, next) = Self::parse_inline(events, idx);
                            if let Some(s) = span {
                                inner.push(s);
                            }
                            if next <= idx {
                                idx += 1;
                            } else {
                                idx = next;
                            }
                        }
                    }
                }
                (Some(InlineSpan::Bold(inner)), idx)
            }

            Event::Start(Tag::Strikethrough) => {
                idx += 1;
                let mut inner = Vec::new();
                while idx < events.len() {
                    match &events[idx] {
                        Event::End(TagEnd::Strikethrough) => {
                            idx += 1;
                            break;
                        }
                        _ => {
                            let (span, next) = Self::parse_inline(events, idx);
                            if let Some(s) = span {
                                inner.push(s);
                            }
                            if next <= idx {
                                idx += 1;
                            } else {
                                idx = next;
                            }
                        }
                    }
                }
                (Some(InlineSpan::Strikethrough(inner)), idx)
            }

            Event::Start(Tag::Link { dest_url, title, .. }) => {
                let url = dest_url.to_string();
                let title = title.to_string();
                idx += 1;
                let mut inner = Vec::new();
                while idx < events.len() {
                    match &events[idx] {
                        Event::End(TagEnd::Link) => {
                            idx += 1;
                            break;
                        }
                        _ => {
                            let (span, next) = Self::parse_inline(events, idx);
                            if let Some(s) = span {
                                inner.push(s);
                            }
                            if next <= idx {
                                idx += 1;
                            } else {
                                idx = next;
                            }
                        }
                    }
                }
                (Some(InlineSpan::Link { url, title, text: inner }), idx)
            }

            Event::Start(Tag::Image { dest_url, title, .. }) => {
                let url = dest_url.to_string();
                let title = title.to_string();
                idx += 1;
                let mut alt = String::new();
                while idx < events.len() {
                    match &events[idx] {
                        Event::End(TagEnd::Image) => {
                            idx += 1;
                            break;
                        }
                        Event::Text(t) => {
                            alt.push_str(t);
                            idx += 1;
                        }
                        _ => {
                            idx += 1;
                        }
                    }
                }
                (Some(InlineSpan::Image { url, title, alt }), idx)
            }

            _ => (None, idx + 1),
        }
    }

    fn try_parse_alert(children: &mut Vec<DocNode>) -> Option<DocNode> {
        if children.is_empty() {
            return None;
        }

        let alert_kind = match &mut children[0] {
            DocNode::Paragraph(spans) => {
                if spans.is_empty() {
                    return None;
                }

                let mut kind = None;
                let mut spans_consumed = 0;
                let mut remainder_text = String::new();

                // Check pattern 1: spans[0] is Text and starts with [!TAG]
                if let Some(InlineSpan::Text(t0)) = spans.get(0) {
                    let t0_trim = t0.trim_start();
                    let upper = t0_trim.to_ascii_uppercase();
                    for (k, tag) in [
                        (AlertKind::Note, "[!NOTE]"),
                        (AlertKind::Tip, "[!TIP]"),
                        (AlertKind::Important, "[!IMPORTANT]"),
                        (AlertKind::Warning, "[!WARNING]"),
                        (AlertKind::Caution, "[!CAUTION]"),
                    ] {
                        if upper.starts_with(tag) {
                            kind = Some(k);
                            spans_consumed = 1;
                            remainder_text = t0_trim[tag.len()..].trim_start().to_string();
                            break;
                        }
                    }
                }

                // Check pattern 2: spans are split into Text("["), Text("!TAG"), Text("]...")
                if kind.is_none() && spans.len() >= 3 {
                    if let (Some(InlineSpan::Text(t0)), Some(InlineSpan::Text(t1)), Some(InlineSpan::Text(t2))) =
                        (spans.get(0), spans.get(1), spans.get(2))
                    {
                        if t0.trim_start() == "[" {
                            let tag_upper = t1.trim().to_ascii_uppercase();
                            let detected_kind = match tag_upper.as_str() {
                                "!NOTE" => Some(AlertKind::Note),
                                "!TIP" => Some(AlertKind::Tip),
                                "!IMPORTANT" => Some(AlertKind::Important),
                                "!WARNING" => Some(AlertKind::Warning),
                                "!CAUTION" => Some(AlertKind::Caution),
                                _ => None,
                            };

                            if let Some(k) = detected_kind {
                                let t2_trim = t2.trim_start();
                                if t2_trim.starts_with(']') {
                                    kind = Some(k);
                                    spans_consumed = 3;
                                    remainder_text = t2_trim[1..].trim_start().to_string();
                                }
                            }
                        }
                    }
                }

                let Some(alert_kind) = kind else {
                    return None;
                };

                for _ in 0..spans_consumed {
                    spans.remove(0);
                }

                if !remainder_text.is_empty() {
                    spans.insert(0, InlineSpan::Text(remainder_text));
                } else if let Some(InlineSpan::SoftBreak) | Some(InlineSpan::HardBreak) = spans.first() {
                    spans.remove(0);
                }

                alert_kind
            }
            _ => return None,
        };

        if let DocNode::Paragraph(spans) = &mut children[0] {
            if spans.is_empty() {
                children.remove(0);
            }
        }

        Some(DocNode::Alert {
            kind: alert_kind,
            children: std::mem::take(children),
        })
    }

    pub fn search(&self, query: &str) -> Vec<SearchMatch> {
        let mut results = Vec::new();
        if query.trim().is_empty() {
            return results;
        }

        let query_lower = query.to_lowercase();

        for (line_idx, line) in self.raw.lines().enumerate() {
            let line_lower = line.to_lowercase();
            let mut start = 0;
            while let Some(pos) = line_lower[start..].find(&query_lower) {
                let char_offset = start + pos;
                results.push(SearchMatch {
                    line_index: line_idx,
                    char_offset,
                    length: query.chars().count(),
                    preview: line.to_string(),
                });
                start = char_offset + query_lower.len().max(1);
                if start >= line_lower.len() {
                    break;
                }
            }
        }

        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_heading_parsing() {
        let md = "# Title 1\n\n## Subtitle 2\n\n### Section 3";
        let doc = Document::parse(md);
        assert_eq!(doc.headings.len(), 3);
        assert_eq!(doc.headings[0].level, 1);
        assert_eq!(doc.headings[0].title, "Title 1");
        assert_eq!(doc.headings[1].level, 2);
        assert_eq!(doc.headings[1].title, "Subtitle 2");
        assert_eq!(doc.headings[2].level, 3);
        assert_eq!(doc.headings[2].title, "Section 3");
    }

    #[test]
    fn test_stats() {
        let md = "Hello world! This is a test markdown document.\nSecond line here.";
        let doc = Document::parse(md);
        assert_eq!(doc.stats.word_count, 11);
        assert_eq!(doc.stats.line_count, 2);
        assert_eq!(doc.stats.reading_time_mins, 1);
    }

    #[test]
    fn test_search() {
        let md = "First line with keyword.\nSecond line without.\nThird line with keyword again.";
        let doc = Document::parse(md);
        let matches = doc.search("keyword");
        assert_eq!(matches.len(), 2);
        assert_eq!(matches[0].line_index, 0);
        assert_eq!(matches[1].line_index, 2);
    }

    #[test]
    fn test_code_block() {
        let md = "```rust\nfn main() {}\n```";
        let doc = Document::parse(md);
        assert_eq!(doc.nodes.len(), 1);
        match &doc.nodes[0] {
            DocNode::CodeBlock { lang, code } => {
                assert_eq!(lang, "rust");
                assert_eq!(code.trim(), "fn main() {}");
            }
            _ => panic!("Expected CodeBlock"),
        }
    }

    #[test]
    fn test_alert_parsing() {
        let md = "> [!NOTE]\n> This is an informative note.\n\n> [!WARNING]\n> Cautionary warning message.";
        let doc = Document::parse(md);
        assert_eq!(doc.nodes.len(), 2);

        match &doc.nodes[0] {
            DocNode::Alert { kind, children } => {
                assert_eq!(*kind, AlertKind::Note);
                assert_eq!(children.len(), 1);
            }
            _ => panic!("Expected Alert node for NOTE, got {:?}", doc.nodes[0]),
        }

        match &doc.nodes[1] {
            DocNode::Alert { kind, children } => {
                assert_eq!(*kind, AlertKind::Warning);
                assert_eq!(children.len(), 1);
            }
            _ => panic!("Expected Alert node for WARNING"),
        }
    }
}
