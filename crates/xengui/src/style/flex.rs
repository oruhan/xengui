// SPDX-License-Identifier: Apache-2.0
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
/// Available `FlexDirection` choices.
pub enum FlexDirection {
    #[default]
    /// The `Row` variant.
    Row,
    /// The `RowReverse` variant.
    RowReverse,
    /// The `Column` variant.
    Column,
    /// The `ColumnReverse` variant.
    ColumnReverse,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
/// Available `FlexWrap` choices.
pub enum FlexWrap {
    #[default]
    /// The `NoWrap` variant.
    NoWrap,
    /// The `Wrap` variant.
    Wrap,
    /// The `WrapReverse` variant.
    WrapReverse,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
/// Available `Align` choices.
pub enum Align {
    #[default]
    /// The `Stretch` variant.
    Stretch,
    /// The `Start` variant.
    Start,
    /// The `End` variant.
    End,
    /// The `Center` variant.
    Center,
    /// The `Baseline` variant.
    Baseline,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
/// Available `JustifyContent` choices.
pub enum JustifyContent {
    #[default]
    /// The `Start` variant.
    Start,
    /// The `End` variant.
    End,
    /// The `Center` variant.
    Center,
    /// The `SpaceBetween` variant.
    SpaceBetween,
    /// The `SpaceAround` variant.
    SpaceAround,
    /// The `SpaceEvenly` variant.
    SpaceEvenly,
}
