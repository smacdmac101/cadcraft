//! Hover palettes are a view of the saved layout, never a mutation of the drawing or placement.
use super::*;
use egui::{Align2, Id, Order, Rect, Sense, pos2, vec2};

#[derive(Clone, Default)]
struct Hover {
    open: Option<Panel>,
    outside_since: Option<f64>,
    explicitly_opened: bool,
}

fn contains(node: &Node<Panel>, panel: Panel) -> bool {
    match node {
        Node::Tabs { panels, .. } => panels.contains(&panel),
        Node::Stack { entries } => entries.iter().any(|entry| entry.panel == panel),
        Node::Split { first, second, .. } => contains(first, panel) || contains(second, panel),
    }
}

// Follow the saved split tree rather than a palette's default side. Stacked/tabbed palettes
// inherit the outer side and width, including after the user moves the whole group.
fn side_and_width(workspace: &Workspace, panel: Panel) -> (Zone, f32) {
    let mut side = panel.default_side();
    let mut width = if panel == Panel::ToolSets { 220.0 } else { 300.0 };
    let mut node = workspace.layout.root.as_ref();
    while let Some(Node::Split { axis, size, first, second }) = node {
        let in_first = contains(first, panel);
        if *axis == SplitAxis::Horizontal {
            if in_first && contains(second, Panel::Canvas) {
                side = Zone::Left;
            } else if !in_first && contains(first, Panel::Canvas) {
                side = Zone::Right;
            }
            match (in_first, size) {
                (true, SplitSize::FixedFirst(value)) | (false, SplitSize::FixedSecond(value)) => width = *value,
                _ => {}
            }
        }
        node = Some(if in_first { first } else { second });
    }
    let limits = panel_limits(&panel);
    (side, width.clamp(limits.min.x, limits.max.x))
}

fn title_menu(content: &mut Content<'_>, ui: &mut egui::Ui, panel: Panel, rect: Rect) {
    content.panel_menu(ui, &panel);
    if ui.button(crate::tl!("Float panel")).clicked() {
        content.queued.push(("ui.dock", json!({"operation":"float","panel":panel.id(),"rect":[rect.left(),rect.top(),rect.width(),rect.height()]})));
        ui.close();
    }
    if ui.button(crate::tl!("Close panel")).clicked() {
        content.queued.push(("ui.dock", json!({"operation":"close","panel":panel.id()})));
        ui.close();
    }
}

fn tab_button(ui: &mut egui::Ui, panel: Panel, tab: Rect, floating: bool, open: bool) -> egui::Response {
    let t = crate::theme::Tokens::get();
    let label = crate::i18n::t(panel.label());
    let galley = ui.painter().layout_no_wrap(label.to_string(), crate::theme::body(), t.text);
    let response = if floating {
        ui.allocate_exact_size(tab.size(), Sense::click()).1
    } else {
        ui.interact(tab, Id::new("cadcraft-auto-hide").with(panel), Sense::click())
    };
    let tab = response.rect;
    crate::icons::describe_control(ui, &response, &format!("{}: {label}", crate::tl!("Auto-hide")), None, false);
    ui.painter().rect_filled(tab, 2.0, if response.hovered() || open { t.control_hover } else { t.chrome });
    if floating {
        ui.painter().with_clip_rect(tab).galley(tab.left_center() + vec2(8.0, -galley.size().y / 2.0), galley, t.text);
    } else {
        ui.painter().with_clip_rect(tab).add(
            egui::epaint::TextShape::new(tab.center() - galley.size() / 2.0, galley, t.text)
                .with_angle_and_anchor(std::f32::consts::FRAC_PI_2, Align2::CENTER_CENTER),
        );
    }
    response.on_hover_text(label)
}

pub(super) fn show(content: &mut Content<'_>, ui: &mut egui::Ui, workspace: &Workspace, viewport: Rect, reveal: Option<Panel>) -> (Rect, Vec<Rect>) {
    if !viewport.is_finite() || viewport.width() < 60.0 || viewport.height() < 30.0 {
        return (viewport, Vec::new());
    }
    let ctx = ui.ctx().clone();
    let id = Id::new("cadcraft-auto-hide");
    let mut hover = ctx.data_mut(|data| data.get_temp::<Hover>(id).unwrap_or_default());
    let panels: Vec<_> = workspace.auto_hide.iter().copied().filter(|panel| workspace.layout.contains(panel)).collect();
    if !panels.iter().any(|panel| Some(*panel) == hover.open) {
        hover = Hover::default();
    }
    if let Some(panel) = reveal.filter(|panel| panels.contains(panel)) {
        hover.open = Some(panel);
        hover.explicitly_opened = true;
        hover.outside_since = None;
    }
    let t = crate::theme::Tokens::get();
    let mut left_y = viewport.top() + 4.0;
    let mut right_y = left_y;
    let mut has_left = false;
    let mut has_right = false;
    let mut active_tab = None;
    let mut blockers = Vec::new();
    let side_count = |side| {
        panels
            .iter()
            .filter(|panel| {
                !workspace.layout.floating.iter().any(|group| group.panels.contains(panel)) && side_and_width(workspace, **panel).0 == side
            })
            .count()
            .max(1) as f32
    };
    for panel in &panels {
        let floating = workspace.layout.floating.iter().find(|group| group.panels.contains(panel)).map(|group| group.rect);
        let (side, width) = side_and_width(workspace, *panel);
        let label = crate::i18n::t(panel.label());
        let galley = ui.painter().layout_no_wrap(label.to_string(), crate::theme::body(), t.text);
        let (tab, flyout) = if let Some([x, y, width, height]) = floating {
            let rect = Rect::from_min_size(pos2(x, y), vec2(width.min(viewport.width()), height.min(viewport.height())));
            let rect = rect.translate(vec2(
                (viewport.left() - rect.left()).max(0.0) - (rect.right() - viewport.right()).max(0.0),
                (viewport.top() - rect.top()).max(0.0) - (rect.bottom() - viewport.bottom()).max(0.0),
            ));
            (Rect::from_min_size(rect.min, vec2(rect.width(), 26.0)), rect)
        } else {
            let length = (galley.size().x + 24.0).clamp(60.0, 180.0).min(((viewport.height() - 4.0) / side_count(side) - 4.0).max(16.0));
            let (x, y) = if side == Zone::Left {
                has_left = true;
                let y = left_y;
                left_y += length + 4.0;
                (viewport.left(), y)
            } else {
                has_right = true;
                let y = right_y;
                right_y += length + 4.0;
                (viewport.right() - 28.0, y)
            };
            let tab = Rect::from_min_size(pos2(x, y), vec2(28.0, length)).intersect(viewport);
            let width = width.min((viewport.width() - 56.0).max(1.0));
            let x = if side == Zone::Left { viewport.left() + 28.0 } else { viewport.right() - 28.0 - width };
            (tab, Rect::from_min_size(pos2(x, viewport.top()), vec2(width, viewport.height())))
        };
        blockers.push(tab);
        let response = if floating.is_some() && hover.open == Some(*panel) {
            // The expanded title replaces the collapsed bar, including its input layer.
            // Keeping both areas would cover the pin control after a dock/float move.
            ui.interact(tab, id.with(("open-title", panel)), Sense::hover())
        } else if floating.is_some() {
            // A floating bar needs its own layer; a same-layer canvas rendered afterwards
            // would otherwise consume hover and clicks at the same coordinates.
            egui::Area::new(id.with(("bar", panel)))
                .order(Order::Foreground)
                .fixed_pos(tab.min)
                .movable(false)
                .constrain_to(viewport)
                .show(&ctx, |ui| tab_button(ui, *panel, tab, true, hover.open == Some(*panel)))
                .inner
        } else {
            tab_button(ui, *panel, tab, false, hover.open == Some(*panel))
        };
        response.context_menu(|ui| title_menu(content, ui, *panel, flyout));
        if response.hovered() || response.clicked() || response.secondary_clicked() {
            hover.open = Some(*panel);
            hover.outside_since = None;
        }
        if hover.open == Some(*panel) {
            active_tab = Some((*panel, tab, flyout));
        }
    }
    let mut canvas_rect = viewport;
    if has_left {
        canvas_rect.min.x += 28.0;
    }
    if has_right {
        canvas_rect.max.x -= 28.0;
    }
    // Reserve only the narrow rails; hovering overlays the canvas without changing its zoom.
    if let Some((panel, tab, flyout)) = active_tab {
        blockers.push(flyout);
        let area =
            egui::Area::new(id.with(("flyout", panel))).order(Order::Foreground).fixed_pos(flyout.min).movable(false).constrain_to(viewport).show(
                &ctx,
                |ui| {
                    egui::Frame::NONE.fill(t.panel).stroke(egui::Stroke::new(1.0, t.border)).show(ui, |ui| {
                        ui.set_min_size(flyout.size() - vec2(2.0, 2.0));
                        ui.set_max_size(flyout.size() - vec2(2.0, 2.0));
                        let header = ui
                            .horizontal(|ui| {
                                ui.label(crate::i18n::t(panel.label()));
                                if ui.small_button(crate::tl!("Keep Open")).clicked() {
                                    content.queued.push(("ui.dock", json!({"operation":"autoHide","panel":panel.id(),"enabled":false})));
                                }
                            })
                            .response;
                        header.context_menu(|ui| title_menu(content, ui, panel, flyout));
                        ui.separator();
                        let body = ui.available_rect_before_wrap();
                        ui.scope_builder(egui::UiBuilder::new().id_salt(("palette", panel)).max_rect(body), |ui| content.body(ui, &panel));
                    });
                },
            );
        let pointer = ctx.input(|input| input.pointer.hover_pos());
        let inside = pointer.is_some_and(|p| tab.union(area.response.rect).expand(3.0).contains(p));
        if inside || ctx.input(|input| input.pointer.any_pressed()) {
            hover.explicitly_opened = false;
        }
        let focused_here = ctx.memory(|memory| memory.focused()).and_then(|focused| ctx.read_response(focused)).is_some_and(|response| {
            response.sense.senses_drag() && response.layer_id == area.response.layer_id && area.response.rect.intersects(response.rect)
        });
        let busy = egui::Popup::is_any_open(&ctx) || ctx.input(|input| input.pointer.any_down()) || focused_here || hover.explicitly_opened;
        let now = ctx.input(|input| input.time);
        if inside || busy {
            hover.outside_since = None;
        } else if now - *hover.outside_since.get_or_insert(now) >= 0.35 {
            hover = Hover::default();
            // This frame already painted the flyout. Native event-driven rendering needs
            // one more frame to remove it even when the pointer has stopped moving.
            ctx.request_repaint();
        } else {
            ctx.request_repaint_after(std::time::Duration::from_millis(100));
        }
        if ctx.input(|input| input.key_pressed(egui::Key::Escape) || (!inside && input.pointer.any_pressed())) && !egui::Popup::is_any_open(&ctx) {
            hover = Hover::default();
            ctx.request_repaint();
        }
    }
    ctx.data_mut(|data| data.insert_temp(id, hover));
    (canvas_rect, blockers)
}

/// Filtering hidden palettes can remove split ancestors. Map divider gestures back to
/// the saved tree by the surviving panel identities on either side of that divider.
pub(super) fn canonical_action(action: Action<Panel>, visible: &Layout<Panel>, saved: &Layout<Panel>) -> Option<Action<Panel>> {
    let Action::ResizeSplit { path, size } = action else {
        return Some(action);
    };
    let mut node = visible.root.as_ref()?;
    for second in path {
        let Node::Split { first, second: other, .. } = node else {
            return None;
        };
        node = if second { other } else { first };
    }
    let Node::Split { first, second, .. } = node else {
        return None;
    };
    let first_panel = Panel::ALL.iter().chain(Panel::ON_DEMAND).copied().find(|panel| contains(first, *panel))?;
    let second_panel = Panel::ALL.iter().chain(Panel::ON_DEMAND).copied().find(|panel| contains(second, *panel))?;
    let mut node = saved.root.as_ref()?;
    let mut path = Vec::new();
    while let Node::Split { first, second, .. } = node {
        if contains(first, first_panel) && contains(second, second_panel) {
            return Some(Action::ResizeSplit { path, size });
        }
        let in_second = contains(second, first_panel);
        path.push(in_second);
        node = if in_second { second } else { first };
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn hidden_split_ancestors_do_not_redirect_the_visible_resize_handle() {
        let workspace = Workspace::default();
        let mut visible = workspace.layout.clone();
        visible.apply(Action::Close { panel: Panel::ToolSets }).unwrap();
        let action = Action::ResizeSplit { path: Vec::new(), size: SplitSize::FixedSecond(350.0) };
        assert_eq!(
            canonical_action(action, &visible, &workspace.layout),
            Some(Action::ResizeSplit { path: vec![true], size: SplitSize::FixedSecond(350.0) })
        );
        assert_eq!(side_and_width(&workspace, Panel::Layers), (Zone::Right, 300.0));
        assert_eq!(side_and_width(&workspace, Panel::ToolSets), (Zone::Left, 220.0));
    }
}
