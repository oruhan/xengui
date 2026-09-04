// SPDX-License-Identifier: Apache-2.0
use super::{Background, Color, Edges, Length};
use crate::{Border, Outline, properties::StyleValue};
use std::cell::{Cell, RefCell};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// Available `ThemeMode` choices.
pub enum ThemeMode {
    /// The `Light` variant.
    Light,
    /// The `Dark` variant.
    Dark,
    /// The `Auto` variant.
    Auto,
}

#[derive(Clone, Debug, PartialEq)]
/// Data and behavior represented by `Theme`.
pub struct Theme {
    name: String,
    mode: ThemeMode,

    /* Colors */
    /// Theme token used for `background`.
    pub background: Color,
    /// Theme token used for `on_background`.
    pub on_background: Color,

    // Primary
    /// Theme token used for `inverse_primary`.
    pub inverse_primary: Color,
    /// Theme token used for `primary`.
    pub primary: Color,
    /// Theme token used for `on_primary`.
    pub on_primary: Color,
    /// Theme token used for `primary_container`.
    pub primary_container: Color,
    /// Theme token used for `on_primary_container`.
    pub on_primary_container: Color,

    /// Theme token used for `primary_fixed`.
    pub primary_fixed: Color,
    /// Theme token used for `primary_fixed_dim`.
    pub primary_fixed_dim: Color,
    /// Theme token used for `on_primary_fixed`.
    pub on_primary_fixed: Color,
    /// Theme token used for `on_primary_fixed_variant`.
    pub on_primary_fixed_variant: Color,

    // Secondary
    /// Theme token used for `inverse_secondary`.
    pub inverse_secondary: Color,
    /// Theme token used for `secondary`.
    pub secondary: Color,
    /// Theme token used for `on_secondary`.
    pub on_secondary: Color,
    /// Theme token used for `secondary_container`.
    pub secondary_container: Color,
    /// Theme token used for `on_secondary_container`.
    pub on_secondary_container: Color,

    /// Theme token used for `secondary_fixed`.
    pub secondary_fixed: Color,
    /// Theme token used for `secondary_fixed_dim`.
    pub secondary_fixed_dim: Color,
    /// Theme token used for `on_secondary_fixed`.
    pub on_secondary_fixed: Color,
    /// Theme token used for `on_secondary_fixed_variant`.
    pub on_secondary_fixed_variant: Color,

    // Tertiary
    /// Theme token used for `inverse_tertiary`.
    pub inverse_tertiary: Color,
    /// Theme token used for `tertiary`.
    pub tertiary: Color,
    /// Theme token used for `on_tertiary`.
    pub on_tertiary: Color,
    /// Theme token used for `tertiary_container`.
    pub tertiary_container: Color,
    /// Theme token used for `on_tertiary_container`.
    pub on_tertiary_container: Color,

    /// Theme token used for `tertiary_fixed`.
    pub tertiary_fixed: Color,
    /// Theme token used for `tertiary_fixed_dim`.
    pub tertiary_fixed_dim: Color,
    /// Theme token used for `on_tertiary_fixed`.
    pub on_tertiary_fixed: Color,
    /// Theme token used for `on_tertiary_fixed_variant`.
    pub on_tertiary_fixed_variant: Color,

    // Info
    /// Theme token used for `info`.
    pub info: Color,
    /// Theme token used for `on_info`.
    pub on_info: Color,
    /// Theme token used for `info_container`.
    pub info_container: Color,
    /// Theme token used for `on_info_container`.
    pub on_info_container: Color,

    // Error
    /// Theme token used for `error`.
    pub error: Color,
    /// Theme token used for `on_error`.
    pub on_error: Color,
    /// Theme token used for `error_container`.
    pub error_container: Color,
    /// Theme token used for `on_error_container`.
    pub on_error_container: Color,

    // Warning
    /// Theme token used for `warning`.
    pub warning: Color,
    /// Theme token used for `on_warning`.
    pub on_warning: Color,
    /// Theme token used for `warning_container`.
    pub warning_container: Color,
    /// Theme token used for `on_warning_container`.
    pub on_warning_container: Color,

    // Success
    /// Theme token used for `success`.
    pub success: Color,
    /// Theme token used for `on_success`.
    pub on_success: Color,
    /// Theme token used for `success_container`.
    pub success_container: Color,
    /// Theme token used for `on_success_container`.
    pub on_success_container: Color,

    // Surface
    /// Theme token used for `surface_dim`.
    pub surface_dim: Color,
    /// Theme token used for `surface`.
    pub surface: Color,
    /// Theme token used for `surface_bright`.
    pub surface_bright: Color,

    /// Theme token used for `inverse_surface`.
    pub inverse_surface: Color,
    /// Theme token used for `inverse_on_surface`.
    pub inverse_on_surface: Color,

    /// Theme token used for `surface_container_low`.
    pub surface_container_low: Color,
    /// Theme token used for `surface_container_lowest`.
    pub surface_container_lowest: Color,
    /// Theme token used for `surface_container`.
    pub surface_container: Color,
    /// Theme token used for `surface_container_high`.
    pub surface_container_high: Color,
    /// Theme token used for `surface_container_highest`.
    pub surface_container_highest: Color,

    /// Theme token used for `on_surface`.
    pub on_surface: Color,
    /// Theme token used for `on_surface_variant`.
    pub on_surface_variant: Color,

    // Outline
    /// Theme token used for `outline`.
    pub outline: Color,
    /// Theme token used for `outline_variant`.
    pub outline_variant: Color,

    // Scrim & Shadow
    /// Theme token used for `scrim`.
    pub scrim: Color,
    /// Theme token used for `shadow`.
    pub shadow: Color,

    /* -------------------------------------- */
    /// Theme token used for `selection`.
    pub selection: Color,
    /// Theme token used for `selection_color`.
    pub selection_color: Color,
    /// Theme token used for `selection_border_color`.
    pub selection_border_color: Color,
    /// Theme token used for `selection_border_width`.
    pub selection_border_width: Length,
    /// Theme token used for `selection_border_radius`.
    pub selection_border_radius: Length,
    /// Theme token used for `caret_color`.
    pub caret_color: Color,

    /// Theme token used for `scrollbar_thumb`.
    pub scrollbar_thumb: Color,
    /// Theme token used for `scrollbar_track`.
    pub scrollbar_track: Color,
    /// Theme token used for `scrollbar_button`.
    pub scrollbar_button: Color,
    /// Theme token used for `scrollbar_arrow`.
    pub scrollbar_arrow: Color,
    /// Theme token used for `scrollbar_thumb_border`.
    pub scrollbar_thumb_border: Color,
    /// Theme token used for `scrollbar_track_border`.
    pub scrollbar_track_border: Color,

    /// Theme token used for `radius_xs`.
    pub radius_xs: Length,
    /// Theme token used for `radius_sm`.
    pub radius_sm: Length,
    /// Theme token used for `radius_md`.
    pub radius_md: Length,
    /// Theme token used for `radius_lg`.
    pub radius_lg: Length,
    /// Theme token used for `radius_xl`.
    pub radius_xl: Length,
    /// Theme token used for `radius_2xl`.
    pub radius_2xl: Length,
    /// Theme token used for `radius_3xl`.
    pub radius_3xl: Length,
    /// Theme token used for `radius_4xl`.
    pub radius_4xl: Length,

    /// Theme token used for `space_xs`.
    pub space_xs: Length,
    /// Theme token used for `space_sm`.
    pub space_sm: Length,
    /// Theme token used for `space_md`.
    pub space_md: Length,
    /// Theme token used for `space_lg`.
    pub space_lg: Length,
    /// Theme token used for `space_xl`.
    pub space_xl: Length,
    /// Theme token used for `space_2xl`.
    pub space_2xl: Length,
    /// Theme token used for `space_3xl`.
    pub space_3xl: Length,
    /// Theme token used for `space_4xl`.
    pub space_4xl: Length,

    /* Typography */
    /// Theme token used for `text_xs`.
    pub text_xs: Length,
    /// Theme token used for `text_sm`.
    pub text_sm: Length,
    /// Theme token used for `text_md`.
    pub text_md: Length,
    /// Theme token used for `text_lg`.
    pub text_lg: Length,
    /// Theme token used for `text_xl`.
    pub text_xl: Length,
    /// Theme token used for `text_2xl`.
    pub text_2xl: Length,
    /// Theme token used for `text_3xl`.
    pub text_3xl: Length,
    /// Theme token used for `text_4xl`.
    pub text_4xl: Length,

    /// Theme token used for `border_width`.
    pub border_width: Length,
}

impl Theme {
    /// Creates a value with its default configuration.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            mode: ThemeMode::Light,

            /* Colors */
            background: Color::WHITE,
            on_background: Color::NEUTRAL_950,

            // Primary
            inverse_primary: Color::BLUE_700,
            primary: Color::BLUE_500,
            on_primary: Color::WHITE,
            primary_container: Color::BLUE_100,
            on_primary_container: Color::BLUE_900,

            primary_fixed: Color::BLUE_500,
            primary_fixed_dim: Color::BLUE_600,
            on_primary_fixed: Color::WHITE,
            on_primary_fixed_variant: Color::BLUE_900,

            // Secondary
            inverse_secondary: Color::BLUE_700,
            secondary: Color::BLUE_600,
            on_secondary: Color::WHITE,
            secondary_container: Color::BLUE_100,
            on_secondary_container: Color::BLUE_900,

            secondary_fixed: Color::BLUE_600,
            secondary_fixed_dim: Color::BLUE_700,
            on_secondary_fixed: Color::WHITE,
            on_secondary_fixed_variant: Color::BLUE_900,

            // Tertiary
            inverse_tertiary: Color::BLUE_800,
            tertiary: Color::BLUE_700,
            on_tertiary: Color::WHITE,
            tertiary_container: Color::BLUE_200,
            on_tertiary_container: Color::BLUE_900,

            tertiary_fixed: Color::BLUE_700,
            tertiary_fixed_dim: Color::BLUE_800,
            on_tertiary_fixed: Color::WHITE,
            on_tertiary_fixed_variant: Color::BLUE_900,

            // Info
            info: Color::CYAN_600,
            on_info: Color::WHITE,
            info_container: Color::CYAN_100,
            on_info_container: Color::CYAN_900,

            // Error
            error: Color::RED_500,
            on_error: Color::WHITE,
            error_container: Color::RED_100,
            on_error_container: Color::RED_900,

            // Warning
            warning: Color::AMBER_500,
            on_warning: Color::BLACK,
            warning_container: Color::AMBER_100,
            on_warning_container: Color::AMBER_900,

            // Success
            success: Color::GREEN_600,
            on_success: Color::WHITE,
            success_container: Color::GREEN_100,
            on_success_container: Color::GREEN_900,

            // Surface
            surface_dim: Color::NEUTRAL_100,
            surface: Color::NEUTRAL_50,
            surface_bright: Color::WHITE,

            inverse_surface: Color::NEUTRAL_900,
            inverse_on_surface: Color::NEUTRAL_50,

            surface_container_low: Color::NEUTRAL_100,
            surface_container_lowest: Color::WHITE,
            surface_container: Color::NEUTRAL_100,
            surface_container_high: Color::NEUTRAL_200,
            surface_container_highest: Color::NEUTRAL_300,

            on_surface: Color::NEUTRAL_900,
            on_surface_variant: Color::NEUTRAL_600,

            // Outline
            outline: Color::NEUTRAL_500,
            outline_variant: Color::NEUTRAL_300,

            // Scrim & Shadow
            scrim: Color::BLACK,
            shadow: Color::BLACK,
            /* -------------------------------------- */

            /* Text cursor */
            caret_color: Color::WHITE,
            selection: Color::BLUE_500.with_alpha(80),
            selection_color: Color::BLUE_200,
            selection_border_color: Color::TRANSPARENT,
            selection_border_width: Length::px(0.0),
            selection_border_radius: Length::px(4.0),

            /* Scrollbar */
            scrollbar_thumb: Color::NEUTRAL_400,
            scrollbar_track: Color::NEUTRAL_100,
            scrollbar_button: Color::NEUTRAL_300,
            scrollbar_arrow: Color::NEUTRAL_400,
            scrollbar_thumb_border: Color::TRANSPARENT,
            scrollbar_track_border: Color::TRANSPARENT,

            /* Corner radius */
            radius_xs: Length::px(2.0),
            radius_sm: Length::px(4.0),
            radius_md: Length::px(6.0),
            radius_lg: Length::px(8.0),
            radius_xl: Length::px(12.0),
            radius_2xl: Length::px(16.0),
            radius_3xl: Length::px(24.0),
            radius_4xl: Length::px(9999.0),

            /* Spacing */
            space_xs: Length::px(2.0),
            space_sm: Length::px(4.0),
            space_md: Length::px(8.0),
            space_lg: Length::px(12.0),
            space_xl: Length::px(16.0),
            space_2xl: Length::px(24.0),
            space_3xl: Length::px(32.0),
            space_4xl: Length::px(48.0),

            /* Typography */
            text_xs: Length::px(10.0),
            text_sm: Length::px(13.0),
            text_md: Length::px(15.0),
            text_lg: Length::px(18.0),
            text_xl: Length::px(20.0),
            text_2xl: Length::px(24.0),
            text_3xl: Length::px(32.0),
            text_4xl: Length::px(48.0),

            border_width: Length::px(1.0),
        }
    }

    /// Returns or updates the `light` value.
    pub fn light() -> Self {
        Self::new("light")
            .mode(ThemeMode::Light)
            /* Colors */
            .background(Color::WHITE)
            .on_background(Color::NEUTRAL_950)
            // Primary
            .inverse_primary(Color::BLUE_700)
            .primary(Color::BLUE_500)
            .on_primary(Color::WHITE)
            .primary_container(Color::BLUE_100)
            .on_primary_container(Color::BLUE_900)
            .primary_fixed(Color::BLUE_500)
            .primary_fixed_dim(Color::BLUE_600)
            .on_primary_fixed(Color::WHITE)
            .on_primary_fixed_variant(Color::BLUE_900)
            // Secondary
            .inverse_secondary(Color::BLUE_700)
            .secondary(Color::BLUE_600)
            .on_secondary(Color::WHITE)
            .secondary_container(Color::BLUE_100)
            .on_secondary_container(Color::BLUE_900)
            .secondary_fixed(Color::BLUE_600)
            .secondary_fixed_dim(Color::BLUE_700)
            .on_secondary_fixed(Color::WHITE)
            .on_secondary_fixed_variant(Color::BLUE_900)
            // Tertiary
            .inverse_tertiary(Color::BLUE_800)
            .tertiary(Color::BLUE_700)
            .on_tertiary(Color::WHITE)
            .tertiary_container(Color::BLUE_200)
            .on_tertiary_container(Color::BLUE_900)
            .tertiary_fixed(Color::BLUE_700)
            .tertiary_fixed_dim(Color::BLUE_800)
            .on_tertiary_fixed(Color::WHITE)
            .on_tertiary_fixed_variant(Color::BLUE_900)
            // Info
            .info(Color::CYAN_600)
            .on_info(Color::WHITE)
            .info_container(Color::CYAN_100)
            .on_info_container(Color::CYAN_900)
            // Error
            .error(Color::RED_500)
            .on_error(Color::WHITE)
            .error_container(Color::RED_100)
            .on_error_container(Color::RED_900)
            // Warning
            .warning(Color::AMBER_500)
            .on_warning(Color::BLACK)
            .warning_container(Color::AMBER_100)
            .on_warning_container(Color::AMBER_900)
            // Success
            .success(Color::GREEN_600)
            .on_success(Color::WHITE)
            .success_container(Color::GREEN_100)
            .on_success_container(Color::GREEN_900)
            // Surface
            .surface_dim(Color::NEUTRAL_100)
            .surface(Color::NEUTRAL_50)
            .surface_bright(Color::WHITE)
            .inverse_surface(Color::NEUTRAL_900)
            .inverse_on_surface(Color::NEUTRAL_50)
            .surface_container_low(Color::NEUTRAL_100)
            .surface_container_lowest(Color::WHITE)
            .surface_container(Color::NEUTRAL_100)
            .surface_container_high(Color::NEUTRAL_200)
            .surface_container_highest(Color::NEUTRAL_300)
            .on_surface(Color::NEUTRAL_900)
            .on_surface_variant(Color::NEUTRAL_600)
            // Outline
            .outline(Color::NEUTRAL_500)
            .outline_variant(Color::NEUTRAL_300)
            // Scrim & Shadow
            .scrim(Color::BLACK)
            .shadow(Color::BLACK)
            /* XenGui */
            // Selection
            .selection(Color::BLUE_500.with_alpha(80))
            .selection_color(Color::WHITE)
            .selection_border_color(Color::TRANSPARENT)
            .selection_border_width(Length::px(0.0))
            .selection_border_radius(Length::px(4.0))
            .caret_color(Color::BLUE_500)
            // Scrollbar
            .scrollbar_thumb(Color::NEUTRAL_400)
            .scrollbar_track(Color::NEUTRAL_100)
            .scrollbar_button(Color::NEUTRAL_300)
            .scrollbar_arrow(Color::NEUTRAL_400)
            .scrollbar_thumb_border(Color::TRANSPARENT)
            .scrollbar_track_border(Color::TRANSPARENT)
    }

    /// Returns or updates the `dark` value.
    pub fn dark() -> Self {
        Self::new("dark")
            .mode(ThemeMode::Dark)
            /* Colors */
            .background(Color::BLACK)
            .on_background(Color::NEUTRAL_50)
            // Primary
            .inverse_primary(Color::BLUE_300)
            .primary(Color::BLUE_400)
            .on_primary(Color::BLUE_950)
            .primary_container(Color::BLUE_800)
            .on_primary_container(Color::BLUE_100)
            .primary_fixed(Color::BLUE_500)
            .primary_fixed_dim(Color::BLUE_600)
            .on_primary_fixed(Color::BLUE_950)
            .on_primary_fixed_variant(Color::BLUE_900)
            // Secondary
            .inverse_secondary(Color::BLUE_300)
            .secondary(Color::BLUE_400)
            .on_secondary(Color::BLUE_950)
            .secondary_container(Color::BLUE_800)
            .on_secondary_container(Color::BLUE_100)
            .secondary_fixed(Color::BLUE_600)
            .secondary_fixed_dim(Color::BLUE_700)
            .on_secondary_fixed(Color::BLUE_950)
            .on_secondary_fixed_variant(Color::BLUE_900)
            // Tertiary
            .inverse_tertiary(Color::BLUE_300)
            .tertiary(Color::BLUE_400)
            .on_tertiary(Color::BLUE_950)
            .tertiary_container(Color::BLUE_800)
            .on_tertiary_container(Color::BLUE_100)
            .tertiary_fixed(Color::BLUE_700)
            .tertiary_fixed_dim(Color::BLUE_800)
            .on_tertiary_fixed(Color::BLUE_950)
            .on_tertiary_fixed_variant(Color::BLUE_900)
            // Info
            .info(Color::CYAN_400)
            .on_info(Color::CYAN_950)
            .info_container(Color::CYAN_800)
            .on_info_container(Color::CYAN_100)
            // Error
            .error(Color::RED_400)
            .on_error(Color::RED_950)
            .error_container(Color::RED_800)
            .on_error_container(Color::RED_100)
            // Warning
            .warning(Color::AMBER_400)
            .on_warning(Color::AMBER_950)
            .warning_container(Color::AMBER_800)
            .on_warning_container(Color::AMBER_100)
            // Success
            .success(Color::GREEN_400)
            .on_success(Color::GREEN_950)
            .success_container(Color::GREEN_800)
            .on_success_container(Color::GREEN_100)
            // Surface
            .surface_dim(Color::NEUTRAL_950)
            .surface(Color::NEUTRAL_900)
            .surface_bright(Color::NEUTRAL_800)
            .inverse_surface(Color::NEUTRAL_100)
            .inverse_on_surface(Color::NEUTRAL_900)
            .surface_container_lowest(Color::NEUTRAL_950)
            .surface_container_low(Color::NEUTRAL_900)
            .surface_container(Color::NEUTRAL_800)
            .surface_container_high(Color::NEUTRAL_800)
            .surface_container_highest(Color::NEUTRAL_700)
            .on_surface(Color::NEUTRAL_50)
            .on_surface_variant(Color::NEUTRAL_300)
            // Outline
            .outline(Color::NEUTRAL_800)
            .outline_variant(Color::NEUTRAL_700)
            // Scrim & Shadow
            .scrim(Color::BLACK)
            .shadow(Color::BLACK)
            /* XenGui */
            // Selection
            .selection(Color::BLUE_500.with_alpha(80))
            .selection_color(Color::BLUE_200)
            .selection_border_color(Color::TRANSPARENT)
            .selection_border_width(Length::px(0.0))
            .selection_border_radius(Length::px(4.0))
            .caret_color(Color::BLUE_400)
            // Scrollbar
            .scrollbar_thumb(Color::NEUTRAL_600)
            .scrollbar_track(Color::NEUTRAL_900)
            .scrollbar_button(Color::NEUTRAL_700)
            .scrollbar_arrow(Color::NEUTRAL_600)
            .scrollbar_thumb_border(Color::TRANSPARENT)
            .scrollbar_track_border(Color::TRANSPARENT)
    }

    /// Returns or updates the `auto` value.
    pub fn auto() -> Self {
        let mut theme = Self::light();
        theme.mode = ThemeMode::Auto;
        theme
    }

    /// Returns or updates the `mode` value.
    pub fn mode(mut self, mode: ThemeMode) -> Self {
        self.mode = mode;
        self
    }

    /// Returns or updates the `background` value.
    pub fn background(mut self, color: Color) -> Self {
        self.background = color;
        self
    }

    /// Registers the `on_background` callback.
    pub fn on_background(mut self, color: Color) -> Self {
        self.on_background = color;
        self
    }

    /// Returns or updates the `inverse_primary` value.
    pub fn inverse_primary(mut self, color: Color) -> Self {
        self.inverse_primary = color;
        self
    }

    /// Returns or updates the `primary` value.
    pub fn primary(mut self, color: Color) -> Self {
        self.primary = color;
        self
    }

    /// Registers the `on_primary` callback.
    pub fn on_primary(mut self, color: Color) -> Self {
        self.on_primary = color;
        self
    }

    /// Returns or updates the `primary_container` value.
    pub fn primary_container(mut self, color: Color) -> Self {
        self.primary_container = color;
        self
    }

    /// Registers the `on_primary_container` callback.
    pub fn on_primary_container(mut self, color: Color) -> Self {
        self.on_primary_container = color;
        self
    }

    /// Returns or updates the `primary_fixed` value.
    pub fn primary_fixed(mut self, color: Color) -> Self {
        self.primary_fixed = color;
        self
    }

    /// Returns or updates the `primary_fixed_dim` value.
    pub fn primary_fixed_dim(mut self, color: Color) -> Self {
        self.primary_fixed_dim = color;
        self
    }

    /// Registers the `on_primary_fixed` callback.
    pub fn on_primary_fixed(mut self, color: Color) -> Self {
        self.on_primary_fixed = color;
        self
    }

    /// Registers the `on_primary_fixed_variant` callback.
    pub fn on_primary_fixed_variant(mut self, color: Color) -> Self {
        self.on_primary_fixed_variant = color;
        self
    }

    /// Returns or updates the `inverse_secondary` value.
    pub fn inverse_secondary(mut self, color: Color) -> Self {
        self.inverse_secondary = color;
        self
    }

    /// Returns or updates the `secondary` value.
    pub fn secondary(mut self, color: Color) -> Self {
        self.secondary = color;
        self
    }

    /// Registers the `on_secondary` callback.
    pub fn on_secondary(mut self, color: Color) -> Self {
        self.on_secondary = color;
        self
    }

    /// Returns or updates the `secondary_container` value.
    pub fn secondary_container(mut self, color: Color) -> Self {
        self.secondary_container = color;
        self
    }

    /// Registers the `on_secondary_container` callback.
    pub fn on_secondary_container(mut self, color: Color) -> Self {
        self.on_secondary_container = color;
        self
    }

    /// Returns or updates the `secondary_fixed` value.
    pub fn secondary_fixed(mut self, color: Color) -> Self {
        self.secondary_fixed = color;
        self
    }

    /// Returns or updates the `secondary_fixed_dim` value.
    pub fn secondary_fixed_dim(mut self, color: Color) -> Self {
        self.secondary_fixed_dim = color;
        self
    }

    /// Registers the `on_secondary_fixed` callback.
    pub fn on_secondary_fixed(mut self, color: Color) -> Self {
        self.on_secondary_fixed = color;
        self
    }

    /// Registers the `on_secondary_fixed_variant` callback.
    pub fn on_secondary_fixed_variant(mut self, color: Color) -> Self {
        self.on_secondary_fixed_variant = color;
        self
    }

    /// Returns or updates the `inverse_tertiary` value.
    pub fn inverse_tertiary(mut self, color: Color) -> Self {
        self.inverse_tertiary = color;
        self
    }

    /// Returns or updates the `tertiary` value.
    pub fn tertiary(mut self, color: Color) -> Self {
        self.tertiary = color;
        self
    }

    /// Registers the `on_tertiary` callback.
    pub fn on_tertiary(mut self, color: Color) -> Self {
        self.on_tertiary = color;
        self
    }

    /// Returns or updates the `tertiary_container` value.
    pub fn tertiary_container(mut self, color: Color) -> Self {
        self.tertiary_container = color;
        self
    }

    /// Registers the `on_tertiary_container` callback.
    pub fn on_tertiary_container(mut self, color: Color) -> Self {
        self.on_tertiary_container = color;
        self
    }

    /// Returns or updates the `tertiary_fixed` value.
    pub fn tertiary_fixed(mut self, color: Color) -> Self {
        self.tertiary_fixed = color;
        self
    }

    /// Returns or updates the `tertiary_fixed_dim` value.
    pub fn tertiary_fixed_dim(mut self, color: Color) -> Self {
        self.tertiary_fixed_dim = color;
        self
    }

    /// Registers the `on_tertiary_fixed` callback.
    pub fn on_tertiary_fixed(mut self, color: Color) -> Self {
        self.on_tertiary_fixed = color;
        self
    }

    /// Registers the `on_tertiary_fixed_variant` callback.
    pub fn on_tertiary_fixed_variant(mut self, color: Color) -> Self {
        self.on_tertiary_fixed_variant = color;
        self
    }

    /// Returns or updates the `info` value.
    pub fn info(mut self, color: Color) -> Self {
        self.info = color;
        self
    }

    /// Registers the `on_info` callback.
    pub fn on_info(mut self, color: Color) -> Self {
        self.on_info = color;
        self
    }

    /// Returns or updates the `info_container` value.
    pub fn info_container(mut self, color: Color) -> Self {
        self.info_container = color;
        self
    }

    /// Registers the `on_info_container` callback.
    pub fn on_info_container(mut self, color: Color) -> Self {
        self.on_info_container = color;
        self
    }

    /// Returns or updates the `error` value.
    pub fn error(mut self, color: Color) -> Self {
        self.error = color;
        self
    }

    /// Registers the `on_error` callback.
    pub fn on_error(mut self, color: Color) -> Self {
        self.on_error = color;
        self
    }

    /// Returns or updates the `error_container` value.
    pub fn error_container(mut self, color: Color) -> Self {
        self.error_container = color;
        self
    }

    /// Registers the `on_error_container` callback.
    pub fn on_error_container(mut self, color: Color) -> Self {
        self.on_error_container = color;
        self
    }

    /// Returns or updates the `warning` value.
    pub fn warning(mut self, color: Color) -> Self {
        self.warning = color;
        self
    }

    /// Registers the `on_warning` callback.
    pub fn on_warning(mut self, color: Color) -> Self {
        self.on_warning = color;
        self
    }

    /// Returns or updates the `warning_container` value.
    pub fn warning_container(mut self, color: Color) -> Self {
        self.warning_container = color;
        self
    }

    /// Registers the `on_warning_container` callback.
    pub fn on_warning_container(mut self, color: Color) -> Self {
        self.on_warning_container = color;
        self
    }

    /// Returns or updates the `success` value.
    pub fn success(mut self, color: Color) -> Self {
        self.success = color;
        self
    }

    /// Registers the `on_success` callback.
    pub fn on_success(mut self, color: Color) -> Self {
        self.on_success = color;
        self
    }

    /// Returns or updates the `success_container` value.
    pub fn success_container(mut self, color: Color) -> Self {
        self.success_container = color;
        self
    }

    /// Registers the `on_success_container` callback.
    pub fn on_success_container(mut self, color: Color) -> Self {
        self.on_success_container = color;
        self
    }

    /// Returns or updates the `surface_dim` value.
    pub fn surface_dim(mut self, color: Color) -> Self {
        self.surface_dim = color;
        self
    }

    /// Returns or updates the `surface_bright` value.
    pub fn surface_bright(mut self, color: Color) -> Self {
        self.surface_bright = color;
        self
    }

    /// Returns or updates the `inverse_surface` value.
    pub fn inverse_surface(mut self, color: Color) -> Self {
        self.inverse_surface = color;
        self
    }

    /// Returns or updates the `inverse_on_surface` value.
    pub fn inverse_on_surface(mut self, color: Color) -> Self {
        self.inverse_on_surface = color;
        self
    }

    /// Returns or updates the `surface_container_low` value.
    pub fn surface_container_low(mut self, color: Color) -> Self {
        self.surface_container_low = color;
        self
    }

    /// Returns or updates the `surface_container_lowest` value.
    pub fn surface_container_lowest(mut self, color: Color) -> Self {
        self.surface_container_lowest = color;
        self
    }

    /// Returns or updates the `surface_container` value.
    pub fn surface_container(mut self, color: Color) -> Self {
        self.surface_container = color;
        self
    }

    /// Returns or updates the `surface_container_high` value.
    pub fn surface_container_high(mut self, color: Color) -> Self {
        self.surface_container_high = color;
        self
    }

    /// Returns or updates the `surface_container_highest` value.
    pub fn surface_container_highest(mut self, color: Color) -> Self {
        self.surface_container_highest = color;
        self
    }

    /// Registers the `on_surface` callback.
    pub fn on_surface(mut self, color: Color) -> Self {
        self.on_surface = color;
        self
    }

    /// Registers the `on_surface_variant` callback.
    pub fn on_surface_variant(mut self, color: Color) -> Self {
        self.on_surface_variant = color;
        self
    }

    /// Returns or updates the `outline` value.
    pub fn outline(mut self, color: Color) -> Self {
        self.outline = color;
        self
    }

    /// Returns or updates the `outline_variant` value.
    pub fn outline_variant(mut self, color: Color) -> Self {
        self.outline_variant = color;
        self
    }

    /// Returns or updates the `scrim` value.
    pub fn scrim(mut self, color: Color) -> Self {
        self.scrim = color;
        self
    }

    /// Returns or updates the `shadow` value.
    pub fn shadow(mut self, color: Color) -> Self {
        self.shadow = color;
        self
    }

    /// Returns or updates the `surface` value.
    pub fn surface(mut self, color: Color) -> Self {
        self.surface = color;
        self
    }

    /// Returns or updates the `selection` value.
    pub fn selection(mut self, color: Color) -> Self {
        self.selection = color;
        self
    }

    /// Returns or updates the `selection_color` value.
    pub fn selection_color(mut self, color: Color) -> Self {
        self.selection_color = color;
        self
    }

    /// Returns or updates the `caret_color` value.
    pub fn caret_color(mut self, color: Color) -> Self {
        self.caret_color = color;
        self
    }

    /// Returns or updates the `scrollbar_thumb` value.
    pub fn scrollbar_thumb(mut self, color: Color) -> Self {
        self.scrollbar_thumb = color;
        self
    }

    /// Returns or updates the `scrollbar_track` value.
    pub fn scrollbar_track(mut self, color: Color) -> Self {
        self.scrollbar_track = color;
        self
    }

    /// Returns or updates the `scrollbar_button` value.
    pub fn scrollbar_button(mut self, color: Color) -> Self {
        self.scrollbar_button = color;
        self
    }

    /// Returns or updates the `scrollbar_arrow` value.
    pub fn scrollbar_arrow(mut self, color: Color) -> Self {
        self.scrollbar_arrow = color;
        self
    }

    /// Returns or updates the `scrollbar_thumb_border` value.
    pub fn scrollbar_thumb_border(mut self, color: Color) -> Self {
        self.scrollbar_thumb_border = color;
        self
    }

    /// Returns or updates the `scrollbar_track_border` value.
    pub fn scrollbar_track_border(mut self, color: Color) -> Self {
        self.scrollbar_track_border = color;
        self
    }

    /// Returns or updates the `selection_border_width` value.
    pub fn selection_border_width(mut self, width: Length) -> Self {
        self.selection_border_width = width;
        self
    }

    /// Returns or updates the `selection_border_color` value.
    pub fn selection_border_color(mut self, color: Color) -> Self {
        self.selection_border_color = color;
        self
    }

    /// Returns or updates the `selection_border_radius` value.
    pub fn selection_border_radius(mut self, radius: Length) -> Self {
        self.selection_border_radius = radius;
        self
    }

    /* Radius: start */
    /// Returns or updates the `radius_xs` value.
    pub fn radius_xs(mut self, radius: impl Into<Length>) -> Self {
        self.radius_xs = radius.into();
        self
    }

    /// Returns or updates the `radius_sm` value.
    pub fn radius_sm(mut self, radius: impl Into<Length>) -> Self {
        self.radius_sm = radius.into();
        self
    }

    /// Returns or updates the `radius_md` value.
    pub fn radius_md(mut self, radius: impl Into<Length>) -> Self {
        self.radius_md = radius.into();
        self
    }

    /// Returns or updates the `radius_lg` value.
    pub fn radius_lg(mut self, radius: impl Into<Length>) -> Self {
        self.radius_lg = radius.into();
        self
    }

    /// Returns or updates the `radius_xl` value.
    pub fn radius_xl(mut self, radius: impl Into<Length>) -> Self {
        self.radius_xl = radius.into();
        self
    }

    /// Returns or updates the `radius_2xl` value.
    pub fn radius_2xl(mut self, radius: impl Into<Length>) -> Self {
        self.radius_2xl = radius.into();
        self
    }

    /// Returns or updates the `radius_3xl` value.
    pub fn radius_3xl(mut self, radius: impl Into<Length>) -> Self {
        self.radius_3xl = radius.into();
        self
    }

    /// Returns or updates the `radius_4xl` value.
    pub fn radius_4xl(mut self, radius: impl Into<Length>) -> Self {
        self.radius_4xl = radius.into();
        self
    }
    /* Radius: end */

    /* Padding: start */
    /// Returns or updates the `space_xs` value.
    pub fn space_xs(mut self, space: impl Into<Length>) -> Self {
        self.space_xs = space.into();
        self
    }

    /// Returns or updates the `space_sm` value.
    pub fn space_sm(mut self, space: impl Into<Length>) -> Self {
        self.space_sm = space.into();
        self
    }

    /// Returns or updates the `space_md` value.
    pub fn space_md(mut self, space: impl Into<Length>) -> Self {
        self.space_md = space.into();
        self
    }

    /// Returns or updates the `space_lg` value.
    pub fn space_lg(mut self, space: impl Into<Length>) -> Self {
        self.space_lg = space.into();
        self
    }

    /// Returns or updates the `space_xl` value.
    pub fn space_xl(mut self, space: impl Into<Length>) -> Self {
        self.space_xl = space.into();
        self
    }

    /// Returns or updates the `space_2xl` value.
    pub fn space_2xl(mut self, space: impl Into<Length>) -> Self {
        self.space_2xl = space.into();
        self
    }

    /// Returns or updates the `space_3xl` value.
    pub fn space_3xl(mut self, space: impl Into<Length>) -> Self {
        self.space_3xl = space.into();
        self
    }

    /// Returns or updates the `space_4xl` value.
    pub fn space_4xl(mut self, space: impl Into<Length>) -> Self {
        self.space_4xl = space.into();
        self
    }
    /* Padding: end */

    /// Returns or updates the `border_width` value.
    pub fn border_width(mut self, width: impl Into<Length>) -> Self {
        self.border_width = width.into();
        self
    }

    /// Returns or updates the `name` value.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns whether the `is_dark` condition is satisfied.
    pub const fn is_dark(&self) -> bool {
        matches!(self.mode, ThemeMode::Dark)
    }

    /// Returns whether the `is_auto` condition is satisfied.
    pub const fn is_auto(&self) -> bool {
        matches!(self.mode, ThemeMode::Auto)
    }

    /// Rebuilds the color set from `Theme::light()`/`Theme::dark()` based on
    /// `system_is_dark`, while keeping every non-color field (radius,
    /// spacing, typography, border width, name) from `self` untouched.
    /// A no-op for non-`Auto` themes.
    pub fn resolved_for_system(&self, system_is_dark: bool) -> Self {
        if !self.is_auto() {
            return self.clone();
        }
        let palette = if system_is_dark {
            Self::dark()
        } else {
            Self::light()
        };
        Self {
            inverse_primary: palette.inverse_primary,
            primary: palette.primary,
            on_primary: palette.on_primary,
            primary_container: palette.primary_container,
            on_primary_container: palette.on_primary_container,
            primary_fixed: palette.primary_fixed,
            primary_fixed_dim: palette.primary_fixed_dim,
            on_primary_fixed: palette.on_primary_fixed,
            on_primary_fixed_variant: palette.on_primary_fixed_variant,

            inverse_secondary: palette.inverse_secondary,
            secondary: palette.secondary,
            on_secondary: palette.on_secondary,
            secondary_container: palette.secondary_container,
            on_secondary_container: palette.on_secondary_container,
            secondary_fixed: palette.secondary_fixed,
            secondary_fixed_dim: palette.secondary_fixed_dim,
            on_secondary_fixed: palette.on_secondary_fixed,
            on_secondary_fixed_variant: palette.on_secondary_fixed_variant,

            inverse_tertiary: palette.inverse_tertiary,
            tertiary: palette.tertiary,
            on_tertiary: palette.on_tertiary,
            tertiary_container: palette.tertiary_container,
            on_tertiary_container: palette.on_tertiary_container,
            tertiary_fixed: palette.tertiary_fixed,
            tertiary_fixed_dim: palette.tertiary_fixed_dim,
            on_tertiary_fixed: palette.on_tertiary_fixed,
            on_tertiary_fixed_variant: palette.on_tertiary_fixed_variant,

            info: palette.info,
            on_info: palette.on_info,
            info_container: palette.info_container,
            on_info_container: palette.on_info_container,

            error: palette.error,
            on_error: palette.on_error,
            error_container: palette.error_container,
            on_error_container: palette.on_error_container,

            warning: palette.warning,
            on_warning: palette.on_warning,
            warning_container: palette.warning_container,
            on_warning_container: palette.on_warning_container,

            success: palette.success,
            on_success: palette.on_success,
            success_container: palette.success_container,
            on_success_container: palette.on_success_container,

            surface_dim: palette.surface_dim,
            surface: palette.surface,
            surface_bright: palette.surface_bright,
            inverse_surface: palette.inverse_surface,
            inverse_on_surface: palette.inverse_on_surface,
            surface_container_low: palette.surface_container_low,
            surface_container_lowest: palette.surface_container_lowest,
            surface_container: palette.surface_container,
            surface_container_high: palette.surface_container_high,
            surface_container_highest: palette.surface_container_highest,
            on_surface: palette.on_surface,
            on_surface_variant: palette.on_surface_variant,

            outline: palette.outline,
            outline_variant: palette.outline_variant,

            scrim: palette.scrim,
            shadow: palette.shadow,

            selection: palette.selection,
            selection_color: palette.selection_color,
            selection_border_color: palette.selection_border_color,
            caret_color: palette.caret_color,

            scrollbar_thumb: palette.scrollbar_thumb,
            scrollbar_track: palette.scrollbar_track,
            scrollbar_button: palette.scrollbar_button,
            scrollbar_arrow: palette.scrollbar_arrow,
            scrollbar_thumb_border: palette.scrollbar_thumb_border,
            scrollbar_track_border: palette.scrollbar_track_border,

            ..self.clone()
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::light()
    }
}

// Which theme should become active on the next render pass; requested via
// `set_active_theme`/`set_active_theme_by_name` from anywhere in user code.
/// Available `ThemeSwitch` choices.
pub enum ThemeSwitch {
    /// The `Index` variant.
    Index(usize),
    /// The `Name` variant.
    Name(String),
}

thread_local! {
    static CURRENT_THEME: RefCell<Theme> = RefCell::new(Theme::default());
    static THEME_SWITCH: RefCell<Option<ThemeSwitch>> = const { RefCell::new(None) };
    // Reflects the OS light/dark preference, refreshed once per painted
    // frame from the `SystemTheme` the render backend receives (see
    // `FrameRenderer::render_frame`). Only consulted for `Theme::auto()`
    // themes - a theme with an explicit Light/Dark mode ignores it.
    static SYSTEM_IS_DARK: Cell<bool> = const { Cell::new(true) };
}

/// Updates the `set_current_theme` value.
pub fn set_current_theme(theme: Theme) {
    CURRENT_THEME.with(|cell| {
        *cell.borrow_mut() = theme;
    });
}

/// Updates the OS light/dark flag used to resolve `Theme::auto()` themes.
/// Called once per frame by the render pipeline - not meant to be called
/// directly by application code.
pub fn set_system_is_dark(is_dark: bool) {
    SYSTEM_IS_DARK.with(|cell| cell.set(is_dark));
}

/// Returns or updates the `take_theme_switch` value.
pub fn take_theme_switch() -> Option<ThemeSwitch> {
    THEME_SWITCH.with(|cell| cell.borrow_mut().take())
}

/// Returns or updates the `current_theme` value.
pub fn current_theme() -> Theme {
    CURRENT_THEME.with(|cell| {
        let theme = cell.borrow().clone();
        if theme.is_auto() {
            theme.resolved_for_system(SYSTEM_IS_DARK.with(Cell::get))
        } else {
            theme
        }
    })
}

/// Switches the app's active theme by index into `AppConfig::themes`,
/// triggering a rebuild on the next frame.
pub fn set_active_theme(index: usize) {
    THEME_SWITCH.with(|cell| {
        *cell.borrow_mut() = Some(ThemeSwitch::Index(index));
    });
    crate::hooks::mark_dirty_and_redraw();
}

/// Switches the app's active theme by matching `Theme::name()` against
/// `AppConfig::themes`, triggering a rebuild on the next frame.
pub fn set_active_theme_by_name(name: impl Into<String>) {
    THEME_SWITCH.with(|cell| {
        *cell.borrow_mut() = Some(ThemeSwitch::Name(name.into()));
    });
    crate::hooks::mark_dirty_and_redraw();
}

/// Data and behavior represented by `ValueMarker`.
pub struct ValueMarker;
/// Data and behavior represented by `FnMarker`.
pub struct FnMarker;

/// Behavior required from `IntoThemed` implementations.
pub trait IntoThemed<T, Marker> {
    /// Returns or updates the `resolve_themed` value.
    fn resolve_themed(self) -> T;
}

impl IntoThemed<Color, ValueMarker> for Color {
    fn resolve_themed(self) -> Color {
        self
    }
}

impl<F: FnOnce(&Theme) -> Color> IntoThemed<Color, FnMarker> for F {
    fn resolve_themed(self) -> Color {
        CURRENT_THEME.with(|cell| self(&cell.borrow()))
    }
}

impl IntoThemed<Background, ValueMarker> for Color {
    fn resolve_themed(self) -> Background {
        Background::Color(self)
    }
}

impl IntoThemed<Background, ValueMarker> for Background {
    fn resolve_themed(self) -> Background {
        self
    }
}

impl<T, F> IntoThemed<Background, FnMarker> for F
where
    T: Into<Background>,
    F: FnOnce(&Theme) -> T,
{
    fn resolve_themed(self) -> Background {
        CURRENT_THEME.with(|cell| self(&cell.borrow()).into())
    }
}

impl<T: Into<Length>> IntoThemed<Length, ValueMarker> for T {
    fn resolve_themed(self) -> Length {
        self.into()
    }
}

impl<F: FnOnce(&Theme) -> Length> IntoThemed<Length, FnMarker> for F {
    fn resolve_themed(self) -> Length {
        CURRENT_THEME.with(|cell| self(&cell.borrow()))
    }
}

impl<T: Into<Edges>> IntoThemed<Edges, ValueMarker> for T {
    fn resolve_themed(self) -> Edges {
        self.into()
    }
}

impl<F: FnOnce(&Theme) -> Edges> IntoThemed<Edges, FnMarker> for F {
    fn resolve_themed(self) -> Edges {
        CURRENT_THEME.with(|cell| self(&cell.borrow()))
    }
}

impl IntoThemed<Border, ValueMarker> for Border {
    fn resolve_themed(self) -> Border {
        self
    }
}

impl<F: FnOnce(&Theme) -> Border> IntoThemed<Border, FnMarker> for F {
    fn resolve_themed(self) -> Border {
        CURRENT_THEME.with(|cell| self(&cell.borrow()))
    }
}

impl IntoThemed<StyleValue<Outline>, ValueMarker> for Outline {
    fn resolve_themed(self) -> StyleValue<Outline> {
        StyleValue::Value(self)
    }
}

impl IntoThemed<StyleValue<Outline>, ValueMarker> for StyleValue<Outline> {
    fn resolve_themed(self) -> StyleValue<Outline> {
        self
    }
}

impl<F: FnOnce(&Theme) -> Outline> IntoThemed<StyleValue<Outline>, FnMarker> for F {
    fn resolve_themed(self) -> StyleValue<Outline> {
        StyleValue::Value(CURRENT_THEME.with(|cell| self(&cell.borrow())))
    }
}

impl IntoThemed<f32, ValueMarker> for f32 {
    fn resolve_themed(self) -> f32 {
        self
    }
}

impl<F: FnOnce(&Theme) -> f32> IntoThemed<f32, FnMarker> for F {
    fn resolve_themed(self) -> f32 {
        CURRENT_THEME.with(|cell| self(&cell.borrow()))
    }
}
