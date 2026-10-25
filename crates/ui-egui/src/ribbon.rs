//! Tabbed command ribbon. Layout preferences are UI state; drawing actions use the registry.
use crate::{
    CadApp,
    icons::{self, Icon},
    theme::Tokens,
};
use egui::{Sense, vec2};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

pub mod context;
mod floating;

type Tool = (Icon, &'static str, &'static str);

pub struct Panel {
    pub id: &'static str,
    pub label: &'static str,
    source: &'static str,
    tools: &'static [Tool],
    launcher: Option<&'static str>,
}
pub struct Tab {
    pub id: &'static str,
    pub label: &'static str,
    pub panels: &'static [Panel],
}

const fn palette(id: &'static str, label: &'static str, source: &'static str) -> Panel {
    Panel { id, label, source, tools: &[], launcher: None }
}
const fn panel(id: &'static str, label: &'static str, tools: &'static [Tool], launcher: Option<&'static str>) -> Panel {
    Panel { id, label, source: "", tools, launcher }
}

pub const TABS: &[Tab] = &[
    Tab {
        id: "home",
        label: "Home",
        panels: &[
            palette("draw", "Draw", "Draw"),
            palette("modify", "Modify", "Modify"),
            palette("annotation", "Annotation", "Text"),
            panel(
                "layers",
                "Layers",
                &[
                    (Icon::Layers, "ui.dialog.layers", "Layers"),
                    (Icon::MakeCurrent, "laymcur", "Make Current"),
                    (Icon::LayerIso, "layiso", "Isolate Layer"),
                    (Icon::LayerOff, "layoff", "Layer Off"),
                ],
                Some("ui.dialog.layers"),
            ),
            palette("block", "Block", "Block"),
            panel(
                "properties",
                "Properties",
                &[
                    (Icon::Properties, "ui.panel.properties", "Properties Inspector"),
                    (Icon::MatchProp, "matchprop", "Match Properties"),
                    (Icon::QuickSelect, "qselect", "Quick Select..."),
                ],
                Some("ui.panel.properties"),
            ),
            panel(
                "utilities",
                "Utilities",
                &[
                    (Icon::Measure, "dist", "Measure Distance"),
                    (Icon::Area, "area", "Area"),
                    (Icon::List, "list", "List"),
                    (Icon::Measure, "ui.panel.quickcalc", "QuickCalc"),
                ],
                None,
            ),
            panel(
                "clipboard",
                "Clipboard",
                &[(Icon::Copy, "copyclip", "Copy"), (Icon::Insert, "pasteclip", "Paste"), (Icon::Erase, "cutclip", "Cut")],
                None,
            ),
        ],
    },
    Tab {
        id: "insert",
        label: "Insert",
        panels: &[
            palette("block", "Block", "Block"),
            panel(
                "attributes",
                "Attributes",
                &[
                    (Icon::AttDef, "attdef", "Define Attributes"),
                    (Icon::Text, "attedit", "Edit Attributes"),
                    (Icon::BlockEdit, "battman", "Block Attribute Manager"),
                    (Icon::Redo, "attsync", "Synchronize Attributes"),
                ],
                None,
            ),
            panel(
                "references",
                "References",
                &[(Icon::Attach, "xattach", "Attach DWG"), (Icon::Blocks, "xref", "External References"), (Icon::Insert, "xbind", "Bind Reference")],
                None,
            ),
            panel("import", "Import", &[(Icon::Import, "import", "Import"), (Icon::Attach, "attach", "Attach")], None),
        ],
    },
    Tab {
        id: "annotate",
        label: "Annotate",
        panels: &[
            palette("text", "Text", "Text"),
            palette("dimensions", "Dimensions", "Dimension"),
            palette("leaders", "Leaders", "Leader"),
            palette("tables", "Tables", "Table"),
            panel("styles", "Styles", &[(Icon::Text, "style", "Text Style..."), (Icon::DimLinear, "dimstyle", "Dimension Style...")], None),
        ],
    },
    Tab { id: "parametric", label: "Parametric", panels: &[palette("constraints", "Constraints", "Parametric")] },
    Tab {
        id: "view",
        label: "View",
        panels: &[
            panel(
                "navigate",
                "Navigate",
                &[
                    (Icon::ZoomExtents, "zoom.extents", "Zoom Extents"),
                    (Icon::ZoomWindow, "zoom.window", "Zoom Window"),
                    (Icon::Pan, "pan", "Pan"),
                    (Icon::Orbit, "3dorbit", "Orbit"),
                ],
                None,
            ),
            panel(
                "palettes",
                "Palettes",
                &[
                    (Icon::Properties, "ui.panel.properties", "Properties Inspector"),
                    (Icon::Layers, "ui.panel.layers", "Layers Palette"),
                    (Icon::Measure, "ui.panel.quickcalc", "QuickCalc"),
                    (Icon::Blocks, "ui.toggle.toolsets", "Tool Sets"),
                ],
                None,
            ),
        ],
    },
    Tab {
        id: "manage",
        label: "Manage",
        panels: &[
            panel(
                "settings",
                "Settings",
                &[
                    (Icon::Gear, "units", "Units..."),
                    (Icon::Parameters, "ui.dialog.parameters", "Parameters Manager"),
                    (Icon::Text, "style", "Text Style..."),
                ],
                None,
            ),
            panel("cleanup", "Drawing Utilities", &[(Icon::Erase, "purge", "Purge"), (Icon::List, "audit", "Audit")], None),
        ],
    },
    Tab {
        id: "output",
        label: "Output",
        panels: &[
            panel(
                "plot",
                "Plot",
                &[
                    (Icon::Plot, "plot", "Print"),
                    (Icon::Publish, "publish", "Batch Publish"),
                    (Icon::PageSetup, "pagesetup", "Page Setup Manager"),
                    (Icon::Preview, "preview", "Plot Preview"),
                ],
                Some("pagesetup"),
            ),
            panel("export", "Export", &[(Icon::Export, "export", "Export"), (Icon::Share, "share", "Share")], None),
        ],
    },
    Tab {
        id: "modeling",
        label: "Modeling",
        panels: &[
            palette("primitives", "Primitives", "modeling:Primitives"),
            palette("solids", "Solid", "modeling:Solid"),
            palette("boolean", "Boolean", "modeling:Boolean"),
            palette("view", "View", "modeling:View"),
        ],
    },
];

pub(crate) fn tools(panel: &Panel) -> Vec<Tool> {
    if panel.source.is_empty() {
        return panel.tools.to_vec();
    }
    let (groups, name) =
        panel.source.strip_prefix("modeling:").map_or((crate::palettes::DRAFTING, panel.source), |name| (crate::palettes::MODELING, name));
    let mut result: Vec<Tool> = groups
        .iter()
        .find(|(group, ..)| *group == name)
        .map(|(_, large, small)| large.iter().chain(small.iter()).copied().collect())
        .unwrap_or_default();
    if panel.source == "Draw"
        && let Some((_, large, small)) = crate::palettes::DRAFTING.iter().find(|(name, ..)| *name == "Hatch")
    {
        result.extend(large.iter().chain(small.iter()).copied());
    }
    result
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum MinimizeMode {
    #[default]
    Tabs,
    PanelTitles,
    PanelButtons,
    Cycle,
}
impl MinimizeMode {
    fn id(self) -> &'static str {
        match self {
            Self::Tabs => "tabs",
            Self::PanelTitles => "panelTitles",
            Self::PanelButtons => "panelButtons",
            Self::Cycle => "cycle",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct State {
    pub active_tab: String,
    pub hidden_tabs: Vec<String>,
    /// Qualified IDs (for example home.draw), so a shared panel can differ between tabs.
    pub hidden_panels: Vec<String>,
    pub minimized: bool,
    pub minimize_mode: MinimizeMode,
    pub cycle_position: u8,
    pub show_titles: bool,
    pub contextual_tabs: bool,
    pub floating_panels: Vec<floating::FloatingPanel>,
}
impl Default for State {
    fn default() -> Self {
        Self {
            active_tab: "home".into(),
            hidden_tabs: vec!["modeling".into()],
            hidden_panels: Vec::new(),
            minimized: false,
            minimize_mode: MinimizeMode::Tabs,
            cycle_position: 0,
            show_titles: true,
            contextual_tabs: true,
            floating_panels: Vec::new(),
        }
    }
}
impl State {
    pub fn validate(&self) -> Result<(), String> {
        if self.cycle_position > 2
            || !floating::valid(&self.floating_panels)
            || self.hidden_tabs.len() > TABS.len()
            || self.hidden_panels.len() > 64
            || !TABS.iter().any(|t| t.id == self.active_tab)
            || self.hidden_tabs.iter().any(|id| !TABS.iter().any(|t| t.id == id))
            || self
                .hidden_panels
                .iter()
                .any(|id| !TABS.iter().chain(context::TABS).any(|t| t.panels.iter().any(|p| *id == format!("{}.{}", t.id, p.id))))
        {
            return Err("Invalid ribbon tab or panel settings".into());
        }
        Ok(())
    }
    pub fn tab_visible(&self, id: &str) -> bool {
        !self.hidden_tabs.iter().any(|hidden| hidden == id)
    }
    pub fn panel_visible(&self, tab: &str, id: &str) -> bool {
        !self.hidden_panels.contains(&format!("{tab}.{id}"))
    }
    fn active(&self) -> Option<&'static Tab> {
        tab(&self.active_tab).filter(|t| self.tab_visible(t.id)).or_else(|| TABS.iter().find(|t| self.tab_visible(t.id)))
    }
    fn docked(&self, tab: &str, panel: &str) -> bool {
        self.panel_visible(tab, panel) && !self.floating_panels.iter().any(|p| p.tab == tab && p.panel == panel)
    }
    fn compact_mode(&self) -> MinimizeMode {
        match (self.minimize_mode, self.cycle_position) {
            (MinimizeMode::Cycle, 0) => MinimizeMode::PanelButtons,
            (MinimizeMode::Cycle, 1) => MinimizeMode::PanelTitles,
            (MinimizeMode::Cycle, _) => MinimizeMode::Tabs,
            (mode, _) => mode,
        }
    }
}
fn tab(id: &str) -> Option<&'static Tab> {
    TABS.iter().chain(context::TABS).find(|t| t.id == id)
}
fn visibility(hidden: &mut Vec<String>, id: &str, visible: bool) {
    hidden.retain(|h| h != id);
    if !visible {
        hidden.push(id.into());
    }
}

pub(crate) fn route(app: &mut CadApp, id: &str, p: &Value) -> Option<Result<Value, String>> {
    if !matches!(
        id,
        "ui.ribbon.tab"
            | "ui.ribbon.panel"
            | "ui.ribbon.options"
            | "ui.ribbon.reset"
            | "ui.ribbon.cycle"
            | "ui.ribbon.float"
            | "ui.ribbon.return"
            | "ui.toggle.ribbon"
            | "ribbon"
            | "ribbonclose"
            | "ui.toolsets.ribbon"
            | "ui.toolsets.palette"
            | "ui.toolsets.tools.reset"
    ) {
        return None;
    }
    Some(configure(app, id, p))
}
fn configure(app: &mut CadApp, id: &str, p: &Value) -> Result<Value, String> {
    if matches!(id, "ui.ribbon.float" | "ui.ribbon.return") {
        return floating::configure(app, id, p);
    }
    if id == "ui.toolsets.tools.reset" {
        app.ui.hidden_tools.clear();
        return Ok(json!({"hiddenTools":[]}));
    }
    if id == "ui.toolsets.palette" {
        crate::docking::command(app, false, &json!({"operation":"open","panel":"toolSets"}))?;
        if app.ui.ribbon.active_tab == "modeling" {
            app.ui.toolset_tab = "Modeling".into();
        }
        app.ui.show_ribbon = false;
        return Ok(json!({"showRibbon":false,"showToolsets":true}));
    }
    if id == "ui.toolsets.ribbon" {
        let mut next = app.ui.ribbon.clone();
        next.validate()?;
        next.active_tab = if app.ui.toolset_tab == "Modeling" { "modeling" } else { "home" }.into();
        let active = next.active_tab.clone();
        visibility(&mut next.hidden_tabs, &active, true);
        crate::docking::command(app, false, &json!({"operation":"close","panel":"toolSets"}))?;
        app.ui.ribbon = next;
        app.ui.show_ribbon = true;
        return Ok(json!({"showRibbon":true,"showToolsets":false}));
    }
    if id == "ui.ribbon.reset" {
        crate::docking::command(app, false, &json!({"operation":"close","panel":"toolSets"}))?;
        app.ui.ribbon = State::default();
        app.ribbon_context = context::Runtime::default();
        app.ui.show_ribbon = true;
        return Ok(json!({"ribbon":app.ui.ribbon}));
    }
    if matches!(id, "ui.toggle.ribbon" | "ribbon" | "ribbonclose") {
        let default = match id {
            "ribbon" => true,
            "ribbonclose" => false,
            _ => !app.ui.show_ribbon,
        };
        let shown = bool_param(p, "on", default)?;
        if shown {
            crate::docking::command(app, false, &json!({"operation":"close","panel":"toolSets"}))?;
        }
        app.ui.show_ribbon = shown;
        return Ok(json!({"visible":app.ui.show_ribbon}));
    }
    let mut next = app.ui.ribbon.clone();
    next.validate()?;
    match id {
        "ui.ribbon.tab" => {
            let t = p.get("tab").and_then(Value::as_str).and_then(tab).ok_or("Unknown ribbon tab")?;
            if context::TABS.iter().any(|c| c.id == t.id) {
                if p.get("visible").is_some() || context::available(app).is_none_or(|c| c.id != t.id) {
                    return Err("Contextual ribbon tab is unavailable".into());
                }
                app.ribbon_context.active = true;
                return Ok(json!({"contextualTab": t.id}));
            }
            if p.get("visible").is_some() {
                visibility(&mut next.hidden_tabs, t.id, bool_param(p, "visible", true)?);
            } else {
                visibility(&mut next.hidden_tabs, t.id, true);
                next.active_tab = t.id.into();
            }
            if let Some(active) = next.active() {
                next.active_tab = active.id.into();
            }
        }
        "ui.ribbon.panel" => {
            let t = match p.get("tab") {
                Some(value) => value.as_str().and_then(tab).ok_or("Unknown ribbon tab")?,
                None => context::active(app).ok_or("Select a ribbon tab first")?,
            };
            let panel =
                p.get("panel").and_then(Value::as_str).and_then(|id| t.panels.iter().find(|panel| panel.id == id)).ok_or("Unknown ribbon panel")?;
            let visible = bool_param(p, "visible", !next.panel_visible(t.id, panel.id))?;
            visibility(&mut next.hidden_panels, &format!("{}.{}", t.id, panel.id), visible);
        }
        "ui.ribbon.options" => {
            next.minimized = bool_param(p, "minimized", next.minimized)?;
            next.show_titles = bool_param(p, "titles", next.show_titles)?;
            next.contextual_tabs = bool_param(p, "contextualTabs", next.contextual_tabs)?;
            if let Some(mode) = p.get("minimizeMode") {
                next.minimize_mode = serde_json::from_value(mode.clone()).map_err(|_| "Unknown ribbon minimize mode")?;
                next.cycle_position = 0;
            }
        }
        "ui.ribbon.cycle" => {
            if next.minimize_mode == MinimizeMode::Cycle && next.minimized && next.cycle_position < 2 {
                next.cycle_position += 1;
            } else {
                next.minimized = !next.minimized;
                next.cycle_position = 0;
            }
        }
        _ => {}
    }
    next.validate()?;
    if id == "ui.ribbon.tab" && p.get("visible").is_none() {
        app.ribbon_context.active = false;
    }
    app.ui.ribbon = next;
    Ok(json!({"ribbon":app.ui.ribbon}))
}
fn bool_param(p: &Value, key: &str, default: bool) -> Result<bool, String> {
    p.get(key).map_or(Ok(default), |v| v.as_bool().ok_or_else(|| format!("{key} must be true or false")))
}

/// Shared recovery controls remain accessible from Panels even when the entire ribbon is hidden.
pub(crate) fn customize(app: &mut CadApp, ui: &mut egui::Ui) {
    let mut shown = app.ui.show_ribbon;
    if ui.checkbox(&mut shown, crate::tl!("Ribbon")).changed() {
        let _ = app.run("ui.toggle.ribbon", json!({"on":shown}));
    }
    ui.menu_button(crate::tl!("Show Tabs"), |ui| {
        for t in TABS {
            let mut shown = app.ui.ribbon.tab_visible(t.id);
            if ui.checkbox(&mut shown, crate::i18n::t(t.label)).changed() {
                let _ = app.run("ui.ribbon.tab", json!({"tab":t.id,"visible":shown}));
            }
        }
    });
    if let Some(t) = context::active(app) {
        ui.menu_button(crate::tl!("Show Panels"), |ui| {
            ui.label(crate::i18n::t(t.label));
            for p in t.panels {
                let mut shown = app.ui.ribbon.panel_visible(t.id, p.id);
                if ui.checkbox(&mut shown, crate::i18n::t(p.label)).changed() {
                    let _ = app.run("ui.ribbon.panel", json!({"tab":t.id,"panel":p.id,"visible":shown}));
                }
            }
        });
    }
    let mut contextual = app.ui.ribbon.contextual_tabs;
    if ui.checkbox(&mut contextual, crate::tl!("Show Contextual Tabs")).changed() {
        let _ = app.run("ui.ribbon.options", json!({"contextualTabs":contextual}));
    }
    if ui.button(crate::tl!("Return All Panels to Ribbon")).clicked() {
        let _ = app.run("ui.ribbon.return", json!({"all":true}));
        ui.close();
    }
    let mut minimized = app.ui.ribbon.minimized;
    if ui.checkbox(&mut minimized, crate::tl!("Minimize Ribbon")).changed() {
        let _ = app.run("ui.ribbon.options", json!({"minimized":minimized}));
    }
    display_menu(app, ui);
    let mut titles = app.ui.ribbon.show_titles;
    if ui.checkbox(&mut titles, crate::tl!("Show Panel Titles")).changed() {
        let _ = app.run("ui.ribbon.options", json!({"titles":titles}));
    }
    if ui.add_enabled(!app.ui.hidden_tools.is_empty(), egui::Button::new(crate::tl!("Show All Tools"))).clicked() {
        let _ = app.run("ui.toolsets.tools.reset", json!({}));
        ui.close();
    }
    if ui.button(crate::tl!("Move to Tool Sets")).clicked() {
        let _ = app.run("ui.toolsets.palette", json!({}));
        ui.close();
    }
    if ui.button(crate::tl!("Restore Ribbon")).clicked() {
        let _ = app.run("ui.ribbon.reset", json!({}));
        ui.close();
    }
}

fn display_menu(app: &mut CadApp, ui: &mut egui::Ui) {
    for (mode, label) in [
        (MinimizeMode::Tabs, crate::tl!("Minimize to Tabs")),
        (MinimizeMode::PanelTitles, crate::tl!("Minimize to Panel Titles")),
        (MinimizeMode::PanelButtons, crate::tl!("Minimize to Panel Buttons")),
        (MinimizeMode::Cycle, crate::tl!("Cycle Through All")),
    ] {
        if ui.selectable_label(app.ui.ribbon.minimize_mode == mode, label).clicked() {
            let _ = app.run("ui.ribbon.options", json!({"minimizeMode":mode.id(),"minimized":true}));
            ui.close();
        }
    }
}

fn enabled(app: &CadApp, command: &str) -> bool {
    cadcraft_engine::find_command(command).map_or_else(|| crate::menus::is_ui_command(command), |c| (c.enabled)(&app.session).is_ok())
}
fn tool_button(ui: &mut egui::Ui, tool: Tool, large: bool, panel_label: &str) -> egui::Response {
    let (icon, _, label) = tool;
    let t = Tokens::get();
    let (r, response) = ui.allocate_exact_size(if large { vec2(68.0, 74.0) } else { vec2(98.0, 22.0) }, Sense::click());
    if response.hovered() || response.has_focus() {
        ui.painter().rect_filled(r, 3.0, t.control_hover);
    }
    if response.has_focus() {
        ui.painter().rect_stroke(r.shrink(1.0), 3.0, egui::Stroke::new(1.0, t.accent), egui::StrokeKind::Inside);
    }
    let icon_rect = if large {
        egui::Rect::from_center_size(egui::pos2(r.center().x, r.top() + 22.0), vec2(32.0, 32.0))
    } else {
        egui::Rect::from_min_size(r.min + vec2(2.0, 2.0), vec2(18.0, 18.0))
    };
    icons::paint(ui.painter(), icon_rect, icon, !ui.is_enabled());
    let mut text = egui::text::LayoutJob::simple(
        crate::i18n::t(label).to_string(),
        egui::FontId::proportional(12.0),
        if ui.is_enabled() { t.text } else { t.text_dim },
        if large { 66.0 } else { 74.0 },
    );
    text.wrap.max_rows = if large { 2 } else { 1 };
    text.wrap.break_anywhere = !large;
    let text = ui.painter().layout_job(text);
    let pos = if large { egui::pos2(r.center().x - text.size().x / 2.0, r.top() + 43.0) } else { r.min + vec2(24.0, 3.0) };
    ui.painter().galley(pos, text, t.text);
    let accessible = format!("{} / {}: {}", crate::tl!("Ribbon"), crate::i18n::t(panel_label), crate::i18n::t(label));
    icons::describe_control(ui, &response, &accessible, None, false);
    crate::control::record_widget(ui, &response, &accessible, "button", None);
    response.on_hover_text(crate::i18n::t(label))
}

fn show_panel(app: &mut CadApp, ui: &mut egui::Ui, tab_id: &str, panel: &Panel, titles: bool, clicked: &mut Option<&'static str>) {
    let t = Tokens::get();
    let (rect, _) = ui.allocate_exact_size(vec2(188.0, 112.0), Sense::hover());
    ui.painter().rect_filled(rect, 2.0, t.panel);
    let mut local_context = ui.interact(rect, ui.id().with("panel_context"), Sense::click());
    let tools: Vec<_> = tools(panel).into_iter().filter(|(_, command, _)| !app.ui.hidden_tools.iter().any(|hidden| hidden == command)).collect();
    for (index, tool) in tools.iter().take(4).enumerate() {
        let large = index == 0;
        let offset = if large { vec2(4.0, 4.0) } else { vec2(80.0, 4.0 + (index - 1) as f32 * 26.0) };
        let bounds = egui::Rect::from_min_size(rect.min + offset, if large { vec2(68.0, 74.0) } else { vec2(98.0, 22.0) });
        let response = ui
            .scope_builder(egui::UiBuilder::new().id_salt(("tool", tool.1)).max_rect(bounds).layout(egui::Layout::top_down(egui::Align::Min)), |ui| {
                ui.add_enabled_ui(enabled(app, tool.1), |ui| tool_button(ui, *tool, large, panel.label)).inner
            })
            .inner;
        local_context |= response.clone();
        if response.clicked() {
            *clicked = Some(tool.1);
        }
    }
    let footer = egui::Rect::from_min_max(rect.min + vec2(4.0, 82.0), rect.max - vec2(4.0, 4.0));
    ui.scope_builder(egui::UiBuilder::new().id_salt("footer").max_rect(footer).layout(egui::Layout::left_to_right(egui::Align::Center)), |ui| {
        if titles {
            let reserved = if tools.len() > 4 { 48.0 } else { 0.0 } + if panel.launcher.is_some() { 28.0 } else { 0.0 };
            let title = ui.add_sized(
                vec2((footer.width() - reserved).max(32.0), 22.0),
                egui::Label::new(egui::RichText::new(crate::i18n::t(panel.label)).size(11.0)).truncate().sense(Sense::click_and_drag()),
            );
            local_context |= title.clone();
            title.clone().on_hover_text(crate::tl!("Drag panel title to float"));
            if title.drag_stopped()
                && let Some(pos) = title.interact_pointer_pos()
                && !rect.contains(pos)
            {
                let _ = app.run("ui.ribbon.float", json!({"tab":tab_id,"panel":panel.id,"position":[pos.x,pos.y]}));
            }
            title.widget_info(|| {
                egui::WidgetInfo::labeled(
                    egui::WidgetType::Label,
                    ui.is_enabled(),
                    format!("{}: {}", crate::tl!("Ribbon Panel"), crate::i18n::t(panel.label)),
                )
            });
        }
        if tools.len() > 4 {
            let more = ui.menu_button(crate::tl!("More"), |ui| {
                for tool in tools.iter().skip(4) {
                    if ui.add_enabled(enabled(app, tool.1), egui::Button::new(crate::i18n::t(tool.2))).clicked() {
                        *clicked = Some(tool.1);
                        ui.close();
                    }
                }
            });
            more.response.widget_info(|| {
                egui::WidgetInfo::labeled(
                    egui::WidgetType::Button,
                    ui.is_enabled(),
                    format!("{}: {}", crate::tl!("More"), crate::i18n::t(panel.label)),
                )
            });
        }
        if let Some(command) = panel.launcher {
            let response = ui.small_button("↗").on_hover_text(crate::i18n::t(panel.label));
            icons::describe_control(ui, &response, &format!("{}: {}", crate::tl!("Ribbon Panel"), crate::i18n::t(panel.label)), None, false);
            local_context |= response.clone();
            if response.clicked() {
                *clicked = Some(command);
            }
        }
    });
    local_context.context_menu(|ui| panel_menu(app, ui, tab_id, panel));
}

fn panel_menu(app: &mut CadApp, ui: &mut egui::Ui, tab_id: &str, panel: &Panel) {
    let floating = app.ui.ribbon.floating_panels.iter().any(|p| p.tab == tab_id && p.panel == panel.id);
    let label = if floating { crate::tl!("Return to Ribbon") } else { crate::tl!("Float Panel") };
    if ui.button(label).clicked() {
        let _ = app.run(if floating { "ui.ribbon.return" } else { "ui.ribbon.float" }, json!({"tab":tab_id,"panel":panel.id}));
        ui.close();
    }
    customize(app, ui);
}

pub fn show(app: &mut CadApp, ui: &mut egui::Ui) {
    if !app.ui.show_ribbon {
        return;
    }
    if app.ui.ribbon.validate().is_err() {
        app.ui.ribbon = State::default();
    }
    context::sync(app);
    let t = Tokens::get();
    let minimized = app.ui.ribbon.minimized;
    let compact = app.ui.ribbon.compact_mode();
    let titles = app.ui.ribbon.show_titles;
    let mut clicked = None;
    egui::Panel::top("cc_ribbon")
        .exact_size(if !minimized {
            160.0
        } else {
            match compact {
                MinimizeMode::PanelTitles => 66.0,
                MinimizeMode::PanelButtons => 108.0,
                _ => 30.0,
            }
        })
        .frame(egui::Frame::NONE.fill(t.chrome_dark).inner_margin(egui::Margin::symmetric(6, 2)))
        .show(ui, |ui| {
            let mut context = ui.interact(ui.max_rect(), ui.id().with("ribbon_context"), Sense::CLICK);
            ui.horizontal(|ui| {
                egui::ScrollArea::horizontal().id_salt("ribbon_tabs").max_width((ui.available_width() - 162.0).max(32.0)).show(ui, |ui| {
                    ui.horizontal(|ui| {
                        for tab in TABS.iter().filter(|t| app.ui.ribbon.tab_visible(t.id)).chain(context::available(app)).collect::<Vec<_>>() {
                            let active = context::active(app).is_some_and(|t| t.id == tab.id);
                            let contextual = context::TABS.iter().any(|t| t.id == tab.id);
                            let text = egui::RichText::new(crate::i18n::t(tab.label));
                            let response = ui.selectable_label(active, if contextual && !active { text.color(t.accent).strong() } else { text });
                            response.widget_info(|| {
                                egui::WidgetInfo::selected(
                                    egui::WidgetType::SelectableLabel,
                                    ui.is_enabled(),
                                    active,
                                    format!("{}: {}", crate::tl!("Ribbon Tab"), crate::i18n::t(tab.label)),
                                )
                            });
                            context |= response.clone();
                            if response.clicked() || response.secondary_clicked() {
                                let _ = app.run("ui.ribbon.tab", json!({"tab":tab.id}));
                            }
                            if response.double_clicked() {
                                let _ = app.run("ui.ribbon.options", json!({"minimized":!minimized}));
                                egui::Popup::close_all(ui.ctx());
                            } else if minimized && compact == MinimizeMode::Tabs {
                                let toggle = response.clicked().then_some(egui::SetOpenCommand::Toggle);
                                egui::Popup::from_response(&response)
                                    .width((ui.ctx().content_rect().width() - 24.0).max(32.0))
                                    .open_memory(toggle)
                                    .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
                                    .show(|ui| flyout(app, ui, tab, None, &mut clicked));
                            }
                        }
                        if context::active(app).is_none() {
                            ui.label(crate::tl!("No ribbon tabs are visible"));
                        }
                    });
                });
                let response = icons::button(
                    ui,
                    if minimized { Icon::ChevronDown } else { Icon::ChevronUp },
                    22.0,
                    if app.ui.ribbon.minimize_mode == MinimizeMode::Cycle {
                        "Cycle Ribbon Display"
                    } else if minimized {
                        "Expand Ribbon"
                    } else {
                        "Minimize Ribbon"
                    },
                    false,
                );
                context |= response.clone();
                if response.clicked() {
                    let _ = app.run("ui.ribbon.cycle", json!({}));
                    egui::Popup::close_all(ui.ctx());
                }
                let display = ui.menu_button(" ", |ui| display_menu(app, ui)).response;
                icons::paint(ui.painter(), egui::Rect::from_center_size(display.rect.center(), vec2(12.0, 12.0)), Icon::ChevronDown, false);
                display.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, ui.is_enabled(), crate::tl!("Ribbon Display")));
                display.on_hover_text(crate::tl!("Ribbon Display"));
                ui.menu_button(crate::tl!("Ribbon Options"), |ui| customize(app, ui));
            });
            if minimized && matches!(compact, MinimizeMode::PanelTitles | MinimizeMode::PanelButtons) {
                ui.separator();
                if let Some(tab) = context::active(app) {
                    egui::ScrollArea::horizontal().id_salt("compact_ribbon_panels").show(ui, |ui| {
                        ui.horizontal_top(|ui| {
                            let panels: Vec<_> = tab.panels.iter().filter(|p| app.ui.ribbon.docked(tab.id, p.id)).collect();
                            for panel in panels {
                                ui.push_id((tab.id, panel.id), |ui| {
                                    let label = crate::i18n::t(panel.label);
                                    let response = if compact == MinimizeMode::PanelButtons {
                                        let (rect, response) = ui.allocate_exact_size(vec2(104.0, 65.0), Sense::click());
                                        ui.painter().rect_filled(rect, 2.0, if response.hovered() { t.control_hover } else { t.panel });
                                        if let Some((icon, ..)) = tools(panel).first() {
                                            icons::paint(
                                                ui.painter(),
                                                egui::Rect::from_center_size(rect.center_top() + vec2(0.0, 20.0), vec2(28.0, 28.0)),
                                                *icon,
                                                false,
                                            );
                                        }
                                        ui.painter().text(
                                            rect.center_bottom() - vec2(0.0, 12.0),
                                            egui::Align2::CENTER_CENTER,
                                            label,
                                            egui::FontId::proportional(11.0),
                                            t.text,
                                        );
                                        icons::paint(
                                            ui.painter(),
                                            egui::Rect::from_center_size(rect.right_top() + vec2(-12.0, 12.0), vec2(10.0, 10.0)),
                                            Icon::ChevronDown,
                                            false,
                                        );
                                        response
                                    } else {
                                        let width = ui.painter().layout_no_wrap(label.to_string(), crate::theme::body(), t.text).size().x + 32.0;
                                        let response = ui.add_sized(vec2(width, 24.0), egui::Button::new(label));
                                        icons::paint(
                                            ui.painter(),
                                            egui::Rect::from_center_size(response.rect.right_center() - vec2(9.0, 0.0), vec2(10.0, 10.0)),
                                            Icon::ChevronDown,
                                            false,
                                        );
                                        response
                                    };
                                    icons::describe_control(
                                        ui,
                                        &response,
                                        &format!("{}: {}", crate::tl!("Ribbon Panel"), crate::i18n::t(panel.label)),
                                        None,
                                        false,
                                    );
                                    response.context_menu(|ui| panel_menu(app, ui, tab.id, panel));
                                    egui::Popup::menu(&response)
                                        .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
                                        .show(|ui| flyout(app, ui, tab, Some(panel.id), &mut clicked));
                                });
                            }
                        });
                    });
                }
            }
            if !minimized {
                ui.separator();
                egui::ScrollArea::horizontal().id_salt("ribbon_panels").show(ui, |ui| {
                    ui.horizontal_top(|ui| {
                        if let Some(tab) = context::active(app) {
                            let panels: Vec<_> = tab.panels.iter().filter(|p| app.ui.ribbon.docked(tab.id, p.id)).collect();
                            if panels.is_empty() {
                                ui.label(crate::tl!("No ribbon panels are visible"));
                                if ui.button(crate::tl!("Restore Ribbon")).clicked() {
                                    let _ = app.run("ui.ribbon.reset", json!({}));
                                }
                            }
                            for panel in panels {
                                ui.push_id((tab.id, panel.id), |ui| show_panel(app, ui, tab.id, panel, titles, &mut clicked));
                            }
                        } else if ui.button(crate::tl!("Restore Ribbon")).clicked() {
                            let _ = app.run("ui.ribbon.reset", json!({}));
                        }
                    });
                });
            }
            context.context_menu(|ui| customize(app, ui));
        });
    floating::show(app, ui.ctx(), &mut clicked);
    if let Some(command) = clicked {
        egui::Popup::close_all(ui.ctx());
        app.start(command);
    }
}

fn flyout(app: &mut CadApp, ui: &mut egui::Ui, tab: &Tab, panel: Option<&str>, clicked: &mut Option<&'static str>) {
    let width = (ui.ctx().content_rect().width() - 24.0).max(32.0);
    if panel.is_none() {
        // Recompute on every opening/resize instead of retaining the generic popup width.
        ui.set_width(width);
    }
    let context = ui.interact(ui.max_rect(), ui.id().with("flyout_context"), Sense::CLICK);
    egui::ScrollArea::horizontal().id_salt(("ribbon_flyout", tab.id, panel)).max_width(width).show(ui, |ui| {
        ui.horizontal_top(|ui| {
            let panels: Vec<_> = tab.panels.iter().filter(|p| panel.is_none_or(|id| id == p.id) && app.ui.ribbon.docked(tab.id, p.id)).collect();
            for panel in panels {
                ui.push_id((tab.id, panel.id), |ui| show_panel(app, ui, tab.id, panel, app.ui.ribbon.show_titles, clicked));
            }
        });
    });
    context.context_menu(|ui| customize(app, ui));
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn enabled_ribbon_actions_resolve_and_legacy_modeling_placeholders_stay_disabled() {
        let app = CadApp::new(cadcraft_engine::Session::default(), crate::Services::default());
        for tab in TABS.iter().chain(context::TABS) {
            for panel in tab.panels {
                let tools = tools(panel);
                assert!(!tools.is_empty(), "{}.{}", tab.id, panel.id);
                for (_, command, _) in tools {
                    let known = cadcraft_engine::find_command(command).is_some() || crate::menus::is_ui_command(command);
                    assert!(known || panel.source.starts_with("modeling:"), "{}.{}: {command}", tab.id, panel.id);
                    if !known {
                        assert!(!enabled(&app, command));
                    }
                }
            }
        }
    }
    #[test]
    fn every_tool_set_command_has_a_ribbon_home() {
        let represented: std::collections::HashSet<_> =
            TABS.iter().flat_map(|tab| tab.panels).flat_map(tools).map(|(_, command, _)| command).collect();
        for groups in [crate::palettes::DRAFTING, crate::palettes::MODELING] {
            for (_, large, small) in groups {
                for (_, command, _) in large.iter().chain(small.iter()) {
                    assert!(represented.contains(command), "Missing Tool Sets command: {command}");
                }
            }
        }
    }
}
