// SPDX-License-Identifier: Apache-2.0
#[derive(Clone, Copy, Debug, Default, PartialEq)]
/// Data and behavior represented by `LayoutBox`.
pub struct LayoutBox {
    /// The `x` value carried by this type.
    pub x: f32,
    /// The `y` value carried by this type.
    pub y: f32,
    /// The `width` value carried by this type.
    pub width: f32,
    /// The `height` value carried by this type.
    pub height: f32,
}

impl LayoutBox {
    /// Returns whether the `contains_rounded` condition is satisfied.
    pub fn contains_rounded(&self, point: (f32, f32), radius: f32) -> bool {
        let (px, py) = point;

        if px < self.x || px > self.x + self.width || py < self.y || py > self.y + self.height {
            return false;
        }

        if radius <= 0.0 {
            return true;
        }

        let r = radius.min(self.width * 0.5).min(self.height * 0.5);

        if px >= self.x + r && px <= self.x + self.width - r {
            return true;
        }

        if py >= self.y + r && py <= self.y + self.height - r {
            return true;
        }

        let (cx, cy) = if px < self.x + r {
            if py < self.y + r {
                (self.x + r, self.y + r) // top-left
            } else {
                (self.x + r, self.y + self.height - r) // bottom-left
            }
        } else if py < self.y + r {
            (self.x + self.width - r, self.y + r) // top-right
        } else {
            (self.x + self.width - r, self.y + self.height - r) // bottom-right
        };

        let dx = px - cx;
        let dy = py - cy;

        dx * dx + dy * dy <= r * r
    }
}
