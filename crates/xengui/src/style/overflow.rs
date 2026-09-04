#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
/// Available `Overflow` choices.
pub enum Overflow {
    #[default]
    /// The `Visible` variant.
    Visible,
    /// The `Hidden` variant.
    Hidden,
    /// The `Scroll` variant.
    Scroll,
    /// The `Auto` variant.
    Auto,
}
