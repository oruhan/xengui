// SPDX-License-Identifier: Apache-2.0
#[derive(Clone, Copy, Debug, PartialEq)]
/// Data and behavior represented by `Color`.
pub struct Color {
    r: f32,
    g: f32,
    b: f32,
    a: f32,
}

/// Data and behavior represented by `OklchColor`.
pub struct OklchColor {
    /// The `l` value carried by this type.
    pub l: f32, // 0.0 - 1.0
    /// The `c` value carried by this type.
    pub c: f32, // 0.0 - 0.4+
    /// The `h` value carried by this type.
    pub h: f32, // 0.0 - 360.0
}

impl Color {
    /// Returns or updates the `r` value.
    pub const fn r(&self) -> f32 {
        self.r
    }

    /// Returns or updates the `g` value.
    pub const fn g(&self) -> f32 {
        self.g
    }

    /// Returns or updates the `b` value.
    pub const fn b(&self) -> f32 {
        self.b
    }

    /// Returns or updates the `a` value.
    pub const fn a(&self) -> f32 {
        self.a
    }
}

impl Color {
    /// Returns or updates the `rgb` value.
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self::rgba(r, g, b, 255)
    }

    /// Returns or updates the `rgba` value.
    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self {
            r: (r as f32) / 255.0,
            g: (g as f32) / 255.0,
            b: (b as f32) / 255.0,
            a: (a as f32) / 255.0,
        }
    }

    /// Returns or updates the `rgb_f32` value.
    pub const fn rgb_f32(r: f32, g: f32, b: f32) -> Self {
        Self::rgba_f32(r, g, b, 1.0)
    }

    /// Returns or updates the `rgba_f32` value.
    pub const fn rgba_f32(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    /// Returns or updates the `to_f32_array` value.
    pub const fn to_f32_array(self) -> [f32; 4] {
        [self.r(), self.g(), self.b(), self.a()]
    }

    /// 0-255
    pub const fn with_alpha(self, alpha: u8) -> Self {
        Self {
            r: self.r,
            g: self.g,
            b: self.b,
            a: (alpha as f32) / 255.0,
        }
    }

    /// Updates the `with_alpha_f32` value.
    pub const fn with_alpha_f32(self, alpha: f32) -> Self {
        Self {
            r: self.r,
            g: self.g,
            b: self.b,
            a: alpha,
        }
    }

    #[allow(clippy::excessive_precision)]
    /// Returns or updates the `oklch` value.
    pub fn oklch(l: f32, c: f32, h: f32) -> Self {
        let l = l / 100.0;
        let h_rad = h.to_radians();

        // 1. Oklch -> Oklab convertion
        let a_lab = c * h_rad.cos();
        let b_lab = c * h_rad.sin();

        // 2. Oklab -> Conical LMS space
        let l_lms = l + 0.3963377774 * a_lab + 0.2158037573 * b_lab;
        let m_lms = l - 0.1055613458 * a_lab - 0.0638541728 * b_lab;
        let s_lms = l - 0.0894841775 * a_lab - 1.291485548 * b_lab;

        // 3. Non-linear LMS transformation
        // Note: Cubes of negative LMS values must mathematically remain
        // negative (e.g., (-0.5)^3 = -0.125), which powi(3) already handles correctly -
        // using powf would produce NaN with a negative base. In combinations of
        // high chroma + restricted hue (out-of-gamut colors), these values
        // can drop into the negative range; this is normal, and the clamp in step 5 tolerates it.
        let l_ = l_lms.powi(3);
        let m_ = m_lms.powi(3);
        let s_ = s_lms.powi(3);

        // 4. LMS -> Linear RGB (Linear sRGB)
        let r_linear = 4.0767416621 * l_ - 3.3077115913 * m_ + 0.2309699292 * s_;
        let g_linear = -1.2684380046 * l_ + 2.6097574011 * m_ - 0.3413193965 * s_;
        let b_linear = -0.0041960863 * l_ - 0.7034186147 * m_ + 1.707614701 * s_;

        // 5. Linear RGB -> Standard sRGB (Gamma correction)
        // Note: powf returns NaN for negative bases; since r_linear/g_linear/b_linear
        // can be negative in out-of-gamut colors, they are first clipped to zero
        // using c.max(0.0) before the gamma curve is applied.
        let f = |c: f32| {
            let c = c.max(0.0);
            if c <= 0.0031308 {
                12.92 * c
            } else {
                1.055 * c.powf(1.0 / 2.4) - 0.055
            }
        };

        // Clamp values into 0.0 - 1.0 range and store
        Self::rgba_f32(
            f(r_linear).clamp(0.0, 1.0),
            f(g_linear).clamp(0.0, 1.0),
            f(b_linear).clamp(0.0, 1.0),
            1.0,
        )
    }

    // Creates a Color from a hex color code. Supports formats with or without the #
    // prefix, both short (#RGB, #RGBA) and long (#RRGGBB, #RRGGBBAA) - consistent with
    // CSS hex syntax. The short format is expanded by duplicating each digit (e.g., F
    // -> FF); if alpha is not provided, full opacity (0xFF) is assumed.
    //
    // In case of invalid length or characters, instead of panicking
    // it falls back to opaque black and logs the error via
    // log::error! - consistent with the error decoding behavior of Image::bytes
    // in the project.
    /// Returns or updates the `hex` value.
    pub fn hex(hex: &str) -> Self {
        let hex = hex.strip_prefix('#').unwrap_or(hex);

        // Expands short formats (#RGB, #RGBA) to long format by
        // duplicating each digit (e.g., "F0A" -> "FF00AA").
        let expanded: String;
        let hex = match hex.len() {
            3 | 4 => {
                expanded = hex.chars().flat_map(|c| [c, c]).collect();
                expanded.as_str()
            }
            _ => hex,
        };

        let parse_channel =
            |s: &str| -> Option<f32> { u8::from_str_radix(s, 16).ok().map(|v| (v as f32) / 255.0) };

        let parsed = match hex.len() {
            6 => (
                parse_channel(&hex[0..2]),
                parse_channel(&hex[2..4]),
                parse_channel(&hex[4..6]),
                Some(1.0),
            ),
            8 => (
                parse_channel(&hex[0..2]),
                parse_channel(&hex[2..4]),
                parse_channel(&hex[4..6]),
                parse_channel(&hex[6..8]),
            ),
            _ => (None, None, None, None),
        };

        match parsed {
            (Some(r), Some(g), Some(b), Some(a)) => Self::rgba_f32(r, g, b, a),
            _ => {
                log::error!("Color::hex invalid hex code: '{hex}'");
                Self::rgba_f32(0.0, 0.0, 0.0, 1.0)
            }
        }
    }

    // Color codes
    /// The `TRANSPARENT` color token.
    pub const TRANSPARENT: Self = Self::rgba(0, 0, 0, 0);
    /// The `BLACK` color token.
    pub const BLACK: Self = Self::rgb(0, 0, 0);
    /// The `WHITE` color token.
    pub const WHITE: Self = Self::rgb(255, 255, 255);

    /* =Red=-------------------------------------------------- */
    /// The `RED_50` color token.
    pub const RED_50: Self = Self::rgb(254, 242, 242);
    /// The `RED_100` color token.
    pub const RED_100: Self = Self::rgb(255, 226, 226);
    /// The `RED_200` color token.
    pub const RED_200: Self = Self::rgb(255, 201, 201);
    /// The `RED_300` color token.
    pub const RED_300: Self = Self::rgb(255, 162, 162);
    /// The `RED_400` color token.
    pub const RED_400: Self = Self::rgb(255, 100, 103);
    /// The `RED_500` color token.
    pub const RED_500: Self = Self::rgb(251, 44, 54);
    /// The `RED_600` color token.
    pub const RED_600: Self = Self::rgb(231, 0, 11);
    /// The `RED_700` color token.
    pub const RED_700: Self = Self::rgb(193, 0, 7);
    /// The `RED_800` color token.
    pub const RED_800: Self = Self::rgb(159, 7, 18);
    /// The `RED_900` color token.
    pub const RED_900: Self = Self::rgb(130, 24, 26);
    /// The `RED_950` color token.
    pub const RED_950: Self = Self::rgb(70, 8, 9);

    /* =Orange=-------------------------------------------------- */
    /// The `ORANGE_50` color token.
    pub const ORANGE_50: Self = Self::rgb(255, 247, 237);
    /// The `ORANGE_100` color token.
    pub const ORANGE_100: Self = Self::rgb(255, 237, 212);
    /// The `ORANGE_200` color token.
    pub const ORANGE_200: Self = Self::rgb(255, 214, 167);
    /// The `ORANGE_300` color token.
    pub const ORANGE_300: Self = Self::rgb(255, 184, 106);
    /// The `ORANGE_400` color token.
    pub const ORANGE_400: Self = Self::rgb(255, 137, 4);
    /// The `ORANGE_500` color token.
    pub const ORANGE_500: Self = Self::rgb(255, 105, 0);
    /// The `ORANGE_600` color token.
    pub const ORANGE_600: Self = Self::rgb(245, 73, 0);
    /// The `ORANGE_700` color token.
    pub const ORANGE_700: Self = Self::rgb(202, 53, 0);
    /// The `ORANGE_800` color token.
    pub const ORANGE_800: Self = Self::rgb(159, 45, 0);
    /// The `ORANGE_900` color token.
    pub const ORANGE_900: Self = Self::rgb(126, 42, 12);
    /// The `ORANGE_950` color token.
    pub const ORANGE_950: Self = Self::rgb(68, 19, 6);

    /* =Amber=-------------------------------------------------- */
    /// The `AMBER_50` color token.
    pub const AMBER_50: Self = Self::rgb(255, 251, 235);
    /// The `AMBER_100` color token.
    pub const AMBER_100: Self = Self::rgb(254, 243, 198);
    /// The `AMBER_200` color token.
    pub const AMBER_200: Self = Self::rgb(254, 230, 133);
    /// The `AMBER_300` color token.
    pub const AMBER_300: Self = Self::rgb(255, 210, 48);
    /// The `AMBER_400` color token.
    pub const AMBER_400: Self = Self::rgb(255, 185, 0);
    /// The `AMBER_500` color token.
    pub const AMBER_500: Self = Self::rgb(254, 154, 0);
    /// The `AMBER_600` color token.
    pub const AMBER_600: Self = Self::rgb(225, 113, 0);
    /// The `AMBER_700` color token.
    pub const AMBER_700: Self = Self::rgb(187, 77, 0);
    /// The `AMBER_800` color token.
    pub const AMBER_800: Self = Self::rgb(151, 60, 0);
    /// The `AMBER_900` color token.
    pub const AMBER_900: Self = Self::rgb(123, 51, 6);
    /// The `AMBER_950` color token.
    pub const AMBER_950: Self = Self::rgb(70, 25, 1);

    /* =Yellow=-------------------------------------------------- */
    /// The `YELLOW_50` color token.
    pub const YELLOW_50: Self = Self::rgb(254, 252, 232);
    /// The `YELLOW_100` color token.
    pub const YELLOW_100: Self = Self::rgb(254, 249, 194);
    /// The `YELLOW_200` color token.
    pub const YELLOW_200: Self = Self::rgb(255, 240, 133);
    /// The `YELLOW_300` color token.
    pub const YELLOW_300: Self = Self::rgb(255, 223, 32);
    /// The `YELLOW_400` color token.
    pub const YELLOW_400: Self = Self::rgb(253, 199, 0);
    /// The `YELLOW_500` color token.
    pub const YELLOW_500: Self = Self::rgb(240, 177, 0);
    /// The `YELLOW_600` color token.
    pub const YELLOW_600: Self = Self::rgb(208, 135, 0);
    /// The `YELLOW_700` color token.
    pub const YELLOW_700: Self = Self::rgb(166, 95, 0);
    /// The `YELLOW_800` color token.
    pub const YELLOW_800: Self = Self::rgb(137, 75, 0);
    /// The `YELLOW_900` color token.
    pub const YELLOW_900: Self = Self::rgb(115, 62, 10);
    /// The `YELLOW_950` color token.
    pub const YELLOW_950: Self = Self::rgb(67, 32, 4);

    /* =Lime=-------------------------------------------------- */
    /// The `LIME_50` color token.
    pub const LIME_50: Self = Self::rgb(247, 254, 231);
    /// The `LIME_100` color token.
    pub const LIME_100: Self = Self::rgb(236, 252, 202);
    /// The `LIME_200` color token.
    pub const LIME_200: Self = Self::rgb(216, 249, 153);
    /// The `LIME_300` color token.
    pub const LIME_300: Self = Self::rgb(187, 244, 81);
    /// The `LIME_400` color token.
    pub const LIME_400: Self = Self::rgb(154, 230, 0);
    /// The `LIME_500` color token.
    pub const LIME_500: Self = Self::rgb(124, 207, 0);
    /// The `LIME_600` color token.
    pub const LIME_600: Self = Self::rgb(94, 165, 0);
    /// The `LIME_700` color token.
    pub const LIME_700: Self = Self::rgb(73, 125, 0);
    /// The `LIME_800` color token.
    pub const LIME_800: Self = Self::rgb(60, 99, 0);
    /// The `LIME_900` color token.
    pub const LIME_900: Self = Self::rgb(53, 83, 14);
    /// The `LIME_950` color token.
    pub const LIME_950: Self = Self::rgb(25, 46, 3);

    /* =Green=-------------------------------------------------- */
    /// The `GREEN_50` color token.
    pub const GREEN_50: Self = Self::rgb(240, 253, 244);
    /// The `GREEN_100` color token.
    pub const GREEN_100: Self = Self::rgb(220, 252, 231);
    /// The `GREEN_200` color token.
    pub const GREEN_200: Self = Self::rgb(185, 248, 207);
    /// The `GREEN_300` color token.
    pub const GREEN_300: Self = Self::rgb(123, 241, 168);
    /// The `GREEN_400` color token.
    pub const GREEN_400: Self = Self::rgb(5, 223, 114);
    /// The `GREEN_500` color token.
    pub const GREEN_500: Self = Self::rgb(0, 201, 80);
    /// The `GREEN_600` color token.
    pub const GREEN_600: Self = Self::rgb(0, 166, 62);
    /// The `GREEN_700` color token.
    pub const GREEN_700: Self = Self::rgb(0, 130, 54);
    /// The `GREEN_800` color token.
    pub const GREEN_800: Self = Self::rgb(1, 102, 48);
    /// The `GREEN_900` color token.
    pub const GREEN_900: Self = Self::rgb(13, 84, 43);
    /// The `GREEN_950` color token.
    pub const GREEN_950: Self = Self::rgb(3, 46, 21);

    /* =Emerald=-------------------------------------------------- */
    /// The `EMERALD_50` color token.
    pub const EMERALD_50: Self = Self::rgb(236, 253, 245);
    /// The `EMERALD_100` color token.
    pub const EMERALD_100: Self = Self::rgb(208, 250, 229);
    /// The `EMERALD_200` color token.
    pub const EMERALD_200: Self = Self::rgb(164, 244, 207);
    /// The `EMERALD_300` color token.
    pub const EMERALD_300: Self = Self::rgb(94, 233, 181);
    /// The `EMERALD_400` color token.
    pub const EMERALD_400: Self = Self::rgb(0, 212, 146);
    /// The `EMERALD_500` color token.
    pub const EMERALD_500: Self = Self::rgb(0, 188, 125);
    /// The `EMERALD_600` color token.
    pub const EMERALD_600: Self = Self::rgb(0, 153, 102);
    /// The `EMERALD_700` color token.
    pub const EMERALD_700: Self = Self::rgb(0, 122, 85);
    /// The `EMERALD_800` color token.
    pub const EMERALD_800: Self = Self::rgb(0, 96, 69);
    /// The `EMERALD_900` color token.
    pub const EMERALD_900: Self = Self::rgb(0, 79, 59);
    /// The `EMERALD_950` color token.
    pub const EMERALD_950: Self = Self::rgb(0, 44, 34);

    /* =Teal=-------------------------------------------------- */
    /// The `TEAL_50` color token.
    pub const TEAL_50: Self = Self::rgb(240, 253, 250);
    /// The `TEAL_100` color token.
    pub const TEAL_100: Self = Self::rgb(203, 251, 241);
    /// The `TEAL_200` color token.
    pub const TEAL_200: Self = Self::rgb(150, 247, 228);
    /// The `TEAL_300` color token.
    pub const TEAL_300: Self = Self::rgb(70, 236, 213);
    /// The `TEAL_400` color token.
    pub const TEAL_400: Self = Self::rgb(0, 213, 190);
    /// The `TEAL_500` color token.
    pub const TEAL_500: Self = Self::rgb(0, 187, 167);
    /// The `TEAL_600` color token.
    pub const TEAL_600: Self = Self::rgb(0, 150, 137);
    /// The `TEAL_700` color token.
    pub const TEAL_700: Self = Self::rgb(0, 120, 111);
    /// The `TEAL_800` color token.
    pub const TEAL_800: Self = Self::rgb(0, 95, 90);
    /// The `TEAL_900` color token.
    pub const TEAL_900: Self = Self::rgb(11, 79, 74);
    /// The `TEAL_950` color token.
    pub const TEAL_950: Self = Self::rgb(2, 47, 46);

    /* =Cyan=-------------------------------------------------- */
    /// The `CYAN_50` color token.
    pub const CYAN_50: Self = Self::rgb(236, 254, 255);
    /// The `CYAN_100` color token.
    pub const CYAN_100: Self = Self::rgb(206, 250, 254);
    /// The `CYAN_200` color token.
    pub const CYAN_200: Self = Self::rgb(162, 244, 253);
    /// The `CYAN_300` color token.
    pub const CYAN_300: Self = Self::rgb(83, 234, 253);
    /// The `CYAN_400` color token.
    pub const CYAN_400: Self = Self::rgb(0, 211, 242);
    /// The `CYAN_500` color token.
    pub const CYAN_500: Self = Self::rgb(0, 184, 219);
    /// The `CYAN_600` color token.
    pub const CYAN_600: Self = Self::rgb(0, 146, 184);
    /// The `CYAN_700` color token.
    pub const CYAN_700: Self = Self::rgb(0, 117, 149);
    /// The `CYAN_800` color token.
    pub const CYAN_800: Self = Self::rgb(0, 95, 120);
    /// The `CYAN_900` color token.
    pub const CYAN_900: Self = Self::rgb(16, 78, 100);
    /// The `CYAN_950` color token.
    pub const CYAN_950: Self = Self::rgb(5, 51, 69);

    /* =Sky=-------------------------------------------------- */
    /// The `SKY_50` color token.
    pub const SKY_50: Self = Self::rgb(240, 249, 255);
    /// The `SKY_100` color token.
    pub const SKY_100: Self = Self::rgb(223, 242, 254);
    /// The `SKY_200` color token.
    pub const SKY_200: Self = Self::rgb(184, 230, 254);
    /// The `SKY_300` color token.
    pub const SKY_300: Self = Self::rgb(116, 212, 255);
    /// The `SKY_400` color token.
    pub const SKY_400: Self = Self::rgb(0, 188, 255);
    /// The `SKY_500` color token.
    pub const SKY_500: Self = Self::rgb(0, 166, 244);
    /// The `SKY_600` color token.
    pub const SKY_600: Self = Self::rgb(0, 132, 209);
    /// The `SKY_700` color token.
    pub const SKY_700: Self = Self::rgb(0, 105, 168);
    /// The `SKY_800` color token.
    pub const SKY_800: Self = Self::rgb(0, 89, 138);
    /// The `SKY_900` color token.
    pub const SKY_900: Self = Self::rgb(2, 74, 112);
    /// The `SKY_950` color token.
    pub const SKY_950: Self = Self::rgb(5, 47, 74);

    /* =Blue=-------------------------------------------------- */
    /// The `BLUE_50` color token.
    pub const BLUE_50: Self = Self::rgb(239, 246, 255);
    /// The `BLUE_100` color token.
    pub const BLUE_100: Self = Self::rgb(219, 234, 254);
    /// The `BLUE_200` color token.
    pub const BLUE_200: Self = Self::rgb(190, 219, 255);
    /// The `BLUE_300` color token.
    pub const BLUE_300: Self = Self::rgb(142, 197, 255);
    /// The `BLUE_400` color token.
    pub const BLUE_400: Self = Self::rgb(81, 162, 255);
    /// The `BLUE_500` color token.
    pub const BLUE_500: Self = Self::rgb(43, 127, 255);
    /// The `BLUE_600` color token.
    pub const BLUE_600: Self = Self::rgb(21, 93, 252);
    /// The `BLUE_700` color token.
    pub const BLUE_700: Self = Self::rgb(20, 71, 230);
    /// The `BLUE_800` color token.
    pub const BLUE_800: Self = Self::rgb(25, 60, 184);
    /// The `BLUE_900` color token.
    pub const BLUE_900: Self = Self::rgb(28, 57, 142);
    /// The `BLUE_950` color token.
    pub const BLUE_950: Self = Self::rgb(22, 36, 86);

    /* =Indigo=-------------------------------------------------- */
    /// The `INDIGO_50` color token.
    pub const INDIGO_50: Self = Self::rgb(238, 242, 255);
    /// The `INDIGO_100` color token.
    pub const INDIGO_100: Self = Self::rgb(224, 231, 255);
    /// The `INDIGO_200` color token.
    pub const INDIGO_200: Self = Self::rgb(198, 210, 255);
    /// The `INDIGO_300` color token.
    pub const INDIGO_300: Self = Self::rgb(163, 179, 255);
    /// The `INDIGO_400` color token.
    pub const INDIGO_400: Self = Self::rgb(124, 134, 255);
    /// The `INDIGO_500` color token.
    pub const INDIGO_500: Self = Self::rgb(97, 95, 255);
    /// The `INDIGO_600` color token.
    pub const INDIGO_600: Self = Self::rgb(79, 57, 246);
    /// The `INDIGO_700` color token.
    pub const INDIGO_700: Self = Self::rgb(67, 45, 215);
    /// The `INDIGO_800` color token.
    pub const INDIGO_800: Self = Self::rgb(55, 42, 172);
    /// The `INDIGO_900` color token.
    pub const INDIGO_900: Self = Self::rgb(49, 44, 133);
    /// The `INDIGO_950` color token.
    pub const INDIGO_950: Self = Self::rgb(30, 26, 77);

    /* =Violet=-------------------------------------------------- */
    /// The `VIOLET_50` color token.
    pub const VIOLET_50: Self = Self::rgb(245, 243, 255);
    /// The `VIOLET_100` color token.
    pub const VIOLET_100: Self = Self::rgb(237, 233, 254);
    /// The `VIOLET_200` color token.
    pub const VIOLET_200: Self = Self::rgb(221, 214, 255);
    /// The `VIOLET_300` color token.
    pub const VIOLET_300: Self = Self::rgb(196, 180, 255);
    /// The `VIOLET_400` color token.
    pub const VIOLET_400: Self = Self::rgb(166, 132, 255);
    /// The `VIOLET_500` color token.
    pub const VIOLET_500: Self = Self::rgb(142, 81, 255);
    /// The `VIOLET_600` color token.
    pub const VIOLET_600: Self = Self::rgb(127, 34, 254);
    /// The `VIOLET_700` color token.
    pub const VIOLET_700: Self = Self::rgb(112, 8, 231);
    /// The `VIOLET_800` color token.
    pub const VIOLET_800: Self = Self::rgb(93, 14, 192);
    /// The `VIOLET_900` color token.
    pub const VIOLET_900: Self = Self::rgb(77, 23, 154);
    /// The `VIOLET_950` color token.
    pub const VIOLET_950: Self = Self::rgb(47, 13, 104);

    /* =Purple=-------------------------------------------------- */
    /// The `PURPLE_50` color token.
    pub const PURPLE_50: Self = Self::rgb(250, 245, 255);
    /// The `PURPLE_100` color token.
    pub const PURPLE_100: Self = Self::rgb(243, 232, 255);
    /// The `PURPLE_200` color token.
    pub const PURPLE_200: Self = Self::rgb(233, 212, 255);
    /// The `PURPLE_300` color token.
    pub const PURPLE_300: Self = Self::rgb(218, 178, 255);
    /// The `PURPLE_400` color token.
    pub const PURPLE_400: Self = Self::rgb(194, 122, 255);
    /// The `PURPLE_500` color token.
    pub const PURPLE_500: Self = Self::rgb(173, 70, 255);
    /// The `PURPLE_600` color token.
    pub const PURPLE_600: Self = Self::rgb(152, 16, 250);
    /// The `PURPLE_700` color token.
    pub const PURPLE_700: Self = Self::rgb(130, 0, 219);
    /// The `PURPLE_800` color token.
    pub const PURPLE_800: Self = Self::rgb(110, 17, 176);
    /// The `PURPLE_900` color token.
    pub const PURPLE_900: Self = Self::rgb(89, 22, 139);
    /// The `PURPLE_950` color token.
    pub const PURPLE_950: Self = Self::rgb(60, 3, 102);

    /* =Fuchsia=-------------------------------------------------- */
    /// The `FUCHSIA_50` color token.
    pub const FUCHSIA_50: Self = Self::rgb(253, 244, 255);
    /// The `FUCHSIA_100` color token.
    pub const FUCHSIA_100: Self = Self::rgb(250, 232, 255);
    /// The `FUCHSIA_200` color token.
    pub const FUCHSIA_200: Self = Self::rgb(246, 207, 255);
    /// The `FUCHSIA_300` color token.
    pub const FUCHSIA_300: Self = Self::rgb(244, 168, 255);
    /// The `FUCHSIA_400` color token.
    pub const FUCHSIA_400: Self = Self::rgb(237, 106, 255);
    /// The `FUCHSIA_500` color token.
    pub const FUCHSIA_500: Self = Self::rgb(225, 42, 251);
    /// The `FUCHSIA_600` color token.
    pub const FUCHSIA_600: Self = Self::rgb(200, 0, 222);
    /// The `FUCHSIA_700` color token.
    pub const FUCHSIA_700: Self = Self::rgb(168, 0, 183);
    /// The `FUCHSIA_800` color token.
    pub const FUCHSIA_800: Self = Self::rgb(138, 1, 148);
    /// The `FUCHSIA_900` color token.
    pub const FUCHSIA_900: Self = Self::rgb(114, 19, 120);
    /// The `FUCHSIA_950` color token.
    pub const FUCHSIA_950: Self = Self::rgb(75, 0, 79);

    /* =Pink=-------------------------------------------------- */
    /// The `PINK_50` color token.
    pub const PINK_50: Self = Self::rgb(253, 242, 248);
    /// The `PINK_100` color token.
    pub const PINK_100: Self = Self::rgb(252, 231, 243);
    /// The `PINK_200` color token.
    pub const PINK_200: Self = Self::rgb(252, 206, 232);
    /// The `PINK_300` color token.
    pub const PINK_300: Self = Self::rgb(253, 165, 213);
    /// The `PINK_400` color token.
    pub const PINK_400: Self = Self::rgb(251, 100, 182);
    /// The `PINK_500` color token.
    pub const PINK_500: Self = Self::rgb(246, 51, 154);
    /// The `PINK_600` color token.
    pub const PINK_600: Self = Self::rgb(230, 0, 118);
    /// The `PINK_700` color token.
    pub const PINK_700: Self = Self::rgb(198, 0, 92);
    /// The `PINK_800` color token.
    pub const PINK_800: Self = Self::rgb(163, 0, 76);
    /// The `PINK_900` color token.
    pub const PINK_900: Self = Self::rgb(134, 16, 67);
    /// The `PINK_950` color token.
    pub const PINK_950: Self = Self::rgb(81, 4, 36);

    /* =Rose=-------------------------------------------------- */
    /// The `ROSE_50` color token.
    pub const ROSE_50: Self = Self::rgb(255, 241, 242);
    /// The `ROSE_100` color token.
    pub const ROSE_100: Self = Self::rgb(255, 228, 230);
    /// The `ROSE_200` color token.
    pub const ROSE_200: Self = Self::rgb(255, 204, 211);
    /// The `ROSE_300` color token.
    pub const ROSE_300: Self = Self::rgb(255, 161, 173);
    /// The `ROSE_400` color token.
    pub const ROSE_400: Self = Self::rgb(255, 99, 126);
    /// The `ROSE_500` color token.
    pub const ROSE_500: Self = Self::rgb(255, 32, 86);
    /// The `ROSE_600` color token.
    pub const ROSE_600: Self = Self::rgb(236, 0, 63);
    /// The `ROSE_700` color token.
    pub const ROSE_700: Self = Self::rgb(199, 0, 54);
    /// The `ROSE_800` color token.
    pub const ROSE_800: Self = Self::rgb(165, 0, 54);
    /// The `ROSE_900` color token.
    pub const ROSE_900: Self = Self::rgb(139, 8, 54);
    /// The `ROSE_950` color token.
    pub const ROSE_950: Self = Self::rgb(77, 2, 24);

    /* =Slate=-------------------------------------------------- */
    /// The `SLATE_50` color token.
    pub const SLATE_50: Self = Self::rgb(248, 250, 252);
    /// The `SLATE_100` color token.
    pub const SLATE_100: Self = Self::rgb(241, 245, 249);
    /// The `SLATE_200` color token.
    pub const SLATE_200: Self = Self::rgb(226, 232, 240);
    /// The `SLATE_300` color token.
    pub const SLATE_300: Self = Self::rgb(202, 213, 226);
    /// The `SLATE_400` color token.
    pub const SLATE_400: Self = Self::rgb(144, 161, 185);
    /// The `SLATE_500` color token.
    pub const SLATE_500: Self = Self::rgb(98, 116, 142);
    /// The `SLATE_600` color token.
    pub const SLATE_600: Self = Self::rgb(69, 85, 108);
    /// The `SLATE_700` color token.
    pub const SLATE_700: Self = Self::rgb(49, 65, 88);
    /// The `SLATE_800` color token.
    pub const SLATE_800: Self = Self::rgb(29, 41, 61);
    /// The `SLATE_900` color token.
    pub const SLATE_900: Self = Self::rgb(15, 23, 43);
    /// The `SLATE_950` color token.
    pub const SLATE_950: Self = Self::rgb(2, 6, 24);

    /* =Gray=-------------------------------------------------- */
    /// The `GRAY_50` color token.
    pub const GRAY_50: Self = Self::rgb(249, 250, 251);
    /// The `GRAY_100` color token.
    pub const GRAY_100: Self = Self::rgb(243, 244, 246);
    /// The `GRAY_200` color token.
    pub const GRAY_200: Self = Self::rgb(229, 231, 235);
    /// The `GRAY_300` color token.
    pub const GRAY_300: Self = Self::rgb(209, 213, 220);
    /// The `GRAY_400` color token.
    pub const GRAY_400: Self = Self::rgb(153, 161, 175);
    /// The `GRAY_500` color token.
    pub const GRAY_500: Self = Self::rgb(106, 114, 130);
    /// The `GRAY_600` color token.
    pub const GRAY_600: Self = Self::rgb(74, 85, 101);
    /// The `GRAY_700` color token.
    pub const GRAY_700: Self = Self::rgb(54, 65, 83);
    /// The `GRAY_800` color token.
    pub const GRAY_800: Self = Self::rgb(30, 41, 57);
    /// The `GRAY_900` color token.
    pub const GRAY_900: Self = Self::rgb(16, 24, 40);
    /// The `GRAY_950` color token.
    pub const GRAY_950: Self = Self::rgb(3, 7, 18);

    /* =Zinc=-------------------------------------------------- */
    /// The `ZINC_50` color token.
    pub const ZINC_50: Self = Self::rgb(250, 250, 250);
    /// The `ZINC_100` color token.
    pub const ZINC_100: Self = Self::rgb(244, 244, 245);
    /// The `ZINC_200` color token.
    pub const ZINC_200: Self = Self::rgb(228, 228, 231);
    /// The `ZINC_300` color token.
    pub const ZINC_300: Self = Self::rgb(212, 212, 216);
    /// The `ZINC_400` color token.
    pub const ZINC_400: Self = Self::rgb(159, 159, 169);
    /// The `ZINC_500` color token.
    pub const ZINC_500: Self = Self::rgb(113, 113, 123);
    /// The `ZINC_600` color token.
    pub const ZINC_600: Self = Self::rgb(82, 82, 92);
    /// The `ZINC_700` color token.
    pub const ZINC_700: Self = Self::rgb(63, 63, 70);
    /// The `ZINC_800` color token.
    pub const ZINC_800: Self = Self::rgb(39, 39, 42);
    /// The `ZINC_900` color token.
    pub const ZINC_900: Self = Self::rgb(24, 24, 27);
    /// The `ZINC_950` color token.
    pub const ZINC_950: Self = Self::rgb(9, 9, 11);

    /* =Neutral=-------------------------------------------------- */
    /// The `NEUTRAL_50` color token.
    pub const NEUTRAL_50: Self = Self::rgb(250, 250, 250);
    /// The `NEUTRAL_100` color token.
    pub const NEUTRAL_100: Self = Self::rgb(245, 245, 245);
    /// The `NEUTRAL_200` color token.
    pub const NEUTRAL_200: Self = Self::rgb(229, 229, 229);
    /// The `NEUTRAL_300` color token.
    pub const NEUTRAL_300: Self = Self::rgb(212, 212, 212);
    /// The `NEUTRAL_400` color token.
    pub const NEUTRAL_400: Self = Self::rgb(161, 161, 161);
    /// The `NEUTRAL_500` color token.
    pub const NEUTRAL_500: Self = Self::rgb(115, 115, 115);
    /// The `NEUTRAL_600` color token.
    pub const NEUTRAL_600: Self = Self::rgb(82, 82, 82);
    /// The `NEUTRAL_700` color token.
    pub const NEUTRAL_700: Self = Self::rgb(64, 64, 64);
    /// The `NEUTRAL_800` color token.
    pub const NEUTRAL_800: Self = Self::rgb(38, 38, 38);
    /// The `NEUTRAL_900` color token.
    pub const NEUTRAL_900: Self = Self::rgb(23, 23, 23);
    /// The `NEUTRAL_950` color token.
    pub const NEUTRAL_950: Self = Self::rgb(10, 10, 10);

    /* =Stone=-------------------------------------------------- */
    /// The `STONE_50` color token.
    pub const STONE_50: Self = Self::rgb(250, 250, 249);
    /// The `STONE_100` color token.
    pub const STONE_100: Self = Self::rgb(245, 245, 244);
    /// The `STONE_200` color token.
    pub const STONE_200: Self = Self::rgb(231, 229, 228);
    /// The `STONE_300` color token.
    pub const STONE_300: Self = Self::rgb(214, 211, 209);
    /// The `STONE_400` color token.
    pub const STONE_400: Self = Self::rgb(166, 160, 155);
    /// The `STONE_500` color token.
    pub const STONE_500: Self = Self::rgb(121, 113, 107);
    /// The `STONE_600` color token.
    pub const STONE_600: Self = Self::rgb(87, 83, 77);
    /// The `STONE_700` color token.
    pub const STONE_700: Self = Self::rgb(68, 64, 59);
    /// The `STONE_800` color token.
    pub const STONE_800: Self = Self::rgb(41, 37, 36);
    /// The `STONE_900` color token.
    pub const STONE_900: Self = Self::rgb(28, 25, 23);
    /// The `STONE_950` color token.
    pub const STONE_950: Self = Self::rgb(12, 10, 9);

    /* =Taupe=------------------------- */
    /// The `TAUPE_50` color token.
    pub const TAUPE_50: Self = Self::rgb(251, 250, 249);
    /// The `TAUPE_100` color token.
    pub const TAUPE_100: Self = Self::rgb(243, 241, 241);
    /// The `TAUPE_200` color token.
    pub const TAUPE_200: Self = Self::rgb(232, 228, 227);
    /// The `TAUPE_300` color token.
    pub const TAUPE_300: Self = Self::rgb(216, 210, 208);
    /// The `TAUPE_400` color token.
    pub const TAUPE_400: Self = Self::rgb(171, 160, 156);
    /// The `TAUPE_500` color token.
    pub const TAUPE_500: Self = Self::rgb(124, 109, 103);
    /// The `TAUPE_600` color token.
    pub const TAUPE_600: Self = Self::rgb(91, 79, 75);
    /// The `TAUPE_700` color token.
    pub const TAUPE_700: Self = Self::rgb(71, 60, 57);
    /// The `TAUPE_800` color token.
    pub const TAUPE_800: Self = Self::rgb(43, 36, 34);
    /// The `TAUPE_900` color token.
    pub const TAUPE_900: Self = Self::rgb(29, 24, 22);
    /// The `TAUPE_950` color token.
    pub const TAUPE_950: Self = Self::rgb(12, 10, 9);

    /* =Mauve=-------------------------------------------------- */
    /// The `MAUVE_50` color token.
    pub const MAUVE_50: Self = Self::rgb(250, 250, 250);
    /// The `MAUVE_100` color token.
    pub const MAUVE_100: Self = Self::rgb(243, 241, 243);
    /// The `MAUVE_200` color token.
    pub const MAUVE_200: Self = Self::rgb(231, 228, 231);
    /// The `MAUVE_300` color token.
    pub const MAUVE_300: Self = Self::rgb(215, 208, 215);
    /// The `MAUVE_400` color token.
    pub const MAUVE_400: Self = Self::rgb(168, 158, 169);
    /// The `MAUVE_500` color token.
    pub const MAUVE_500: Self = Self::rgb(121, 105, 123);
    /// The `MAUVE_600` color token.
    pub const MAUVE_600: Self = Self::rgb(89, 76, 91);
    /// The `MAUVE_700` color token.
    pub const MAUVE_700: Self = Self::rgb(70, 57, 71);
    /// The `MAUVE_800` color token.
    pub const MAUVE_800: Self = Self::rgb(42, 33, 44);
    /// The `MAUVE_900` color token.
    pub const MAUVE_900: Self = Self::rgb(29, 22, 30);
    /// The `MAUVE_950` color token.
    pub const MAUVE_950: Self = Self::rgb(12, 9, 12);

    /* =Mist=-------------------------------------------------- */
    /// The `MIST_50` color token.
    pub const MIST_50: Self = Self::rgb(249, 251, 251);
    /// The `MIST_100` color token.
    pub const MIST_100: Self = Self::rgb(241, 243, 243);
    /// The `MIST_200` color token.
    pub const MIST_200: Self = Self::rgb(232, 232, 227);
    /// The `MIST_300` color token.
    pub const MIST_300: Self = Self::rgb(216, 216, 208);
    /// The `MIST_400` color token.
    pub const MIST_400: Self = Self::rgb(156, 168, 171);
    /// The `MIST_500` color token.
    pub const MIST_500: Self = Self::rgb(103, 120, 124);
    /// The `MIST_600` color token.
    pub const MIST_600: Self = Self::rgb(75, 88, 91);
    /// The `MIST_700` color token.
    pub const MIST_700: Self = Self::rgb(57, 68, 71);
    /// The `MIST_800` color token.
    pub const MIST_800: Self = Self::rgb(34, 41, 43);
    /// The `MIST_900` color token.
    pub const MIST_900: Self = Self::rgb(22, 27, 29);
    /// The `MIST_950` color token.
    pub const MIST_950: Self = Self::rgb(9, 11, 12);

    /* =Olive=-------------------------------------------------- */
    /// The `OLIVE_50` color token.
    pub const OLIVE_50: Self = Self::rgb(251, 251, 249);
    /// The `OLIVE_100` color token.
    pub const OLIVE_100: Self = Self::rgb(244, 244, 240);
    /// The `OLIVE_200` color token.
    pub const OLIVE_200: Self = Self::rgb(232, 232, 227);
    /// The `OLIVE_300` color token.
    pub const OLIVE_300: Self = Self::rgb(216, 216, 208);
    /// The `OLIVE_400` color token.
    pub const OLIVE_400: Self = Self::rgb(171, 171, 156);
    /// The `OLIVE_500` color token.
    pub const OLIVE_500: Self = Self::rgb(124, 124, 103);
    /// The `OLIVE_600` color token.
    pub const OLIVE_600: Self = Self::rgb(91, 91, 75);
    /// The `OLIVE_700` color token.
    pub const OLIVE_700: Self = Self::rgb(71, 71, 57);
    /// The `OLIVE_800` color token.
    pub const OLIVE_800: Self = Self::rgb(43, 43, 34);
    /// The `OLIVE_900` color token.
    pub const OLIVE_900: Self = Self::rgb(29, 29, 22);
    /// The `OLIVE_950` color token.
    pub const OLIVE_950: Self = Self::rgb(12, 12, 9);
}

impl Default for Color {
    fn default() -> Self {
        Self::TRANSPARENT
    }
}
