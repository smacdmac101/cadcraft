//! Detached ribbon panels retain their source identity and original return position.
use super::*;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FloatingPanel {
    pub tab: String,
    pub panel: String,
    pub position: [f32; 2],
    /// Native outer-window position in physical desktop pixels; independent of monitor DPI.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub desktop_position: Option<[f32; 2]>,
}

pub(super) fn valid(panels: &[FloatingPanel]) -> bool {
    panels.len() <= 64
        && panels.iter().enumerate().all(|(i, p)| {
            tab(&p.tab).is_some_and(|t| t.panels.iter().any(|a| a.id == p.panel))
                && p.position.iter().all(|v| v.is_finite() && v.abs() <= 100_000.0)
                && p.desktop_position.is_none_or(|p| p.iter().all(|v| v.is_finite() && v.abs() <= 1_000_000.0))
                && !panels[..i].iter().any(|a| a.tab == p.tab && a.panel == p.panel)
        })
}

pub(super) fn configure(app: &mut CadApp, id: &str, p: &Value) -> Result<Value, String> {
    let mut next = app.ui.ribbon.clone();
    if id == "ui.ribbon.return" && bool_param(p, "all", false)? {
        next.floating_panels.clear();
    } else {
        let t = match p.get("tab") {
            Some(v) => v.as_str().and_then(tab).ok_or("Unknown ribbon tab")?,
            None => context::active(app).ok_or("Select a ribbon tab first")?,
        };
        let panel = p.get("panel").and_then(Value::as_str).and_then(|id| t.panels.iter().find(|a| a.id == id)).ok_or("Unknown ribbon panel")?;
        let existing = next.floating_panels.iter().find(|a| a.tab == t.id && a.panel == panel.id);
        if id == "ui.ribbon.float" {
            let position = match p.get("position") {
                Some(v) => serde_json::from_value::<[f32; 2]>(v.clone()).map_err(|_| "Invalid floating panel position")?,
                None => existing.map_or([80.0, 260.0], |p| p.position),
            };
            let desktop_position = match p.get("desktopPosition") {
                Some(Value::Null) => None,
                Some(v) => Some(serde_json::from_value::<[f32; 2]>(v.clone()).map_err(|_| "Invalid desktop panel position")?),
                None if p.get("position").is_some() => None,
                None => existing.and_then(|p| p.desktop_position),
            };
            next.floating_panels.retain(|a| a.tab != t.id || a.panel != panel.id);
            next.floating_panels.push(FloatingPanel { tab: t.id.into(), panel: panel.id.into(), position, desktop_position });
            visibility(&mut next.hidden_panels, &format!("{}.{}", t.id, panel.id), true);
        } else {
            next.floating_panels.retain(|a| a.tab != t.id || a.panel != panel.id);
        }
    }
    next.validate()?;
    app.ui.ribbon = next;
    Ok(json!({"ribbon":app.ui.ribbon}))
}

pub(super) fn show(app: &mut CadApp, ctx: &egui::Context, clicked: &mut Option<&'static str>) {
    let mut native_ids = Vec::new();
    for f in app.ui.ribbon.floating_panels.clone() {
        let Some(tab) = tab(&f.tab) else { continue };
        let Some(panel) = tab.panels.iter().find(|p| p.id == f.panel) else { continue };
        if !app.ui.ribbon.panel_visible(tab.id, panel.id) {
            continue;
        }
        if context::TABS.iter().any(|t| t.id == tab.id) && context::available(app).is_none_or(|t| t.id != tab.id) {
            continue;
        }
        if !ctx.embed_viewports() {
            let id = native::show(app, ctx, &f, tab, panel, clicked);
            native_ids.push(id);
            continue;
        }
        let mut open = true;
        let response = egui::Window::new(format!("{} / {}", crate::i18n::t(tab.label), crate::i18n::t(panel.label)))
            .id(egui::Id::new(("floating_ribbon_panel", tab.id, panel.id)))
            .current_pos(egui::pos2(f.position[0], f.position[1]))
            .open(&mut open)
            .collapsible(false)
            .auto_sized()
            .constrain(true)
            .show(ctx, |ui| {
                if ui.button(crate::tl!("Return to Ribbon")).clicked() {
                    let _ = app.run("ui.ribbon.return", json!({"tab":tab.id,"panel":panel.id}));
                }
                show_panel(app, ui, tab.id, panel, true, clicked);
            });
        if !open {
            // Closing a detached panel returns it rather than losing its tools.
            let _ = app.run("ui.ribbon.return", json!({"tab":tab.id,"panel":panel.id}));
        } else if let Some(response) = response {
            let pos = response.response.rect.min;
            if f.position != [pos.x, pos.y] && app.ui.ribbon.floating_panels.iter().any(|a| a.tab == f.tab && a.panel == f.panel) {
                let _ =
                    app.run("ui.ribbon.float", json!({"tab":tab.id,"panel":panel.id,"position":[pos.x,pos.y],"desktopPosition":f.desktop_position}));
            }
        }
    }
    native::remember_windows(ctx, native_ids);
}

pub(super) mod native;
pub use native::{Display, inspect, sync_desktop};

pub(super) fn clear(ctx: &egui::Context) {
    native::clear(ctx);
}
