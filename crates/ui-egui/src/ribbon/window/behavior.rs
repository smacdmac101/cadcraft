//! Whole-ribbon placement and transient hover/drag state. No drawing mutation.
use super::*;
use egui::{Id, Pos2, Rect, pos2};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Edge {
    Left,
    Right,
}

#[derive(Clone, Default)]
struct Hover {
    open: bool,
    outside: Option<f64>,
}
#[derive(Clone, Default)]
struct Move {
    position: Option<[f32; 2]>,
    since: f64,
    pending: bool,
}
fn hover_id() -> Id {
    Id::new("whole-ribbon-hover")
}
fn blockers_id() -> Id {
    Id::new("whole-ribbon-blockers")
}
pub(crate) fn blockers(ctx: &egui::Context) -> Vec<Rect> {
    ctx.data_mut(|d| d.get_temp::<Vec<Rect>>(blockers_id()).unwrap_or_default())
}
pub(in crate::ribbon) fn clear(ctx: &egui::Context) {
    ctx.data_mut(|d| d.insert_temp(blockers_id(), Vec::<Rect>::new()));
}
pub(super) fn rolled(ctx: &egui::Context, enabled: bool) -> bool {
    enabled && !ctx.data_mut(|d| d.get_temp::<Hover>(hover_id()).unwrap_or_default()).open
}

fn hover(ctx: &egui::Context, inside: bool, enabled: bool) {
    let mut state = ctx.data_mut(|d| d.get_temp::<Hover>(hover_id()).unwrap_or_default());
    let previous = state.open;
    let busy = egui::Popup::is_any_open(ctx) || ctx.input(|i| i.pointer.any_down()) || ctx.egui_wants_keyboard_input();
    let now = ctx.input(|i| i.time);
    if inside || busy || !enabled {
        state.open = true;
        state.outside = None;
    } else if now - *state.outside.get_or_insert(now) >= 0.35 {
        state.open = false;
    } else {
        ctx.request_repaint_after(std::time::Duration::from_millis(100));
    }
    if state.open != previous {
        ctx.request_repaint();
        ctx.request_repaint_of(egui::ViewportId::ROOT);
    }
    ctx.data_mut(|d| d.insert_temp(hover_id(), state));
}

pub(in crate::ribbon) fn options(app: &mut CadApp, ui: &mut egui::Ui) {
    let state = app.ui.ribbon.window.clone();
    if state.floating || state.anchor.is_some() {
        let mut allow = state.allow_docking;
        if ui.checkbox(&mut allow, crate::tl!("Allow Docking")).changed() {
            let _ = app.run("ui.ribbon.window", json!({"allowDocking":allow}));
        }
        for (edge, label) in [(Edge::Left, crate::tl!("Anchor Left")), (Edge::Right, crate::tl!("Anchor Right"))] {
            if ui.selectable_label(state.anchor == Some(edge), label).clicked() {
                let _ = app.run("ui.ribbon.window", json!({"anchor":edge}));
                ui.close();
            }
        }
        let mut hide = state.auto_hide;
        if ui.checkbox(&mut hide, crate::tl!("Auto-hide")).changed() {
            let _ = app.run("ui.ribbon.window", json!({"autoHide":hide}));
            ui.close();
        }
        ui.separator();
    }
}

fn bar(app: &mut CadApp, ui: &mut egui::Ui, native: bool) -> egui::Response {
    let response = ui.allocate_exact_size(vec2(ui.available_width().min(28.0), ui.available_height().max(32.0)), Sense::click_and_drag()).1;
    let t = Tokens::get();
    ui.painter().rect_filled(response.rect, 0.0, if response.hovered() { t.control_hover } else { t.chrome });
    let galley = ui.painter().layout_no_wrap(crate::tl!("Ribbon").to_string(), crate::theme::body(), t.text);
    ui.painter().with_clip_rect(response.rect).add(
        egui::epaint::TextShape::new(response.rect.center() - galley.size() / 2.0, galley, t.text)
            .with_angle_and_anchor(std::f32::consts::FRAC_PI_2, egui::Align2::CENTER_CENTER),
    );
    response.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, ui.is_enabled(), crate::tl!("Ribbon Auto-hide")));
    crate::control::record_widget(ui, &response, crate::tl!("Ribbon Auto-hide"), "button", None);
    response.context_menu(|ui| customize(app, ui));
    if response.double_clicked() {
        dock(app, ui.ctx());
    }
    if native && response.drag_started() {
        ui.ctx().send_viewport_cmd(egui::ViewportCommand::StartDrag);
    }
    response
}

pub(super) fn floating_body(app: &mut CadApp, ui: &mut egui::Ui, clicked: &mut Option<&'static str>, collapsed: bool) {
    if collapsed {
        bar(app, ui, true);
    } else {
        body(app, ui, clicked);
    }
    let inside = ui.ctx().input(|i| i.pointer.hover_pos().is_some_and(|p| ui.max_rect().contains(p)));
    hover(ui.ctx(), inside, app.ui.ribbon.window.auto_hide);
}

pub(in crate::ribbon) fn anchored(app: &mut CadApp, ui: &mut egui::Ui, clicked: &mut Option<&'static str>) {
    let Some(edge) = app.ui.ribbon.window.anchor else { return };
    let ctx = ui.ctx().clone();
    let hide = app.ui.ribbon.window.auto_hide;
    let open = !rolled(&ctx, hide);
    let width = app.ui.ribbon.window.size[0].clamp(320.0, 560.0).min((ui.available_width() - 32.0).max(32.0));
    let panel = if edge == Edge::Left { egui::Panel::left("ribbon_anchor") } else { egui::Panel::right("ribbon_anchor") };
    let mut rects = Vec::new();
    let rail = panel
        .exact_size(if hide { 32.0 } else { width })
        .resizable(false)
        .frame(egui::Frame::NONE.fill(Tokens::get().chrome_dark))
        .show(ui, |ui| {
            if hide {
                bar(app, ui, false);
            } else {
                anchor_body(app, ui, clicked);
            }
        })
        .response
        .rect;
    rects.push(rail);
    if hide && open {
        let x = if edge == Edge::Left { rail.right() } else { rail.left() - width };
        let area = egui::Area::new(Id::new("ribbon_anchor_flyout"))
            .order(egui::Order::Foreground)
            .fixed_pos(pos2(x, rail.top()))
            .movable(false)
            .show(&ctx, |ui| {
                egui::Frame::NONE.fill(Tokens::get().chrome_dark).inner_margin(6.0).show(ui, |ui| {
                    ui.set_min_size(vec2((width - 12.0).max(20.0), (rail.height() - 12.0).max(20.0)));
                    ui.set_max_size(ui.min_size());
                    anchor_body(app, ui, clicked);
                });
            });
        rects.push(area.response.rect);
    }
    let inside = ctx.input(|i| i.pointer.hover_pos().is_some_and(|p| rects.iter().any(|r| r.expand(3.0).contains(p))));
    hover(&ctx, inside, hide);
    ctx.data_mut(|d| d.insert_temp(blockers_id(), rects));
}
fn anchor_body(app: &mut CadApp, ui: &mut egui::Ui, clicked: &mut Option<&'static str>) {
    ui.horizontal(|ui| {
        if ui.small_button(if app.ui.ribbon.window.auto_hide { crate::tl!("Keep Open") } else { crate::tl!("Auto-hide") }).clicked() {
            let _ = app.run("ui.ribbon.window", json!({"autoHide":!app.ui.ribbon.window.auto_hide}));
        }
        let grip = ui.add(egui::Label::new(crate::tl!("Undock Ribbon")).sense(Sense::click_and_drag()));
        if grip.clicked() || grip.drag_started() {
            let scale = ui.ctx().pixels_per_point();
            let position = ui.ctx().input(|i| {
                i.raw.viewports.get(&egui::ViewportId::ROOT).and_then(|v| v.inner_rect).zip(i.pointer.hover_pos()).map(|(root, p)| {
                    let p = (root.min + p.to_vec2()) * scale;
                    [p.x, p.y]
                })
            });
            let _ = app.run("ui.ribbon.window", json!({"floating":true,"desktopPosition":position}));
        }
    });
    body(app, ui, clicked);
}

/// Snap only after an observed user move settles. Saved/programmatic placement must not dock.
pub(super) fn edge_docking(app: &mut CadApp, ctx: &egui::Context, geometry: &native::Geometry) {
    let id = Id::new("whole-ribbon-edge-move");
    let mut movement = ctx.data_mut(|d| d.get_temp::<Move>(id).unwrap_or_default());
    let now = ctx.input(|i| i.time);
    if geometry.user_moved
        && let Some(p) = geometry.position
    {
        movement = Move { position: Some(p), since: now, pending: true };
    }
    if movement.pending {
        let root = ctx.input(|i| i.raw.viewports.get(&egui::ViewportId::ROOT).and_then(|v| v.inner_rect));
        let ctrl = geometry.ctrl || ctx.input(|i| i.modifiers.ctrl);
        if !app.ui.ribbon.window.allow_docking || ctrl {
            movement.pending = false;
        } else if let (Some(root), Some(p)) = (root, movement.position) {
            let scale = ctx.pixels_per_point();
            let root = root * scale;
            if let Some(target) = target(Pos2::new(p[0], p[1]), root, 28.0 * scale) {
                let preview = match target {
                    None => Rect::from_min_size(root.min / scale, vec2(root.width() / scale, 160.0)),
                    Some(Edge::Left) => Rect::from_min_size(root.min / scale, vec2(32.0, root.height() / scale)),
                    Some(Edge::Right) => {
                        Rect::from_min_size(pos2(root.right() / scale - 32.0, root.top() / scale), vec2(32.0, root.height() / scale))
                    }
                };
                let local = preview.translate(-root.min.to_vec2() / scale);
                ctx.layer_painter(egui::LayerId::new(egui::Order::Tooltip, id)).rect_filled(local, 0.0, Tokens::get().accent.gamma_multiply(0.35));
                if now - movement.since >= 0.3 {
                    let _ = app.run("ui.ribbon.window", json!({"floating":false,"anchor":target,"autoHide":target.is_some()}));
                    movement.pending = false;
                    ctx.request_repaint();
                }
            } else {
                movement.pending = false;
            }
        }
        ctx.request_repaint_after(std::time::Duration::from_millis(100));
    }
    ctx.data_mut(|d| d.insert_temp(id, movement));
}
fn target(p: Pos2, root: Rect, margin: f32) -> Option<Option<Edge>> {
    if !root.expand(margin).contains(p) {
        return None;
    }
    if (p.y - root.top()).abs() <= margin {
        Some(None)
    } else if (p.x - root.left()).abs() <= margin {
        Some(Some(Edge::Left))
    } else if (p.x - root.right()).abs() <= margin {
        Some(Some(Edge::Right))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn edge_targets_use_application_bounds_not_monitor_edges() {
        let root = Rect::from_min_size(pos2(-1500.0, 200.0), vec2(1200.0, 800.0));
        assert_eq!(target(pos2(-800.0, 205.0), root, 28.0), Some(None));
        assert_eq!(target(pos2(-1490.0, 500.0), root, 28.0), Some(Some(Edge::Left)));
        assert_eq!(target(pos2(-310.0, 500.0), root, 28.0), Some(Some(Edge::Right)));
        assert_eq!(target(pos2(0.0, 500.0), root, 28.0), None);
        assert_eq!(target(pos2(-800.0, 500.0), root, 28.0), None);
    }
}
