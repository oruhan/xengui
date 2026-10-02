//! Task-oriented XenGui documentation.

use crate::site_tokens::{fast_effect, page_gutter, radius, space, type_scale, ARTICLE_MAX};
use xen_router::RouteParams;
use xengui::*;
use xengui_icons::codepoints;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum DocsSection {
    #[default]
    Start,
    Concepts,
    Components,
    Styling,
    Rendering,
    Api,
}

impl DocsSection {
    const fn slug(self) -> &'static str {
        match self {
            Self::Start => "quick-start",
            Self::Concepts => "core-concepts",
            Self::Components => "components",
            Self::Styling => "styling-layout",
            Self::Rendering => "rendering",
            Self::Api => "api-reference",
        }
    }

    fn from_hash(hash: &str) -> Self {
        match hash.trim_start_matches('#') {
            "core-concepts" | "state" | "keys" => Self::Concepts,
            "components" | "controls" | "alerts" => Self::Components,
            "styling-layout" | "theme-roles" | "responsive-layout" => Self::Styling,
            "rendering" | "renderer-options" => Self::Rendering,
            "api-reference" | "crates" => Self::Api,
            _ => Self::Start,
        }
    }
}

const NAV_ITEMS: &[(DocsSection, &str)] = &[
    (DocsSection::Start, "Quick start"),
    (DocsSection::Concepts, "Core concepts"),
    (DocsSection::Components, "Components"),
    (DocsSection::Styling, "Styling and layout"),
    (DocsSection::Rendering, "Rendering"),
    (DocsSection::Api, "API reference"),
];

#[cfg(target_arch = "wasm32")]
fn current_hash() -> String {
    web_sys::window()
        .and_then(|window| window.location().hash().ok())
        .unwrap_or_default()
}

#[cfg(not(target_arch = "wasm32"))]
fn current_hash() -> String {
    String::new()
}

fn set_docs_hash(slug: &str) {
    #[cfg(target_arch = "wasm32")]
    if let Some(window) = web_sys::window() {
        let _ = window.location().set_hash(slug);
    }
    #[cfg(not(target_arch = "wasm32"))]
    let _ = slug;
}

fn paragraph(text: &str) -> RichText {
    RichText::new()
        .with_content(text)
        .width(pct!(100.0))
        .min_width(px!(0.0))
        .max_width(px!(760.0))
        .font_size(type_scale::BODY_LG)
        .line_height(px!(type_scale::BODY_LG_LINE))
        .color(|theme: &Theme| theme.on_surface_variant)
}

fn page_heading(kicker: &str, title: &str, description: &str, section: DocsSection) -> View {
    Column::new()
        .width(pct!(100.0))
        .min_width(px!(0.0))
        .gap(0.0, space::MD)
        .child(
            Label::new()
                .label(kicker.to_uppercase())
                .font_size(type_scale::LABEL_MD)
                .font_weight(FontWeight::Bold)
                .letter_spacing(px!(1.0))
                .color(|theme: &Theme| theme.primary),
        )
        .child(anchor_title(section.slug(), title, type_scale::HEADLINE_LG, type_scale::HEADLINE_LG_LINE))
        .child(paragraph(description))
}

fn anchor_title(slug: &'static str, title: &str, size: f32, line_height: f32) -> View {
    Row::new()
        .id(format!("docs-heading-{slug}"))
        .width(pct!(100.0))
        .min_width(px!(0.0))
        .align_items(Align::Center)
        .gap(space::SM, 0.0)
        .child(
            RichText::new()
                .with_content(title)
                .min_width(px!(0.0))
                .font_size(size)
                .line_height(px!(line_height))
                .font_weight(FontWeight::Medium)
                .color(|theme: &Theme| theme.on_surface),
        )
        .child(
            Button::new()
                .material_icon(codepoints::LINK)
                .accessible_label(format!("Link to {title}"))
                .width(px!(40.0))
                .height(px!(40.0))
                .padding(Edges::all(8.0))
                .background(Color::TRANSPARENT)
                .color(|theme: &Theme| theme.on_surface_variant)
                .border(Border::all(0.0, Color::TRANSPARENT).radius(radius::FULL))
                .hover_style(|style: StylePatch, theme: &Theme| style.background(theme.surface_container_high).color(theme.primary))
                .focus_style(|style: StylePatch, theme: &Theme| style.border(Border::all(2.0, theme.primary).radius(radius::FULL)))
                .on_click(move |_| set_docs_hash(slug)),
        )
}

fn section_title(slug: &'static str, title: &str, description: &str) -> View {
    Column::new()
        .width(pct!(100.0))
        .min_width(px!(0.0))
        .gap(0.0, space::SM)
        .child(anchor_title(slug, title, type_scale::TITLE_LG, type_scale::TITLE_LG_LINE))
        .child(
            paragraph(description)
                .font_size(type_scale::BODY_MD)
                .line_height(px!(type_scale::BODY_MD_LINE)),
        )
}

fn code_block(label: &str, code: &str) -> CodeBlock {
    let language = match label {
        "Cargo.toml" => CodeLanguage::Toml,
        "Terminal" => CodeLanguage::Shell,
        _ => CodeLanguage::Rust,
    };
    CodeBlock::new(code)
        .label(label)
        .language(language)
        .code_font("XenMono")
        .copy_label("Copy")
        .copied_label("Copied")
        .border(|theme: &Theme| Border::all(1.0, theme.outline_variant).radius(radius::LG))
}

#[derive(Clone, Copy)]
enum AlertKind {
    Info,
    Warning,
    Success,
    Error,
}

fn alert(kind: AlertKind, title: &str, text: &str) -> View {
    let icon_codepoint = match kind {
        AlertKind::Info => codepoints::INFO,
        AlertKind::Warning => codepoints::WARNING,
        AlertKind::Success => codepoints::CHECK_CIRCLE,
        AlertKind::Error => codepoints::ERROR,
    };
    let fill = move |theme: &Theme| match kind {
        AlertKind::Info => theme.primary_container.with_alpha(120),
        AlertKind::Warning => theme.warning_container.with_alpha(150),
        AlertKind::Success => theme.success_container.with_alpha(140),
        AlertKind::Error => theme.error_container.with_alpha(140),
    };
    let foreground = move |theme: &Theme| match kind {
        AlertKind::Info => theme.on_primary_container,
        AlertKind::Warning => theme.on_warning_container,
        AlertKind::Success => theme.on_success_container,
        AlertKind::Error => theme.on_error_container,
    };
    let accent = move |theme: &Theme| match kind {
        AlertKind::Info => theme.primary,
        AlertKind::Warning => theme.warning,
        AlertKind::Success => theme.success,
        AlertKind::Error => theme.error,
    };

    Row::new()
        .width(pct!(100.0))
        .min_width(px!(0.0))
        .align_items(Align::Start)
        .gap(space::MD, 0.0)
        .padding(Edges::all(space::LG))
        .background(fill)
        .border(move |theme: &Theme| Border::all(1.0, accent(theme).with_alpha(150)).radius(radius::LG))
        .child(StyleBuilder::color(VariableIcon::new(icon_codepoint).size(22.0), accent))
        .child(
            Column::new()
                .min_width(px!(0.0))
                .gap(0.0, space::XS)
                .child(Label::new().label(title).font_size(type_scale::TITLE_SM).font_weight(FontWeight::Bold).color(foreground))
                .child(paragraph(text).font_size(type_scale::BODY_MD).line_height(px!(type_scale::BODY_MD_LINE)).color(foreground)),
        )
}

fn feature_card(title: &str, description: &str) -> View {
    Column::new()
        .flex_basis(Responsive::new(pct!(100.0)).md(pct!(48.0)))
        .gap(0.0, space::SM)
        .padding(Edges::all(space::LG))
        .background(|theme: &Theme| theme.surface_container_low)
        .border(|theme: &Theme| Border::all(1.0, theme.outline_variant).radius(radius::LG))
        .child(Label::new().label(title).font_size(type_scale::TITLE_MD).font_weight(FontWeight::Bold))
        .child(paragraph(description).font_size(type_scale::BODY_MD).line_height(px!(type_scale::BODY_MD_LINE)).max_width(px!(420.0)))
}

fn cards(items: &[(&str, &str)]) -> View {
    let mut row = Row::new().flex_wrap(FlexWrap::Wrap).gap(space::MD, space::MD);
    for (title, description) in items {
        row = row.child(feature_card(title, description));
    }
    row
}

fn start_page() -> View {
    Column::new()
        .gap(0.0, space::XXL)
        .child(page_heading(
            "Documentation",
            "Build your first XenGui application",
            "Create a small task board that demonstrates layout, state, interaction, styling, and several core widgets without hiding the runtime setup.",
            DocsSection::Start,
        ))
        .child(alert(AlertKind::Info, "Two complementary references", "Use this site for learning paths and runnable examples. Use docs.rs for exact public signatures and crate-level API contracts."))
        .child(section_title("dependencies", "1. Add dependencies", "The window runtime, widget tree, and wgpu backend are separate crates."))
        .child(code_block("Cargo.toml", "[dependencies]\nxengui = \"0.2.8\"\nxenframe = \"0.1.2\"\nxengui-wgpu = \"0.1.2\""))
        .child(section_title("first-window", "2. Create a stateful view", "The render closure produces a new widget tree when state changes; reconciliation preserves stable widget identity."))
        .child(code_block(
            "src/main.rs",
            "use xenframe::{App, AppConfig};\nuse xengui::*;\n\nfn main() -> Result<(), Box<dyn std::error::Error>> {\n    let mut app = App::new(AppConfig {\n        title: \"Focus Board\".into(),\n        width: 760,\n        height: 520,\n        ..Default::default()\n    });\n\n    app.render(|| {\n        let (done, set_done) = use_state(false);\n        Box::new(\n            Column::new()\n                .padding(Edges::all(24.0))\n                .gap(0.0, 16.0)\n                .child(Label::new().label(\"Today\").font_size(28.0))\n                .child(\n                    Row::new()\n                        .align_items(Align::Center)\n                        .gap(12.0, 0.0)\n                        .child(Checkbox::new().checked(done).on_change(\n                            move |value, _| set_done.set(value),\n                        ))\n                        .child(Label::new().label(\"Ship a polished XenGui screen\"))\n                )\n                .child(ProgressBar::new().value(if done { 1.0 } else { 0.5 }))\n        )\n    });\n\n    app.run()?;\n    Ok(())\n}",
        ))
        .child(section_title("run", "3. Run it", "Cargo starts the native application; Trunk serves browser applications."))
        .child(code_block("Terminal", "cargo run\n\n# Browser projects\nrustup target add wasm32-unknown-unknown\ntrunk serve --open"))
        .child(alert(AlertKind::Warning, "Active development", "Pin XenGui crate versions and review release notes before upgrading production applications prior to 1.0."))
}

fn concepts_page() -> View {
    Column::new()
        .gap(0.0, space::XXL)
        .child(page_heading("Model", "Core concepts", "XenGui combines a retained widget tree with declarative render functions and explicit state.", DocsSection::Concepts))
        .child(cards(&[
            ("Widget tree", "Every widget participates in measurement, layout, paint, semantics, and input through one Widget contract."),
            ("Controlled state", "use_state returns a value and setter; form controls report changes through callbacks."),
            ("Reconciliation", "Stable keys preserve state and interaction identity when dynamic lists are reordered."),
            ("Effects", "Cleanup runs before an effect is repeated and when its component is unmounted."),
        ]))
        .child(section_title("state", "State", "Keep hook order unconditional and stable across renders."))
        .child(code_block("Component state", "let (done, set_done) = use_state(false);\n\nCheckbox::new()\n    .checked(done)\n    .on_change(move |value, _| set_done.set(value))"))
        .child(section_title("keys", "Stable list keys", "Use domain identifiers instead of list positions for dynamic siblings."))
        .child(code_block("Keyed list", "for item in items {\n    list = list.child(\n        Row::new()\n            .key(item.id.to_string())\n            .child(Label::new().label(item.title))\n    );\n}"))
}

fn components_page() -> View {
    Column::new()
        .gap(0.0, space::XXL)
        .child(page_heading("Widgets", "Components", "Core controls use theme roles and expose hover, focus, pressed, disabled, and controlled-value behavior.", DocsSection::Components))
        .child(section_title("controls", "Interactive controls", "These are live XenGui widgets, not screenshots."))
        .child(
            Row::new()
                .flex_wrap(FlexWrap::Wrap)
                .align_items(Align::Center)
                .gap(space::LG, space::LG)
                .child(Button::new().label("Save changes"))
                .child(Switch::new().checked(true))
                .child(Checkbox::new().checked(false))
                .child(TextBox::new().placeholder("Project name"))
                .child(Badge::new().label("Stable")),
        )
        .child(section_title("alerts", "Alerts", "Semantic alerts use consistent icon size, spacing, border, container, and foreground roles."))
        .child(alert(AlertKind::Info, "Information", "Use for relevant context that does not require immediate action."))
        .child(alert(AlertKind::Warning, "Warning", "Use for a recoverable condition the reader should review."))
        .child(alert(AlertKind::Success, "Success", "Use to confirm a completed operation."))
        .child(alert(AlertKind::Error, "Error", "Use for a failure that blocks the current task."))
}

fn styling_page() -> View {
    Column::new()
        .gap(0.0, space::XXL)
        .child(page_heading("Design system", "Styling and layout", "Theme roles, responsive values, Flexbox, and Grid compose in the same builder chain.", DocsSection::Styling))
        .child(section_title("theme-roles", "Theme roles", "Pair container roles with their matching on-container foreground roles."))
        .child(code_block("Theme roles", "View::new()\n    .padding(Edges::all(20.0))\n    .background(|theme: &Theme| theme.surface_container_low)\n    .color(|theme: &Theme| theme.on_surface)\n    .border(|theme: &Theme|\n        Border::all(1.0, theme.outline_variant).radius(16.0)\n    )"))
        .child(section_title("responsive-layout", "Responsive layout", "Author compact values first, then override them at wider breakpoints."))
        .child(code_block("Responsive layout", "View::new()\n    .padding(Responsive::new(Edges::all(16.0)).md(Edges::all(32.0)))\n    .width(Responsive::new(pct!(100.0)).lg(px!(960.0)))\n    .display(Display::Grid)\n    .grid_template_columns(vec![GridTrack::Fr(1.0), GridTrack::Fr(1.0)])"))
}

fn rendering_page() -> View {
    Column::new()
        .gap(0.0, space::XXL)
        .child(page_heading("Pipeline", "Rendering", "xengui produces backend-neutral paint commands; xengui-wgpu turns those commands into GPU work.", DocsSection::Rendering))
        .child(cards(&[
            ("Surface recovery", "Lost and outdated surfaces are reconfigured; timeout and occlusion return controlled outcomes."),
            ("Retained staging", "Frame and pipeline staging buffers retain their high-water capacity for steady-state reuse."),
            ("Text shaping", "Measurement and drawing share shaped glyph buffers when the text run is unchanged."),
            ("Browser fallback", "Adapter limits determine whether the browser uses WebGPU or the supported compatibility path."),
        ]))
        .child(section_title("renderer-options", "Renderer options", "Choose a presentation policy and sample count explicitly when application requirements differ from defaults."))
        .child(code_block("Renderer configuration", "use xengui_wgpu::{PresentModePreference, RendererOptions, SampleCount};\n\nlet config = AppConfig {\n    renderer: RendererOptions {\n        present_mode: PresentModePreference::Vsync,\n        sample_count: SampleCount::X4,\n        ..Default::default()\n    },\n    ..Default::default()\n};"))
}

fn api_card(name: &str, description: &str, href: &str) -> View {
    feature_card(name, description).child(
        Link::new()
            .label("Open rustdoc →")
            .href(href)
            .target_blank(true)
            .font_size(type_scale::LABEL_LG)
            .color(|theme: &Theme| theme.primary)
            .text_decoration(TextDecoration::UNDERLINE)
            .hover_style(|style: StylePatch, theme: &Theme| style.color(theme.on_surface)),
    )
}

fn api_page() -> View {
    Column::new()
        .gap(0.0, space::XXL)
        .child(page_heading("Reference", "API reference", "docs.rs is the authoritative source for public types, methods, traits, and error contracts.", DocsSection::Api))
        .child(section_title("crates", "Crates", "Open the generated API reference for the part of the stack you are using."))
        .child(
            Row::new()
                .flex_wrap(FlexWrap::Wrap)
                .gap(space::MD, space::MD)
                .child(api_card("xengui", "Widgets, layout, styling, input, hooks, and paint.", "https://docs.rs/xengui/latest/xengui/"))
                .child(api_card("xenframe", "Window lifecycle and platform integration.", "https://docs.rs/xenframe/latest/xenframe/"))
                .child(api_card("xengui-wgpu", "The wgpu render backend and renderer options.", "https://docs.rs/xengui-wgpu/latest/xengui_wgpu/"))
                .child(api_card("xen-router", "Client-side routing and browser history synchronization.", "https://docs.rs/xen-router/latest/xen_router/")),
        )
}

fn nav_button(active: DocsSection, section: DocsSection, title: &'static str, set_active: SetState<DocsSection>, compact: bool) -> Button {
    let selected = active == section;
    Button::new()
        .width(if compact { px!(164.0) } else { pct!(100.0) })
        .flex_shrink(0.0)
        .label(title)
        .text_align(TextAlign::Start)
        .font_size(type_scale::LABEL_LG)
        .font_weight(if selected { FontWeight::Bold } else { FontWeight::Medium })
        .color(move |theme: &Theme| if selected { theme.on_surface } else { theme.on_surface_variant })
        .background(move |theme: &Theme| if selected { theme.surface_container_high } else { Color::TRANSPARENT })
        .border(Border::all(0.0, Color::TRANSPARENT).radius(radius::MD))
        .padding(Edges::symmetric(space::MD, 10.0))
        .transition_colors(fast_effect())
        .hover_style(|style: StylePatch, theme: &Theme| style.background(theme.surface_container_high).color(theme.on_surface))
        .focus_style(|style: StylePatch, theme: &Theme| style.border(Border::all(2.0, theme.primary).radius(radius::MD)))
        .on_click(move |_| {
            set_docs_hash(section.slug());
            set_active.set(section);
        })
}

pub fn page(_params: &RouteParams) -> Box<dyn Widget> {
    let (active, set_active) = use_state(DocsSection::from_hash(&current_hash()));
    let compact = !responsive_bool(Breakpoint::Expanded, true);
    let mut navigation = View::new()
        .display(Display::Flex)
        .flex_direction(if compact { FlexDirection::Row } else { FlexDirection::Column })
        .width(pct!(100.0))
        .gap(if compact { space::SM } else { 0.0 }, if compact { 0.0 } else { space::XS })
        .overflow_x(if compact { Overflow::Auto } else { Overflow::Visible });
    for (section, title) in NAV_ITEMS {
        navigation = navigation.child(nav_button(active, *section, title, set_active.clone(), compact));
    }

    let content = match active {
        DocsSection::Start => start_page(),
        DocsSection::Concepts => concepts_page(),
        DocsSection::Components => components_page(),
        DocsSection::Styling => styling_page(),
        DocsSection::Rendering => rendering_page(),
        DocsSection::Api => api_page(),
    };

    Box::new(
        View::new()
            .display(Display::Flex)
            .flex_direction(if compact { FlexDirection::Column } else { FlexDirection::Row })
            .align_items(Align::Start)
            .width(pct!(100.0))
            .min_width(px!(0.0))
            .padding(page_gutter(24.0, 64.0))
            .gap(Responsive::new(px!(24.0)).md(px!(40.0)), px!(32.0))
            .child(
                Column::new()
                    .position(if compact { Position::Relative } else { Position::Sticky })
                    .top(if compact { 0.0 } else { 96.0 })
                    .width(if compact { pct!(100.0) } else { px!(248.0) })
                    .flex_shrink(0.0)
                    .gap(0.0, space::MD)
                    .child(Label::new().label("DOCUMENTATION").font_size(type_scale::LABEL_SM).font_weight(FontWeight::Bold).letter_spacing(px!(1.0)).color(|theme: &Theme| theme.primary))
                    .child(navigation),
            )
            .child(
                Column::new()
                    .width(pct!(100.0))
                    .max_width(px!(ARTICLE_MAX))
                    .min_width(px!(0.0))
                    .align_items(Align::Start)
                    .child(content),
            ),
    )
}
