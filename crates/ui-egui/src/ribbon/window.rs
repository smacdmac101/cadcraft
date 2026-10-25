//! Whole-ribbon palette, independent of detached individual panels.
use super::*;
use floating::native;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct State {
    pub floating: bool,
    pub desktop_position: Option<[f32; 2]>,
    pub size: [f32; 2],
}

impl Default for State {
    fn default() -> Self {
        Self { floating: false, desktop_position: None, size: [360.0, 520.0] }
    }
}

impl State {
    pub(super) fn valid(&self) -> bool {
        self.desktop_position.is_none_or(|p| p.iter().all(|v| v.is_finite() && v.abs() <= 1_000_000.0))
            && self.size.iter().all(|v| v.is_finite() && (220.0..=8192.0).contains(v))
            && self.size[0] >= 300.0
    }
}

pub(super) fn configure(app: &mut CadApp, p: &Value) -> Result<Value, String> {
    let mut next = app.ui.ribbon.clone();
    next.window.floating = bool_param(p, "floating", next.window.floating)?;
    if let Some(v) = p.get("desktopPosition") {
        next.window.desktop_position =
            if v.is_null() { None } else { Some(serde_json::from_value(v.clone()).map_err(|_| "Invalid ribbon position")?) };
    }
    if let Some(v) = p.get("size") {
        next.window.size = serde_json::from_value(v.clone()).map_err(|_| "Invalid ribbon size")?;
    }
    next.validate()?;
    app.ui.ribbon = next;
    Ok(json!({"ribbon":app.ui.ribbon}))
}

pub(super) fn show(app: &mut CadApp, ctx: &egui::Context, clicked: &mut Option<&'static str>) {
    let state = app.ui.ribbon.window.clone();
    if ctx.embed_viewports() {
        let mut open = true;
        egui::Window::new(crate::tl!("Ribbon"))
            .id(egui::Id::new("whole_ribbon"))
            .default_pos([80.0, 180.0])
            .default_size(state.size)
            .min_size([300.0, 220.0])
            .open(&mut open)
            .collapsible(false)
            .show(ctx, |ui| body(app, ui, clicked));
        if !open {
            let _ = app.run("ribbonclose", json!({}));
        }
        return;
    }
    let id = egui::ViewportId::from_hash_of("cadcraft-whole-ribbon");
    let geometry = native::show_window(
        app,
        ctx,
        native::WindowSpec {
            id,
            title: format!("CADCraft — {}", crate::tl!("Ribbon")),
            saved: state.desktop_position,
            offset: [80.0, 180.0],
            size: state.size,
            resizable: true,
        },
        clicked,
        body,
    );
    native::remember_windows(ctx, vec![id]);
    if geometry.closed {
        let _ = app.run("ribbonclose", json!({}));
    } else if app.ui.ribbon.window.floating {
        let _ = app.run(
            "ui.ribbon.window",
            json!({
                "desktopPosition":geometry.position.or(state.desktop_position),
                "size":geometry.size.unwrap_or(state.size),
            }),
        );
    }
}

fn body(app: &mut CadApp, ui: &mut egui::Ui, clicked: &mut Option<&'static str>) {
    let background = ui.interact(ui.max_rect(), ui.id().with("whole_ribbon_context"), Sense::click());
    ui.horizontal(|ui| {
        let grip = ui.add(egui::Label::new(crate::tl!("Ribbon")).sense(Sense::click_and_drag()));
        grip.clone().on_hover_text(crate::tl!("Double-click to dock ribbon"));
        if grip.double_clicked() {
            dock(app, ui.ctx());
        }
        if grip.drag_started() && !ui.ctx().embed_viewports() {
            ui.ctx().send_viewport_cmd(egui::ViewportCommand::StartDrag);
        }
        if ui.button(crate::tl!("Dock Ribbon")).clicked() {
            dock(app, ui.ctx());
        }
        ui.menu_button(crate::tl!("Ribbon Options"), |ui| customize(app, ui));
    });
    ui.separator();
    let tabs: Vec<_> = TABS.iter().filter(|t| app.ui.ribbon.tab_visible(t.id)).chain(context::available(app)).collect();
    let vertical = ui.available_width() < 600.0;
    let temporary = !vertical && app.ui.ribbon.minimized && app.ui.ribbon.compact_mode() == MinimizeMode::Tabs;
    if vertical {
        ui.horizontal_top(|ui| {
            ui.vertical(|ui| {
                ui.set_width(110.0);
                egui::ScrollArea::vertical().id_salt("floating_tabs").show(ui, |ui| tab_buttons(app, ui, &tabs, clicked, false));
            });
            ui.separator();
            egui::ScrollArea::vertical().id_salt("floating_panels").show(ui, |ui| {
                ui.vertical(|ui| panels(app, ui, clicked, true));
            });
        });
    } else {
        egui::ScrollArea::horizontal().id_salt("floating_tabs").show(ui, |ui| {
            ui.horizontal(|ui| tab_buttons(app, ui, &tabs, clicked, temporary));
        });
        ui.separator();
        if !temporary {
            egui::ScrollArea::horizontal().id_salt("floating_panels").show(ui, |ui| {
                ui.horizontal_top(|ui| panels(app, ui, clicked, false));
            });
        }
    }
    background.context_menu(|ui| customize(app, ui));
}

fn dock(app: &mut CadApp, ctx: &egui::Context) {
    let _ = app.run("ui.ribbon.window", json!({"floating":false}));
    ctx.send_viewport_cmd_to(egui::ViewportId::ROOT, egui::ViewportCommand::Focus);
    ctx.request_repaint_of(egui::ViewportId::ROOT);
}

fn tab_buttons(app: &mut CadApp, ui: &mut egui::Ui, tabs: &[&Tab], clicked: &mut Option<&'static str>, temporary: bool) {
    for tab in tabs {
        let active = context::active(app).is_some_and(|t| t.id == tab.id);
        let contextual = context::TABS.iter().any(|t| t.id == tab.id);
        let text = egui::RichText::new(crate::i18n::t(tab.label));
        let response = ui.selectable_label(active, if contextual && !active { text.color(Tokens::get().accent).strong() } else { text });
        crate::control::record_widget(ui, &response, &format!("{}: {}", crate::tl!("Ribbon Tab"), crate::i18n::t(tab.label)), "tab", None);
        if response.clicked() || response.secondary_clicked() {
            let _ = app.run("ui.ribbon.tab", json!({"tab":tab.id}));
        }
        response.context_menu(|ui| customize(app, ui));
        if temporary {
            egui::Popup::from_response(&response)
                .open_memory(response.clicked().then_some(egui::SetOpenCommand::Toggle))
                .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
                .show(|ui| flyout(app, ui, tab, None, clicked));
        }
    }
}

fn panels(app: &mut CadApp, ui: &mut egui::Ui, clicked: &mut Option<&'static str>, compact: bool) {
    let Some(tab) = context::active(app) else {
        ui.label(crate::tl!("No ribbon tabs are visible"));
        return;
    };
    for panel in tab.panels.iter().filter(|p| app.ui.ribbon.docked(tab.id, p.id)).collect::<Vec<_>>() {
        ui.push_id((tab.id, panel.id), |ui| {
            if compact || app.ui.ribbon.minimized {
                let titles = !compact && app.ui.ribbon.compact_mode() == MinimizeMode::PanelTitles;
                let response =
                    ui.add_sized([188.0, if titles { 24.0 } else { 58.0 }], egui::Button::new(format!("    {}", crate::i18n::t(panel.label))));
                if !titles && let Some((icon, ..)) = tools(panel).first() {
                    icons::paint(
                        ui.painter(),
                        egui::Rect::from_center_size(response.rect.left_center() + vec2(24.0, 0.0), vec2(28.0, 28.0)),
                        *icon,
                        false,
                    );
                }
                crate::control::record_widget(
                    ui,
                    &response,
                    &format!("{}: {}", crate::tl!("Ribbon Panel"), crate::i18n::t(panel.label)),
                    "button",
                    None,
                );
                response.context_menu(|ui| panel_menu(app, ui, tab.id, panel));
                egui::Popup::menu(&response).show(|ui| flyout(app, ui, tab, Some(panel.id), clicked));
            } else {
                show_panel(app, ui, tab.id, panel, app.ui.ribbon.show_titles, clicked);
            }
        });
    }
}
