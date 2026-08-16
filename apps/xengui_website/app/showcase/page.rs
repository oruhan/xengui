use std::time::Duration;
use xen_router::RouteParams;
use xengui::*;

#[derive(Clone, Copy, PartialEq)]
enum Role {
    User,
    Assistant,
}

#[derive(Clone, PartialEq)]
struct ChatMessage {
    role: Role,
    text: String,
}

const CONVERSATIONS: &[&str] = &[
    "XenGui responsive API tasarımı",
    "wgpu render backend soruları",
    "Layout motoru hata ayıklama",
    "Material Design 3 renk paleti",
    "Animasyon geçiş süreleri",
];

fn sidebar_conversation(title: &str, collapsed: bool) -> View {
    let mut item = View::new()
        .display(Display::Flex)
        .flex_direction(FlexDirection::Row)
        .align_items(Align::Center)
        .padding(Edges::symmetric(12, 10))
        .border(|theme: &Theme| Border::all(1, Color::TRANSPARENT).radius(theme.radius_md))
        .transition_colors(Transition::new(Duration::from_millis(120)).easing(Easing::EaseOut))
        .hover_style(|ctx: StylePatch, theme: &Theme| ctx.background(theme.surface_container_high));

    if !collapsed {
        item = item.child(
            Label::new()
                .label(title)
                .font_size(13)
                .color(|theme: &Theme| theme.on_surface)
        );
    }

    item
}

fn message_bubble(message: &ChatMessage) -> View {
    let is_user = message.role == Role::User;

    let bubble = View::new()
        .display(Display::Flex)
        .max_width(Responsive::new(pct!(90.0)).md(pct!(70.0)))
        .padding(Edges::symmetric(16, 12))
        .background(|theme: &Theme| if false { theme.primary } else { theme.surface })
        .border(|theme: &Theme| Border::all(1, theme.outline_variant).radius(theme.radius_lg))
        .child(
            Label::new()
                .label(message.text.clone())
                .font_size(14)
                .line_height(px!(21.0))
                .color(|theme: &Theme| theme.on_surface)
        );

    let bubble = if is_user {
        bubble
            .background(|theme: &Theme| theme.primary_container)
            .border(|theme: &Theme| Border::all(1, theme.primary_container).radius(theme.radius_lg))
    } else {
        bubble
    };

    View::new()
        .display(Display::Flex)
        .width(pct!(100.0))
        .justify_content(if is_user { JustifyContent::End } else { JustifyContent::Start })
        .child(bubble)
}

pub struct ShowcasePage {
    base: WidgetBase,
    layout_box: LayoutBox,
    inner: Vec<Box<dyn Widget>>,
    hooks_id: WidgetId,
}

impl ShowcasePage {
    pub fn new() -> Self {
        Self {
            base: WidgetBase::new(Interaction::new()),
            layout_box: LayoutBox::default(),
            inner: Vec::new(),
            hooks_id: WidgetId::new_unique(),
        }
    }
}

impl Default for ShowcasePage {
    fn default() -> Self {
        Self::new()
    }
}

impl Render for ShowcasePage {
    fn render(&self) -> Box<dyn Widget> {
        let (messages, set_messages) = use_state(
            vec![ChatMessage {
                role: Role::Assistant,
                text: "How can I help you today?".to_string(),
            }]
        );
        let (draft, set_draft) = use_state(String::new());
        let (collapsed, set_collapsed) = use_state(false);

        // Small screens default to a collapsed sidebar unless the user
        // explicitly opened it, matching a typical mobile chat layout.
        let effective_collapsed = collapsed || !responsive_bool(Breakpoint::Md, true);

        let sidebar_width = if effective_collapsed { px!(64.0) } else { px!(272.0) };

        let mut conversation_list = View::new()
            .display(Display::Flex)
            .flex_direction(FlexDirection::Column)
            .gap(0, 2)
            .padding(Edges::symmetric(8, 8))
            .overflow_y(Overflow::Auto)
            .flex_grow(1.0);

        for title in CONVERSATIONS {
            conversation_list = conversation_list.child(
                sidebar_conversation(title, effective_collapsed)
            );
        }

        let toggle_icon = if effective_collapsed { ">" } else { "<" };
        let set_collapsed_toggle = set_collapsed.clone();
        let collapsed_for_toggle = collapsed;

        let sidebar_header = View::new()
            .display(Display::Flex)
            .flex_direction(FlexDirection::Row)
            .align_items(Align::Center)
            .justify_content(JustifyContent::SpaceBetween)
            .padding(px!(12))
            .child(
                Button::new()
                    .label(toggle_icon)
                    .background(Color::TRANSPARENT)
                    .color(|theme: &Theme| theme.on_surface)
                    .padding(Edges::symmetric(8, 6))
                    .border(|theme: &Theme|
                        Border::all(1, theme.outline_variant).radius(theme.radius_sm)
                    )
                    .on_click(move |_ctx| set_collapsed_toggle.set(!collapsed_for_toggle))
            );

        let set_messages_new = set_messages.clone();
        let new_chat_button = if effective_collapsed {
            View::new()
        } else {
            View::new()
                .padding(Edges::only(12, 0, 12, 8))
                .child(
                    Button::new()
                        .label("+ Yeni Sohbet")
                        .background(|theme: &Theme| theme.primary)
                        .color(|theme: &Theme| theme.on_primary)
                        .padding(Edges::symmetric(0, 10))
                        .width(pct!(100.0))
                        .border(|theme: &Theme|
                            Border::all(1, theme.primary).radius(theme.radius_md)
                        )
                        .on_click(move |_ctx| {
                            set_messages_new.set(
                                vec![ChatMessage {
                                    role: Role::Assistant,
                                    text: "Yeni bir sohbete başladın.".to_string(),
                                }]
                            );
                        })
                )
        };

        let sidebar = View::new()
            .display(Display::Flex)
            .flex_direction(FlexDirection::Column)
            .width(sidebar_width)
            .height(pct!(100.0))
            .flex_shrink(0.0)
            .background(|theme: &Theme| theme.surface_container_lowest)
            .border(|theme: &Theme| Border::right(1, theme.outline_variant))
            .transition_all(Transition::new(Duration::from_millis(180)).easing(Easing::EaseInOut))
            .child(sidebar_header)
            .child(new_chat_button)
            .child(conversation_list);

        let mut message_list = View::new()
            .key("showcase_message_list")
            .display(Display::Flex)
            .flex_direction(FlexDirection::Column)
            .gap(0, 12)
            .padding(Responsive::new(Edges::symmetric(16, 24)).md(Edges::symmetric(120, 32)))
            .width(pct!(100.0));

        for message in messages.iter() {
            message_list = message_list.child(message_bubble(message));
        }

        let message_scroll = View::new()
            .key("showcase_message_scroll")
            .display(Display::Flex)
            .flex_direction(FlexDirection::Column)
            .flex_grow(1.0)
            .overflow_y(Overflow::Auto)
            /*.pin_scroll_to_bottom(true)*/
            .child(message_list);

        let draft_for_send = draft.clone();
        let set_messages_send = set_messages.clone();
        let set_draft_send = set_draft.clone();
        let messages_for_send = messages.clone();

        let send_message = move || {
            let text = draft_for_send.trim().to_string();
            if text.is_empty() {
                return;
            }
            let mut next = messages_for_send.clone();
            next.push(ChatMessage { role: Role::User, text });
            next.push(ChatMessage {
                role: Role::Assistant,
                text: "api.reply_message".to_string(),
            });
            set_messages_send.set(next);
            set_draft_send.set(String::new());
        };

        let send_message_click = send_message.clone();
        let send_message_submit = send_message;

        let composer = View::new()
            .display(Display::Flex)
            .flex_direction(FlexDirection::Row)
            .align_items(Align::Center)
            .gap(8, 0)
            .padding(Responsive::new(Edges::symmetric(16, 16)).md(Edges::symmetric(120, 20)))
            .border(|theme: &Theme| Border::top(1, theme.outline_variant))
            .background(|theme: &Theme| theme.surface_container_lowest)
            .child(
                TextBox::new()
                    .value(draft.clone())
                    .placeholder("How can I help you today?")
                    .padding(Edges::symmetric(14, 10))
                    .flex_grow(1.0)
                    .border(|theme: &Theme|
                        Border::all(1, theme.outline_variant).radius(theme.radius_xl)
                    )
                    .on_change(move |value, _ctx| set_draft.set(value.to_string()))
                    .on_submit(move |_value, _ctx| send_message_submit())
            )
            .child(
                Button::new()
                    .label("Gönder")
                    .background(|theme: &Theme| theme.primary)
                    .color(|theme: &Theme| theme.on_primary)
                    .padding(Edges::symmetric(18, 10))
                    .border(|theme: &Theme| Border::all(1, theme.primary).radius(theme.radius_xl))
                    .transition_all(
                        Transition::new(Duration::from_millis(150)).easing(Easing::EaseInOut)
                    )
                    .pressed_style(|ctx: StylePatch, _theme: &Theme| ctx.scale(0.96))
                    .on_click(move |_ctx| send_message_click())
            );

        let main = View::new()
            .display(Display::Flex)
            .flex_direction(FlexDirection::Column)
            .flex_grow(1.0)
            //.height(pct!(100.0))
            .background(|theme: &Theme| theme.background)
            .child(message_scroll)
            .child(composer);

        Box::new(
            View::new()
                .display(Display::Flex)
                .flex_direction(FlexDirection::Row)
                .width(pct!(100.0))
                .height(pct!(100.0))
                .child(sidebar)
                .child(main.height(pct!(100.0)))
        )
    }
}

xengui::impl_composite_widget!(ShowcasePage);

pub fn page(_params: &RouteParams) -> Box<dyn Widget> {
    Box::new(
        View::new()
            .display(Display::Flex)
            .flex_direction(FlexDirection::Column)
            .height(pct!(100.0))
            .child(ShowcasePage::new())
    )
}
