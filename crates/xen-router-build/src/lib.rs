// SPDX-License-Identifier: Apache-2.0
//! build.rs codegen for xen-router's file-based routing.
//!
//! Scans an `app/` directory using Next.js App Router conventions
//! (`page.rs`, `layout.rs`, `notfound.rs`, `[param]` and `[...rest]`
//! folders, `(group)` folders) and emits a Rust source file defining
//! `build_router() -> xen_router::Router`.
//!
//! Usage from a downstream crate's own `build.rs`:
//!
//! ```ignore
//! fn main() {
//!     xen_router_build::generate("app");
//! }
//! ```
//!
//! Then, in application code:
//!
//! ```ignore
//! include!(concat!(env!("OUT_DIR"), "/xen_router_generated.rs"));
//!
//! fn main() {
//!     let router = build_router();
//!     // ... router.build() inside App::render
//! }
//! ```

use std::env;
use std::fs;
use std::path::{ Path, PathBuf };

struct DirNode {
    name: String,
    page: Option<PathBuf>,
    layout: Option<PathBuf>,
    notfound: Option<PathBuf>,
    children: Vec<DirNode>,
}

enum Segment {
    Literal(String),
    Dynamic(String),
    CatchAll(String),
    // A `(name)` folder - contributes to layout nesting but is skipped
    // entirely in the URL pattern.
    Group,
}

fn classify(folder_name: &str) -> Segment {
    if folder_name.starts_with('(') && folder_name.ends_with(')') {
        return Segment::Group;
    }
    if folder_name.starts_with('[') && folder_name.ends_with(']') {
        let inner = &folder_name[1..folder_name.len() - 1];
        if let Some(rest) = inner.strip_prefix("...") {
            return Segment::CatchAll(rest.to_string());
        }
        return Segment::Dynamic(inner.to_string());
    }
    Segment::Literal(folder_name.to_string())
}

fn walk(dir: &Path, name: &str) -> DirNode {
    let mut node = DirNode {
        name: name.to_string(),
        page: None,
        layout: None,
        notfound: None,
        children: Vec::new(),
    };

    let Ok(entries) = fs::read_dir(dir) else {
        return node;
    };

    let mut entries: Vec<_> = entries.filter_map(Result::ok).collect();
    // Deterministic output regardless of the OS's own directory iteration order.
    entries.sort_by_key(|e| e.file_name());

    for entry in entries {
        let path = entry.path();
        if path.is_dir() {
            let child_name = entry.file_name().to_string_lossy().to_string();
            node.children.push(walk(&path, &child_name));
            continue;
        }

        match path.file_name().and_then(|n| n.to_str()) {
            Some("page.rs") => {
                node.page = Some(path);
            }
            Some("layout.rs") => {
                node.layout = Some(path);
            }
            Some("notfound.rs") => {
                node.notfound = Some(path);
            }
            _ => {}
        }
    }

    node
}

fn sanitize(path: &str) -> String {
    let mut out = String::new();
    let mut last_was_underscore = false;

    for c in path.chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c);
            last_was_underscore = false;
        } else if !last_was_underscore {
            out.push('_');
            last_was_underscore = true;
        }
    }

    out.trim_matches('_').to_string()
}

#[derive(Clone)]
struct LayoutRef {
    // Stable component() key: the layout's own directory path, so
    // sibling routes under the same layout share its hook state.
    key: String,
    module: String,
}

struct RouteEntry {
    pattern: String,
    page_module: String,
    layouts: Vec<LayoutRef>,
}

struct ModDecl {
    module: String,
    abs_path: PathBuf,
}

#[allow(clippy::too_many_arguments)]
fn collect(
    node: &DirNode,
    url_stack: &mut Vec<String>,
    layout_stack: &mut Vec<LayoutRef>,
    dir_path_stack: &mut Vec<String>,
    routes: &mut Vec<RouteEntry>,
    mods: &mut Vec<ModDecl>,
    root_notfound: &mut Option<String>,
    warnings: &mut Vec<String>
) {
    dir_path_stack.push(node.name.clone());
    let dir_path = dir_path_stack.join("/");

    if let Some(layout_path) = &node.layout {
        let module = format!("layout_{}", sanitize(&dir_path));
        mods.push(ModDecl { module: module.clone(), abs_path: layout_path.clone() });
        layout_stack.push(LayoutRef { key: format!("layout:{dir_path}"), module });
    }

    if let Some(notfound_path) = &node.notfound {
        let module = format!("notfound_{}", sanitize(&dir_path));
        mods.push(ModDecl { module: module.clone(), abs_path: notfound_path.clone() });
        if dir_path_stack.len() == 1 {
            *root_notfound = Some(module);
        } else {
            warnings.push(
                format!(
                    "xen-router-build: notfound.rs at '{dir_path}' is ignored - only the root app/notfound.rs is currently supported"
                )
            );
        }
    }

    if let Some(page_path) = &node.page {
        let module = format!("page_{}", sanitize(&dir_path));
        mods.push(ModDecl { module: module.clone(), abs_path: page_path.clone() });

        let pattern = if url_stack.is_empty() {
            "/".to_string()
        } else {
            format!("/{}", url_stack.join("/"))
        };

        routes.push(RouteEntry { pattern, page_module: module, layouts: layout_stack.clone() });
    }

    for child in &node.children {
        let segment = classify(&child.name);
        let pushed_url = match &segment {
            Segment::Literal(s) => {
                url_stack.push(s.clone());
                true
            }
            Segment::Dynamic(name) => {
                url_stack.push(format!(":{name}"));
                true
            }
            Segment::CatchAll(name) => {
                url_stack.push(format!("*{name}"));
                true
            }
            Segment::Group => false,
        };

        collect(
            child,
            url_stack,
            layout_stack,
            dir_path_stack,
            routes,
            mods,
            root_notfound,
            warnings
        );

        if pushed_url {
            url_stack.pop();
        }
    }

    if node.layout.is_some() {
        layout_stack.pop();
    }
    dir_path_stack.pop();
}

// Wraps the page in each of its layouts, innermost first, giving every
// layout level its own component() identity (keyed by that layout's
// directory, not by the full route pattern) so it keeps its hook state
// across sibling-route navigation, matching Next.js layout persistence.
fn emit_route_closure(route: &RouteEntry) -> String {
    let mut expr = format!("{}::page(&__params)", route.page_module);

    for layout in route.layouts.iter().rev() {
        expr = format!(
            "::xengui::component({key:?}, {{ let __params = __params.clone(); move || {module}::layout(&__params, {inner}) }})",
            key = layout.key,
            module = layout.module,
            inner = expr
        );
    }

    format!(
        "router = router.route({pattern:?}, move |__params: &::xen_router::RouteParams| {{ let __params = __params.clone(); {expr} }});\n",
        pattern = route.pattern
    )
}

/// Scans `app_dir` (relative to `CARGO_MANIFEST_DIR`) and writes
/// `xen_router_generated.rs` into `OUT_DIR`, defining
/// `pub fn build_router() -> ::xen_router::Router`.
///
/// Call this from the downstream crate's own `build.rs`.
pub fn generate(app_dir: &str) {
    let manifest_dir = env
        ::var("CARGO_MANIFEST_DIR")
        .expect("xen-router-build: CARGO_MANIFEST_DIR not set - call generate() from build.rs");
    let out_dir = env
        ::var("OUT_DIR")
        .expect("xen-router-build: OUT_DIR not set - call generate() from build.rs");

    let app_path = Path::new(&manifest_dir).join(app_dir);
    println!("cargo:rerun-if-changed={}", app_path.display());

    let root = walk(&app_path, app_dir);

    let mut url_stack = Vec::new();
    let mut layout_stack = Vec::new();
    let mut dir_path_stack = Vec::new();
    let mut routes = Vec::new();
    let mut mods = Vec::new();
    let mut root_notfound = None;
    let mut warnings = Vec::new();

    collect(
        &root,
        &mut url_stack,
        &mut layout_stack,
        &mut dir_path_stack,
        &mut routes,
        &mut mods,
        &mut root_notfound,
        &mut warnings
    );

    for warning in &warnings {
        println!("cargo:warning={warning}");
    }

    let mut out = String::new();
    out.push_str("// Generated by xen-router-build - do not edit by hand.\n\n");

    for m in &mods {
        println!("cargo:rerun-if-changed={}", m.abs_path.display());
        out.push_str(
            &format!("#[path = {:?}]\nmod {};\n", m.abs_path.display().to_string(), m.module)
        );
    }
    out.push('\n');

    out.push_str("pub fn build_router() -> ::xen_router::Router {\n");
    out.push_str("    let mut router = ::xen_router::Router::new();\n");
    for route in &routes {
        out.push_str("    ");
        out.push_str(&emit_route_closure(route));
    }
    if let Some(module) = &root_notfound {
        out.push_str(&format!("    router = router.not_found(|| {module}::not_found());\n"));
    }
    out.push_str("    router\n");
    out.push_str("}\n");

    let out_file = Path::new(&out_dir).join("xen_router_generated.rs");
    fs::write(&out_file, out).unwrap_or_else(|e|
        panic!("xen-router-build: failed to write {}: {e}", out_file.display())
    );
}
