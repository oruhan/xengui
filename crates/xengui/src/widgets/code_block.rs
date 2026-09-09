// SPDX-License-Identifier: Apache-2.0
//! Selectable, syntax-highlighted source code with one-click copying.

use crate::{
    Align, Border, BorderRadius, Button, Color, Column, Edges, FontWeight, Interaction, Label,
    LayoutBox, Length, Overflow, Render, RichText, Row, Style, StyleBuilder, TextSpan,
    VariableIcon, View, Widget, WidgetBase, WidgetId, pct,
};
use smol_str::SmolStr;
use xen_clipboard::Clipboard;

/// A language hint used by [`CodeBlock`]'s built-in lightweight highlighter.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum CodeLanguage {
    /// Rust source code.
    Rust,
    /// JavaScript source code.
    JavaScript,
    /// TypeScript source code.
    TypeScript,
    /// HTML markup.
    Html,
    /// CSS source code.
    Css,
    /// JSON data.
    Json,
    /// TOML configuration.
    Toml,
    /// Shell source code.
    Shell,
    /// Source without language-specific keywords.
    #[default]
    PlainText,
    /// A display name for a language handled by the generic lexer.
    Custom(SmolStr),
}

impl CodeLanguage {
    /// Infers a language from a language name, alias, or common filename.
    pub fn from_name(name: &str) -> Self {
        let normalized = name.trim().to_ascii_lowercase();
        if normalized.ends_with(".rs") {
            return Self::Rust;
        }
        match normalized.as_str() {
            "rust" | "rs" => Self::Rust,
            "javascript" | "js" | "jsx" => Self::JavaScript,
            "typescript" | "ts" | "tsx" => Self::TypeScript,
            "html" | "htm" => Self::Html,
            "css" => Self::Css,
            "json" => Self::Json,
            "toml" | "cargo.toml" => Self::Toml,
            "shell" | "sh" | "bash" | "zsh" | "terminal" => Self::Shell,
            "" | "text" | "txt" | "plain" | "plaintext" => Self::PlainText,
            _ => Self::Custom(SmolStr::new(name)),
        }
    }

    /// Returns the human-readable language name.
    pub fn name(&self) -> &str {
        match self {
            Self::Rust => "Rust",
            Self::JavaScript => "JavaScript",
            Self::TypeScript => "TypeScript",
            Self::Html => "HTML",
            Self::Css => "CSS",
            Self::Json => "JSON",
            Self::Toml => "TOML",
            Self::Shell => "Shell",
            Self::PlainText => "Text",
            Self::Custom(name) => name,
        }
    }
}

impl From<&str> for CodeLanguage {
    fn from(value: &str) -> Self {
        Self::from_name(value)
    }
}

impl From<String> for CodeLanguage {
    fn from(value: String) -> Self {
        Self::from_name(&value)
    }
}

impl From<SmolStr> for CodeLanguage {
    fn from(value: SmolStr) -> Self {
        Self::from_name(&value)
    }
}

/// Colors used by a [`CodeBlock`] and its syntax highlighter.
#[derive(Clone, Debug, PartialEq)]
pub struct CodeBlockTheme {
    /// Main code area background.
    pub background: Color,
    /// Header background.
    pub header_background: Color,
    /// Outer and header divider color.
    pub border: Color,
    /// Default source text color.
    pub text: Color,
    /// Header label color.
    pub label: Color,
    /// Comment token color.
    pub comment: Color,
    /// Keyword token color.
    pub keyword: Color,
    /// String token color.
    pub string: Color,
    /// Numeric token color.
    pub number: Color,
    /// Function token color.
    pub function: Color,
    /// Type token color.
    pub type_name: Color,
    /// Punctuation token color.
    pub punctuation: Color,
    /// Selected source foreground.
    pub selection_text: Color,
    /// Selected source background.
    pub selection_background: Color,
    /// Copy button foreground.
    pub copy_text: Color,
    /// Copy button background.
    pub copy_background: Color,
}

impl Default for CodeBlockTheme {
    fn default() -> Self {
        Self {
            background: Color::NEUTRAL_950,
            header_background: Color::rgb(15, 15, 16),
            border: Color::NEUTRAL_800,
            text: Color::NEUTRAL_100,
            label: Color::NEUTRAL_300,
            comment: Color::rgb(127, 132, 142),
            keyword: Color::rgb(198, 146, 234),
            string: Color::rgb(152, 195, 121),
            number: Color::rgb(209, 154, 102),
            function: Color::rgb(97, 175, 239),
            type_name: Color::rgb(229, 192, 123),
            punctuation: Color::rgb(171, 178, 191),
            selection_text: Color::WHITE,
            selection_background: Color::rgba(48, 112, 208, 180),
            copy_text: Color::NEUTRAL_200,
            copy_background: Color::NEUTRAL_900,
        }
    }
}

/// Function signature for replacing [`CodeBlock`]'s built-in highlighter.
pub type SyntaxHighlighter = fn(&str, &CodeLanguage, &CodeBlockTheme) -> Vec<TextSpan>;

/// A source-code panel with multiline selection, syntax highlighting, and copy support.
pub struct CodeBlock {
    base: WidgetBase,
    layout_box: LayoutBox,
    inner: Vec<Box<dyn Widget>>,
    hooks_id: WidgetId,
    code: SmolStr,
    label: Option<SmolStr>,
    language: CodeLanguage,
    theme: CodeBlockTheme,
    font: SmolStr,
    font_size: Length,
    line_height: Length,
    copy_label: SmolStr,
    show_header: bool,
    show_copy_button: bool,
    highlighter: SyntaxHighlighter,
}

impl CodeBlock {
    /// Creates a code block containing `code`.
    pub fn new(code: impl Into<SmolStr>) -> Self {
        Self {
            base: WidgetBase::new(Interaction::new()),
            layout_box: LayoutBox::default(),
            inner: Vec::new(),
            hooks_id: WidgetId::new_unique(),
            code: code.into(),
            label: None,
            language: CodeLanguage::PlainText,
            theme: CodeBlockTheme::default(),
            font: SmolStr::new("monospace"),
            font_size: Length::px(13.0),
            line_height: Length::px(21.0),
            copy_label: SmolStr::new("Copy"),
            show_header: true,
            show_copy_button: true,
            highlighter: highlight_code,
        }
    }

    /// Sets the source code.
    pub fn code(mut self, code: impl Into<SmolStr>) -> Self {
        self.code = code.into();
        self
    }

    /// Sets the header label. When omitted, the language name is used.
    pub fn label(mut self, label: impl Into<SmolStr>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Selects the language used for highlighting.
    pub fn language(mut self, language: impl Into<CodeLanguage>) -> Self {
        self.language = language.into();
        self
    }

    /// Replaces all built-in colors.
    pub fn code_theme(mut self, theme: CodeBlockTheme) -> Self {
        self.theme = theme;
        self
    }

    /// Replaces all built-in colors.
    pub fn theme(self, theme: CodeBlockTheme) -> Self {
        self.code_theme(theme)
    }

    /// Sets the code font. The default is the generic `monospace` family.
    pub fn code_font(mut self, font: impl Into<SmolStr>) -> Self {
        self.font = font.into();
        self
    }

    /// Sets the source font size.
    pub fn code_font_size(mut self, size: impl Into<Length>) -> Self {
        self.font_size = size.into();
        self
    }

    /// Sets the source line height.
    pub fn code_line_height(mut self, height: impl Into<Length>) -> Self {
        self.line_height = height.into();
        self
    }

    /// Sets the copy button text.
    pub fn copy_label(mut self, label: impl Into<SmolStr>) -> Self {
        self.copy_label = label.into();
        self
    }

    /// Shows or hides the complete header row.
    pub fn show_header(mut self, show: bool) -> Self {
        self.show_header = show;
        self
    }

    /// Shows or hides the copy button while retaining the header label.
    pub fn show_copy_button(mut self, show: bool) -> Self {
        self.show_copy_button = show;
        self
    }

    /// Replaces the built-in syntax highlighter.
    pub fn highlighter(mut self, highlighter: SyntaxHighlighter) -> Self {
        self.highlighter = highlighter;
        self
    }
}

impl Default for CodeBlock {
    fn default() -> Self {
        Self::new("")
    }
}

impl StyleBuilder for CodeBlock {
    fn style_mut(&mut self) -> &mut Style {
        &mut self.base.style
    }

    fn mark_dirty(&mut self) {
        self.base.dirty = true;
    }
}

impl Render for CodeBlock {
    fn render(&self) -> Box<dyn Widget> {
        const BORDER_WIDTH: f32 = 1.0;
        const CONTAINER_RADIUS: f32 = 20.0;
        const INNER_RADIUS: f32 = CONTAINER_RADIUS - BORDER_WIDTH;

        let mut root = Column::new()
            .width(pct!(100.0))
            .background(self.theme.background)
            .border(Border::all(BORDER_WIDTH, self.theme.border).radius(CONTAINER_RADIUS))
            .overflow(Overflow::Hidden, Overflow::Hidden);

        if self.show_header {
            let title = self
                .label
                .clone()
                .unwrap_or_else(|| SmolStr::new(self.language.name()));
            let mut header = Row::new()
                .width(pct!(100.0))
                .align_items(Align::Center)
                .gap(9.0, 0.0)
                .padding(Edges::only(18.0, 10.0, 12.0, 10.0))
                .background(self.theme.header_background)
                .border(
                    Border::bottom(BORDER_WIDTH, self.theme.border)
                        .radius(BorderRadius::top(INNER_RADIUS)),
                )
                .child(
                    VariableIcon::new(xengui_icons::codepoints::CODE_BLOCKS)
                        .size(17.0)
                        .color(self.theme.label),
                )
                .child(
                    Label::new()
                        .label(title)
                        .flex_grow(1.0)
                        .font_size(12.0)
                        .font_weight(FontWeight::SemiBold)
                        .color(self.theme.label),
                );

            if self.show_copy_button {
                let code = self.code.to_string();
                header = header.child(
                    Button::new()
                        .label(self.copy_label.clone())
                        .font_size(11.0)
                        .font_weight(FontWeight::SemiBold)
                        .padding(Edges::symmetric(10.0, 6.0))
                        .color(self.theme.copy_text)
                        .background(self.theme.copy_background)
                        .border(Border::all(1.0, self.theme.border).radius(8.0))
                        .on_click(move |_ctx| {
                            Clipboard::new().set_text(code.clone(), |result| {
                                if let Err(error) = result {
                                    log::error!("CodeBlock copy failed: {error}");
                                }
                            });
                        }),
                );
            }
            root = root.child(header);
        }

        let spans = (self.highlighter)(&self.code, &self.language, &self.theme);
        root = root.child(
            View::new()
                .width(pct!(100.0))
                .padding(Edges::all(20.0))
                .overflow_x(Overflow::Auto)
                .child(
                    RichText::new()
                        .spans(spans)
                        .selectable(true)
                        .preserve_whitespace(true)
                        .wrap(false)
                        .font(self.font.clone())
                        .font_size(self.font_size)
                        .line_height(self.line_height)
                        .color(self.theme.text)
                        .selection_color(self.theme.selection_text)
                        .selection_background(self.theme.selection_background),
                ),
        );
        Box::new(root)
    }
}

crate::impl_composite_widget!(CodeBlock);

fn keyword(language: &CodeLanguage, word: &str) -> bool {
    let words: &[&str] = match language {
        CodeLanguage::Rust => &[
            "as", "async", "await", "break", "const", "continue", "crate", "dyn", "else", "enum",
            "extern", "false", "fn", "for", "if", "impl", "in", "let", "loop", "match", "mod",
            "move", "mut", "pub", "ref", "return", "self", "Self", "static", "struct", "super",
            "trait", "true", "type", "unsafe", "use", "where", "while",
        ],
        CodeLanguage::JavaScript | CodeLanguage::TypeScript => &[
            "async",
            "await",
            "break",
            "case",
            "catch",
            "class",
            "const",
            "continue",
            "debugger",
            "default",
            "delete",
            "do",
            "else",
            "export",
            "extends",
            "false",
            "finally",
            "for",
            "from",
            "function",
            "if",
            "import",
            "in",
            "instanceof",
            "let",
            "new",
            "null",
            "of",
            "return",
            "static",
            "super",
            "switch",
            "this",
            "throw",
            "true",
            "try",
            "typeof",
            "var",
            "void",
            "while",
            "yield",
            "interface",
            "namespace",
            "private",
            "protected",
            "public",
            "readonly",
            "type",
            "implements",
            "declare",
            "keyof",
        ],
        CodeLanguage::Css => &[
            "and",
            "important",
            "not",
            "only",
            "or",
            "screen",
            "supports",
        ],
        CodeLanguage::Shell => &[
            "case", "do", "done", "elif", "else", "esac", "fi", "for", "function", "if", "in",
            "select", "then", "time", "until", "while",
        ],
        CodeLanguage::Json => &["false", "null", "true"],
        _ => &[],
    };
    words.contains(&word)
}

fn builtin_type(language: &CodeLanguage, word: &str) -> bool {
    let builtins: &[&str] = match language {
        CodeLanguage::Rust => &[
            "bool", "char", "f32", "f64", "i8", "i16", "i32", "i64", "i128", "isize", "str", "u8",
            "u16", "u32", "u64", "u128", "usize", "Option", "Result", "Self", "String", "Vec",
        ],
        CodeLanguage::JavaScript | CodeLanguage::TypeScript => &[
            "Array", "BigInt", "Boolean", "Error", "Map", "Number", "Object", "Promise", "Set",
            "String", "Symbol", "any", "boolean", "never", "number", "string", "unknown", "void",
        ],
        _ => &[],
    };
    builtins.contains(&word)
}

fn push_span(spans: &mut Vec<TextSpan>, text: &str, color: Option<Color>) {
    if text.is_empty() {
        return;
    }
    if let Some(last) = spans.last_mut()
        && last.color == color
        && last.weight.is_none()
        && last.style.is_none()
        && last.decoration.is_none()
    {
        let mut joined = last.text.to_string();
        joined.push_str(text);
        last.text = SmolStr::new(joined);
        return;
    }
    let mut span = TextSpan::new(text);
    span.color = color;
    spans.push(span);
}

/// Highlights source using a dependency-free lexer suitable for UI previews and documentation.
pub fn highlight_code(
    code: &str,
    language: &CodeLanguage,
    theme: &CodeBlockTheme,
) -> Vec<TextSpan> {
    let mut spans = Vec::new();
    let mut index = 0;
    let mut in_block_comment = false;

    while index < code.len() {
        let rest = &code[index..];
        if in_block_comment {
            let end = rest.find("*/").map_or(rest.len(), |found| found + 2);
            push_span(&mut spans, &rest[..end], Some(theme.comment));
            index += end;
            in_block_comment = !rest[..end].ends_with("*/");
            continue;
        }

        if matches!(language, CodeLanguage::Html) && rest.starts_with("<!--") {
            let end = rest.find("-->").map_or(rest.len(), |found| found + 3);
            push_span(&mut spans, &rest[..end], Some(theme.comment));
            index += end;
            continue;
        }

        let hash_comment = matches!(language, CodeLanguage::Shell | CodeLanguage::Toml);
        if rest.starts_with("//") || (hash_comment && rest.starts_with('#')) {
            let end = rest.find('\n').unwrap_or(rest.len());
            push_span(&mut spans, &rest[..end], Some(theme.comment));
            index += end;
            continue;
        }
        if rest.starts_with("/*") {
            let end = rest.find("*/").map_or(rest.len(), |found| found + 2);
            push_span(&mut spans, &rest[..end], Some(theme.comment));
            index += end;
            in_block_comment = !rest[..end].ends_with("*/");
            continue;
        }

        let first = rest.chars().next().expect("non-empty source remainder");
        if matches!(first, '\'' | '"' | '`') {
            let mut escaped = false;
            let mut end = first.len_utf8();
            for c in rest[first.len_utf8()..].chars() {
                end += c.len_utf8();
                if c == first && !escaped {
                    break;
                }
                escaped = c == '\\' && !escaped;
                if c != '\\' {
                    escaped = false;
                }
            }
            push_span(&mut spans, &rest[..end], Some(theme.string));
            index += end;
            continue;
        }

        if first.is_ascii_digit() {
            let end = rest
                .char_indices()
                .skip(1)
                .find_map(|(byte, c)| {
                    (!c.is_ascii_alphanumeric() && !matches!(c, '.' | '_' | '+' | '-'))
                        .then_some(byte)
                })
                .unwrap_or(rest.len());
            push_span(&mut spans, &rest[..end], Some(theme.number));
            index += end;
            continue;
        }

        if first.is_alphabetic() || first == '_' {
            let end = rest
                .char_indices()
                .skip(1)
                .find_map(|(byte, c)| (!(c.is_alphanumeric() || c == '_')).then_some(byte))
                .unwrap_or(rest.len());
            let word = &rest[..end];
            let color = if keyword(language, word) {
                Some(theme.keyword)
            } else if builtin_type(language, word)
                || word.chars().next().is_some_and(char::is_uppercase)
            {
                Some(theme.type_name)
            } else if rest[end..].trim_start().starts_with('(')
                || rest[end..].trim_start().starts_with("!(")
            {
                Some(theme.function)
            } else {
                None
            };
            push_span(&mut spans, word, color);
            index += end;
            continue;
        }

        let color = if first.is_whitespace() {
            None
        } else {
            Some(theme.punctuation)
        };
        let length = first.len_utf8();
        push_span(&mut spans, &rest[..length], color);
        index += length;
    }

    spans
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn highlighting_preserves_source_exactly() {
        let source = "fn main() {\n    // hi\n    println!(\"ok\");\n}\n";
        let spans = highlight_code(source, &CodeLanguage::Rust, &CodeBlockTheme::default());
        let rebuilt: String = spans.iter().map(|span| span.text.as_str()).collect();
        assert_eq!(rebuilt, source);
        assert!(
            spans
                .iter()
                .any(|span| span.color == Some(CodeBlockTheme::default().keyword))
        );
        assert!(
            spans
                .iter()
                .any(|span| span.color == Some(CodeBlockTheme::default().comment))
        );
    }

    #[test]
    fn language_names_accept_filenames_and_aliases() {
        assert_eq!(CodeLanguage::from_name("Cargo.toml"), CodeLanguage::Toml);
        assert_eq!(CodeLanguage::from_name("TSX"), CodeLanguage::TypeScript);
        assert_eq!(CodeLanguage::from_name("Terminal"), CodeLanguage::Shell);
    }
}
