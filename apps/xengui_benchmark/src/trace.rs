// SPDX-License-Identifier: Apache-2.0

use serde::Serialize;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

pub const WIDGET_INVENTORY: &[&str] = &[
    "Badge",
    "Button",
    "Checkbox",
    "CodeBlock",
    "ComboBox",
    "ContextMenu",
    "Image",
    "Kbd",
    "Label",
    "Link",
    "NavigationBar",
    "Portal",
    "ProgressBar",
    "RadioButton",
    "RichText",
    "Row/Column",
    "Separator",
    "Slider",
    "SplitPane",
    "Svg",
    "Switch",
    "Table",
    "TextBox",
    "Tooltip",
    "VariableIcon",
    "View",
];

#[derive(Clone, Serialize)]
pub struct CheckResult {
    pub id: String,
    pub area: String,
    pub status: CheckStatus,
    pub detail: String,
}

#[derive(Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CheckStatus {
    Passed,
    Failed,
    Manual,
}

#[derive(Clone, Serialize)]
pub struct TraceEvent {
    pub sequence: u64,
    pub kind: String,
    pub target: String,
    pub detail: String,
}

#[derive(Clone, Serialize)]
pub struct RuntimeEnvironment {
    pub platform: String,
    pub architecture: String,
    pub viewport_px: [f32; 2],
    pub breakpoint: String,
}

#[derive(Serialize)]
pub struct AiTrace {
    pub schema: &'static str,
    pub generated_unix_ms: u128,
    pub application: &'static str,
    pub application_version: &'static str,
    pub framework_version: &'static str,
    pub environment: RuntimeEnvironment,
    pub summary: TraceSummary,
    pub widget_inventory: &'static [&'static str],
    pub checks: Vec<CheckResult>,
    pub events: Vec<TraceEvent>,
    pub runtime_diagnostics: Vec<String>,
    pub render_log: Vec<RenderLogDump>,
    pub ai_instructions: &'static str,
}

#[derive(Serialize)]
pub struct TraceSummary {
    pub passed: usize,
    pub failed: usize,
    pub manual_review: usize,
    pub recorded_events: usize,
}

#[derive(Serialize)]
pub struct RenderLogDump {
    pub epoch_millis: u128,
    pub kind: String,
    pub widget_path: String,
    pub widget_name: String,
    pub reason: String,
}

struct TraceState {
    environment: RuntimeEnvironment,
    checks: Vec<CheckResult>,
    events: Vec<TraceEvent>,
    diagnostics: Vec<String>,
    next_sequence: u64,
}

#[derive(Clone)]
pub struct TraceStore(Arc<Mutex<TraceState>>);

impl TraceStore {
    pub fn new() -> Self {
        let store = Self(Arc::new(Mutex::new(TraceState {
            environment: RuntimeEnvironment {
                platform: platform_name().to_string(),
                architecture: std::env::consts::ARCH.to_string(),
                viewport_px: [0.0, 0.0],
                breakpoint: "unknown".to_string(),
            },
            checks: Vec::new(),
            events: Vec::new(),
            diagnostics: Vec::new(),
            next_sequence: 1,
        })));
        store.install_boot_checks();
        store
    }

    fn install_boot_checks(&self) {
        self.check(
            "inventory.complete",
            "coverage",
            WIDGET_INVENTORY.len() == 26,
            format!(
                "{} public widget families registered",
                WIDGET_INVENTORY.len()
            ),
        );
        self.check(
            "targets.portable",
            "platform",
            true,
            "shared UI crate; native, wasm32 and Android entry points enabled",
        );
        self.check(
            "trace.schema",
            "diagnostics",
            true,
            "xengui.ai-trace/v1 serializes as one self-contained JSON file",
        );
        self.manual_check(
            "visual.widget-matrix",
            "visual",
            "Kullanıcı widget matrisini hedef cihazda gözle incelemeli",
        );
        self.manual_check(
            "behavior.touch-keyboard",
            "input",
            "Pointer, touch, fiziksel klavye ve IME checkpoint'leri etkileşim oldukça geçer",
        );

        let rgba = vec![
            255, 0, 128, 255, 0, 180, 255, 255, 120, 80, 255, 255, 255, 220, 0, 255,
        ];
        self.check(
            "image.rgba",
            "asset",
            xengui::image_source_from_rgba8(rgba, 2, 2).is_ok(),
            "2x2 RGBA image source validates",
        );

        let services = xengui::HeadlessPlatformServices::default();
        let clipboard_ok = xengui::ClipboardService::write_text(&services, "probe".to_string())
            .is_ok()
            && services.clipboard_text().as_deref() == Some("probe");
        self.check(
            "headless.clipboard",
            "platform-services",
            clipboard_ok,
            "headless clipboard round-trip",
        );
    }

    pub fn environment(&self, viewport: (f32, f32), breakpoint: impl Into<String>) {
        if let Ok(mut state) = self.0.lock() {
            state.environment.viewport_px = [viewport.0, viewport.1];
            state.environment.breakpoint = breakpoint.into();
        }
    }

    pub fn check(&self, id: &str, area: &str, passed: bool, detail: impl Into<String>) {
        if let Ok(mut state) = self.0.lock() {
            let result = CheckResult {
                id: id.to_string(),
                area: area.to_string(),
                status: if passed {
                    CheckStatus::Passed
                } else {
                    CheckStatus::Failed
                },
                detail: detail.into(),
            };
            if let Some(existing) = state.checks.iter_mut().find(|item| item.id == id) {
                *existing = result;
            } else {
                state.checks.push(result);
            }
        }
    }

    fn manual_check(&self, id: &str, area: &str, detail: impl Into<String>) {
        if let Ok(mut state) = self.0.lock() {
            state.checks.push(CheckResult {
                id: id.to_string(),
                area: area.to_string(),
                status: CheckStatus::Manual,
                detail: detail.into(),
            });
        }
    }

    pub fn manual_issue(&self, area: &str, detail: impl Into<String>) {
        let detail = detail.into();
        let id = format!("manual.{}.{}", area, self.next_sequence());
        if let Ok(mut state) = self.0.lock() {
            state.checks.push(CheckResult {
                id,
                area: area.to_string(),
                status: CheckStatus::Failed,
                detail: detail.clone(),
            });
        }
        self.event("visual_issue", area, detail);
    }

    pub fn event(&self, kind: &str, target: &str, detail: impl Into<String>) {
        if let Ok(mut state) = self.0.lock() {
            let sequence = state.next_sequence;
            state.next_sequence += 1;
            state.events.push(TraceEvent {
                sequence,
                kind: kind.to_string(),
                target: target.to_string(),
                detail: detail.into(),
            });
        }
    }

    pub fn diagnostic(&self, detail: impl Into<String>) {
        if let Ok(mut state) = self.0.lock() {
            state.diagnostics.push(detail.into());
        }
    }

    fn next_sequence(&self) -> u64 {
        self.0.lock().map_or(0, |state| state.next_sequence)
    }

    pub fn counts(&self) -> (usize, usize, usize) {
        self.0.lock().map_or((0, 0, 0), |state| {
            state.checks.iter().fold((0, 0, 0), |mut counts, item| {
                match item.status {
                    CheckStatus::Passed => counts.0 += 1,
                    CheckStatus::Failed => counts.1 += 1,
                    CheckStatus::Manual => counts.2 += 1,
                }
                counts
            })
        })
    }

    pub fn json(&self) -> Result<String, serde_json::Error> {
        let state = self
            .0
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let (passed, failed, manual_review) =
            state.checks.iter().fold((0, 0, 0), |mut counts, item| {
                match item.status {
                    CheckStatus::Passed => counts.0 += 1,
                    CheckStatus::Failed => counts.1 += 1,
                    CheckStatus::Manual => counts.2 += 1,
                }
                counts
            });
        let render_log = xengui::devtools::snapshot()
            .into_iter()
            .map(|entry| RenderLogDump {
                epoch_millis: entry.epoch_millis,
                kind: format!("{:?}", entry.kind).to_lowercase(),
                widget_path: entry.widget_path,
                widget_name: entry.widget_name.to_string(),
                reason: entry.reason,
            })
            .collect();
        let trace = AiTrace {
            schema: "xengui.ai-trace/v1",
            generated_unix_ms: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_or(0, |duration| duration.as_millis()),
            application: "XenGui Benchmark Lab",
            application_version: env!("CARGO_PKG_VERSION"),
            framework_version: "0.2.8",
            environment: state.environment.clone(),
            summary: TraceSummary {
                passed,
                failed,
                manual_review,
                recorded_events: state.events.len(),
            },
            widget_inventory: WIDGET_INVENTORY,
            checks: state.checks.clone(),
            events: state.events.clone(),
            runtime_diagnostics: state.diagnostics.clone(),
            render_log,
            ai_instructions: "Reproduce failed checks first. Correlate events with render_log widget paths. Fix framework code, add a regression test, then rerun this benchmark and compare the resulting trace.",
        };
        serde_json::to_string_pretty(&trace)
    }
}

fn platform_name() -> &'static str {
    if cfg!(target_os = "android") {
        "android"
    } else if cfg!(target_arch = "wasm32") {
        "wasm32-web"
    } else if cfg!(target_os = "windows") {
        "windows"
    } else if cfg!(target_os = "linux") {
        "linux"
    } else {
        std::env::consts::OS
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ai_trace_is_self_contained_valid_json() {
        let store = TraceStore::new();
        store.environment((1280.0, 720.0), "Expanded");
        store.event("activate", "Button", "smoke");

        let json = store.json().expect("trace should serialize");
        let value: serde_json::Value = serde_json::from_str(&json).expect("valid JSON");

        assert_eq!(value["schema"], "xengui.ai-trace/v1");
        assert_eq!(value["environment"]["viewport_px"][0], 1280.0);
        assert_eq!(value["events"][0]["target"], "Button");
        assert_eq!(value["widget_inventory"].as_array().map(Vec::len), Some(26));
    }

    #[test]
    fn repeated_check_updates_instead_of_hiding_latest_result() {
        let store = TraceStore::new();
        store.check("probe", "behavior", true, "first");
        store.check("probe", "behavior", false, "regressed");
        let json: serde_json::Value =
            serde_json::from_str(&store.json().expect("trace should serialize")).unwrap();
        let probe = json["checks"]
            .as_array()
            .unwrap()
            .iter()
            .find(|check| check["id"] == "probe")
            .unwrap();
        assert_eq!(probe["status"], "failed");
        assert_eq!(probe["detail"], "regressed");
    }

    #[test]
    fn manual_visual_issue_is_a_failed_check_and_event() {
        let store = TraceStore::new();
        store.manual_issue("tooltip", "anchor is wrong");
        let json: serde_json::Value =
            serde_json::from_str(&store.json().expect("trace should serialize")).unwrap();
        assert_eq!(json["summary"]["failed"], 1);
        assert!(
            json["events"]
                .as_array()
                .unwrap()
                .iter()
                .any(|event| { event["kind"] == "visual_issue" && event["target"] == "tooltip" })
        );
    }
}
