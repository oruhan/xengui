// SPDX-License-Identifier: Apache-2.0
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[cfg(not(target_arch = "wasm32"))]
use xenframe::WindowPosition;
use xenframe::{App, AppConfig};
use xengui::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(not(target_arch = "wasm32"))]
    let _ = env_logger::try_init();

    let mut app = App::new(AppConfig {
        title: "Focus Board · XenGui".into(),
        #[cfg(not(target_arch = "wasm32"))]
        width: 960,
        #[cfg(not(target_arch = "wasm32"))]
        height: 640,
        #[cfg(not(target_arch = "wasm32"))]
        position: WindowPosition::Center,
        ..Default::default()
    });

    app.with_font(
        "Inter",
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../xengui_website/fonts/Inter-VariableFont.ttf"
        ))
        .to_vec(),
    );

    app.render(|| {
        let (done, set_done) = use_state(false);
        let (draft, set_draft) = use_state(String::new());
        let (note, set_note) = use_state("Review focus and keyboard states".to_string());
        let progress = if done { 1.0 } else { 0.5 };

        let draft_for_add = draft.clone();
        let set_note_for_add = set_note.clone();
        let set_draft_for_add = set_draft.clone();

        Box::new(
            View::new()
                .font("Inter")
                .width(pct!(100.0))
                .height(pct!(100.0))
                .padding(Edges::all(40.0))
                .background(|theme: &Theme| theme.background)
                .child(
                    Column::new()
                        .width(pct!(100.0))
                        .height(pct!(100.0))
                        .gap(0.0, 24.0)
                        .padding(Edges::all(32.0))
                        .background(|theme: &Theme| theme.surface_container_low)
                        .border(|theme: &Theme| {
                            Border::all(1.0, theme.outline_variant).radius(28.0)
                        })
                        .child(
                            Row::new()
                                .align_items(Align::End)
                                .justify_content(JustifyContent::SpaceBetween)
                                .child(
                                    Column::new()
                                        .gap(0.0, 6.0)
                                        .child(
                                            Label::new()
                                                .label("FOCUS BOARD")
                                                .font_size(12.0)
                                                .font_weight(FontWeight::Bold)
                                                .letter_spacing(px!(1.0))
                                                .color(|theme: &Theme| theme.primary),
                                        )
                                        .child(
                                            Label::new()
                                                .label("Today")
                                                .font_size(32.0)
                                                .line_height(px!(40.0))
                                                .font_weight(FontWeight::Medium),
                                        ),
                                )
                                .child(Badge::new().label(if done {
                                    "Complete"
                                } else {
                                    "In progress"
                                })),
                        )
                        .child(
                            Column::new()
                                .gap(0.0, 10.0)
                                .child(
                                    Row::new()
                                        .justify_content(JustifyContent::SpaceBetween)
                                        .child(Label::new().label("Daily progress"))
                                        .child(
                                            Label::new()
                                                .label(format!("{}%", (progress * 100.0) as u32))
                                                .font_weight(FontWeight::Bold),
                                        ),
                                )
                                .child(ProgressBar::new().value(progress)),
                        )
                        .child(
                            Row::new()
                                .align_items(Align::Center)
                                .gap(16.0, 0.0)
                                .padding(Edges::all(20.0))
                                .background(|theme: &Theme| theme.surface)
                                .border(|theme: &Theme| {
                                    Border::all(1.0, theme.outline_variant).radius(16.0)
                                })
                                .child(
                                    Checkbox::new()
                                        .checked(done)
                                        .on_change(move |value, _| set_done.set(value)),
                                )
                                .child(
                                    Column::new()
                                        .gap(0.0, 4.0)
                                        .child(
                                            Label::new()
                                                .label("Ship a polished XenGui screen")
                                                .font_size(16.0)
                                                .font_weight(FontWeight::SemiBold),
                                        )
                                        .child(
                                            Label::new()
                                                .label(note)
                                                .font_size(14.0)
                                                .color(|theme: &Theme| theme.on_surface_variant),
                                        ),
                                ),
                        )
                        .child(
                            Row::new()
                                .align_items(Align::Center)
                                .gap(12.0, 0.0)
                                .child(
                                    TextBox::new()
                                        .value(draft)
                                        .placeholder("Add a short note…")
                                        .accessible_label("Task note")
                                        .height(px!(48.0))
                                        .flex_grow(1.0)
                                        .padding(Edges::symmetric(16.0, 0.0))
                                        .border(|theme: &Theme| {
                                            Border::all(1.0, theme.outline).radius(24.0)
                                        })
                                        .on_change(move |value, _| {
                                            set_draft.set(value.to_string())
                                        }),
                                )
                                .child(
                                    Button::new()
                                        .label("Update note")
                                        .height(px!(48.0))
                                        .padding(Edges::symmetric(20.0, 0.0))
                                        .background(|theme: &Theme| theme.primary)
                                        .color(|theme: &Theme| theme.on_primary)
                                        .border(Border::all(0.0, Color::TRANSPARENT).radius(24.0))
                                        .on_click(move |_| {
                                            let next = draft_for_add.trim();
                                            if !next.is_empty() {
                                                set_note_for_add.set(next.to_string());
                                                set_draft_for_add.set(String::new());
                                            }
                                        }),
                                ),
                        ),
                ),
        )
    });

    app.run()?;
    Ok(())
}
