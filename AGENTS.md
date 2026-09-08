## Material Design 3 / M3 Expressive

The Material Design 3 / M3 Expressive specification used by this project is located at:

`external/skills/m3-expressive/`

Before writing, modifying, or reviewing any Material Design UI code, read:

`external/skills/m3-expressive/SKILL.md`

Then follow its **"Where to look"** table to the appropriate reference file for the task.

This applies to, but is not limited to:

- Components
- Layout and spacing
- dp values
- Corner radii and shapes
- Motion and spring parameters
- Typography and type styles
- Color roles and theming
- Elevation
- Adaptive and responsive layouts
- Component variants and interaction behavior

### Specification lookup rules

Do not rely on memory for Material Design values, token names, component names, geometry, or behavior.

Always verify the relevant value or behavior in the local M3 specification before using it.

Use the following references:

- Exact token values such as type scale, radii, motion springs, and elevation:
  `external/skills/m3-expressive/references/tokens.md`

- Per-component dimensions and dp geometry:
  `external/skills/m3-expressive/references/component-tokens.md`

- Component behavior, variants, interaction rules, and do/don't guidance:
  `external/skills/m3-expressive/references/components/*.md`

- Platform API availability and platform-specific support:
  `external/skills/m3-expressive/references/platforms.md`

If the required value, token, behavior, or component rule is not defined by the M3 specification, do not present an invented value as part of Material Design.

Clearly label such choices as:

> my design decision (not in M3)

Project-specific design decisions are allowed when necessary, but they must remain distinguishable from specification-defined behavior.

### Implementation priority

When implementing or reviewing Material UI:

1. Read `SKILL.md`.
2. Locate the relevant specification reference.
3. Verify the exact component, token, geometry, behavior, or API.
4. Implement according to the specification.
5. Only introduce project-specific behavior when the specification does not define the requirement.
6. Explicitly identify any such deviation or extension as a project design decision.

Do not silently approximate specification values.

---

## Icons

Use the project's `xen-icons` crate as the primary icon source.

Prefer Material icons already available through `xen-icons`.

### Icon selection rules

1. Before introducing a new icon asset, check whether an appropriate Material icon already exists in `xen-icons`.
2. If it exists, use the `xen-icons` version.
3. Do not add arbitrary SVG files for icons that are already available through `xen-icons`.
4. If the required icon does not exist in `xen-icons`, do not automatically fall back to an unthemed, generic, or visually inconsistent SVG.
5. Any fallback icon must match the project's Material Design visual language and theming system.
6. Prefer adding the appropriate Material icon to `xen-icons` rather than embedding a one-off SVG directly into a component.

Avoid:

- Random third-party SVG icons
- Unthemed SVG assets
- Generic icons that conflict with Material icon geometry
- Component-local icon assets when the icon belongs in `xen-icons`
- Duplicating an icon already provided by `xen-icons`

Icons should remain consistent with the project's Material Design system in shape, sizing, weight, optical alignment, and theming.
