// SPDX-License-Identifier: Apache-2.0
// hide console window on windows subsystem
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use web_time::Duration;
use xenframe::{ App, AppConfig };

#[cfg(not(target_arch = "wasm32"))]
use xenframe::WindowPosition;
use xengui::{ properties::StyleValue, * };

// ---------------------------------------------------------------------
// Icons - hand-authored inline SVGs (fill/stroke resolve to the widget's
// own `color`, matching CSS `currentColor`), so every icon tints along
// with its button's hover/pressed state automatically.
// ---------------------------------------------------------------------

const ICON_LOGO: &str =
    r##"<svg viewBox="0 0 24 24">
<circle cx="12" cy="12" r="10" fill="#FF8A80"/>
<path d="M9.5 8 L9.5 16 L16.2 12 Z" fill="#210300"/>
</svg>"##;

const ICON_PLAY: &str =
    r#"<svg viewBox="0 0 24 24"><path d="M8 5 L8 19 L19 12 Z" fill="currentColor"/></svg>"#;

const ICON_PAUSE: &str =
    r#"<svg viewBox="0 0 24 24">
<rect x="6" y="5" width="4" height="14" rx="1" fill="currentColor"/>
<rect x="14" y="5" width="4" height="14" rx="1" fill="currentColor"/>
</svg>"#;

const ICON_SKIP_NEXT: &str =
    r#"<svg viewBox="0 0 24 24">
<path d="M6 5 L6 19 L15 12 Z" fill="currentColor"/>
<rect x="16" y="5" width="2.4" height="14" fill="currentColor"/>
</svg>"#;

const ICON_SKIP_PREVIOUS: &str =
    r#"<svg viewBox="0 0 24 24">
<rect x="5.6" y="5" width="2.4" height="14" fill="currentColor"/>
<path d="M18 5 L18 19 L9 12 Z" fill="currentColor"/>
</svg>"#;

const ICON_SHUFFLE: &str =
    r#"<svg viewBox="0 0 24 24">
<path d="M16 3 L21 3 L21 8" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
<line x1="4" y1="20" x2="21" y2="3" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
<path d="M21 16 L21 21 L16 21" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
<line x1="15" y1="15" x2="21" y2="21" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
<line x1="4" y1="4" x2="9" y2="9" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
</svg>"#;

const ICON_REPEAT: &str =
    r#"<svg viewBox="0 0 24 24">
<path d="M7 7 L17 7 L17 4 L21 8 L17 12 L17 9 L7 9 Z" fill="currentColor"/>
<path d="M17 17 L7 17 L7 20 L3 16 L7 12 L7 15 L17 15 Z" fill="currentColor"/>
</svg>"#;

const ICON_VOLUME: &str =
    r#"<svg viewBox="0 0 24 24">
<path d="M4 9 L4 15 L8 15 L13 20 L13 4 L8 9 Z" fill="currentColor"/>
<path d="M15.5 9 L18 12 L15.5 15" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
<path d="M18.5 6 L22 12 L18.5 18" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
</svg>"#;

const ICON_VOLUME_MUTE: &str =
    r#"<svg viewBox="0 0 24 24">
<path d="M4 9 L4 15 L8 15 L13 20 L13 4 L8 9 Z" fill="currentColor"/>
<line x1="16" y1="9" x2="22" y2="15" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
<line x1="22" y1="9" x2="16" y2="15" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
</svg>"#;

const ICON_SEARCH: &str =
    r#"<svg viewBox="0 0 24 24">
<circle cx="10.5" cy="10.5" r="6.5" fill="none" stroke="currentColor" stroke-width="2"/>
<line x1="20" y1="20" x2="15.2" y2="15.2" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
</svg>"#;

const ICON_HOME: &str =
    r#"<svg viewBox="0 0 24 24">
<path d="M3 11 L12 4 L21 11" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
<path d="M5 10 L5 21 L19 21 L19 10" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
</svg>"#;

const ICON_LIBRARY: &str =
    r#"<svg viewBox="0 0 24 24">
<rect x="3" y="4" width="18" height="3" rx="1.2" fill="currentColor"/>
<rect x="3" y="10.5" width="18" height="3" rx="1.2" fill="currentColor"/>
<rect x="3" y="17" width="11" height="3" rx="1.2" fill="currentColor"/>
</svg>"#;

const ICON_HEART: &str =
    r#"<svg viewBox="0 0 24 24">
<path d="M12 20 C12 20 3 14.5 3 8.6 C3 5.8 5.2 4 7.6 4 C9.4 4 10.9 5 12 6.5 C13.1 5 14.6 4 16.4 4 C18.8 4 21 5.8 21 8.6 C21 14.5 12 20 12 20 Z" fill="currentColor"/>
</svg>"#;

const ICON_MUSIC_NOTE: &str =
    r#"<svg viewBox="0 0 24 24">
<circle cx="9" cy="17" r="3" fill="currentColor"/>
<rect x="10.6" y="4" width="1.8" height="13" fill="currentColor"/>
<path d="M12.4 4 L18 5.6 L18 8.4 L12.4 6.8 Z" fill="currentColor"/>
</svg>"#;

const ICON_MINIMIZE: &str =
    r#"<svg viewBox="0 0 24 24"><line x1="5" y1="12" x2="19" y2="12" stroke="currentColor" stroke-width="2" stroke-linecap="round"/></svg>"#;

const ICON_MAXIMIZE: &str =
    r#"<svg viewBox="0 0 24 24"><rect x="5.5" y="5.5" width="13" height="13" rx="1.5" fill="none" stroke="currentColor" stroke-width="1.6"/></svg>"#;

const ICON_CLOSE: &str =
    r#"<svg viewBox="0 0 24 24">
<line x1="6" y1="6" x2="18" y2="18" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
<line x1="18" y1="6" x2="6" y2="18" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
</svg>"#;

// ---------------------------------------------------------------------
// Sample data
// ---------------------------------------------------------------------

#[derive(Clone)]
struct Track {
    title: String,
    artist: String,
    duration_secs: u32,
    art_color: Color,
}

fn sample_tracks() -> Vec<Track> {
    let raw: [(&str, &str, u32, Color); 8] = [
        ("Midnight City Lights", "Nova Sound", 214, Color::INDIGO_400),
        ("Golden Hour", "Wilder Skies", 187, Color::AMBER_400),
        ("Ocean Drive", "Coast Radio", 245, Color::CYAN_400),
        ("Paper Hearts", "June Ames", 198, Color::ROSE_400),
        ("Static & Stars", "Nova Sound", 230, Color::VIOLET_400),
        ("Slow Burn", "Wilder Skies", 176, Color::ORANGE_400),
        ("Glass Room", "June Ames", 205, Color::TEAL_400),
        ("Afterglow", "Coast Radio", 221, Color::PINK_400),
    ];

    raw.into_iter()
        .map(|(title, artist, duration_secs, art_color)| Track {
            title: title.to_string(),
            artist: artist.to_string(),
            duration_secs,
            art_color,
        })
        .collect()
}

// Warm coral accent layered on top of the stock dark palette, so
// active/"now playing" states read as a music app instead of a generic
// blue Material accent.
fn music_theme() -> Theme {
    Theme::dark()
        .primary(Color::rgb(255, 138, 128))
        .on_primary(Color::rgb(38, 6, 3))
        .primary_container(Color::rgb(93, 26, 18))
        .on_primary_container(Color::rgb(255, 218, 214))
        .secondary_container(Color::rgb(63, 40, 38))
        .on_secondary_container(Color::rgb(255, 218, 214))
}

fn format_duration(total_secs: u32) -> String {
    format!("{}:{:02}", total_secs / 60, total_secs % 60)
}

// ---------------------------------------------------------------------
// Small reusable pieces
// ---------------------------------------------------------------------

fn icon_button(
    icon: &str,
    color: Color,
    size: f32,
    on_click: impl FnMut(&mut EventCtx) + 'static
) -> Button {
    Button::new()
        .width(px!(size))
        .height(px!(size))
        .icon(icon)
        .icon_size(size * 0.5, size * 0.5)
        .color(color)
        .background(Color::TRANSPARENT)
        .border(Border::all(0.0, Color::TRANSPARENT).radius(size * 0.5))
        .transition_all(Transition::new(Duration::from_millis(140)).easing(Easing::EaseOut))
        .hover_style(|s, theme: &Theme| s.background(theme.surface_container_high))
        .pressed_style(|s, theme: &Theme|
            s.background(theme.surface_container_highest).scale(0.88).content_scale(1.0)
        )
        .on_click(on_click)
}

fn play_pause_button(
    theme: &Theme,
    is_playing: bool,
    on_click: impl FnMut(&mut EventCtx) + 'static
) -> Button {
    let icon = if is_playing { ICON_PAUSE } else { ICON_PLAY };
    Button::new()
        .width(px!(42.0))
        .height(px!(42.0))
        .icon(icon)
        .icon_size(17.0, 17.0)
        .color(theme.on_primary)
        .background(theme.primary)
        .border(Border::all(0.0, Color::TRANSPARENT).radius(21.0))
        .transition_all(Transition::new(Duration::from_millis(150)).easing(Easing::EaseOut))
        .hover_style(|s, theme: &Theme| s.background(theme.primary_fixed_dim))
        .pressed_style(|s, theme: &Theme|
            s.background(theme.primary_fixed_dim).scale(0.9).content_scale(1.0)
        )
        .on_click(on_click)
}

fn window_control_button(
    icon: &str,
    color: Color,
    hover_bg: Color,
    on_click: impl FnMut(&mut EventCtx) + 'static
) -> Button {
    Button::new()
        .width(px!(44.0))
        .height(pct!(100.0))
        .icon(icon)
        .icon_size(15.0, 15.0)
        .color(color)
        .background(Color::TRANSPARENT)
        .transition_all(Transition::new(Duration::from_millis(140)).easing(Easing::EaseOut))
        .hover_style(move |s, _theme: &Theme| s.background(hover_bg))
        .pressed_style(move |s, theme: &Theme|
            s.background(theme.surface_container_highest).scale(0.9).content_scale(1.0)
        )
        .on_click(on_click)
}

fn window_close_button(color: Color) -> Button {
    Button::new()
        .width(px!(44.0))
        .height(pct!(100.0))
        .icon(ICON_CLOSE)
        .icon_size(14.0, 14.0)
        .color(color)
        .background(Color::TRANSPARENT)
        .transition_all(Transition::new(Duration::from_millis(140)).easing(Easing::EaseOut))
        .hover_style(|s, _theme: &Theme| s.background(Color::rgb(196, 43, 28)).color(Color::WHITE))
        .pressed_style(|s, _theme: &Theme|
            s.background(Color::rgb(150, 30, 20)).color(Color::WHITE).scale(0.9).content_scale(1.0)
        )
        .on_click(|_ctx| xenframe::close_window())
}

fn album_art_block(size: f32, icon_size: f32, color: Color) -> View {
    View::new()
        .width(px!(size))
        .height(px!(size))
        .align_items(Align::Center)
        .justify_content(JustifyContent::Center)
        .background(color)
        .border(Border::all(0.0, Color::TRANSPARENT).radius(8.0))
        .child(
            Svg::from_string(ICON_MUSIC_NOTE)
                .color(Color::WHITE.with_alpha_f32(0.92))
                .width(icon_size as u32)
                .height(icon_size as u32)
        )
}

// ---------------------------------------------------------------------
// Titlebar
// ---------------------------------------------------------------------

fn build_titlebar(theme: &Theme, current: &Track) -> View {
    let brand = Row::new()
        .align_items(Align::Center)
        .gap(10.0, 0.0)
        .padding(Edges::only(14.0, 0.0, 0.0, 0.0))
        .child(Svg::from_string(ICON_LOGO).width(20).height(20))
        .child(
            Label::new()
                .label("Pearl Music")
                .font_size(px!(13.5))
                .font_weight(FontWeight::SemiBold)
                .color(theme.on_surface)
        );

    let now_playing_label = format!("{} — {}", current.title, current.artist);

    let center = View::new()
        .flex_grow(1.0)
        .height(pct!(100.0))
        .align_items(Align::Center)
        .justify_content(JustifyContent::Center)
        .window_drag_region(true)
        .child(
            Label::new()
                .label(now_playing_label)
                .font_size(px!(12.0))
                .color(theme.on_surface_variant)
        );

    let controls = Row::new()
        .height(pct!(100.0))
        .child(
            window_control_button(
                ICON_MINIMIZE,
                theme.on_surface_variant,
                theme.surface_container_high,
                |_ctx| xenframe::minimize_window()
            )
        )
        .child(
            window_control_button(
                ICON_MAXIMIZE,
                theme.on_surface_variant,
                theme.surface_container_high,
                |_ctx| xenframe::toggle_maximize_window()
            )
        )
        .child(window_close_button(theme.on_surface_variant));

    Row::new()
        .width(pct!(100.0))
        .height(px!(40.0))
        .min_height(px!(40.0))
        .align_items(Align::Center)
        .background(theme.surface_container)
        .border(Border::bottom(1.0, theme.outline_variant))
        .child(brand)
        .child(center)
        .child(controls)
}

// ---------------------------------------------------------------------
// Sidebar
// ---------------------------------------------------------------------

fn nav_item(
    theme: &Theme,
    icon: &str,
    label: &str,
    active: bool,
    on_click: impl FnMut(&mut EventCtx) + 'static
) -> Button {
    let (bg, fg) = if active {
        (theme.secondary_container, theme.on_secondary_container)
    } else {
        (Color::TRANSPARENT, theme.on_surface_variant)
    };

    Button::new()
        .label(label)
        .icon(icon)
        .icon_size(19.0, 19.0)
        .icon_gap(14.0)
        .justify_content(JustifyContent::Start)
        .align_items(Align::Center)
        .width(pct!(100.0))
        .padding(Edges::symmetric(14.0, 10.0))
        .border(Border::all(0.0, Color::TRANSPARENT).radius(20.0))
        .background(bg)
        .color(fg)
        .font_size(px!(13.5))
        .transition_all(Transition::new(Duration::from_millis(160)).easing(Easing::EaseOut))
        .hover_style(move |s, theme: &Theme| {
            if active { s } else { s.background(theme.surface_container_high) }
        })
        .pressed_style(|s, _theme: &Theme| s.scale(0.98).content_scale(1.0))
        .on_click(on_click)
}

fn playlists_list(theme: &Theme) -> View {
    let names: [(&str, Color); 5] = [
        ("Liked Songs", Color::RED_400),
        ("Chill Mix", Color::TEAL_400),
        ("Workout", Color::ORANGE_400),
        ("Road Trip", Color::INDIGO_400),
        ("Focus Flow", Color::EMERALD_400),
    ];

    let mut list = Column::new().flex_grow(1.0).overflow_y(Overflow::Auto).gap(0.0, 2.0);

    for (name, color) in names {
        list = list.child(
            Row::new()
                .align_items(Align::Center)
                .gap(10.0, 0.0)
                .padding(Edges::symmetric(14.0, 8.0))
                .border(Border::all(0.0, Color::TRANSPARENT).radius(8.0))
                .transition_all(Transition::new(Duration::from_millis(140)).easing(Easing::EaseOut))
                .hover_background(theme.surface_container_high)
                .child(
                    View::new()
                        .width(px!(6.0))
                        .height(px!(6.0))
                        .background(color)
                        .border(Border::all(0.0, Color::TRANSPARENT).radius(3.0))
                )
                .child(Label::new().label(name).font_size(px!(13.0)).color(theme.on_surface))
        );
    }

    list
}

fn build_sidebar(theme: &Theme, active_nav: usize, set_active_nav: SetState<usize>) -> View {
    let nav_column = Column::new()
        .gap(0.0, 2.0)
        .child(
            nav_item(theme, ICON_HOME, "Home", active_nav == 0, {
                let set_active_nav = set_active_nav.clone();
                move |_ctx| set_active_nav.set(0)
            })
        )
        .child(
            nav_item(theme, ICON_SEARCH, "Search", active_nav == 1, {
                let set_active_nav = set_active_nav.clone();
                move |_ctx| set_active_nav.set(1)
            })
        )
        .child(
            nav_item(theme, ICON_LIBRARY, "Library", active_nav == 2, move |_ctx|
                set_active_nav.set(2)
            )
        );

    Column::new()
        .width(px!(240.0))
        .min_width(px!(240.0))
        .height(pct!(100.0))
        .background(theme.surface)
        .border(Border::right(1.0, theme.outline_variant))
        .padding(Edges::only(12.0, 16.0, 12.0, 16.0))
        .gap(0.0, 10.0)
        .child(nav_column)
        .child(View::new().width(pct!(100.0)).height(px!(1.0)).background(theme.outline_variant))
        .child(
            Label::new()
                .label("PLAYLISTS")
                .font_size(px!(11.0))
                .font_weight(FontWeight::SemiBold)
                .color(theme.on_surface_variant)
                .padding(Edges::only(10.0, 4.0, 0.0, 8.0))
        )
        .child(playlists_list(theme))
}

// ---------------------------------------------------------------------
// Content: header + song list
// ---------------------------------------------------------------------

fn song_row(
    theme: &Theme,
    index: usize,
    track: &Track,
    is_current: bool,
    set_current_track: SetState<usize>,
    set_is_playing: SetState<bool>,
    set_progress: SetState<f32>
) -> View {
    let art = album_art_block(42.0, 16.0, track.art_color);

    let title_color = if is_current { theme.primary } else { theme.on_surface };

    let texts = Column::new()
        .flex_grow(1.0)
        .justify_content(JustifyContent::Center)
        .gap(0.0, 2.0)
        .child(Label::new().label(track.title.clone()).font_size(px!(13.5)).color(title_color))
        .child(
            Label::new()
                .label(track.artist.clone())
                .font_size(px!(12.0))
                .color(theme.on_surface_variant)
        );

    let duration_label = Label::new()
        .label(format_duration(track.duration_secs))
        .font_size(px!(12.0))
        .color(theme.on_surface_variant);

    Row::new()
        .width(pct!(100.0))
        .align_items(Align::Center)
        .gap(12.0, 0.0)
        .padding(Edges::symmetric(12.0, 8.0))
        .border(Border::all(0.0, Color::TRANSPARENT).radius(10.0))
        .background(if is_current { theme.surface_container_high } else { Color::TRANSPARENT })
        .transition_all(Transition::new(Duration::from_millis(140)).easing(Easing::EaseOut))
        .hover_background(theme.surface_container_high)
        .pressed_style(|s, _theme: &Theme| s.scale(0.99).content_scale(1.0))
        .child(art)
        .child(texts)
        .child(duration_label)
        .on_click(move |ctx| {
            set_current_track.set(index);
            set_is_playing.set(true);
            set_progress.set(0.0);
            ctx.request_redraw();
        })
}

fn build_content(
    theme: &Theme,
    tracks: &[Track],
    current_track: usize,
    set_current_track: SetState<usize>,
    set_is_playing: SetState<bool>,
    set_progress: SetState<f32>
) -> View {
    let search_box = Row::new()
        .align_items(Align::Center)
        .gap(10.0, 0.0)
        .width(px!(320.0))
        .padding(Edges::symmetric(16.0, 9.0))
        .background(theme.surface_container_high)
        .border(Border::all(0.0, Color::TRANSPARENT).radius(24.0))
        .child(Svg::from_string(ICON_SEARCH).color(theme.on_surface_variant).width(16).height(16))
        .child(
            TextBox::new()
                .placeholder("Search songs, artists...")
                .flex_grow(1.0)
                .font_size(px!(13.0))
                .background(Color::TRANSPARENT)
                .border(Border::all(0.0, Color::TRANSPARENT))
                .outline(StyleValue::None)
                .padding(Edges::all(0.0))
                .color(theme.on_surface)
        );

    let header = Row::new()
        .width(pct!(100.0))
        .align_items(Align::Center)
        .justify_content(JustifyContent::SpaceBetween)
        .padding(Edges::only(24.0, 22.0, 24.0, 14.0))
        .child(
            Label::new()
                .label("Your Library")
                .font_size(px!(22.0))
                .font_weight(FontWeight::Bold)
                .color(theme.on_background)
        )
        .child(search_box);

    let mut song_list = Column::new()
        .width(pct!(100.0))
        .gap(2.0, 2.0)
        .padding(Edges::only(24.0, 0.0, 24.0, 16.0));

    for (index, track) in tracks.iter().enumerate() {
        let row = song_row(
            theme,
            index,
            track,
            index == current_track,
            set_current_track.clone(),
            set_is_playing.clone(),
            set_progress.clone()
        ).key(format!("track_{index}"));
        song_list = song_list.child(row);
    }

    Column::new()
        .flex_grow(1.0)
        .height(pct!(100.0))
        .overflow_x(Overflow::Hidden)
        .overflow_y(Overflow::Auto)
        .background(theme.background)
        .child(header)
        .child(song_list)
}

// ---------------------------------------------------------------------
// Bottom player bar
// ---------------------------------------------------------------------

#[allow(clippy::too_many_arguments)]
fn build_player_bar(
    theme: &Theme,
    tracks: &[Track],
    current_track_index: usize,
    is_playing: bool,
    progress: f32,
    volume: f32,
    shuffle_on: bool,
    repeat_on: bool,
    set_current_track: SetState<usize>,
    set_is_playing: SetState<bool>,
    set_progress: SetState<f32>,
    set_volume: SetState<f32>,
    set_shuffle_on: SetState<bool>,
    set_repeat_on: SetState<bool>
) -> View {
    let track = &tracks[current_track_index];
    let tracks_len = tracks.len();

    let left = Row::new()
        .width(px!(300.0))
        .align_items(Align::Center)
        .gap(12.0, 0.0)
        .child(album_art_block(52.0, 20.0, track.art_color))
        .child(
            Column::new()
                .gap(0.0, 2.0)
                .child(
                    Label::new()
                        .label(track.title.clone())
                        .font_size(px!(13.0))
                        .color(theme.on_surface)
                )
                .child(
                    Label::new()
                        .label(track.artist.clone())
                        .font_size(px!(11.5))
                        .color(theme.on_surface_variant)
                )
        )
        .child(icon_button(ICON_HEART, theme.on_surface_variant, 34.0, |_ctx| {}));

    let shuffle_color = if shuffle_on { theme.primary } else { theme.on_surface_variant };
    let repeat_color = if repeat_on { theme.primary } else { theme.on_surface_variant };

    let prev_track = {
        let set_current_track = set_current_track.clone();
        let set_progress = set_progress.clone();
        move |ctx: &mut EventCtx| {
            let next = if current_track_index == 0 {
                tracks_len - 1
            } else {
                current_track_index - 1
            };
            set_current_track.set(next);
            set_progress.set(0.0);
            ctx.request_redraw();
        }
    };

    let next_track = {
        let set_current_track = set_current_track.clone();
        let set_progress = set_progress.clone();
        move |ctx: &mut EventCtx| {
            let next = (current_track_index + 1) % tracks_len;
            set_current_track.set(next);
            set_progress.set(0.0);
            ctx.request_redraw();
        }
    };

    let toggle_play = {
        let set_is_playing = set_is_playing.clone();
        move |ctx: &mut EventCtx| {
            set_is_playing.set(!is_playing);
            ctx.request_redraw();
        }
    };

    let toggle_shuffle = move |_ctx: &mut EventCtx| set_shuffle_on.set(!shuffle_on);
    let toggle_repeat = move |_ctx: &mut EventCtx| set_repeat_on.set(!repeat_on);

    let transport = Row::new()
        .align_items(Align::Center)
        .gap(16.0, 0.0)
        .child(icon_button(ICON_SHUFFLE, shuffle_color, 32.0, toggle_shuffle))
        .child(icon_button(ICON_SKIP_PREVIOUS, theme.on_surface, 32.0, prev_track))
        .child(play_pause_button(theme, is_playing, toggle_play))
        .child(icon_button(ICON_SKIP_NEXT, theme.on_surface, 32.0, next_track))
        .child(icon_button(ICON_REPEAT, repeat_color, 32.0, toggle_repeat));

    let elapsed = (progress * (track.duration_secs as f32)) as u32;

    let seek_row = Row::new()
        .align_items(Align::Center)
        .gap(10.0, 0.0)
        .width(px!(440.0))
        .child(
            Label::new()
                .label(format_duration(elapsed))
                .font_size(px!(11.0))
                .color(theme.on_surface_variant)
        )
        .child(
            Slider::new()
                .value(progress)
                .flex_grow(1.0)
                .fill_color(theme.primary)
                .track_color(theme.surface_container_highest)
                .on_change(move |value, ctx| {
                    set_progress.set(value);
                    ctx.request_redraw();
                })
        )
        .child(
            Label::new()
                .label(format_duration(track.duration_secs))
                .font_size(px!(11.0))
                .color(theme.on_surface_variant)
        );

    let center = Column::new()
        .align_items(Align::Center)
        .gap(0.0, 8.0)
        .child(transport)
        .child(seek_row);

    let volume_icon = if volume <= 0.001 { ICON_VOLUME_MUTE } else { ICON_VOLUME };
    let toggle_mute = {
        let set_volume = set_volume.clone();
        move |_ctx: &mut EventCtx| set_volume.set(if volume > 0.0 { 0.0 } else { 0.7 })
    };

    let volume_row = Row::new()
        .width(px!(300.0))
        .justify_content(JustifyContent::End)
        .align_items(Align::Center)
        .gap(8.0, 0.0)
        .child(icon_button(volume_icon, theme.on_surface_variant, 32.0, toggle_mute))
        .child(
            Slider::new()
                .value(volume)
                .width(px!(110.0))
                .fill_color(theme.primary)
                .track_color(theme.surface_container_highest)
                .on_change(move |value, ctx| {
                    set_volume.set(value);
                    ctx.request_redraw();
                })
        );

    Row::new()
        .width(pct!(100.0))
        .height(px!(88.0))
        .min_height(px!(88.0))
        .align_items(Align::Center)
        .justify_content(JustifyContent::SpaceBetween)
        .padding(Edges::symmetric(20.0, 0.0))
        .background(theme.surface_container)
        .border(Border::top(1.0, theme.outline_variant))
        .child(left)
        .child(center)
        .child(volume_row)
}

// ---------------------------------------------------------------------
// App entry point
// ---------------------------------------------------------------------

fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(target_arch = "wasm32")]
    {
        console_error_panic_hook::set_once();
        let _ = console_log::init_with_level(log::Level::Info);
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = env_logger::Builder
            ::new()
            .filter_module("xengui", log::LevelFilter::Info)
            .filter_level(log::LevelFilter::Warn)
            .format_timestamp(None)
            .try_init();
    }

    let config = AppConfig {
        #[cfg(not(target_arch = "wasm32"))]
        title: "Pearl Music".into(),
        #[cfg(not(target_arch = "wasm32"))]
        width: 1180,
        #[cfg(not(target_arch = "wasm32"))]
        height: 760,
        #[cfg(not(target_arch = "wasm32"))]
        position: WindowPosition::Center,
        #[cfg(not(target_arch = "wasm32"))]
        decorations: false,
        #[cfg(not(target_arch = "wasm32"))]
        start_maximized: true,
        themes: vec![music_theme()],
        active_theme: 0,
        theme_mode: xenframe::AppThemeMode::Fixed,
        ..Default::default()
    };

    let mut app = App::new(config);

    app.with_font(
        "Noto_Sans",
        include_bytes!(
            concat!(env!("CARGO_MANIFEST_DIR"), "/fonts/NotoSans-VariableFont.ttf")
        ).to_vec()
    );

    let tracks = sample_tracks();

    app.render(move || {
        let theme = xengui::current_theme();

        let (current_track, set_current_track) = use_state(0usize);
        let (is_playing, set_is_playing) = use_state(false);
        let (progress, set_progress) = use_state(0.32f32);
        let (volume, set_volume) = use_state(0.7f32);
        let (active_nav, set_active_nav) = use_state(0usize);
        let (shuffle_on, set_shuffle_on) = use_state(false);
        let (repeat_on, set_repeat_on) = use_state(false);

        let current = &tracks[current_track];

        // Mirrors the currently playing track onto the OS window
        // title/taskbar, independent of the in-app titlebar label above.
        let os_title = format!(
            "{} {} - {}",
            if is_playing {
                "▶"
            } else {
                "❚❚"
            },
            current.title,
            current.artist
        );
        let dep_key = ((current_track as u64) << 1) | (is_playing as u64);
        use_effect(
            {
                let os_title = os_title.clone();
                move || {
                    xenframe::set_window_title(&os_title);
                }
            },
            [dep_key]
        );

        let titlebar = build_titlebar(&theme, current);
        let sidebar = build_sidebar(&theme, active_nav, set_active_nav.clone());
        let content = build_content(
            &theme,
            &tracks,
            current_track,
            set_current_track.clone(),
            set_is_playing.clone(),
            set_progress.clone()
        );
        let player_bar = build_player_bar(
            &theme,
            &tracks,
            current_track,
            is_playing,
            progress,
            volume,
            shuffle_on,
            repeat_on,
            set_current_track.clone(),
            set_is_playing.clone(),
            set_progress.clone(),
            set_volume.clone(),
            set_shuffle_on.clone(),
            set_repeat_on.clone()
        );

        let main_view = Column::new()
            .width(pct!(100.0))
            .height(pct!(100.0))
            .background(theme.background)
            .child(titlebar)
            .child(Row::new().width(pct!(100.0)).flex_grow(1.0).child(sidebar).child(content))
            .child(player_bar);

        Box::new(main_view)
    });

    if let Err(e) = app.run() {
        eprintln!("Error running app: {:?}", e);
    }

    Ok(())
}
