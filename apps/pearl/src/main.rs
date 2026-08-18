// SPDX-License-Identifier: Apache-2.0
// hide console window on windows subsystem
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod components;
use components::{ AlbumArt, IconButton };

use web_time::Duration;
use xenframe::{ App, AppConfig };

#[cfg(not(target_arch = "wasm32"))]
use xenframe::WindowPosition;
use xengui::{ properties::StyleValue, * };
use xengui_icons::{ IconAxes, codepoints };

// ---------------------------------------------------------------------
// Sample data
// ---------------------------------------------------------------------

#[derive(Clone, PartialEq)]
struct Track {
    title: String,
    artist: String,
    album: String,
    explicit_content: bool,
    duration_secs: u32,
    art_color: Color,
}

#[derive(Clone)]
struct Playlist {
    id: u32,
    name: String,
    color: Color,
    track_count: u32,
}

fn sample_tracks() -> Vec<Track> {
    let raw: [(&str, &str, &str, bool, u32, Color); 8] = [
        ("Thunder", "Imagine Dragons", "Evolve", false, 214, Color::VIOLET_400),
        ("Starboy (feat. Daft Punk)", "The Weeknd", "Starboy", true, 187, Color::AMBER_400),
        ("Everlong", "Foo Fighters", "The Colour And The Shape", false, 245, Color::CYAN_400),
        ("Bohemian Rhapsody", "Queen", "A Night at The Opera", false, 198, Color::ROSE_400),
        ("Killer Queen", "Queen", "Queen Rock Montreal", false, 230, Color::PURPLE_400),
        ("Kyoto (feat. Sirah)", "Skrillex", "Bangarang EP", true, 176, Color::ORANGE_400),
        (
            "Miss You (Bonus Track)",
            "Oliver Tree & Robin Schulz",
            "Alone In A Crowd",
            false,
            205,
            Color::TEAL_400,
        ),
        ("Hurt", "Oliver Tree", "Ugly Is Beautiful", true, 221, Color::PINK_400),
    ];

    raw.into_iter()
        .map(|(title, artist, album, explicit_content, duration_secs, art_color)| Track {
            title: title.to_string(),
            artist: artist.to_string(),
            album: album.to_string(),
            explicit_content,
            duration_secs,
            art_color,
        })
        .collect()
}

fn sample_playlists() -> Vec<Playlist> {
    vec![
        Playlist { id: 1, name: "Liked Songs".into(), color: Color::VIOLET_400, track_count: 0 },
        Playlist { id: 2, name: "Playlist 1".into(), color: Color::PURPLE_400, track_count: 0 },
        Playlist { id: 3, name: "Playlist 2".into(), color: Color::INDIGO_400, track_count: 0 }
    ]
}

// Violet-based Material 3 dark palette; every derived color follows from
// the primary/secondary/tertiary hues instead of the stock blue theme.
fn pearl_theme() -> Theme {
    Theme::dark()
        .primary(Color::VIOLET_500)
        .on_primary(Color::VIOLET_950)
        .primary_container(Color::VIOLET_800)
        .on_primary_container(Color::VIOLET_100)
        .primary_fixed(Color::VIOLET_500)
        .primary_fixed_dim(Color::VIOLET_600)
        .on_primary_fixed(Color::VIOLET_950)
        .on_primary_fixed_variant(Color::VIOLET_900)
        .secondary(Color::VIOLET_300)
        .on_secondary(Color::VIOLET_950)
        .secondary_container(Color::VIOLET_900)
        .on_secondary_container(Color::VIOLET_100)
        .tertiary(Color::PURPLE_300)
        .on_tertiary(Color::PURPLE_950)
        .tertiary_container(Color::PURPLE_900)
        .on_tertiary_container(Color::PURPLE_100)
        .outline(Color::rgb(58, 48, 72))
        .outline_variant(Color::rgb(36, 30, 46))
        .surface(Color::rgb(0, 0, 0))
        .surface_bright(Color::rgb(0, 0, 0))
        .surface_dim(Color::rgb(7, 6, 10))
        .surface_container_lowest(Color::rgb(0, 0, 0))
        .surface_container_low(Color::rgb(0, 0, 0))
        .surface_container(Color::rgb(0, 0, 0))
        .surface_container_high(Color::rgb(24, 20, 31))
        .surface_container_highest(Color::rgb(30, 26, 39))
        .background(Color::rgb(0, 0, 0))
        .on_background(Color::VIOLET_50)
        .on_surface(Color::VIOLET_50)
        .on_surface_variant(Color::VIOLET_200)
        .caret_color(Color::VIOLET_400)
        .selection(Color::VIOLET_500.with_alpha(80))
        .selection_color(Color::VIOLET_100)
}

fn format_duration(total_secs: u32) -> String {
    format!("{}:{:02}", total_secs / 60, total_secs % 60)
}

fn play_pause_button(
    theme: &Theme,
    is_playing: bool,
    on_click: impl FnMut(&mut EventCtx) + 'static
) -> View {
    let codepoint = if is_playing { codepoints::PAUSE } else { codepoints::PLAY_ARROW };
    View::new()
        .width(px!(42.0))
        .height(px!(42.0))
        .align_items(Align::Center)
        .justify_content(JustifyContent::Center)
        .color(theme.on_primary)
        .background(theme.primary)
        .cursor(Cursor::Pointer)
        .border(Border::all(0.0, Color::TRANSPARENT).radius(21.0))
        .transition_all(Transition::new(Duration::from_millis(150)).easing(Easing::EaseOut))
        .hover_style(|s, theme: &Theme| s.background(theme.primary_fixed_dim))
        .pressed_style(|s, theme: &Theme|
            s.background(theme.primary_fixed_dim).scale(0.9).content_scale(1.0)
        )
        .child(
            VariableIcon::new(codepoint)
                .size(24.0)
                .axes(IconAxes::default().weight(500.0).fill(1.0))
        )
        .on_click(on_click)
}

fn window_control_button(
    codepoint: char,
    color: Color,
    hover_bg: Color,
    on_click: impl FnMut(&mut EventCtx) + 'static
) -> View {
    View::new()
        .width(px!(44.0))
        .height(pct!(100.0))
        .align_items(Align::Center)
        .justify_content(JustifyContent::Center)
        .background(Color::TRANSPARENT)
        .color(color)
        .transition_all(Transition::new(Duration::from_millis(140)).easing(Easing::EaseOut))
        .hover_style(move |s, _theme: &Theme| s.background(hover_bg))
        .pressed_style(move |s, theme: &Theme| s.background(theme.surface_container_highest))
        .child(VariableIcon::new(codepoint).size(16.0))
        .on_click(on_click)
}

fn window_close_button(color: Color) -> View {
    View::new()
        .width(px!(44.0))
        .height(pct!(100.0))
        .align_items(Align::Center)
        .justify_content(JustifyContent::Center)
        .background(Color::TRANSPARENT)
        .color(color)
        .transition_all(Transition::new(Duration::from_millis(140)).easing(Easing::EaseOut))
        .hover_style(|s, _theme: &Theme| s.background(Color::rgb(196, 43, 28)).color(Color::WHITE))
        .pressed_style(|s, _theme: &Theme|
            s.background(Color::rgb(150, 30, 20)).color(Color::WHITE)
        )
        .child(VariableIcon::new(codepoints::CLOSE).size(16.0))
        .on_click(|_ctx| xenframe::close_window())
}

fn brand_logo(theme: &Theme) -> View {
    View::new()
        .width(px!(22.0))
        .height(px!(22.0))
        .align_items(Align::Center)
        .justify_content(JustifyContent::Center)
        .background(theme.primary)
        .color(theme.on_primary)
        .border(Border::all(0.0, Color::TRANSPARENT).radius(11.0))
        .child(VariableIcon::new(codepoints::MUSIC_NOTE).size(13.0))
}

// Search bar has a static 1px border; the inner TextBox gets its own
// focus outline since the framework has no CSS `:focus-within` equivalent
// to react to a child's focus from the wrapping container.
fn search_box(theme: &Theme) -> View {
    Row::new()
        .align_items(Align::Center)
        .gap(10.0, 0.0)
        .width(pct!(100.0))
        .max_width(px!(360.0))
        .padding(Edges::symmetric(16.0, 9.0))
        .background(theme.surface_container_high)
        .border(Border::all(1.0, theme.outline_variant).radius(24.0))
        .transition_all(Transition::new(Duration::from_millis(140)).easing(Easing::EaseOut))
        .focus_within_style(|s, theme: &Theme|
            s
                .border(Border::all(1.5, theme.primary).radius(24.0))
                .outline(Outline::new(1.0, theme.primary.with_alpha(70), None, 0.0))
        )
        .child(VariableIcon::new(codepoints::SEARCH).size(20.0).color(theme.on_surface_variant))
        .child(
            TextBox::new()
                .placeholder("Search songs, artists, playlists...")
                .flex_grow(1.0)
                .font_size(px!(14.0))
                .background(Color::TRANSPARENT)
                .border(Border::all(0.0, Color::TRANSPARENT))
                .outline(StyleValue::None)
                .padding(Edges::all(0.0))
                .color(theme.on_surface)
        )
}

// ---------------------------------------------------------------------
// Sidebar / navigation
// ---------------------------------------------------------------------

fn nav_item(
    theme: &Theme,
    codepoint: char,
    axes: IconAxes,
    label: &str,
    is_active: bool,
    on_click: impl FnMut(&mut EventCtx) + 'static
) -> View {
    let (bg, fg) = if is_active {
        (theme.secondary_container, theme.on_secondary_container)
    } else {
        (Color::TRANSPARENT, theme.on_surface_variant)
    };

    View::new()
        .display(Display::Flex)
        .flex_direction(FlexDirection::Row)
        .align_items(Align::Center)
        .gap(14.0, 0.0)
        .width(pct!(100.0))
        .color(fg)
        .padding(Edges::symmetric(14.0, 10.0))
        .border(Border::all(0.0, Color::TRANSPARENT).radius(20.0))
        .background(bg)
        .transition_all(Transition::new(Duration::from_millis(160)).easing(Easing::EaseOut))
        .hover_style(move |s, theme: &Theme| {
            if is_active { s } else { s.background(theme.surface_container_high) }
        })
        .pressed_style(|s, _theme: &Theme| s.scale(0.98).content_scale(1.0))
        .child(VariableIcon::new(codepoint).size(19.0).axes(axes))
        .child(Label::new().label(label.to_string()).font_size(px!(13.5)))
        .on_click(on_click)
}

fn playlist_row(
    theme: Theme,
    playlist: Playlist,
    set_playlists: SetState<Vec<Playlist>>
) -> Box<dyn Widget> {
    let id = playlist.id;

    component(format!("playlist_row_{id}"), move || {
        let (editing, set_editing) = use_state(false);
        let (draft, set_draft) = use_state(playlist.name.to_string());

        let set_playlists_del = set_playlists.clone();
        let set_playlists_rename = set_playlists.clone();
        let set_editing_start = set_editing.clone();

        let name_widget: Box<dyn Widget> = if editing {
            Box::new(
                TextBox::new()
                    .value(draft.clone())
                    .font_size(px!(13.0))
                    .padding(Edges::all(0.0))
                    .background(Color::TRANSPARENT)
                    .border(Border::all(0.0, Color::TRANSPARENT))
                    .outline(StyleValue::None)
                    .color(theme.on_surface)
                    .on_change(move |value, _ctx| set_draft.set(value.to_string()))
                    .on_submit(move |value, ctx| {
                        let new_name = value.to_string();
                        set_playlists_rename.update(move |list| {
                            if let Some(p) = list.iter_mut().find(|p| p.id == id) {
                                p.name = new_name.clone();
                            }
                        });
                        set_editing.set(false);
                        ctx.request_redraw();
                    })
            )
        } else {
            Box::new(
                Label::new()
                    .label(playlist.name.clone())
                    .font_size(px!(13.0))
                    .color(theme.on_surface)
            )
        };

        let mut row = Row::new()
            .key(format!("playlist_{id}"))
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
                    .background(playlist.color)
                    .border(Border::all(0.0, Color::TRANSPARENT).radius(3.0))
            )
            .child(
                Column::new()
                    .flex_grow(1.0)
                    .gap(0.0, 1.0)
                    .child_boxed(name_widget)
                    .child(
                        Label::new()
                            .label(format!("{} songs", playlist.track_count))
                            .font_size(px!(11.0))
                            .color(theme.on_surface_variant)
                    )
            )
            .child(
                IconButton::new(codepoints::EDIT)
                    .color(theme.on_surface_variant)
                    .size(26.0)
                    .on_click(move |ctx| {
                        set_editing_start.set(true);
                        ctx.request_redraw();
                    })
            )
            .child(
                IconButton::new(codepoints::CLOSE)
                    .color(theme.on_surface_variant)
                    .size(26.0)
                    .on_click(move |ctx| {
                        set_playlists_del.update(move |list| list.retain(|p| p.id != id));
                        ctx.request_redraw();
                    })
            );

        row = row.on_click(move |_ctx| xen_router::push(format!("/playlist/{id}")));

        Box::new(row) as Box<dyn Widget>
    })
}

fn playlists_list(
    theme: &Theme,
    playlists: &[Playlist],
    set_playlists: SetState<Vec<Playlist>>
) -> View {
    let mut list = Column::new().flex_grow(1.0).overflow_y(Overflow::Auto).gap(0.0, 2.0);
    for playlist in playlists {
        list = list.child_boxed(
            playlist_row(theme.clone(), playlist.clone(), set_playlists.clone())
        );
    }
    list
}

fn add_playlist_row(
    theme: &Theme,
    next_playlist_id: u32,
    set_playlists: SetState<Vec<Playlist>>,
    set_next_playlist_id: SetState<u32>
) -> View {
    Row::new()
        .align_items(Align::Center)
        .gap(10.0, 0.0)
        .padding(Edges::symmetric(14.0, 8.0))
        .color(theme.on_surface_variant)
        .border(Border::all(0.0, Color::TRANSPARENT).radius(8.0))
        .transition_all(Transition::new(Duration::from_millis(140)).easing(Easing::EaseOut))
        .hover_background(theme.surface_container_high)
        .child(
            View::new()
                .width(px!(20.0))
                .height(px!(20.0))
                .align_items(Align::Center)
                .justify_content(JustifyContent::Center)
                .background(theme.surface_container_highest)
                .border(Border::all(0.0, Color::TRANSPARENT).radius(10.0))
                .child(VariableIcon::new(codepoints::ADD).size(13.0))
        )
        .child(Label::new().label("New Playlist").font_size(px!(13.0)))
        .on_click(move |ctx| {
            let id = next_playlist_id;
            set_playlists.update(move |list| {
                list.push(Playlist {
                    id,
                    name: format!("New Playlist {id}"),
                    color: Color::VIOLET_400,
                    track_count: 0,
                });
            });
            set_next_playlist_id.set(id + 1);
            ctx.request_redraw();
        })
}

fn build_sidebar(
    theme: &Theme,
    current_path: &str,
    playlists: &[Playlist],
    set_playlists: SetState<Vec<Playlist>>,
    next_playlist_id: u32,
    set_next_playlist_id: SetState<u32>
) -> View {
    let nav_column = Column::new()
        .gap(0.0, 2.0)
        .child({
            let is_active = current_path == "/";
            nav_item(
                theme,
                codepoints::HOME,
                IconAxes::default().fill(if is_active { 1.0 } else { 0.0 }),
                "Home",
                is_active,
                |_ctx| {
                    xen_router::push("/");
                }
            )
        })
        .child({
            let is_active = current_path == "/search";
            nav_item(theme, codepoints::SEARCH, IconAxes::default(), "Search", is_active, |_ctx| {
                xen_router::push("/search");
            })
        })
        .child({
            let is_active = current_path == "/library";
            nav_item(
                theme,
                codepoints::LIBRARY_MUSIC,
                IconAxes::default().fill(if is_active { 1.0 } else { 0.0 }),
                "Library",
                is_active,
                |_ctx| {
                    xen_router::push("/library");
                }
            )
        });

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
        .child(playlists_list(theme, playlists, set_playlists.clone()))
        .child(add_playlist_row(theme, next_playlist_id, set_playlists, set_next_playlist_id))
}

// ---------------------------------------------------------------------
// Titlebar - every part of the title bar (brand, center, buttons area)
// marks itself as a drag region, so grabbing the label or the logo icon
// drags the window too, not just an empty stretch of the bar.
// ---------------------------------------------------------------------

#[cfg(not(target_arch = "wasm32"))]
fn window_controls_row(theme: &Theme) -> View {
    Row::new()
        .height(pct!(100.0))
        .child(
            window_control_button(
                codepoints::MINIMIZE,
                theme.on_surface_variant,
                theme.surface_container_high,
                |_ctx| xenframe::minimize_window()
            )
        )
        .child(
            window_control_button(
                codepoints::MAXIMIZE,
                theme.on_surface_variant,
                theme.surface_container_high,
                |_ctx| xenframe::toggle_maximize_window()
            )
        )
        .child(window_close_button(theme.on_surface_variant))
}

// Browser has no native window chrome to control - the titlebar itself
// still renders, just without minimize/maximize/close.
#[cfg(target_arch = "wasm32")]
fn window_controls_row(_theme: &Theme) -> View {
    Row::new().height(pct!(100.0))
}

fn build_titlebar(theme: &Theme, current: &Track) -> View {
    let brand = Row::new()
        .align_items(Align::Center)
        .gap(8.0, 0.0)
        .padding(Edges::only(10.0, 0.0, 0.0, 0.0))
        .window_drag_region(true)
        .child(brand_logo(theme))
        .child(
            Label::new()
                .label("Pearl")
                .font_size(px!(13.5))
                .font_weight(FontWeight::SemiBold)
                .color(theme.on_surface)
        );

    let now_playing_label = format!("{} - {}", current.title, current.artist);

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

    //let controls = window_controls_row(theme);

    Row::new()
        .width(pct!(100.0))
        .height(px!(40.0))
        .min_height(px!(40.0))
        .align_items(Align::Center)
        .background(theme.surface_container.with_alpha_f32(0.75))
        .backdrop_filter(Filter::Blur(px!(16.0)))
        .border(Border::bottom(1.0, theme.outline_variant))
        .window_drag_region(true)
        .child(brand)
        .child(center)
}

// ---------------------------------------------------------------------
// Pages
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
    let art = AlbumArt::new(track.art_color).size(42.0).icon_size(16.0);

    let title_color = if is_current { theme.primary } else { theme.on_surface };

    let texts = Column::new()
        .flex_grow(1.0)
        .justify_content(JustifyContent::Center)
        .gap(0.0, 2.0)
        .child(
            Label::new()
                .font_weight(FontWeight::SemiBold)
                .label(track.title.clone())
                .font_size(px!(13.5))
                .color(title_color)
        )
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

fn build_home_page(
    theme: &Theme,
    tracks: &[Track],
    current_track: usize,
    set_current_track: SetState<usize>,
    set_is_playing: SetState<bool>,
    set_progress: SetState<f32>
) -> Box<dyn Widget> {
    let header = Row::new()
        .width(pct!(100.0))
        .align_items(Align::Center)
        .child(
            Label::new()
                .label("Recently Played")
                .font_size(px!(20.0))
                .font_weight(FontWeight::Bold)
                .color(theme.on_background)
        );

    let mut song_list = Column::new().width(pct!(100.0)).gap(2.0, 2.0);

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

    Box::new(
        Column::new()
            .flex_grow(1.0)
            .min_height(pct!(100.0))
            .overflow_x(Overflow::Hidden)
            .overflow_y(Overflow::Scroll)
            .padding(Edges::only(22, 20, 0, 20))
            .gap(0, 16)
            .background(theme.background)
            .child(header)
            .child(
                View::new()
                    .display(Display::Flex)
                    .flex_direction(FlexDirection::Column)
                    .background(Color::NEUTRAL_800)
                    .border(Border::all(1.0, Color::NEUTRAL_600).radius(20))
                    .padding(px!(14))
                    .gap(0, 12)
                    .child(AlbumArt::new(Color::BLUE_500).size(128.0).icon_size(48.0))
                    .child(
                        View::new()
                            .display(Display::Flex)
                            .flex_direction(FlexDirection::Column)
                            .gap(0, 4)
                            .child(
                                Label::new()
                                    .label("Miss You (Bonus Track)")
                                    .font_weight(FontWeight::Bold)
                                    .font_size(16)
                            )
                            .child(
                                Label::new()
                                    .label("Track • Oliver Tree & Robin Schulz")
                                    .font_weight(FontWeight::Medium)
                                    .font_size(14)
                            )
                    )
            )
            .child(song_list)
    )
}

fn build_search_page(theme: &Theme) -> Box<dyn Widget> {
    let genres: [(&str, Color); 8] = [
        ("Pop", Color::VIOLET_500),
        ("Hip-Hop", Color::PURPLE_500),
        ("Chill", Color::INDIGO_500),
        ("Rock", Color::ROSE_500),
        ("Electronic", Color::CYAN_500),
        ("Jazz", Color::AMBER_500),
        ("Focus", Color::TEAL_500),
        ("Party", Color::PINK_500),
    ];

    let header = Row::new()
        .width(pct!(100.0))
        .padding(Edges::only(24.0, 22.0, 24.0, 16.0))
        .child(search_box(theme));

    let mut grid = View::new()
        .display(Display::Flex)
        .flex_direction(FlexDirection::Row)
        .flex_wrap(FlexWrap::Wrap)
        .gap(12.0, 12.0)
        .padding(Edges::only(24.0, 0.0, 24.0, 96.0));

    for (name, color) in genres {
        grid = grid.child(
            View::new()
                .width(px!(160.0))
                .height(px!(90.0))
                .padding(Edges::all(12.0))
                .background(color)
                .color(Color::WHITE)
                .border(Border::all(0.0, Color::TRANSPARENT).radius(12.0))
                .child(
                    Label::new().label(name).font_size(px!(15.0)).font_weight(FontWeight::SemiBold)
                )
        );
    }

    Box::new(
        Column::new()
            .flex_grow(1.0)
            .height(pct!(100.0))
            .overflow_y(Overflow::Auto)
            .background(theme.background)
            .child(header)
            .child(grid)
    )
}

fn build_library_page(theme: &Theme, playlists: &[Playlist]) -> Box<dyn Widget> {
    let header = Row::new()
        .width(pct!(100.0))
        .align_items(Align::Center)
        .padding(Edges::only(24.0, 22.0, 24.0, 14.0))
        .child(
            Label::new()
                .label("Your Library")
                .font_size(px!(22.0))
                .font_weight(FontWeight::Bold)
                .color(theme.on_background)
        );

    let mut grid = View::new()
        .display(Display::Flex)
        .flex_direction(FlexDirection::Row)
        .flex_wrap(FlexWrap::Wrap)
        .gap(16.0, 16.0)
        .padding(Edges::only(24.0, 0.0, 24.0, 96.0));

    if playlists.is_empty() {
        grid = grid.child(
            Label::new()
                .label("No playlists yet — create one from the sidebar.")
                .font_size(px!(13.0))
                .color(theme.on_surface_variant)
        );
    }

    for playlist in playlists {
        grid = grid.child(
            Column::new()
                .width(px!(170.0))
                .gap(0.0, 8.0)
                .key(format!("lib_playlist_{}", playlist.id))
                .child(
                    View::new()
                        .width(px!(170.0))
                        .height(px!(170.0))
                        .align_items(Align::Center)
                        .justify_content(JustifyContent::Center)
                        .background(playlist.color)
                        .color(Color::WHITE.with_alpha_f32(0.9))
                        .border(Border::all(0.0, Color::TRANSPARENT).radius(12.0))
                        .child(VariableIcon::new(codepoints::LIBRARY_MUSIC).size(48.0))
                )
                .child(
                    Label::new()
                        .label(playlist.name.clone())
                        .font_size(px!(13.5))
                        .color(theme.on_surface)
                )
                .child(
                    Label::new()
                        .label(format!("{} songs", playlist.track_count))
                        .font_size(px!(11.5))
                        .color(theme.on_surface_variant)
                )
        );
    }

    Box::new(
        Column::new()
            .flex_grow(1.0)
            .height(pct!(100.0))
            .overflow_y(Overflow::Auto)
            .background(theme.background)
            .child(header)
            .child(grid)
    )
}

fn build_playlist_page(theme: &Theme, playlist: &Playlist, tracks: &[Track]) -> Box<dyn Widget> {
    let header = Row::new()
        .width(pct!(100.0))
        .align_items(Align::Center)
        .gap(16.0, 0.0)
        .padding(Edges::only(24.0, 22.0, 24.0, 14.0))
        .child(AlbumArt::new(playlist.color).size(88.0).icon_size(34.0))
        .child(
            Column::new()
                .gap(0.0, 4.0)
                .child(
                    Label::new()
                        .label("PLAYLIST")
                        .font_size(px!(11.0))
                        .font_weight(FontWeight::SemiBold)
                        .color(theme.on_surface_variant)
                )
                .child(
                    Label::new()
                        .label(playlist.name.clone())
                        .font_size(px!(24.0))
                        .font_weight(FontWeight::Bold)
                        .color(theme.on_background)
                )
                .child(
                    View::new()
                        .display(Display::Flex)
                        .flex_direction(FlexDirection::Row)
                        .align_items(Align::Center)
                        .justify_content(JustifyContent::Start)
                        .child(
                            Label::new()
                                .label(
                                    format!(
                                        "{} songs",
                                        playlist.track_count.max(tracks.len() as u32)
                                    )
                                )
                                .font_size(px!(12.5))
                                .color(theme.on_surface_variant)
                        )
                        .child(
                            Label::new()
                                .label(" - ")
                                .font_size(px!(12.5))
                                .color(theme.on_surface_variant)
                        )
                        .child(
                            Label::new()
                                .label(
                                    format!(
                                        "{} hours",
                                        playlist.track_count.max(tracks.len() as u32)
                                    )
                                )
                                .font_size(px!(12.5))
                                .color(theme.on_surface_variant)
                        )
                )
        );

    let mut list = Column::new()
        .width(pct!(100.0))
        .gap(2.0, 2.0)
        .padding(Edges::only(24.0, 0.0, 24.0, 96.0));

    // Sample data has no real per-playlist track association yet, so every
    // playlist page previews the same shared library for now.
    for (index, track) in tracks.iter().enumerate() {
        list = list.child(
            Row::new()
                .key(format!("pl_track_{index}"))
                .width(pct!(100.0))
                .align_items(Align::Center)
                .gap(12.0, 0.0)
                .padding(Edges::symmetric(12.0, 8.0))
                .border(Border::all(0.0, Color::TRANSPARENT).radius(10.0))
                .transition_all(Transition::new(Duration::from_millis(140)).easing(Easing::EaseOut))
                .hover_background(theme.surface_container_high)
                .child(AlbumArt::new(track.art_color).size(42.0).icon_size(16.0))
                .child(
                    Column::new()
                        .flex_grow(1.0)
                        .gap(0.0, 2.0)
                        .child(
                            Label::new()
                                .label(track.title.clone())
                                .font_weight(FontWeight::Bold)
                                .font_size(px!(13.5))
                                .color(theme.on_surface)
                        )
                        .child(
                            Label::new()
                                .label(track.artist.clone())
                                .font_size(px!(12.0))
                                .color(theme.on_surface_variant)
                        )
                )
                .child(
                    Label::new()
                        .label(format_duration(track.duration_secs))
                        .font_size(px!(12.0))
                        .color(theme.on_surface_variant)
                )
        );
    }

    Box::new(
        Column::new()
            .flex_grow(1.0)
            .height(pct!(100.0))
            .overflow_y(Overflow::Auto)
            .background(theme.background)
            .child(header)
            .child(list)
    )
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
    show_side_panels: bool,
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
        .child(AlbumArt::new(track.art_color).size(52.0).icon_size(20.0))
        .child(
            Column::new()
                .gap(0.0, 2.0)
                .child(
                    Label::new()
                        .label(track.title.clone())
                        .font_weight(FontWeight::Bold)
                        .font_size(px!(14.0))
                        .color(theme.on_surface)
                )
                .child(
                    Label::new()
                        .label(track.artist.clone())
                        .font_size(px!(13))
                        .color(theme.on_surface_variant)
                )
        )
        .child(IconButton::new(codepoints::FAVORITE).color(theme.on_surface_variant).size(34.0));

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
        .child(
            IconButton::new(codepoints::SHUFFLE)
                .color(shuffle_color)
                .size(36.0)
                .cursor(Cursor::Pointer)
                .on_click(toggle_shuffle)
        )
        .child(
            IconButton::new(codepoints::SKIP_PREVIOUS)
                .color(theme.on_surface)
                .size(36.0)
                .cursor(Cursor::Pointer)
                .axes(IconAxes::default().weight(500.0).fill(1.0))
                .on_click(prev_track)
        )
        .child(play_pause_button(theme, is_playing, toggle_play))
        .child(
            IconButton::new(codepoints::SKIP_NEXT)
                .color(theme.on_surface)
                .size(36.0)
                .cursor(Cursor::Pointer)
                .axes(IconAxes::default().weight(500.0).fill(1.0))
                .on_click(next_track)
        )
        .child(
            IconButton::new(codepoints::REPEAT)
                .color(repeat_color)
                .size(36.0)
                .cursor(Cursor::Pointer)
                .on_click(toggle_repeat)
        );

    let elapsed = (progress * (track.duration_secs as f32)) as u32;

    let seek_row = Row::new()
        .align_items(Align::Center)
        .gap(10.0, 0.0)
        .width(px!(if show_side_panels { 440.0 } else { 220.0 }))
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

    let mut bar = Row::new()
        .width(pct!(100.0))
        .height(px!(88.0))
        .min_height(px!(88.0))
        .align_items(Align::Center)
        .justify_content(JustifyContent::SpaceBetween)
        .padding(Edges::symmetric(20.0, 0.0))
        .background(theme.surface_container)
        .border(Border::top(1.0, theme.outline_variant));

    if show_side_panels {
        bar = bar.child(left);
    }
    bar = bar.child(center);

    if show_side_panels {
        let volume_icon = if volume <= 0.001 {
            codepoints::VOLUME_OFF
        } else {
            codepoints::VOLUME_UP
        };
        let toggle_mute = {
            let set_volume = set_volume.clone();
            move |_ctx: &mut EventCtx| set_volume.set(if volume > 0.0 { 0.0 } else { 0.7 })
        };

        let volume_row = Row::new()
            .width(px!(300.0))
            .justify_content(JustifyContent::End)
            .align_items(Align::Center)
            .gap(4.0, 0.0)
            .child(
                IconButton::new(volume_icon)
                    .color(theme.on_surface_variant)
                    .size(36.0)
                    .cursor(Cursor::Pointer)
                    .axes(IconAxes::default().weight(500.0).fill(1.0))
                    .on_click(toggle_mute)
            )
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

        bar = bar.child(volume_row);
    }

    bar
}

fn build_mini_player(
    theme: &Theme,
    track: &Track,
    is_playing: bool,
    progress: f32,
    toggle_play: impl Fn(&mut EventCtx) + 'static
) -> View {
    let seek = Slider::new()
        .position(Position::Sticky)
        .value(progress)
        .bottom(0)
        .width(pct!(100.0))
        .fill_color(theme.primary)
        .track_color(theme.surface_container_highest.with_alpha_f32(0.8));

    let info_row = Row::new()
        .width(pct!(100.0))
        .align_items(Align::Center)
        .gap(10.0, 0.0)
        .padding(Edges::only(10, 10, 8, 2))
        .child(AlbumArt::new(track.art_color).size(40.0).icon_size(16.0))
        .child(
            Column::new()
                .flex_grow(1.0)
                .gap(0.0, 2.0)
                .child(
                    Label::new()
                        .label(track.title.clone())
                        .font_size(px!(13))
                        .font_weight(FontWeight::Bold)
                        .color(theme.on_surface)
                )
                .child(
                    Label::new()
                        .label(track.artist.clone())
                        .font_size(px!(12.0))
                        .color(theme.on_surface_variant)
                )
        )
        .child(
            IconButton::new(if is_playing { codepoints::PAUSE } else { codepoints::PLAY_ARROW })
                .color(theme.on_surface)
                .size(40.0)
                .axes(IconAxes::default().fill(1.0))
                .cursor(Cursor::Pointer)
                .on_click(toggle_play)
        );

    Column::new()
        .width(pct!(95.0))
        .background(theme.surface_container.with_alpha_f32(0.72))
        .backdrop_filter(Filter::Blur(px!(20.0)))
        .border(Border::all(1.0, theme.outline_variant.with_alpha_f32(0.5)).radius(20.0))
        .child(info_row)
        .child(seek)
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
        themes: vec![pearl_theme()],
        active_theme: 0,
        theme_mode: xenframe::AppThemeMode::Fixed,
        ..Default::default()
    };

    let mut app = App::new(config);

    app.with_font(
        "Nunito",
        include_bytes!(
            concat!(env!("CARGO_MANIFEST_DIR"), "/fonts/Nunito-VariableFont_wght.ttf")
        ).to_vec()
    );

    let tracks = sample_tracks();

    app.render(move || {
        let theme = xengui::current_theme();

        let (current_track, set_current_track) = use_state(0usize);
        let (is_playing, set_is_playing) = use_state(false);
        let (progress, set_progress) = use_state(0.32f32);
        let (volume, set_volume) = use_state(0.7f32);
        let (shuffle_on, set_shuffle_on) = use_state(false);
        let (repeat_on, set_repeat_on) = use_state(false);
        let (playlists, set_playlists) = use_state(sample_playlists());
        let (next_playlist_id, set_next_playlist_id) = use_state(4u32);

        let current_path = xen_router::current_path();
        let current = tracks[current_track].clone();

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

        // Sidebar only shows at Md+; below that we switch to the floating
        // bottom nav bubble, matching a phone/tablet-friendly layout.
        let show_sidebar = xengui::responsive_bool(xengui::Breakpoint::Md, true);

        let titlebar = build_titlebar(&theme, &current);

        let content: Box<dyn Widget> = {
            let theme_home = theme.clone();
            let tracks_home = tracks.clone();
            let set_current_track_home = set_current_track.clone();
            let set_is_playing_home = set_is_playing.clone();
            let set_progress_home = set_progress.clone();

            let theme_search = theme.clone();

            let theme_library = theme.clone();
            let playlists_library = playlists.clone();

            xen_router::Router
                ::new()
                .route("/", move |_params| {
                    build_home_page(
                        &theme_home,
                        &tracks_home,
                        current_track,
                        set_current_track_home.clone(),
                        set_is_playing_home.clone(),
                        set_progress_home.clone()
                    )
                })
                .route("/search", move |_params| { build_search_page(&theme_search) })
                .route("/playlist/:id", {
                    let theme_playlist = theme.clone();
                    let playlists_playlist = playlists.clone();
                    let tracks_playlist = tracks.clone();
                    move |params| {
                        let id: u32 = params
                            .get("id")
                            .and_then(|s| s.parse().ok())
                            .unwrap_or(0);
                        match playlists_playlist.iter().find(|p| p.id == id) {
                            Some(playlist) =>
                                build_playlist_page(&theme_playlist, playlist, &tracks_playlist),
                            None => Box::new(View::new()) as Box<dyn Widget>,
                        }
                    }
                })
                .route("/library", move |_params| {
                    build_library_page(&theme_library, &playlists_library)
                })
                .not_found(|| Box::new(View::new()) as Box<dyn Widget>)
                .build()
        };

        let mut body = Row::new().width(pct!(100.0)).flex_grow(1.0);

        if show_sidebar {
            body = body.child(
                build_sidebar(
                    &theme,
                    &current_path,
                    &playlists,
                    set_playlists.clone(),
                    next_playlist_id,
                    set_next_playlist_id.clone()
                )
            );
        }
        body = body.child_boxed(content);

        let mut main_view = Column::new()
            .width(pct!(100.0))
            .height(pct!(100.0))
            .background(theme.background)
            .font("Nunito")
            .child(titlebar)
            .child(body);

        if show_sidebar {
            let player_bar = build_player_bar(
                &theme,
                &tracks,
                current_track,
                is_playing,
                progress,
                volume,
                shuffle_on,
                repeat_on,
                show_sidebar,
                set_current_track.clone(),
                set_is_playing.clone(),
                set_progress.clone(),
                set_volume.clone(),
                set_shuffle_on.clone(),
                set_repeat_on.clone()
            );
            main_view = main_view.child(player_bar);
        } else {
            let active_index = match current_path.as_str() {
                "/search" => 1,
                "/library" => 2,
                _ => 0,
            };

            let nav = NavigationBar::new()
                .item(NavItem::new(codepoints::HOME, "Home"))
                .item(NavItem::new(codepoints::SEARCH, "Search"))
                .item(NavItem::new(codepoints::LIBRARY_MUSIC, "Library"))
                .active_index(active_index)
                .on_select(|index| {
                    let path = match index {
                        1 => "/search",
                        2 => "/library",
                        _ => "/",
                    };
                    xen_router::push(path);
                });

            let set_is_playing_mini = set_is_playing.clone();
            let mini_player = build_mini_player(
                &theme,
                &tracks[current_track],
                is_playing,
                progress,
                move |ctx| {
                    set_is_playing_mini.set(!is_playing);
                    ctx.request_redraw();
                }
            );

            // Mini player sits directly above the pill nav, sharing one
            // fixed bottom-anchored stack so they always move together.
            let floating_stack = Column::new()
                .position(Position::Fixed)
                .bottom(px!(16.0))
                .left(px!(0.0))
                .width(pct!(100.0))
                .align_items(Align::Center)
                .gap(0.0, 10.0)
                .child(mini_player)
                .child(nav);

            main_view = main_view.child(floating_stack);
        }

        Box::new(main_view)
    });

    if let Err(e) = app.run() {
        eprintln!("Error running app: {:?}", e);
    }

    Ok(())
}
