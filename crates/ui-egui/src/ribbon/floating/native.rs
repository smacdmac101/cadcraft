//! OS windows for sticky ribbon panels. Embedded hosts keep the existing egui window renderer.
use super::*;
use egui::{Pos2, Rect, Vec2, ViewportBuilder, ViewportCommand, ViewportId, WindowLevel};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Display {
    /// Physical desktop pixels, including negative monitor origins.
    pub rect: Rect,
    pub scale: f32,
}

#[derive(Clone, Default)]
struct Desktop {
    displays: Vec<Display>,
    windows: Vec<ViewportId>,
    minimized: bool,
    focused: bool,
}

#[derive(Clone, Default)]
struct Placement {
    initialized: bool,
    saved: Option<[f32; 2]>,
    displays: Vec<Display>,
    position_deadline: Option<f64>,
    target: Pos2,
    size: Option<[f32; 2]>,
    size_deadline: Option<f64>,
    rolled: bool,
}

fn desktop_id() -> egui::Id {
    egui::Id::new("ribbon-native-desktop")
}

pub fn sync_desktop(ctx: &egui::Context, displays: Vec<Display>) {
    let minimized = ctx.input(|i| i.raw.viewports.get(&ViewportId::ROOT).is_some_and(|v| v.minimized == Some(true)));
    let previous = ctx.data_mut(|d| d.get_temp::<Desktop>(desktop_id()).unwrap_or_default());
    let focused = ctx.input(|i| i.raw.viewports.values().any(|v| v.focused == Some(true)));
    if focused != previous.focused {
        for id in &previous.windows {
            ctx.send_viewport_cmd_to(*id, ViewportCommand::WindowLevel(if focused { WindowLevel::AlwaysOnTop } else { WindowLevel::Normal }));
        }
    }
    if minimized != previous.minimized {
        for id in &previous.windows {
            ctx.send_viewport_cmd_to(*id, ViewportCommand::Visible(!minimized));
        }
    }
    let mut displays: Vec<_> =
        displays.into_iter().filter(|d| d.rect.is_finite() && d.rect.is_positive() && d.scale.is_finite() && d.scale > 0.0).take(64).collect();
    displays.sort_by(|a, b| a.rect.min.x.total_cmp(&b.rect.min.x).then(a.rect.min.y.total_cmp(&b.rect.min.y)));
    ctx.data_mut(|d| d.insert_temp(desktop_id(), Desktop { displays, minimized, focused, ..previous }));
}

pub(in crate::ribbon) fn clear(ctx: &egui::Context) {
    ctx.data_mut(|d| d.get_temp_mut_or_default::<Desktop>(desktop_id()).windows.clear());
}

pub(in crate::ribbon) fn remember_windows(ctx: &egui::Context, windows: Vec<ViewportId>) {
    ctx.data_mut(|d| d.get_temp_mut_or_default::<Desktop>(desktop_id()).windows.extend(windows));
}

/// Recover a missing-monitor position, keeping the whole caption reachable. Do not run this
/// on every mouse move: users must be free to drag across gaps between displays.
#[cfg(test)]
fn recover(position: Pos2, displays: &[Display], root: Pos2, zoom: f32) -> Pos2 {
    recover_size(position, displays, root, zoom, vec2(220.0, 200.0))
}

fn recover_size(position: Pos2, displays: &[Display], root: Pos2, zoom: f32, window_size: Vec2) -> Pos2 {
    let contains = |rect: Rect, p: Pos2| p.x >= rect.min.x && p.x < rect.max.x && p.y >= rect.min.y && p.y < rect.max.y;
    let target = displays
        .iter()
        .find(|d| contains(d.rect, position))
        .or_else(|| displays.iter().find(|d| contains(d.rect, root)))
        .or_else(|| displays.first());
    let Some(display) = target else { return position };
    let size = window_size * display.scale * zoom;
    let max = (display.rect.max - size).max(display.rect.min);
    position.clamp(display.rect.min, max)
}

pub(super) fn show(
    app: &mut CadApp,
    ctx: &egui::Context,
    f: &FloatingPanel,
    tab: &Tab,
    panel: &Panel,
    clicked: &mut Option<&'static str>,
) -> ViewportId {
    let id = ViewportId::from_hash_of(("cadcraft-native-ribbon", tab.id, panel.id));
    let spec = WindowSpec {
        id,
        title: format!("CADCraft — {} / {}", crate::i18n::t(tab.label), crate::i18n::t(panel.label)),
        saved: f.desktop_position,
        offset: f.position,
        size: [204.0, 154.0],
        resizable: false,
        rolled: false,
    };
    let geometry = show_window(app, ctx, spec, clicked, |app, ui, clicked| body(app, ui, tab, panel, clicked));
    if geometry.closed {
        let _ = app.run("ui.ribbon.return", json!({"tab":tab.id,"panel":panel.id}));
    } else if let Some(position) = geometry.position
        && app.ui.ribbon.floating_panels.iter().any(|f| f.tab == tab.id && f.panel == panel.id)
    {
        let _ = app.run("ui.ribbon.float", json!({"tab":tab.id,"panel":panel.id,"desktopPosition":position}));
    }
    id
}

pub(in crate::ribbon) struct WindowSpec {
    pub id: ViewportId,
    pub title: String,
    pub saved: Option<[f32; 2]>,
    pub offset: [f32; 2],
    pub size: [f32; 2],
    pub resizable: bool,
    pub rolled: bool,
}

#[derive(Default)]
pub(in crate::ribbon) struct Geometry {
    pub closed: bool,
    pub position: Option<[f32; 2]>,
    pub size: Option<[f32; 2]>,
    pub user_moved: bool,
    pub ctrl: bool,
}

pub(in crate::ribbon) fn show_window(
    app: &mut CadApp,
    ctx: &egui::Context,
    spec: WindowSpec,
    clicked: &mut Option<&'static str>,
    mut render: impl FnMut(&mut CadApp, &mut egui::Ui, &mut Option<&'static str>),
) -> Geometry {
    let id = spec.id;
    let desktop = ctx.data_mut(|d| d.get_temp::<Desktop>(desktop_id()).unwrap_or_default());
    let root_scale = ctx.pixels_per_point();
    let root = ctx.input(|i| i.raw.viewports.get(&ViewportId::ROOT).and_then(|v| v.inner_rect).map_or(Pos2::ZERO, |r| r.min * root_scale));
    let exists = ctx.input(|i| i.raw.viewports.contains_key(&id));
    let state_id = egui::Id::new(("ribbon-native-placement", id));
    let mut placement = if exists { ctx.data_mut(|d| d.get_temp::<Placement>(state_id).unwrap_or_default()) } else { Placement::default() };
    let focused = ctx.input(|i| i.raw.viewports.values().any(|v| v.focused == Some(true)));
    let builder = ViewportBuilder::default()
        .with_title(spec.title)
        .with_inner_size(spec.size)
        .with_min_inner_size(if spec.rolled {
            [32.0, 32.0]
        } else if spec.resizable {
            [300.0, 220.0]
        } else {
            spec.size
        })
        .with_decorations(!spec.rolled)
        .with_minimize_button(false)
        .with_maximize_button(false)
        .with_resizable(spec.resizable)
        .with_taskbar(false)
        .with_active(false)
        .with_visible(!desktop.minimized)
        .with_window_level(if focused { WindowLevel::AlwaysOnTop } else { WindowLevel::Normal });
    let mut geometry = Geometry::default();
    ctx.show_viewport_immediate(id, builder, |ui, class| {
        if class == egui::ViewportClass::EmbeddedWindow {
            render(app, ui, clicked);
            return;
        }
        let child = ui.ctx().clone();
        if child.input(|i| i.viewport().close_requested()) {
            geometry.closed = true;
            child.send_viewport_cmd_to(ViewportId::ROOT, ViewportCommand::Focus);
            child.request_repaint_of(ViewportId::ROOT);
            return;
        }
        crate::menus::shortcuts(app, &child);
        crate::cmdline::keyboard(app, &child);
        let scale = child.pixels_per_point();
        let now = child.input(|i| i.time);
        geometry.ctrl = child.input(|i| i.modifiers.ctrl);
        if !placement.initialized || placement.saved != spec.saved || placement.displays != desktop.displays || placement.rolled != spec.rolled {
            let requested = spec.saved.map_or(root + vec2(spec.offset[0], spec.offset[1]) * root_scale, |p| Pos2::new(p[0], p[1]));
            let target = recover_size(requested, &desktop.displays, root, child.zoom_factor(), vec2(spec.size[0] + 16.0, spec.size[1] + 46.0));
            placement = Placement {
                initialized: true,
                saved: spec.saved,
                displays: desktop.displays.clone(),
                position_deadline: Some(now + 1.0),
                target,
                rolled: spec.rolled,
                ..Default::default()
            };
        }
        let observed = child.input(|i| i.viewport().outer_rect);
        let positioning = placement.position_deadline.is_some();
        if placement.position_deadline.is_some() {
            if observed.is_some_and(|r| (r.min * scale).distance(placement.target) <= 1.0)
                || placement.position_deadline.is_some_and(|deadline| now >= deadline)
            {
                placement.position_deadline = None;
            } else {
                child.send_viewport_cmd(ViewportCommand::OuterPosition(placement.target / scale));
                child.request_repaint();
            }
        }
        if placement.position_deadline.is_none()
            && let Some(rect) = observed
        {
            let pos = rect.min * scale;
            let position = [pos.x, pos.y];
            if spec.saved != Some(position) {
                geometry.position = Some(position);
                geometry.user_moved = !positioning;
                placement.saved = Some(position);
            }
        }
        if spec.resizable {
            if placement.size != Some(spec.size) {
                placement.size = Some(spec.size);
                placement.size_deadline = Some(now + 1.0);
            }
            let observed_size = child.input(|i| i.viewport().inner_rect).map(|r| r.size());
            let requested_size = vec2(spec.size[0], spec.size[1]);
            if placement.size_deadline.is_some() {
                if observed_size.is_some_and(|size| (size - requested_size).length() <= 1.0)
                    || placement.size_deadline.is_some_and(|deadline| now >= deadline)
                {
                    placement.size_deadline = None;
                } else {
                    // Discard/sizing passes can render repeatedly before the OS applies
                    // commands. Bound retries by elapsed UI time, not callback count.
                    child.send_viewport_cmd(ViewportCommand::InnerSize(requested_size));
                    child.request_repaint();
                }
            }
            if placement.size_deadline.is_none()
                && let Some(size) = observed_size
            {
                let size = [size.x, size.y];
                if size != spec.size {
                    geometry.size = Some(size);
                    placement.size = Some(size);
                }
            }
        }
        ui.painter().rect_filled(ui.max_rect(), 0.0, Tokens::get().chrome_dark);
        egui::Frame::new().fill(Tokens::get().chrome_dark).inner_margin(6.0).show(ui, |ui| render(app, ui, clicked));
        if clicked.is_some() {
            egui::Popup::close_all(&child);
            child.send_viewport_cmd_to(ViewportId::ROOT, ViewportCommand::Focus);
            child.request_repaint_of(ViewportId::ROOT);
        }
    });
    ctx.data_mut(|d| d.insert_temp(state_id, placement));
    geometry
}

fn body(app: &mut CadApp, ui: &mut egui::Ui, tab: &Tab, panel: &Panel, clicked: &mut Option<&'static str>) {
    ui.push_id(("native_ribbon_body", tab.id, panel.id), |ui| {
        let response = ui.button(crate::tl!("Return to Ribbon"));
        crate::control::record_widget(ui, &response, crate::tl!("Return to Ribbon"), "button", None);
        if response.clicked() {
            let _ = app.run("ui.ribbon.return", json!({"tab":tab.id,"panel":panel.id}));
            ui.ctx().send_viewport_cmd_to(ViewportId::ROOT, ViewportCommand::Focus);
            ui.ctx().request_repaint_of(ViewportId::ROOT);
        }
        show_panel(app, ui, tab.id, panel, true, clicked);
    });
}

pub fn inspect(ctx: &egui::Context) -> Value {
    let desktop = ctx.data_mut(|d| d.get_temp::<Desktop>(desktop_id()).unwrap_or_default());
    let embedded = ctx.embed_viewports();
    let zoom = ctx.zoom_factor();
    ctx.input(|i| json!({
        "embedded": embedded,
        "applicationFocused": desktop.focused,
        "displays": desktop.displays.iter().map(|d| json!({"rect":[d.rect.min.x,d.rect.min.y,d.rect.width(),d.rect.height()],"scale":d.scale})).collect::<Vec<_>>(),
        "windows": desktop.windows.iter().filter_map(|id| i.raw.viewports.get(id).map(|v| json!({
            "viewport":format!("{:016x}",id.0.value()), "title":v.title,
            "outerRect":v.outer_rect.map(|r| [r.min.x,r.min.y,r.width(),r.height()]),
            "physicalOuterRect":v.outer_rect.zip(v.native_pixels_per_point).map(|(r,p)| {let r=r*(p*zoom); [r.min.x,r.min.y,r.width(),r.height()]}),
            "nativePixelsPerPoint":v.native_pixels_per_point,"focused":v.focused,"minimized":v.minimized,
        }))).collect::<Vec<_>>()
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn recovery_keeps_secondary_negative_origins_and_handles_removed_displays() {
        let primary = Display { rect: Rect::from_min_size(Pos2::ZERO, vec2(1920.0, 1080.0)), scale: 1.5 };
        let secondary = Display { rect: Rect::from_min_size(egui::pos2(-2560.0, 0.0), vec2(2560.0, 1440.0)), scale: 1.0 };
        let pos = egui::pos2(-2200.0, 300.0);
        assert_eq!(recover(pos, &[primary, secondary], egui::pos2(100.0, 100.0), 1.0), pos);
        let right = Display { rect: Rect::from_min_size(egui::pos2(1920.0, 0.0), vec2(2560.0, 1440.0)), scale: 1.0 };
        assert_eq!(recover(egui::pos2(1920.0, 100.0), &[primary, right], Pos2::ZERO, 1.0), egui::pos2(1920.0, 100.0));
        let recovered = recover(pos, &[primary], egui::pos2(100.0, 100.0), 1.0);
        assert!(primary.rect.contains(recovered));
        assert!(primary.rect.contains(recovered + vec2(330.0, 300.0)));
    }
    #[test]
    fn recovery_keeps_caption_on_screen_at_mixed_dpi_and_allows_unknown_layouts() {
        let monitor = Display { rect: Rect::from_min_size(egui::pos2(1920.0, -200.0), vec2(3840.0, 2160.0)), scale: 2.0 };
        let pos = recover(egui::pos2(5700.0, 1900.0), &[monitor], Pos2::ZERO, 1.25);
        assert!(monitor.rect.contains(pos + vec2(550.0, 500.0)));
        assert_eq!(recover(egui::pos2(-500.0, 10.0), &[], Pos2::ZERO, 1.0), egui::pos2(-500.0, 10.0));
    }
}
