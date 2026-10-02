use crate::site_tokens::{fast_spatial, radius, type_scale};
use xen_router::RouteParams;
use xengui::*;

const HERO_CODE: &str =
    "use xengui::*;\n\nfn view() -> View {\n    Column::new()\n        .gap(0.0, 12.0)\n        .child(Label::new()\n            .label(\"Today\"))\n        .child(Button::new()\n            .label(\"Add task\"))\n}";

fn eyebrow(label: &str) -> View {
    Row::new()
        .align_items(Align::Center)
        .gap(8.0, 0.0)
        .child(
            View::new()
                .width(px!(7.0))
                .height(px!(7.0))
                .background(Color::BLUE_500)
                .border(Border::all(0.0, Color::TRANSPARENT).radius(7.0))
        )
        .child(
            Label::new()
                .label(label)
                .font_size(type_scale::LABEL_MD)
                .font_weight(FontWeight::SemiBold)
                .letter_spacing(px!(0.8))
                .color(|theme: &Theme| theme.on_surface_variant)
        )
}

fn code_window() -> View {
    Column::new()
        .width(Responsive::new(pct!(100.0)).lg(pct!(45.0)))
        .min_width(px!(0.0))
        .background(Color::NEUTRAL_950)
        .border(Border::all(1.0, Color::NEUTRAL_800).radius(16.0))
        .overflow_x(Overflow::Hidden)
        .overflow_y(Overflow::Hidden)
        .box_shadow(
            BoxShadow::new(0.0, 18.0, 48.0, Color::BLACK.with_alpha(42)).direction(
                ShadowDirection::Bottom
            )
        )
        .child(
            CodeBlock::new(HERO_CODE)
                .label("Rust")
                .language(CodeLanguage::Rust)
                .code_font("XenMono")
                .copy_label("Copy")
                .copied_label("Copied")
                .border(Border::none())
        )
        .child(
            Row::new()
                .align_items(Align::Center)
                .justify_content(JustifyContent::SpaceBetween)
                .padding(Edges::symmetric(18.0, 13.0))
                .background(Color::hex("#111827"))
                .border(Border::top(1.0, Color::NEUTRAL_800).radius(BorderRadius::bottom(15.0)))
                .child(
                    Label::new().label("native / wasm32").font_size(type_scale::LABEL_SM).color(Color::NEUTRAL_400)
                )
                .child(
                    Label::new()
                        .label("cargo run")
                        .font_size(type_scale::LABEL_SM)
                        .font_weight(FontWeight::SemiBold)
                        .color(Color::BLUE_300)
                )
        )
}

fn stat(value: &str, label: &str) -> View {
    Column::new()
        .flex_basis(Responsive::new(pct!(48.0)).lg(pct!(23.0)))
        .gap(0.0, 5.0)
        .padding(Edges::only(0.0, 20.0, 0.0, 20.0))
        .border(|theme: &Theme| Border::top(1.0, theme.outline_variant))
        .child(
            Label::new()
                .label(value)
                .font_size(type_scale::TITLE_LG)
                .font_weight(FontWeight::SemiBold)
                .color(|theme: &Theme| theme.on_background)
        )
        .child(
            Label::new()
                .label(label)
                .font_size(type_scale::BODY_SM)
                .color(|theme: &Theme| theme.on_surface_variant)
        )
}

fn feature(index: &str, title: &str, text: &str) -> View {
    Column::new()
        .flex_basis(Responsive::new(pct!(100.0)).md(pct!(31.0)))
        .min_width(px!(0.0))
        .gap(0.0, 14.0)
        .padding(Edges::only(0.0, 22.0, 0.0, 8.0))
        .border(|theme: &Theme| Border::top(1.0, theme.outline_variant))
        .child(
            Label::new()
                .label(index)
                .font_size(type_scale::LABEL_SM)
                .font_weight(FontWeight::SemiBold)
                .letter_spacing(px!(1.0))
                .color(|theme: &Theme| theme.primary)
        )
        .child(
            Label::new()
                .label(title)
                .font_size(type_scale::TITLE_LG)
                .font_weight(FontWeight::SemiBold)
                .color(|theme: &Theme| theme.on_background)
        )
        .child(
            RichText::new()
                .with_content(text)
                .width(pct!(100.0))
                .font_size(type_scale::BODY_MD)
                .line_height(px!(type_scale::BODY_MD_LINE))
                .color(|theme: &Theme| theme.on_surface_variant)
        )
}

pub fn page(_params: &RouteParams) -> Box<dyn Widget> {
    let stacked = !responsive_bool(Breakpoint::Large, true);

    let hero_copy = Column::new()
        .width(Responsive::new(pct!(100.0)).lg(pct!(49.0)))
        .min_width(px!(0.0))
        .align_items(Align::Start)
        .gap(0.0, 22.0)
        .child(eyebrow("CROSS-PLATFORM RUST UI"))
        .child(
            RichText::new()
                .with_content("Build clear, responsive interfaces in Rust.")
                .width(pct!(100.0))
                .max_width(px!(720.0))
                .font_size(Responsive::new(px!(type_scale::DISPLAY_SM)).md(px!(type_scale::DISPLAY_MD)).lg(px!(type_scale::DISPLAY_LG)))
                .line_height(Responsive::new(px!(type_scale::DISPLAY_SM_LINE)).md(px!(type_scale::DISPLAY_MD_LINE)).lg(px!(type_scale::DISPLAY_LG_LINE)).resolve())
                .font_weight(FontWeight::Medium)
                .letter_spacing(px!(-0.2))
                .color(|theme: &Theme| theme.on_background)
        )
        .child(
            RichText::new()
                .with_content(
                    "Use one retained widget tree for desktop and browser applications, with shared layout, input, state, and rendering APIs."
                )
                .width(pct!(100.0))
                .max_width(px!(610.0))
                .font_size(Responsive::new(px!(type_scale::BODY_MD)).md(px!(type_scale::BODY_LG)))
                .line_height(px!(type_scale::BODY_LG_LINE))
                .color(|theme: &Theme| theme.on_surface_variant)
        )
        .child(
            View::new()
                .display(Display::Flex)
                .flex_direction(FlexDirection::Row)
                .flex_wrap(FlexWrap::Wrap)
                .gap(10.0, 10.0)
                .child(
                    xen_router
                        ::link("/docs")
                        .label("Read the quick start")
                        .font_weight(FontWeight::SemiBold)
                        .background(|theme: &Theme| theme.on_background)
                        .color(|theme: &Theme| theme.background)
                        .height(px!(56.0))
                        .padding(Edges::symmetric(24.0, 0.0))
                        .border(Border::all(0.0, Color::TRANSPARENT).radius(radius::FULL))
                        .transform_origin(TransformOrigin::CENTER)
                        .content_scale(1.0)
                        .transition_transform(fast_spatial())
                        .pressed_style(|style: StylePatch, _theme: &Theme| style.scale(0.98))
                )
                .child(
                    Link::new()
                        .href("https://github.com/randseas/xengui")
                        .target_blank(true)
                        .label("View on GitHub")
                        .font_weight(FontWeight::Medium)
                        .background(Color::TRANSPARENT)
                        .color(|theme: &Theme| theme.on_background)
                        .height(px!(56.0))
                        .padding(Edges::symmetric(24.0, 0.0))
                        .border(|theme: &Theme| Border::all(1.0, theme.outline).radius(radius::FULL))
                        .hover_style(|style: StylePatch, theme: &Theme| style.background(theme.surface_container_high))
                        .focus_style(|style: StylePatch, theme: &Theme| style.border(Border::all(2.0, theme.primary).radius(radius::FULL)))
                )
        )
        .child(
            Label::new()
                .label("Apache 2.0 · Rust 1.92+ · v0.2.8")
                .font_size(type_scale::BODY_SM)
                .color(|theme: &Theme| theme.on_surface_variant)
        );

    let hero = View::new()
        .display(Display::Flex)
        .flex_direction(if stacked { FlexDirection::Column } else { FlexDirection::Row })
        .align_items(if stacked { Align::Stretch } else { Align::Center })
        .justify_content(JustifyContent::SpaceBetween)
        .gap(Responsive::new(px!(44.0)).lg(px!(72.0)), px!(44.0))
        .padding(
            Responsive::new(Edges::only(20.0, 56.0, 20.0, 48.0))
                .md(Edges::only(64.0, 80.0, 64.0, 64.0))
                .lg(Edges::only(120.0, 104.0, 120.0, 88.0))
        )
        .child(hero_copy)
        .child(code_window());

    let stats = View::new()
        .display(Display::Flex)
        .flex_wrap(FlexWrap::Wrap)
        .gap(16.0, 16.0)
        .padding(
            Responsive::new(Edges::only(20.0, 0.0, 20.0, 56.0))
                .md(Edges::only(64.0, 0.0, 64.0, 72.0))
                .lg(Edges::only(120.0, 0.0, 120.0, 88.0))
        )
        .child(stat("1", "shared widget model"))
        .child(stat("2", "desktop and web targets"))
        .child(stat("Flex + Grid", "layout primitives"))
        .child(stat("Rust", "state and interaction"));

    let principles = Column::new()
        .gap(0.0, 32.0)
        .padding(
            Responsive::new(Edges::only(20.0, 56.0, 20.0, 80.0))
                .md(Edges::only(64.0, 72.0, 64.0, 96.0))
                .lg(Edges::only(120.0, 88.0, 120.0, 120.0))
        )
        .child(
            Column::new()
                .gap(0.0, 10.0)
                .child(eyebrow("WHY XENGUI"))
                .child(
                    RichText::new()
                        .with_content("A focused UI stack you can understand.")
                        .width(pct!(100.0))
                        .font_size(Responsive::new(px!(type_scale::HEADLINE_MD)).md(px!(type_scale::DISPLAY_SM)))
                        .line_height(px!(type_scale::DISPLAY_SM_LINE))
                        .font_weight(FontWeight::Medium)
                        .color(|theme: &Theme| theme.on_background)
                )
        )
        .child(
            View::new()
                .display(Display::Flex)
                .flex_wrap(FlexWrap::Wrap)
                .gap(18.0, 18.0)
                .child(
                    feature(
                        "01 / MODEL",
                        "Declarative and retained",
                        "Build readable widget trees while reconciliation preserves identity and interaction state across updates."
                    )
                )
                .child(
                    feature(
                        "02 / RENDER",
                        "Backend-aware rendering",
                        "Text, SVG, and surfaces become backend-neutral paint commands consumed by the wgpu renderer."
                    )
                )
                .child(
                    feature(
                        "03 / SCALE",
                        "Responsive by construction",
                        "Breakpoints, theme roles, Flexbox, and Grid compose in the same Rust builder API."
                    )
                )
        );

    Box::new(
        Column::new()
            .width(pct!(100.0))
            .min_width(px!(0.0))
            .child(hero)
            .child(stats)
            .child(principles)
    )
}
