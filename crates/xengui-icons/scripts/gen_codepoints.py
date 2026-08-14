#!/usr/bin/env python3
"""Regenerates xengui-icons's codepoints module from Google's Material
Symbols .codepoints file. Names not valid as Rust identifiers (starting
with a digit, or matching a Rust keyword) get an underscore prefix.

Run from the crate root: python3 scripts/gen_codepoints.py
"""

import os
import urllib.request

SOURCE_URL = (
    "https://raw.githubusercontent.com/google/material-design-icons/"
    "master/variablefont/MaterialSymbolsOutlined%5BFILL%2CGRAD%2Copsz%2Cwght%5D.codepoints"
)

# This script lives in <crate>/scripts/, output goes to <crate>/src/...
CRATE_ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
OUTPUT_PATH = os.path.join(CRATE_ROOT, "src", "material_symbols", "codepoints.rs")

RUST_KEYWORDS = {
    "as",
    "break",
    "const",
    "continue",
    "crate",
    "dyn",
    "else",
    "enum",
    "extern",
    "false",
    "fn",
    "for",
    "if",
    "impl",
    "in",
    "let",
    "loop",
    "match",
    "mod",
    "move",
    "mut",
    "pub",
    "ref",
    "return",
    "self",
    "Self",
    "static",
    "struct",
    "super",
    "trait",
    "true",
    "type",
    "unsafe",
    "use",
    "where",
    "while",
    "async",
    "await",
    "try",
}

# Extra alias names not present under that spelling in Google's list,
# mapped to an existing const name they should point at.
ALIASES = {
    "PLUS": "ADD",
    "MINUS": "REMOVE",
}


def to_rust_const_name(icon_name: str) -> str:
    name = icon_name.upper()
    if name[0].isdigit():
        name = f"_{name}"
    if name.lower() in RUST_KEYWORDS:
        name = f"{name}_"
    return name


def main():
    with urllib.request.urlopen(SOURCE_URL) as resp:
        lines = resp.read().decode("utf-8").splitlines()

    entries = []
    seen_names = set()
    for line in lines:
        line = line.strip()
        if not line:
            continue
        icon_name, hex_code = line.split()
        const_name = to_rust_const_name(icon_name)
        if const_name in seen_names:
            continue
        seen_names.add(const_name)
        entries.append((const_name, hex_code))

    entries.sort()
    entries_by_name = dict(entries)

    os.makedirs(os.path.dirname(OUTPUT_PATH), exist_ok=True)

    with open(OUTPUT_PATH, "w") as f:
        f.write("// SPDX-License-Identifier: Apache-2.0\n")
        f.write(
            "//! Auto-generated from Google's Material Symbols .codepoints file.\n"
            "//! Run `python3 scripts/gen_codepoints.py` from the crate root to regenerate.\n"
        )
        for const_name, hex_code in entries:
            f.write(f"pub const {const_name}: char = '\\u{{{hex_code}}}';\n")
        for alias_name, target_name in sorted(ALIASES.items()):
            if target_name in entries_by_name:
                f.write(f"pub const {alias_name}: char = {target_name};\n")
        f.write("\n")

    print(f"wrote {len(entries)} codepoints (+{len(ALIASES)} aliases) to {OUTPUT_PATH}")


if __name__ == "__main__":
    main()
