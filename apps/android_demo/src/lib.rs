use std::cell::RefCell;
use std::collections::HashMap;
use web_time::Duration;
use xen_router::Router;
use xenframe::{App, AppConfig, AppThemeMode, WindowPosition};
use xengui::{
    Align, Border, BorderRadius, Breakpoint, Color, Display, Easing, Edges, FlexDirection,
    FontWeight, JustifyContent, Label, Length, Overflow, ScrollState, SetState, StyleBuilder,
    Switch, TextBox, Theme, Transition, VariableIcon, View, Widget, pct, px, responsive_bool,
    use_effect, use_state,
};
use xengui_icons::{IconAxes, codepoints};

const BACKGROUND: Color = Color::rgb(15, 23, 24);
const CARD: Color = Color::rgb(32, 43, 46);
const SEARCH: Color = Color::rgb(0, 0, 0);
const TEXT: Color = Color::rgb(222, 231, 234);
const MUTED: Color = Color::rgb(142, 153, 157);

thread_local! {
    static ROUTE_SCROLL_STATES: RefCell<HashMap<String, ScrollState>> = RefCell::new(HashMap::new());
}

fn route_scroll_state(route: impl Into<String>) -> ScrollState {
    let route = route.into();
    ROUTE_SCROLL_STATES.with(|states| {
        states
            .borrow_mut()
            .entry(route)
            .or_insert_with(ScrollState::new)
            .clone()
    })
}

#[derive(Clone, Copy)]
struct Category {
    route: &'static str,
    title: &'static str,
    subtitle: &'static str,
    icon: char,
    accent: Color,
    icon_color: Color,
    details: &'static [&'static str],
}

#[derive(Clone, Copy)]
enum DetailKind {
    Navigation,
    Toggle,
    Value(&'static str),
}

const NETWORK_DETAILS: &[&str] = &[
    "Internet",
    "Calls & SMS",
    "SIMs",
    "Hotspot & tethering",
    "VPN",
    "Private DNS",
];
const DEVICES_DETAILS: &[&str] = &[
    "Pair new device",
    "Previously connected devices",
    "Connection preferences",
    "Chromebook",
    "Nearby Share",
];
const APPS_DETAILS: &[&str] = &[
    "See all apps",
    "Default apps",
    "Screen time",
    "Unused apps",
    "Special app access",
];
const NOTIFICATION_DETAILS: &[&str] = &[
    "App notifications",
    "Notification history",
    "Conversations",
    "Bubbles",
    "Lock screen notifications",
];
const SOUND_DETAILS: &[&str] = &[
    "Media volume",
    "Call volume",
    "Ring & notification volume",
    "Alarm volume",
    "Vibration & haptics",
];
const MODES_DETAILS: &[&str] = &[
    "Do Not Disturb",
    "Bedtime",
    "Driving",
    "Game Dashboard",
    "Flip to Shhh",
];
const DISPLAY_DETAILS: &[&str] = &[
    "Brightness level",
    "Dark theme",
    "Screen timeout",
    "Colors",
    "Display size and text",
    "Screen saver",
];
const WALLPAPER_DETAILS: &[&str] = &[
    "Change wallpaper",
    "Lock screen",
    "Colors",
    "Themed icons",
    "App grid",
];
const STORAGE_DETAILS: &[&str] = &["Apps", "Images", "Videos", "Audio", "Documents", "System"];
const BATTERY_DETAILS: &[&str] = &[
    "Battery usage",
    "Battery Saver",
    "Adaptive Battery",
    "Battery percentage",
];
const SECURITY_DETAILS: &[&str] = &[
    "Screen lock",
    "Face & fingerprint unlock",
    "Google Play system update",
    "Find My Device",
];
const PRIVACY_DETAILS: &[&str] = &[
    "Privacy dashboard",
    "Permission manager",
    "Camera access",
    "Microphone access",
    "Show passwords",
];
const LOCATION_DETAILS: &[&str] = &[
    "Use location",
    "App location permissions",
    "Location services",
    "Recent access",
];
const SAFETY_DETAILS: &[&str] = &[
    "Emergency SOS",
    "Car crash detection",
    "Emergency sharing",
    "Safety check",
];
const PASSWORD_DETAILS: &[&str] = &[
    "Google Password Manager",
    "Autofill service",
    "Passkeys",
    "Add account",
];
const SYSTEM_DETAILS: &[&str] = &[
    "Languages",
    "Keyboard",
    "Gestures",
    "Date & time",
    "Backup",
    "System update",
];
const ABOUT_DETAILS: &[&str] = &[
    "Device name",
    "Android version",
    "IP address",
    "Wi-Fi MAC address",
    "Build number",
];
const PROFILE_DETAILS: &[&str] = &[
    "Manage your Google Account",
    "Google services",
    "Backup",
    "Find My Device",
    "Parental controls",
];

const CATEGORIES: &[Category] = &[
    Category {
        route: "/network",
        title: "Network & internet",
        subtitle: "Mobile, Wi-Fi, hotspot",
        icon: codepoints::WIFI,
        accent: Color::rgb(96, 207, 247),
        icon_color: Color::rgb(0, 78, 111),
        details: NETWORK_DETAILS,
    },
    Category {
        route: "/connected-devices",
        title: "Connected devices",
        subtitle: "Bluetooth, pairing",
        icon: codepoints::DEVICES,
        accent: Color::rgb(96, 207, 247),
        icon_color: Color::rgb(0, 78, 111),
        details: DEVICES_DETAILS,
    },
    Category {
        route: "/apps",
        title: "Apps",
        subtitle: "Assistant, recent apps, default apps",
        icon: codepoints::APPS,
        accent: Color::rgb(151, 193, 255),
        icon_color: Color::rgb(0, 65, 145),
        details: APPS_DETAILS,
    },
    Category {
        route: "/notifications",
        title: "Notifications",
        subtitle: "Notification history, conversations",
        icon: codepoints::NOTIFICATIONS,
        accent: Color::rgb(255, 151, 213),
        icon_color: Color::rgb(151, 0, 85),
        details: NOTIFICATION_DETAILS,
    },
    Category {
        route: "/sound",
        title: "Sound & vibration",
        subtitle: "Volume and haptics",
        icon: codepoints::VOLUME_UP,
        accent: Color::rgb(255, 151, 213),
        icon_color: Color::rgb(151, 0, 85),
        details: SOUND_DETAILS,
    },
    Category {
        route: "/modes",
        title: "Modes",
        subtitle: "Do Not Disturb, Bedtime, Driving",
        icon: codepoints::DO_NOT_DISTURB,
        accent: Color::rgb(255, 151, 213),
        icon_color: Color::rgb(151, 0, 85),
        details: MODES_DETAILS,
    },
    Category {
        route: "/display",
        title: "Display & touch",
        subtitle: "Dark theme, font size, touch",
        icon: codepoints::DISPLAY_SETTINGS,
        accent: Color::rgb(255, 178, 124),
        icon_color: Color::rgb(128, 55, 0),
        details: DISPLAY_DETAILS,
    },
    Category {
        route: "/wallpaper",
        title: "Wallpaper & style",
        subtitle: "Colors, themed icons, app grid",
        icon: codepoints::PALETTE,
        accent: Color::rgb(255, 178, 124),
        icon_color: Color::rgb(128, 55, 0),
        details: WALLPAPER_DETAILS,
    },
    Category {
        route: "/storage",
        title: "Storage",
        subtitle: "74% used · 33.1 GB free",
        icon: codepoints::STORAGE,
        accent: Color::rgb(255, 194, 18),
        icon_color: Color::rgb(94, 67, 0),
        details: STORAGE_DETAILS,
    },
    Category {
        route: "/battery",
        title: "Battery",
        subtitle: "100% · About 2 days left",
        icon: codepoints::BATTERY_FULL,
        accent: Color::rgb(116, 224, 153),
        icon_color: Color::rgb(0, 85, 40),
        details: BATTERY_DETAILS,
    },
    Category {
        route: "/security",
        title: "Security",
        subtitle: "Screen lock, device protection",
        icon: codepoints::SECURITY,
        accent: Color::rgb(151, 193, 255),
        icon_color: Color::rgb(0, 65, 145),
        details: SECURITY_DETAILS,
    },
    Category {
        route: "/privacy",
        title: "Privacy",
        subtitle: "Permissions, account activity",
        icon: codepoints::PRIVACY,
        accent: Color::rgb(160, 202, 255),
        icon_color: Color::rgb(0, 65, 115),
        details: PRIVACY_DETAILS,
    },
    Category {
        route: "/location",
        title: "Location",
        subtitle: "On · 3 apps have access",
        icon: codepoints::LOCATION_ON,
        accent: Color::rgb(116, 224, 153),
        icon_color: Color::rgb(0, 85, 40),
        details: LOCATION_DETAILS,
    },
    Category {
        route: "/safety",
        title: "Safety & emergency",
        subtitle: "Emergency SOS, alerts",
        icon: codepoints::SAFETY_CHECK,
        accent: Color::rgb(255, 180, 171),
        icon_color: Color::rgb(145, 0, 10),
        details: SAFETY_DETAILS,
    },
    Category {
        route: "/passwords",
        title: "Passwords & accounts",
        subtitle: "Saved passwords, autofill",
        icon: codepoints::PASSWORD,
        accent: Color::rgb(211, 188, 255),
        icon_color: Color::rgb(78, 45, 130),
        details: PASSWORD_DETAILS,
    },
    Category {
        route: "/system",
        title: "System",
        subtitle: "Languages, gestures, backup",
        icon: codepoints::SETTINGS,
        accent: Color::rgb(196, 199, 255),
        icon_color: Color::rgb(50, 54, 135),
        details: SYSTEM_DETAILS,
    },
    Category {
        route: "/about",
        title: "About phone",
        subtitle: "Pixel · Android 14",
        icon: codepoints::INFO,
        accent: Color::rgb(196, 199, 255),
        icon_color: Color::rgb(50, 54, 135),
        details: ABOUT_DETAILS,
    },
];

fn spatial_motion() -> Transition {
    Transition::new(Duration::from_millis(350)).easing(Easing::cubic_bezier(0.42, 1.67, 0.21, 0.9))
}

fn effect_motion() -> Transition {
    Transition::new(Duration::from_millis(150)).easing(Easing::cubic_bezier(0.31, 0.94, 0.34, 1.0))
}

fn page_motion() -> Transition {
    Transition::new(Duration::from_millis(400)).easing(Easing::cubic_bezier(0.05, 0.7, 0.1, 1.0))
}

fn icon(codepoint: char, size: f32, filled: bool) -> VariableIcon {
    VariableIcon::new(codepoint).size(size).axes(
        IconAxes::default()
            .fill(if filled { 1.0 } else { 0.0 })
            .weight(500.0)
            .optical_size(size),
    )
}

fn platform_top_inset() -> f32 {
    xengui::safe_area_insets().top
}

fn page_shell(child: impl Widget + 'static) -> Box<dyn Widget> {
    let (entered, set_entered) = use_state(false);
    use_effect(
        move || {
            xengui::task::spawn(async move {
                // Keep the entering offset on screen for one committed frame
                // so the transition manager has a real starting value.
                xengui::task::yield_now().await;
                set_entered.set(true);
            });
        },
        (),
    );

    let (enter_x, enter_y) = if xengui::current_breakpoint() >= Breakpoint::Sm {
        (0.0, -48.0)
    } else {
        match xen_router::navigation_direction() {
            xen_router::NavigationDirection::Backward => (-48.0, 0.0),
            xen_router::NavigationDirection::Forward | xen_router::NavigationDirection::Replace => {
                (48.0, 0.0)
            }
        }
    };

    Box::new(
        View::new()
            .font("Noto_Sans")
            .display(Display::Flex)
            .flex_direction(FlexDirection::Column)
            .align_items(Align::Center)
            .width(pct!(100.0))
            .height(pct!(100.0))
            .background(BACKGROUND)
            .child(
                View::new()
                    .display(Display::Flex)
                    .flex_direction(FlexDirection::Column)
                    .width(pct!(100.0))
                    .max_width(px!(720.0))
                    .height(pct!(100.0))
                    .margin(Edges::only(
                        if entered { 0.0 } else { enter_x },
                        if entered { 0.0 } else { enter_y },
                        0.0,
                        0.0,
                    ))
                    .transition_all(page_motion())
                    .background(BACKGROUND)
                    .child(child),
            ),
    )
}

fn search_entry() -> View {
    View::new()
        .display(Display::Flex)
        .flex_direction(FlexDirection::Row)
        .align_items(Align::Center)
        .gap(16.0, 0.0)
        .width(pct!(100.0))
        .height(px!(56.0))
        .min_height(px!(56.0))
        .padding(Edges::symmetric(24.0, 0.0))
        .background(SEARCH)
        .color(MUTED)
        .border(Border::all(1.0, Color::rgb(61, 72, 75)).radius(28.0))
        .transition_all(spatial_motion())
        .transition_colors(effect_motion())
        .hover_style(|style, _| {
            style
                .background(Color::rgb(22, 31, 33))
                .border(Border::all(1.0, Color::rgb(80, 91, 94)).radius(28.0))
        })
        .pressed_style(|style, _| {
            style.border(Border::all(1.0, Color::rgb(80, 91, 94)).radius(16.0))
        })
        .child(icon(codepoints::SEARCH, 24.0, false))
        .child(
            Label::new()
                .label("Search Settings")
                .font_size(px!(22.0))
                .line_height(px!(28.0))
                .color(MUTED),
        )
        .on_click(|_| xen_router::push("/search"))
}

fn profile_card() -> View {
    View::new()
        .display(Display::Flex)
        .flex_direction(FlexDirection::Row)
        .align_items(Align::Center)
        .gap(16.0, 0.0)
        .width(pct!(100.0))
        .min_height(px!(84.0))
        .padding(Edges::symmetric(16.0, 12.0))
        .background(CARD)
        .border(Border::all(0.0, CARD).radius(28.0))
        .content_scale(1.0)
        .transition_all(spatial_motion())
        .transition_colors(effect_motion())
        .hover_style(|style, _| style.background(Color::rgb(43, 55, 58)).scale(1.01))
        .pressed_style(|style, _| {
            style
                .background(Color::rgb(43, 55, 58))
                .border(Border::all(0.0, CARD).radius(16.0))
        })
        .child(
            View::new()
                .width(px!(48.0))
                .height(px!(48.0))
                .min_width(px!(48.0))
                .align_items(Align::Center)
                .justify_content(JustifyContent::Center)
                .background(Color::rgb(111, 181, 255))
                .color(Color::rgb(0, 56, 101))
                .border(Border::all(0.0, Color::TRANSPARENT).radius(24.0))
                .child(icon(codepoints::PERSON, 28.0, true)),
        )
        .child(
            View::new()
                .display(Display::Flex)
                .flex_direction(FlexDirection::Column)
                .flex_grow(1.0)
                .child(
                    Label::new()
                        .label("Orhan")
                        .font_size(px!(20.0))
                        .line_height(px!(28.0))
                        .font_weight(FontWeight::Bold)
                        .color(TEXT),
                )
                .child(
                    Label::new()
                        .label("Google services and preferences")
                        .font_size(px!(16.0))
                        .line_height(px!(24.0))
                        .color(MUTED),
                ),
        )
        .on_click(|_| xen_router::push("/profile"))
}

fn segmented_radius(first: bool, last: bool) -> BorderRadius {
    match (first, last) {
        (true, true) => BorderRadius::all(28.0),
        (true, false) => BorderRadius::only(28.0, 28.0, 8.0, 8.0),
        (false, true) => BorderRadius::only(8.0, 8.0, 28.0, 28.0),
        (false, false) => BorderRadius::all(8.0),
    }
}

fn category_row(category: Category, first: bool, last: bool) -> View {
    let radius = segmented_radius(first, last);
    View::new()
        .display(Display::Flex)
        .flex_direction(FlexDirection::Row)
        .align_items(Align::Center)
        .gap(16.0, 0.0)
        .width(pct!(100.0))
        .min_height(px!(84.0))
        .padding(Edges::symmetric(16.0, 12.0))
        .background(CARD)
        .border(Border::all(0.0, CARD).radius(radius))
        .content_scale(1.0)
        .transition_all(spatial_motion())
        .transition_colors(effect_motion())
        .hover_style(move |style, _| {
            style
                .background(Color::rgb(43, 55, 58))
                .border(Border::all(0.0, CARD).radius(if first || last { 28.0 } else { 12.0 }))
                .scale(1.01)
        })
        .pressed_style(|style, _| {
            style
                .background(Color::rgb(43, 55, 58))
                .border(Border::all(0.0, CARD).radius(16.0))
        })
        .child(
            View::new()
                .width(px!(48.0))
                .height(px!(48.0))
                .min_width(px!(48.0))
                .align_items(Align::Center)
                .justify_content(JustifyContent::Center)
                .background(category.accent)
                .color(category.icon_color)
                .border(Border::all(0.0, category.accent).radius(24.0))
                .child(icon(category.icon, 26.0, true)),
        )
        .child(
            View::new()
                .display(Display::Flex)
                .flex_direction(FlexDirection::Column)
                .flex_grow(1.0)
                .child(
                    Label::new()
                        .label(category.title)
                        .font_size(px!(20.0))
                        .line_height(px!(28.0))
                        .font_weight(FontWeight::Medium)
                        .color(TEXT),
                )
                .child(
                    Label::new()
                        .label(category.subtitle)
                        .font_size(px!(16.0))
                        .line_height(px!(24.0))
                        .color(MUTED),
                ),
        )
        .on_click(move |_| xen_router::push(category.route))
}

fn category_group(items: &[Category]) -> View {
    let len = items.len();
    let children = items
        .iter()
        .enumerate()
        .map(|(index, item)| {
            Box::new(category_row(*item, index == 0, index + 1 == len)) as Box<dyn Widget>
        })
        .collect();
    View::new()
        .display(Display::Flex)
        .flex_direction(FlexDirection::Column)
        .gap(0.0, 4.0)
        .width(pct!(100.0))
        .children_vec(children)
}

fn home_page() -> Box<dyn Widget> {
    let content = View::new()
        .display(Display::Flex)
        .flex_direction(FlexDirection::Column)
        .width(pct!(100.0))
        .height(pct!(100.0))
        .child(
            View::new()
                .height(px!(platform_top_inset()))
                .min_height(px!(platform_top_inset())),
        )
        .child(
            View::new()
                .display(Display::Flex)
                .flex_direction(FlexDirection::Column)
                .gap(0.0, 16.0)
                .width(pct!(100.0))
                .flex_grow(1.0)
                .overflow_y(Overflow::Auto)
                .scroll_state(route_scroll_state("/"))
                .padding(Edges::only(16.0, 16.0, 16.0, 32.0))
                .child(search_entry())
                .child(profile_card())
                .child(category_group(&CATEGORIES[0..2]))
                .child(category_group(&CATEGORIES[2..8]))
                .child(category_group(&CATEGORIES[8..10]))
                .child(category_group(&CATEGORIES[10..14]))
                .child(category_group(&CATEGORIES[14..17])),
        );
    page_shell(content)
}

fn back_button() -> View {
    View::new()
        .width(px!(48.0))
        .height(px!(48.0))
        .min_width(px!(48.0))
        .align_items(Align::Center)
        .justify_content(JustifyContent::Center)
        .color(TEXT)
        .background(Color::TRANSPARENT)
        .border(Border::all(0.0, Color::TRANSPARENT).radius(24.0))
        .content_scale(1.0)
        .transition_all(spatial_motion())
        .transition_colors(effect_motion())
        .hover_style(|style, _| style.background(Color::rgb(43, 55, 58)).scale(1.06))
        .pressed_style(|style, _| {
            style
                .background(Color::rgb(43, 55, 58))
                .border(Border::all(0.0, Color::TRANSPARENT).radius(12.0))
        })
        .child(icon(codepoints::ARROW_BACK, 24.0, false))
        .on_click(|_| {
            xen_router::back();
        })
}

fn detail_app_bar(title: &'static str, show_back: bool) -> View {
    let mut children: Vec<Box<dyn Widget>> = Vec::new();
    if show_back {
        children.push(Box::new(back_button()));
    }
    children.push(Box::new(
        Label::new()
            .label(title)
            .font_size(px!(22.0))
            .line_height(px!(28.0))
            .font_weight(FontWeight::Medium)
            .color(TEXT),
    ));

    View::new()
        .display(Display::Flex)
        .flex_direction(FlexDirection::Row)
        .align_items(Align::Center)
        .gap(12.0, 0.0)
        .width(pct!(100.0))
        .min_height(px!(64.0))
        .padding(Edges::symmetric(8.0, 8.0))
        .children_vec(children)
}

fn switch_row(
    label: &'static str,
    checked: bool,
    set_checked: SetState<bool>,
    first: bool,
    last: bool,
) -> View {
    let radius = segmented_radius(first, last);
    let switch_setter = set_checked.clone();
    let target_setter = set_checked.clone();
    let target_height = if xengui::is_touch_platform() {
        48.0
    } else {
        32.0
    };
    View::new()
        .display(Display::Flex)
        .flex_direction(FlexDirection::Row)
        .align_items(Align::Center)
        .justify_content(JustifyContent::SpaceBetween)
        .gap(16.0, 0.0)
        .width(pct!(100.0))
        .min_height(px!(72.0))
        .padding(Edges::symmetric(20.0, 10.0))
        .background(CARD)
        .border(Border::all(0.0, CARD).radius(radius))
        .transition_colors(effect_motion())
        .hover_style(|style, _| style.background(Color::rgb(38, 50, 53)))
        .on_click(move |_| set_checked.set(!checked))
        .child(
            Label::new()
                .label(label)
                .font_size(px!(17.0))
                .line_height(px!(24.0))
                .color(TEXT)
                .flex_grow(1.0),
        )
        .child(
            View::new()
                .display(Display::Flex)
                .align_items(Align::Center)
                .justify_content(JustifyContent::Center)
                .width(px!(54.0))
                .height(px!(target_height))
                .on_click(move |_| target_setter.set(!checked))
                .child(
                    Switch::new()
                        .checked(checked)
                        .on_change(move |value, _| switch_setter.set(value)),
                ),
        )
}

fn value_row(label: &'static str, value: &'static str, first: bool, last: bool) -> View {
    let radius = segmented_radius(first, last);
    View::new()
        .display(Display::Flex)
        .flex_direction(FlexDirection::Row)
        .align_items(Align::Center)
        .justify_content(JustifyContent::SpaceBetween)
        .gap(16.0, 0.0)
        .width(pct!(100.0))
        .min_height(px!(72.0))
        .padding(Edges::symmetric(20.0, 10.0))
        .background(CARD)
        .border(Border::all(0.0, CARD).radius(radius))
        .child(
            Label::new()
                .label(label)
                .font_size(px!(17.0))
                .line_height(px!(24.0))
                .color(TEXT)
                .flex_grow(1.0),
        )
        .child(
            Label::new()
                .label(value)
                .font_size(px!(15.0))
                .line_height(px!(20.0))
                .color(MUTED),
        )
}

fn detail_kind(category: Category, index: usize) -> DetailKind {
    match (category.route, index) {
        ("/notifications", 3)
        | ("/sound", 4)
        | ("/display", 1)
        | ("/wallpaper", 3)
        | ("/battery", 1 | 3)
        | ("/privacy", 2..=4)
        | ("/location", 0) => DetailKind::Toggle,
        ("/about", 1) => DetailKind::Value("14"),
        ("/about", 2) => DetailKind::Value("192.168.1.24"),
        ("/about", 3) => DetailKind::Value("02:00:00:00:00:00"),
        ("/about", 4) => DetailKind::Value("AP4A.250205.002"),
        _ => DetailKind::Navigation,
    }
}

fn navigation_row(label: &'static str, route: String, first: bool, last: bool) -> View {
    let radius = segmented_radius(first, last);
    View::new()
        .display(Display::Flex)
        .flex_direction(FlexDirection::Row)
        .align_items(Align::Center)
        .justify_content(JustifyContent::SpaceBetween)
        .gap(16.0, 0.0)
        .width(pct!(100.0))
        .min_height(px!(72.0))
        .padding(Edges::symmetric(20.0, 10.0))
        .background(CARD)
        .border(Border::all(0.0, CARD).radius(radius))
        .content_scale(1.0)
        .transition_all(spatial_motion())
        .transition_colors(effect_motion())
        .hover_style(move |style, _| {
            style
                .background(Color::rgb(43, 55, 58))
                .border(Border::all(0.0, CARD).radius(if first || last { 28.0 } else { 12.0 }))
                .scale(1.01)
        })
        .pressed_style(|style, _| style.background(Color::rgb(43, 55, 58)))
        .child(
            Label::new()
                .label(label)
                .font_size(px!(17.0))
                .line_height(px!(24.0))
                .color(TEXT)
                .flex_grow(1.0),
        )
        .child(
            View::new()
                .color(MUTED)
                .child(icon(codepoints::CHEVRON_RIGHT, 24.0, false)),
        )
        .on_click(move |_| xen_router::push(route.clone()))
}

fn detail_page(category: Category, show_back: bool) -> Box<dyn Widget> {
    let (enabled, set_enabled) = use_state(true);
    let (suggestions, set_suggestions) = use_state(false);
    let (optional, set_optional) = use_state(true);
    let count = category.details.len();
    let mut rows: Vec<Box<dyn Widget>> = Vec::with_capacity(count);
    for (index, label) in category.details.iter().enumerate() {
        let first = index == 0;
        let last = index + 1 == count;
        match detail_kind(category, index) {
            DetailKind::Toggle => {
                let (value, setter) = match index % 3 {
                    0 => (enabled, set_enabled.clone()),
                    1 => (suggestions, set_suggestions.clone()),
                    _ => (optional, set_optional.clone()),
                };
                rows.push(Box::new(switch_row(label, value, setter, first, last)));
            }
            DetailKind::Value(value) => {
                rows.push(Box::new(value_row(label, value, first, last)));
            }
            DetailKind::Navigation => {
                rows.push(Box::new(navigation_row(
                    label,
                    format!("{}/{index}", category.route),
                    first,
                    last,
                )));
            }
        }
    }

    let content = View::new()
        .display(Display::Flex)
        .flex_direction(FlexDirection::Column)
        .width(pct!(100.0))
        .height(pct!(100.0))
        .child(
            View::new()
                .height(px!(platform_top_inset()))
                .min_height(px!(platform_top_inset())),
        )
        .child(detail_app_bar(category.title, show_back))
        .child(
            View::new()
                .display(Display::Flex)
                .flex_direction(FlexDirection::Column)
                .gap(0.0, 20.0)
                .width(pct!(100.0))
                .flex_grow(1.0)
                .overflow_y(Overflow::Auto)
                .scroll_state(route_scroll_state(category.route))
                .padding(Edges::only(16.0, 8.0, 16.0, 32.0))
                .child(
                    View::new()
                        .display(Display::Flex)
                        .flex_direction(FlexDirection::Column)
                        .align_items(Align::Center)
                        .gap(0.0, 12.0)
                        .padding(Edges::only(16.0, 20.0, 16.0, 20.0))
                        .child(
                            View::new()
                                .width(px!(72.0))
                                .height(px!(72.0))
                                .align_items(Align::Center)
                                .justify_content(JustifyContent::Center)
                                .background(category.accent)
                                .color(category.icon_color)
                                .border(Border::all(0.0, category.accent).radius(36.0))
                                .child(icon(category.icon, 36.0, true)),
                        )
                        .child(
                            Label::new()
                                .label(category.subtitle)
                                .font_size(px!(16.0))
                                .line_height(px!(24.0))
                                .color(MUTED),
                        ),
                )
                .child(
                    View::new()
                        .display(Display::Flex)
                        .flex_direction(FlexDirection::Column)
                        .gap(0.0, 4.0)
                        .children_vec(rows),
                ),
        );
    page_shell(content)
}

fn profile_page(show_back: bool) -> Box<dyn Widget> {
    detail_page(
        Category {
            route: "/profile",
            title: "Orhan",
            subtitle: "Google services and preferences",
            icon: codepoints::PERSON,
            accent: Color::rgb(111, 181, 255),
            icon_color: Color::rgb(0, 56, 101),
            details: PROFILE_DETAILS,
        },
        show_back,
    )
}

fn leaf_page(
    category: Category,
    title: &'static str,
    route: String,
    show_back: bool,
) -> Box<dyn Widget> {
    let content = View::new()
        .display(Display::Flex)
        .flex_direction(FlexDirection::Column)
        .width(pct!(100.0))
        .height(pct!(100.0))
        .child(
            View::new()
                .height(px!(platform_top_inset()))
                .min_height(px!(platform_top_inset())),
        )
        .child(detail_app_bar(title, show_back))
        .child(
            View::new()
                .display(Display::Flex)
                .flex_direction(FlexDirection::Column)
                .gap(0.0, 20.0)
                .width(pct!(100.0))
                .flex_grow(1.0)
                .overflow_y(Overflow::Auto)
                .scroll_state(route_scroll_state(route))
                .padding(Edges::only(16.0, 16.0, 16.0, 32.0))
                .child(
                    View::new()
                        .display(Display::Flex)
                        .flex_direction(FlexDirection::Row)
                        .align_items(Align::Center)
                        .gap(16.0, 0.0)
                        .padding(Edges::all(20.0))
                        .background(CARD)
                        .border(Border::all(0.0, CARD).radius(28.0))
                        .child(
                            View::new()
                                .width(px!(48.0))
                                .height(px!(48.0))
                                .min_width(px!(48.0))
                                .align_items(Align::Center)
                                .justify_content(JustifyContent::Center)
                                .background(category.accent)
                                .color(category.icon_color)
                                .border(Border::all(0.0, category.accent).radius(24.0))
                                .child(icon(category.icon, 26.0, true)),
                        )
                        .child(
                            View::new()
                                .display(Display::Flex)
                                .flex_direction(FlexDirection::Column)
                                .flex_grow(1.0)
                                .child(
                                    Label::new()
                                        .label(title)
                                        .font_size(px!(20.0))
                                        .line_height(px!(28.0))
                                        .font_weight(FontWeight::Medium)
                                        .color(TEXT),
                                )
                                .child(
                                    Label::new()
                                        .label(category.title)
                                        .font_size(px!(14.0))
                                        .line_height(px!(20.0))
                                        .color(MUTED),
                                ),
                        ),
                )
                .child(
                    View::new()
                        .padding(Edges::all(20.0))
                        .background(CARD)
                        .border(Border::all(0.0, CARD).radius(28.0))
                        .child(
                            Label::new()
                                .label("Additional settings and information are available on this page.")
                                .font_size(px!(16.0))
                                .line_height(px!(24.0))
                                .color(MUTED),
                        ),
                ),
        );
    page_shell(content)
}

fn nested_route_page(
    category_segment: &str,
    setting_segment: &str,
    show_back: bool,
) -> Box<dyn Widget> {
    let index = setting_segment.parse::<usize>().ok();
    let category = if category_segment == "profile" {
        Some(Category {
            route: "/profile",
            title: "Orhan",
            subtitle: "Google services and preferences",
            icon: codepoints::PERSON,
            accent: Color::rgb(111, 181, 255),
            icon_color: Color::rgb(0, 56, 101),
            details: PROFILE_DETAILS,
        })
    } else {
        CATEGORIES
            .iter()
            .copied()
            .find(|category| category.route.trim_start_matches('/') == category_segment)
    };

    match (category, index) {
        (Some(category), Some(index)) if index < category.details.len() => leaf_page(
            category,
            category.details[index],
            format!("/{category_segment}/{setting_segment}"),
            show_back,
        ),
        _ => not_found_page(),
    }
}

fn search_result(category: Category) -> View {
    View::new()
        .display(Display::Flex)
        .flex_direction(FlexDirection::Row)
        .align_items(Align::Center)
        .gap(16.0, 0.0)
        .width(pct!(100.0))
        .min_height(px!(72.0))
        .padding(Edges::symmetric(16.0, 10.0))
        .background(CARD)
        .border(Border::all(0.0, CARD).radius(20.0))
        .content_scale(1.0)
        .transition_all(spatial_motion())
        .transition_colors(effect_motion())
        .hover_style(|style, _| style.background(Color::rgb(43, 55, 58)).scale(1.01))
        .pressed_style(|style, _| style.background(Color::rgb(43, 55, 58)))
        .child(
            View::new()
                .width(px!(40.0))
                .height(px!(40.0))
                .min_width(px!(40.0))
                .align_items(Align::Center)
                .justify_content(JustifyContent::Center)
                .background(category.accent)
                .color(category.icon_color)
                .border(Border::all(0.0, category.accent).radius(20.0))
                .child(icon(category.icon, 22.0, true)),
        )
        .child(
            Label::new()
                .label(category.title)
                .font_size(px!(17.0))
                .line_height(px!(24.0))
                .font_weight(FontWeight::Medium)
                .color(TEXT)
                .flex_grow(1.0),
        )
        .on_click(move |_| xen_router::push(category.route))
}

fn search_page() -> Box<dyn Widget> {
    let (query, set_query) = use_state(String::new());
    let normalized = query.to_lowercase();
    let results = CATEGORIES
        .iter()
        .copied()
        .filter(|category| {
            normalized.is_empty()
                || category.title.to_lowercase().contains(&normalized)
                || category.subtitle.to_lowercase().contains(&normalized)
        })
        .map(|category| Box::new(search_result(category)) as Box<dyn Widget>)
        .collect();

    let content = View::new()
        .display(Display::Flex)
        .flex_direction(FlexDirection::Column)
        .width(pct!(100.0))
        .height(pct!(100.0))
        .child(
            View::new()
                .height(px!(platform_top_inset()))
                .min_height(px!(platform_top_inset())),
        )
        .child(
            View::new()
                .display(Display::Flex)
                .flex_direction(FlexDirection::Row)
                .align_items(Align::Center)
                .gap(8.0, 0.0)
                .padding(Edges::only(12.0, 12.0, 12.0, 8.0))
                .child(back_button())
                .child(
                    View::new()
                        .display(Display::Flex)
                        .flex_direction(FlexDirection::Row)
                        .align_items(Align::Center)
                        .gap(12.0, 0.0)
                        .height(px!(56.0))
                        .flex_grow(1.0)
                        .padding(Edges::symmetric(16.0, 0.0))
                        .background(SEARCH)
                        .border(Border::all(1.0, Color::rgb(61, 72, 75)).radius(28.0))
                        .transition_colors(effect_motion())
                        .hover_style(|style, _| {
                            style
                                .background(Color::rgb(22, 31, 33))
                                .border(Border::all(1.0, Color::rgb(80, 91, 94)).radius(28.0))
                        })
                        .focus_within_style(|style, _| {
                            style.border(Border::all(1.0, Color::rgb(151, 193, 255)).radius(28.0))
                        })
                        .color(MUTED)
                        .child(icon(codepoints::SEARCH, 24.0, false))
                        .child(
                            TextBox::new()
                                .value(query)
                                .placeholder("Search Settings")
                                .flex_grow(1.0)
                                .height(px!(48.0))
                                .font_size(px!(18.0))
                                .color(TEXT)
                                .background(Color::TRANSPARENT)
                                .border(Border::all(0.0, Color::TRANSPARENT))
                                .on_change(move |value, _| set_query.set(value.to_owned())),
                        ),
                ),
        )
        .child(
            View::new()
                .display(Display::Flex)
                .flex_direction(FlexDirection::Column)
                .gap(0.0, 8.0)
                .width(pct!(100.0))
                .flex_grow(1.0)
                .overflow_y(Overflow::Auto)
                .scroll_state(route_scroll_state("/search"))
                .padding(Edges::only(16.0, 12.0, 16.0, 32.0))
                .child(
                    Label::new()
                        .label(if normalized.is_empty() {
                            "All settings"
                        } else {
                            "Quick results"
                        })
                        .font_size(px!(14.0))
                        .line_height(px!(20.0))
                        .font_weight(FontWeight::Bold)
                        .color(MUTED)
                        .padding(Edges::symmetric(8.0, 0.0)),
                )
                .children_vec(results),
        );
    page_shell(content)
}

fn not_found_page() -> Box<dyn Widget> {
    let content = View::new()
        .display(Display::Flex)
        .flex_direction(FlexDirection::Column)
        .align_items(Align::Center)
        .justify_content(JustifyContent::Center)
        .gap(0.0, 16.0)
        .width(pct!(100.0))
        .height(pct!(100.0))
        .child(icon(codepoints::SEARCH_OFF, 48.0, false))
        .child(
            Label::new()
                .label("Page not found")
                .font_size(px!(24.0))
                .color(TEXT),
        )
        .child(back_button());
    page_shell(content)
}

fn adaptive_detail_placeholder() -> Box<dyn Widget> {
    page_shell(
        View::new()
            .display(Display::Flex)
            .flex_direction(FlexDirection::Column)
            .align_items(Align::Center)
            .justify_content(JustifyContent::Center)
            .gap(0.0, 16.0)
            .width(pct!(100.0))
            .height(pct!(100.0))
            .padding(Edges::all(24.0))
            .child(icon(codepoints::SETTINGS, 48.0, false))
            .child(
                Label::new()
                    .label("Select a setting")
                    .font_size(px!(22.0))
                    .line_height(px!(28.0))
                    .color(TEXT),
            ),
    )
}

fn routed_page(show_back: bool, home_on_root: bool) -> Box<dyn Widget> {
    let mut router = Router::new()
        .route("/", move |_| {
            if home_on_root {
                home_page()
            } else {
                adaptive_detail_placeholder()
            }
        })
        .route("/search", |_| search_page())
        .route("/profile", move |_| profile_page(show_back));
    for category in CATEGORIES {
        let category = *category;
        router = router.route(category.route, move |_| detail_page(category, show_back));
    }
    router
        .route("/:category/:setting", move |params| {
            let category = params.get("category").unwrap_or_default().to_owned();
            let setting = params.get("setting").unwrap_or_default().to_owned();
            xengui::component(format!("settings-leaf:{category}:{setting}"), || {
                nested_route_page(&category, &setting, show_back)
            })
        })
        .not_found(not_found_page)
        .build()
}

fn root() -> Box<dyn Widget> {
    if !responsive_bool(Breakpoint::Sm, true) {
        return routed_page(true, true);
    }

    let sidebar_width: Length = match xengui::current_breakpoint() {
        Breakpoint::Sm => pct!(50.0),
        Breakpoint::Md => px!(360.0),
        Breakpoint::Lg | Breakpoint::Xl | Breakpoint::Xl2 => px!(412.0),
        Breakpoint::Base => pct!(100.0),
    };

    Box::new(
        View::new()
            .display(Display::Flex)
            .flex_direction(FlexDirection::Row)
            .align_items(Align::Stretch)
            .gap(24.0, 0.0)
            .width(pct!(100.0))
            .height(pct!(100.0))
            .padding(Edges::symmetric(24.0, 0.0))
            .background(BACKGROUND)
            .child(
                View::new()
                    .width(sidebar_width)
                    .height(pct!(100.0))
                    .flex_shrink(1.0)
                    .child_boxed(xengui::component("settings-list-pane", || home_page())),
            )
            .child(
                View::new()
                    .height(pct!(100.0))
                    .min_width(px!(0.0))
                    .flex_grow(1.0)
                    .child_boxed(routed_page(true, false)),
            ),
    )
}

fn settings_theme() -> Theme {
    Theme::dark()
        .background(BACKGROUND)
        .surface(BACKGROUND)
        .surface_dim(BACKGROUND)
        .surface_container_low(BACKGROUND)
        .surface_container(CARD)
        .surface_container_high(CARD)
        .surface_container_highest(Color::rgb(43, 55, 58))
        .on_surface(TEXT)
        .on_surface_variant(MUTED)
}

fn create_app() -> App {
    let mut app = App::new(AppConfig {
        title: "Settings".to_owned(),
        width: 640,
        height: 480,
        position: WindowPosition::Center,
        theme_mode: AppThemeMode::Fixed,
        themes: vec![settings_theme()],
        active_theme: 0,
        dark_theme: 0,
        light_theme: 0,
        ..Default::default()
    });
    app.with_font(
        "Noto_Sans",
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../examples/widgets_catalog/fonts/NotoSans-VariableFont.ttf"
        ))
        .to_vec(),
    );
    app.on_system_back(xen_router::back);
    app.render(root);
    app
}

#[cfg(not(target_os = "android"))]
pub fn run_desktop() -> Result<(), Box<dyn std::error::Error>> {
    create_app().run()
}

#[cfg(target_os = "android")]
#[allow(unsafe_code)]
#[unsafe(no_mangle)]
fn android_main(android_app: winit::platform::android::activity::AndroidApp) {
    if let Err(error) = create_app().run_android(android_app) {
        eprintln!("android_demo failed: {error}");
    }
}
