// SPDX-License-Identifier: Apache-2.0
use crate::{ Border, Outline, properties::StyleValue };
use super::{ Background, Color, Edges, Length };
use std::cell::{ Cell, RefCell };

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ThemeMode {
    Light,
    Dark,
    Auto,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Theme {
    name: String,
    mode: ThemeMode,

    /* Colors */
    pub background: Color,
    pub on_background: Color,

    // Primary
    pub inverse_primary: Color,
    pub primary: Color,
    pub on_primary: Color,
    pub primary_container: Color,
    pub on_primary_container: Color,

    pub primary_fixed: Color,
    pub primary_fixed_dim: Color,
    pub on_primary_fixed: Color,
    pub on_primary_fixed_variant: Color,

    // Secondary
    pub inverse_secondary: Color,
    pub secondary: Color,
    pub on_secondary: Color,
    pub secondary_container: Color,
    pub on_secondary_container: Color,

    pub secondary_fixed: Color,
    pub secondary_fixed_dim: Color,
    pub on_secondary_fixed: Color,
    pub on_secondary_fixed_variant: Color,

    // Tertiary
    pub inverse_tertiary: Color,
    pub tertiary: Color,
    pub on_tertiary: Color,
    pub tertiary_container: Color,
    pub on_tertiary_container: Color,

    pub tertiary_fixed: Color,
    pub tertiary_fixed_dim: Color,
    pub on_tertiary_fixed: Color,
    pub on_tertiary_fixed_variant: Color,

    // Info
    pub info: Color,
    pub on_info: Color,
    pub info_container: Color,
    pub on_info_container: Color,

    // Error
    pub error: Color,
    pub on_error: Color,
    pub error_container: Color,
    pub on_error_container: Color,

    // Warning
    pub warning: Color,
    pub on_warning: Color,
    pub warning_container: Color,
    pub on_warning_container: Color,

    // Success
    pub success: Color,
    pub on_success: Color,
    pub success_container: Color,
    pub on_success_container: Color,

    // Surface
    pub surface_dim: Color,
    pub surface: Color,
    pub surface_bright: Color,

    pub inverse_surface: Color,
    pub inverse_on_surface: Color,

    pub surface_container_low: Color,
    pub surface_container_lowest: Color,
    pub surface_container: Color,
    pub surface_container_high: Color,
    pub surface_container_highest: Color,

    pub on_surface: Color,
    pub on_surface_variant: Color,

    // Outline
    pub outline: Color,
    pub outline_variant: Color,

    // Scrim & Shadow
    pub scrim: Color,
    pub shadow: Color,

    /* -------------------------------------- */

    pub selection: Color,
    pub selection_color: Color,
    pub selection_border_color: Color,
    pub selection_border_width: Length,
    pub selection_border_radius: Length,
    pub caret_color: Color,

    pub scrollbar_thumb: Color,
    pub scrollbar_track: Color,
    pub scrollbar_button: Color,
    pub scrollbar_arrow: Color,
    pub scrollbar_thumb_border: Color,
    pub scrollbar_track_border: Color,

    pub radius_xs: Length,
    pub radius_sm: Length,
    pub radius_md: Length,
    pub radius_lg: Length,
    pub radius_xl: Length,
    pub radius_2xl: Length,
    pub radius_3xl: Length,
    pub radius_4xl: Length,

    pub space_xs: Length,
    pub space_sm: Length,
    pub space_md: Length,
    pub space_lg: Length,
    pub space_xl: Length,
    pub space_2xl: Length,
    pub space_3xl: Length,
    pub space_4xl: Length,

    /* Typography */
    pub text_xs: Length,
    pub text_sm: Length,
    pub text_md: Length,
    pub text_lg: Length,
    pub text_xl: Length,
    pub text_2xl: Length,
    pub text_3xl: Length,
    pub text_4xl: Length,

    pub border_width: Length,
}

impl Theme {
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

    pub fn auto() -> Self {
        let mut theme = Self::light();
        theme.mode = ThemeMode::Auto;
        theme
    }

    pub fn mode(mut self, mode: ThemeMode) -> Self {
        self.mode = mode;
        self
    }

    pub fn background(mut self, color: Color) -> Self {
        self.background = color;
        self
    }

    pub fn on_background(mut self, color: Color) -> Self {
        self.on_background = color;
        self
    }

    pub fn inverse_primary(mut self, color: Color) -> Self {
        self.inverse_primary = color;
        self
    }

    pub fn primary(mut self, color: Color) -> Self {
        self.primary = color;
        self
    }

    pub fn on_primary(mut self, color: Color) -> Self {
        self.on_primary = color;
        self
    }

    pub fn primary_container(mut self, color: Color) -> Self {
        self.primary_container = color;
        self
    }

    pub fn on_primary_container(mut self, color: Color) -> Self {
        self.on_primary_container = color;
        self
    }

    pub fn primary_fixed(mut self, color: Color) -> Self {
        self.primary_fixed = color;
        self
    }

    pub fn primary_fixed_dim(mut self, color: Color) -> Self {
        self.primary_fixed_dim = color;
        self
    }

    pub fn on_primary_fixed(mut self, color: Color) -> Self {
        self.on_primary_fixed = color;
        self
    }

    pub fn on_primary_fixed_variant(mut self, color: Color) -> Self {
        self.on_primary_fixed_variant = color;
        self
    }

    pub fn inverse_secondary(mut self, color: Color) -> Self {
        self.inverse_secondary = color;
        self
    }

    pub fn secondary(mut self, color: Color) -> Self {
        self.secondary = color;
        self
    }

    pub fn on_secondary(mut self, color: Color) -> Self {
        self.on_secondary = color;
        self
    }

    pub fn secondary_container(mut self, color: Color) -> Self {
        self.secondary_container = color;
        self
    }

    pub fn on_secondary_container(mut self, color: Color) -> Self {
        self.on_secondary_container = color;
        self
    }

    pub fn secondary_fixed(mut self, color: Color) -> Self {
        self.secondary_fixed = color;
        self
    }

    pub fn secondary_fixed_dim(mut self, color: Color) -> Self {
        self.secondary_fixed_dim = color;
        self
    }

    pub fn on_secondary_fixed(mut self, color: Color) -> Self {
        self.on_secondary_fixed = color;
        self
    }

    pub fn on_secondary_fixed_variant(mut self, color: Color) -> Self {
        self.on_secondary_fixed_variant = color;
        self
    }

    pub fn inverse_tertiary(mut self, color: Color) -> Self {
        self.inverse_tertiary = color;
        self
    }

    pub fn tertiary(mut self, color: Color) -> Self {
        self.tertiary = color;
        self
    }

    pub fn on_tertiary(mut self, color: Color) -> Self {
        self.on_tertiary = color;
        self
    }

    pub fn tertiary_container(mut self, color: Color) -> Self {
        self.tertiary_container = color;
        self
    }

    pub fn on_tertiary_container(mut self, color: Color) -> Self {
        self.on_tertiary_container = color;
        self
    }

    pub fn tertiary_fixed(mut self, color: Color) -> Self {
        self.tertiary_fixed = color;
        self
    }

    pub fn tertiary_fixed_dim(mut self, color: Color) -> Self {
        self.tertiary_fixed_dim = color;
        self
    }

    pub fn on_tertiary_fixed(mut self, color: Color) -> Self {
        self.on_tertiary_fixed = color;
        self
    }

    pub fn on_tertiary_fixed_variant(mut self, color: Color) -> Self {
        self.on_tertiary_fixed_variant = color;
        self
    }

    pub fn info(mut self, color: Color) -> Self {
        self.info = color;
        self
    }

    pub fn on_info(mut self, color: Color) -> Self {
        self.on_info = color;
        self
    }

    pub fn info_container(mut self, color: Color) -> Self {
        self.info_container = color;
        self
    }

    pub fn on_info_container(mut self, color: Color) -> Self {
        self.on_info_container = color;
        self
    }

    pub fn error(mut self, color: Color) -> Self {
        self.error = color;
        self
    }

    pub fn on_error(mut self, color: Color) -> Self {
        self.on_error = color;
        self
    }

    pub fn error_container(mut self, color: Color) -> Self {
        self.error_container = color;
        self
    }

    pub fn on_error_container(mut self, color: Color) -> Self {
        self.on_error_container = color;
        self
    }

    pub fn warning(mut self, color: Color) -> Self {
        self.warning = color;
        self
    }

    pub fn on_warning(mut self, color: Color) -> Self {
        self.on_warning = color;
        self
    }

    pub fn warning_container(mut self, color: Color) -> Self {
        self.warning_container = color;
        self
    }

    pub fn on_warning_container(mut self, color: Color) -> Self {
        self.on_warning_container = color;
        self
    }

    pub fn success(mut self, color: Color) -> Self {
        self.success = color;
        self
    }

    pub fn on_success(mut self, color: Color) -> Self {
        self.on_success = color;
        self
    }

    pub fn success_container(mut self, color: Color) -> Self {
        self.success_container = color;
        self
    }

    pub fn on_success_container(mut self, color: Color) -> Self {
        self.on_success_container = color;
        self
    }

    pub fn surface_dim(mut self, color: Color) -> Self {
        self.surface_dim = color;
        self
    }

    pub fn surface_bright(mut self, color: Color) -> Self {
        self.surface_bright = color;
        self
    }

    pub fn inverse_surface(mut self, color: Color) -> Self {
        self.inverse_surface = color;
        self
    }

    pub fn inverse_on_surface(mut self, color: Color) -> Self {
        self.inverse_on_surface = color;
        self
    }

    pub fn surface_container_low(mut self, color: Color) -> Self {
        self.surface_container_low = color;
        self
    }

    pub fn surface_container_lowest(mut self, color: Color) -> Self {
        self.surface_container_lowest = color;
        self
    }

    pub fn surface_container(mut self, color: Color) -> Self {
        self.surface_container = color;
        self
    }

    pub fn surface_container_high(mut self, color: Color) -> Self {
        self.surface_container_high = color;
        self
    }

    pub fn surface_container_highest(mut self, color: Color) -> Self {
        self.surface_container_highest = color;
        self
    }

    pub fn on_surface(mut self, color: Color) -> Self {
        self.on_surface = color;
        self
    }

    pub fn on_surface_variant(mut self, color: Color) -> Self {
        self.on_surface_variant = color;
        self
    }

    pub fn outline(mut self, color: Color) -> Self {
        self.outline = color;
        self
    }

    pub fn outline_variant(mut self, color: Color) -> Self {
        self.outline_variant = color;
        self
    }

    pub fn scrim(mut self, color: Color) -> Self {
        self.scrim = color;
        self
    }

    pub fn shadow(mut self, color: Color) -> Self {
        self.shadow = color;
        self
    }

    pub fn surface(mut self, color: Color) -> Self {
        self.surface = color;
        self
    }

    pub fn selection(mut self, color: Color) -> Self {
        self.selection = color;
        self
    }

    pub fn selection_color(mut self, color: Color) -> Self {
        self.selection_color = color;
        self
    }

    pub fn caret_color(mut self, color: Color) -> Self {
        self.caret_color = color;
        self
    }

    pub fn scrollbar_thumb(mut self, color: Color) -> Self {
        self.scrollbar_thumb = color;
        self
    }

    pub fn scrollbar_track(mut self, color: Color) -> Self {
        self.scrollbar_track = color;
        self
    }

    pub fn scrollbar_button(mut self, color: Color) -> Self {
        self.scrollbar_button = color;
        self
    }

    pub fn scrollbar_arrow(mut self, color: Color) -> Self {
        self.scrollbar_arrow = color;
        self
    }

    pub fn scrollbar_thumb_border(mut self, color: Color) -> Self {
        self.scrollbar_thumb_border = color;
        self
    }

    pub fn scrollbar_track_border(mut self, color: Color) -> Self {
        self.scrollbar_track_border = color;
        self
    }

    pub fn selection_border_width(mut self, width: Length) -> Self {
        self.selection_border_width = width;
        self
    }

    pub fn selection_border_color(mut self, color: Color) -> Self {
        self.selection_border_color = color;
        self
    }

    pub fn selection_border_radius(mut self, radius: Length) -> Self {
        self.selection_border_radius = radius;
        self
    }

    /* Radius: start */
    pub fn radius_xs(mut self, radius: impl Into<Length>) -> Self {
        self.radius_xs = radius.into();
        self
    }

    pub fn radius_sm(mut self, radius: impl Into<Length>) -> Self {
        self.radius_sm = radius.into();
        self
    }

    pub fn radius_md(mut self, radius: impl Into<Length>) -> Self {
        self.radius_md = radius.into();
        self
    }

    pub fn radius_lg(mut self, radius: impl Into<Length>) -> Self {
        self.radius_lg = radius.into();
        self
    }

    pub fn radius_xl(mut self, radius: impl Into<Length>) -> Self {
        self.radius_xl = radius.into();
        self
    }

    pub fn radius_2xl(mut self, radius: impl Into<Length>) -> Self {
        self.radius_2xl = radius.into();
        self
    }

    pub fn radius_3xl(mut self, radius: impl Into<Length>) -> Self {
        self.radius_3xl = radius.into();
        self
    }

    pub fn radius_4xl(mut self, radius: impl Into<Length>) -> Self {
        self.radius_4xl = radius.into();
        self
    }
    /* Radius: end */

    /* Padding: start */
    pub fn space_xs(mut self, space: impl Into<Length>) -> Self {
        self.space_xs = space.into();
        self
    }

    pub fn space_sm(mut self, space: impl Into<Length>) -> Self {
        self.space_sm = space.into();
        self
    }

    pub fn space_md(mut self, space: impl Into<Length>) -> Self {
        self.space_md = space.into();
        self
    }

    pub fn space_lg(mut self, space: impl Into<Length>) -> Self {
        self.space_lg = space.into();
        self
    }

    pub fn space_xl(mut self, space: impl Into<Length>) -> Self {
        self.space_xl = space.into();
        self
    }

    pub fn space_2xl(mut self, space: impl Into<Length>) -> Self {
        self.space_2xl = space.into();
        self
    }

    pub fn space_3xl(mut self, space: impl Into<Length>) -> Self {
        self.space_3xl = space.into();
        self
    }

    pub fn space_4xl(mut self, space: impl Into<Length>) -> Self {
        self.space_4xl = space.into();
        self
    }
    /* Padding: end */

    pub fn border_width(mut self, width: impl Into<Length>) -> Self {
        self.border_width = width.into();
        self
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub const fn is_dark(&self) -> bool {
        matches!(self.mode, ThemeMode::Dark)
    }

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
        let palette = if system_is_dark { Self::dark() } else { Self::light() };
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
pub enum ThemeSwitch {
    Index(usize),
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

pub fn take_theme_switch() -> Option<ThemeSwitch> {
    THEME_SWITCH.with(|cell| cell.borrow_mut().take())
}

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

pub struct ValueMarker;
pub struct FnMarker;

pub trait IntoThemed<T, Marker> {
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

impl<T, F> IntoThemed<Background, FnMarker> for F where T: Into<Background>, F: FnOnce(&Theme) -> T {
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
