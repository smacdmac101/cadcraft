//! Application panel identities and persistence around the shared docking renderer.
//! Document contents remain owned by their existing view implementations.

use craft_ui::docking::{Action, DockArea, DockContent, DockStyle, Layout, Location, Node, PanelLimits, Permissions, Placement, Zone};
use craft_ui::layout::{SplitAxis, SplitSize};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

mod auto_hide;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Panel {
    Canvas,
    ToolSets,
    QuickCalc,
    Layers,
    Properties,
    #[serde(rename = "externalReferences")]
    ExternalRefs,
}

impl Panel {
    pub const ALL: &[Self] = &[Self::Canvas, Self::ToolSets, Self::Layers, Self::Properties, Self::ExternalRefs];
    pub fn id(self) -> &'static str {
        match self {
            Self::Canvas => "canvas",
            Self::ToolSets => "toolSets",
            Self::QuickCalc => "quickCalc",
            Self::Layers => "layers",
            Self::Properties => "properties",
            Self::ExternalRefs => "externalReferences",
        }
    }
    fn label(self) -> &'static str {
        match self {
            Self::Canvas => "Drawing",
            Self::ToolSets => "Tool Sets",
            Self::QuickCalc => "QuickCalc",
            Self::Layers => "Layers",
            Self::Properties => "Properties",
            Self::ExternalRefs => "External References",
        }
    }
    fn default_side(self) -> Zone {
        match self {
            Self::Canvas => Zone::Right,
            Self::ToolSets => Zone::Left,
            Self::QuickCalc => Zone::Right,
            Self::Layers => Zone::Right,
            Self::Properties => Zone::Right,
            Self::ExternalRefs => Zone::Right,
        }
    }
    /// Palettes outside the default arrangement, opened on request (QUICKCALC).
    pub const ON_DEMAND: &[Self] = &[Self::QuickCalc];
    fn parse(id: &str) -> Result<Self, String> {
        Self::ALL.iter().chain(Self::ON_DEMAND).copied().find(|panel| panel.id() == id).ok_or_else(|| format!("Unknown panel: {id}"))
    }
}

fn permissions(panel: &Panel) -> Permissions {
    if *panel == Panel::Canvas { Permissions::PROTECTED } else { Permissions::default() }
}

fn tabs(panel: Panel) -> Node<Panel> {
    Node::Tabs { panels: vec![panel], active: 0 }
}

fn split(axis: SplitAxis, size: SplitSize, first: Node<Panel>, second: Node<Panel>) -> Node<Panel> {
    Node::Split { axis, size, first: Box::new(first), second: Box::new(second) }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Workspace {
    pub layout: Layout<Panel>,
    /// Closed panels retain their most recent group, side, size or floating rectangle.
    pub hidden: Vec<(Panel, Location<Panel>)>,
    /// Auto-hidden panels remain logically open, with their full placement retained.
    pub auto_hide: Vec<Panel>,
    #[serde(skip)]
    reveal_auto_hide: Option<Panel>,
}

impl Default for Workspace {
    fn default() -> Self {
        Self {
            layout: Layout {
                root: Some(split(
                    SplitAxis::Horizontal,
                    SplitSize::FixedFirst(220.0),
                    tabs(Panel::ToolSets),
                    split(
                        SplitAxis::Horizontal,
                        SplitSize::FixedSecond(300.0),
                        tabs(Panel::Canvas),
                        Node::Stack {
                            entries: vec![
                                craft_ui::docking::StackEntry { panel: Panel::Layers, open: true, height: Some(280.0) },
                                craft_ui::docking::StackEntry { panel: Panel::Properties, open: true, height: None },
                            ],
                        },
                    ),
                )),
                floating: Vec::new(),
            },
            hidden: Vec::new(),
            auto_hide: Vec::new(),
            reveal_auto_hide: None,
        }
    }
}

impl Workspace {
    pub fn validate(&self) -> Result<(), String> {
        self.layout.validate().map_err(|error| error.to_string())?;
        let mut auto_hidden = std::collections::HashSet::new();
        if self.auto_hide.len() > Panel::ALL.len() + Panel::ON_DEMAND.len()
            || self.auto_hide.iter().any(|panel| *panel == Panel::Canvas || !auto_hidden.insert(*panel))
        {
            return Err("Auto-hide requires unique side panel identities".into());
        }
        if !self.layout.contains(&Panel::Canvas) || self.hidden.len() > Panel::ALL.len() + Panel::ON_DEMAND.len() {
            return Err("The panel layout must contain its document view and bounded hidden panels".into());
        }
        // Saved settings cannot turn the protected document view into a floating or closable tab.
        if self.layout.floating.iter().any(|group| group.panels.contains(&Panel::Canvas)) {
            return Err("The document view cannot float".into());
        }
        let mut nodes: Vec<_> = self.layout.root.iter().collect();
        while let Some(node) = nodes.pop() {
            match node {
                Node::Split { first, second, .. } => {
                    nodes.push(first);
                    nodes.push(second);
                }
                Node::Tabs { panels, .. } if panels.contains(&Panel::Canvas) && panels.len() != 1 => {
                    return Err("The document view cannot share a tab group".into());
                }
                Node::Stack { entries } if entries.iter().any(|entry| entry.panel == Panel::Canvas) => {
                    return Err("The document view cannot be collapsed".into());
                }
                _ => {}
            }
        }
        let mut seen = std::collections::HashSet::new();
        for (panel, location) in &self.hidden {
            if *panel == Panel::Canvas || self.layout.contains(panel) || !seen.insert(*panel) {
                return Err("A hidden panel has an invalid or duplicate identity".into());
            }
            if location.anchor == Some(Panel::Canvas) && matches!(location.placement, Placement::Tab { .. } | Placement::Split(Zone::Center)) {
                return Err("A hidden panel cannot restore over the document view".into());
            }
            let mut probe = self.layout.clone();
            if let Err(error) = probe.restore(*panel, location)
                && error != craft_ui::docking::DockError::MissingTarget
            {
                return Err(error.to_string());
            }
        }
        Ok(())
    }

    pub fn apply(&mut self, action: Action<Panel>) -> Result<(), String> {
        self.validate()?;
        let mut next = self.clone();
        if let Action::Close { panel } = &action {
            let location = next.layout.location(panel).map_err(|error| error.to_string())?;
            next.hidden.retain(|(id, _)| id != panel);
            next.hidden.push((*panel, location));
        } else if let Action::Open { panel, .. } | Action::OpenAt { panel, .. } = &action {
            next.hidden.retain(|(id, _)| id != panel);
        }
        next.layout.apply_with_permissions(action, permissions).map_err(|error| error.to_string())?;
        next.validate()?;
        *self = next;
        Ok(())
    }

    /// Explicit opens select the requested tab and expand a collapsed stack entry.
    pub fn reveal_panel(&mut self, panel: Panel) -> Result<(), String> {
        let mut next = self.clone();
        next.set_visible(panel, true)?;
        next.apply(Action::Activate { panel })?;
        *self = next;
        Ok(())
    }

    pub fn set_visible(&mut self, panel: Panel, visible: bool) -> Result<(), String> {
        if self.layout.contains(&panel) == visible {
            return Ok(());
        }
        if !visible {
            return self.apply(Action::Close { panel });
        }
        self.validate()?;
        let mut next = self.clone();
        let location = next.hidden.iter().find(|(id, _)| *id == panel).map(|(_, location)| location.clone());
        // Recreate the last surviving member first. Restoring a member also restores any
        // hidden anchor it needs, so either close order retains the original tab order.
        // Remove this member before recursion to bound malformed/cyclic saved anchors.
        next.hidden.retain(|(id, _)| *id != panel);
        if let Some(anchor) = location.as_ref().and_then(|saved| saved.anchor)
            && !next.layout.contains(&anchor)
            && next.hidden.iter().any(|(id, _)| *id == anchor)
        {
            next.set_visible(anchor, true)?;
        }
        let restored = location.as_ref().is_some_and(|location| next.layout.restore(panel, location).is_ok());
        if !restored {
            next.layout
                .apply_with_permissions(
                    Action::OpenAt { panel, anchor: Panel::Canvas, placement: Placement::Split(panel.default_side()) },
                    permissions,
                )
                .map_err(|error| error.to_string())?;
        }
        next.validate()?;
        *self = next;
        Ok(())
    }

    fn command(&mut self, params: &Value) -> Result<(), String> {
        self.validate()?;
        if let Some(action) = params.get("action") {
            let action: Action<Panel> = serde_json::from_value(action.clone()).map_err(|error| error.to_string())?;
            return self.apply(action);
        }
        let operation = params.get("operation").and_then(Value::as_str).ok_or("operation is required")?;
        if operation == "resizeSplit" {
            let path: Vec<bool> =
                serde_json::from_value(params.get("path").cloned().ok_or("path is required")?).map_err(|error| error.to_string())?;
            let size: SplitSize =
                serde_json::from_value(params.get("size").cloned().ok_or("size is required")?).map_err(|error| error.to_string())?;
            return self.apply(Action::ResizeSplit { path, size });
        }
        let panel = Panel::parse(params.get("panel").and_then(Value::as_str).ok_or("panel is required")?)?;
        let action = match operation {
            "autoHide" => {
                if panel == Panel::Canvas {
                    return Err("The document view cannot auto-hide".into());
                }
                let enabled = params.get("enabled").and_then(Value::as_bool).ok_or("enabled must be a boolean")?;
                self.auto_hide.retain(|id| *id != panel);
                if enabled {
                    self.auto_hide.push(panel);
                }
                return Ok(());
            }
            "open" => {
                self.reveal_panel(panel)?;
                if self.auto_hide.contains(&panel) {
                    self.reveal_auto_hide = Some(panel);
                }
                return Ok(());
            }
            "close" => return self.set_visible(panel, false),
            "dock" => {
                let zone = match params.get("zone").and_then(Value::as_str).unwrap_or("left") {
                    "left" => Zone::Left,
                    "right" => Zone::Right,
                    _ => return Err("dock zone must be left or right".into()),
                };
                let mut next = self.clone();
                next.reveal_panel(panel)?;
                let old = next.layout.location(&panel).map_err(|error| error.to_string())?;
                let width = old
                    .floating
                    .map(|rect| rect[2])
                    .or(match (old.placement, old.size) {
                        (Placement::Split(Zone::Left), Some(SplitSize::FixedFirst(width)))
                        | (Placement::Split(Zone::Right), Some(SplitSize::FixedSecond(width))) => Some(width),
                        _ => None,
                    })
                    .unwrap_or(if panel == Panel::ToolSets { 220.0 } else { 300.0 });
                let limits = panel_limits(&panel);
                let width = width.clamp(limits.min.x, limits.max.x);
                next.set_visible(panel, false)?;
                next.hidden.retain(|(id, _)| *id != panel);
                let location = Location {
                    floating: None,
                    anchor: Some(Panel::Canvas),
                    placement: Placement::Split(zone),
                    size: Some(if zone == Zone::Left { SplitSize::FixedFirst(width) } else { SplitSize::FixedSecond(width) }),
                    stack: None,
                    sibling: vec![Panel::Canvas],
                };
                next.layout.restore(panel, &location).map_err(|error| error.to_string())?;
                next.validate()?;
                *self = next;
                return Ok(());
            }
            "activate" => Action::Activate { panel },
            "setStackOpen" => Action::SetStackOpen { panel, open: params.get("open").and_then(Value::as_bool).ok_or("open must be a boolean")? },
            "resizeStack" => {
                let value = params.get("height").ok_or("height is required")?;
                let height = if value.is_null() {
                    None
                } else {
                    Some(
                        value
                            .as_f64()
                            .filter(|height| height.is_finite() && (0.0..=1_000_000.0).contains(height))
                            .ok_or("height must be finite and bounded")? as f32,
                    )
                };
                Action::ResizeStack { panel, height }
            }
            "float" | "moveFloating" => {
                let rect = params.get("rect").and_then(Value::as_array).filter(|rect| rect.len() == 4).ok_or("rect must be [x,y,width,height]")?;
                let mut values = [0.0f32; 4];
                for (output, value) in values.iter_mut().zip(rect) {
                    *output = value
                        .as_f64()
                        .filter(|number| number.is_finite() && number.abs() <= 1_000_000.0)
                        .ok_or("rect values must be finite and bounded")? as f32;
                }
                if operation == "float" { Action::Float { panel, rect: values } } else { Action::MoveFloating { panel, rect: values } }
            }
            "move" => {
                let anchor = Panel::parse(params.get("target").and_then(Value::as_str).ok_or("target is required")?)?;
                let placement = match params.get("zone").and_then(Value::as_str).unwrap_or("center") {
                    "center" => Placement::Split(Zone::Center),
                    "left" => Placement::Split(Zone::Left),
                    "right" => Placement::Split(Zone::Right),
                    "top" => Placement::Split(Zone::Top),
                    "bottom" => Placement::Split(Zone::Bottom),
                    _ => return Err("zone must be center, left, right, top or bottom".into()),
                };
                let placement = if let Some(before) = params.get("before") {
                    if !matches!(placement, Placement::Split(Zone::Center)) {
                        return Err("before only applies to a center move".into());
                    }
                    Placement::Tab {
                        before: if before.is_null() {
                            None
                        } else {
                            Some(Panel::parse(before.as_str().ok_or("before must be a panel ID or null")?)?)
                        },
                    }
                } else {
                    placement
                };
                Action::Move { panel, anchor, placement }
            }
            _ => {
                return Err(
                    "operation must be open, close, dock, autoHide, activate, float, move, moveFloating, resizeSplit, setStackOpen or resizeStack"
                        .into(),
                );
            }
        };
        self.apply(action)
    }
}

fn sync(workspace: &mut Workspace, app: &crate::CadApp) -> Result<(), String> {
    workspace.set_visible(Panel::ToolSets, app.ui.show_toolsets)?;
    if !app.ui.show_palettes {
        workspace.set_visible(Panel::Layers, false)?;
        workspace.set_visible(Panel::Properties, false)?;
    } else if !workspace.layout.contains(&Panel::Layers) && !workspace.layout.contains(&Panel::Properties) {
        // Reopen the last surviving accordion entry before restoring its earlier sibling.
        workspace.set_visible(Panel::Properties, true)?;
        workspace.set_visible(Panel::Layers, true)?;
    }
    Ok(())
}

fn publish_visibility(app: &mut crate::CadApp, workspace: &Workspace) {
    app.ui.show_toolsets = workspace.layout.contains(&Panel::ToolSets);
    app.ui.show_palettes = workspace.layout.contains(&Panel::Layers) || workspace.layout.contains(&Panel::Properties);
}

pub fn command(app: &mut crate::CadApp, reset: bool, params: &Value) -> Result<Value, String> {
    let mut workspace = app.ui.docking.clone();
    if reset {
        workspace = Workspace::default();
        if app.ui.show_ribbon {
            workspace.set_visible(Panel::ToolSets, false)?;
        }
    } else {
        sync(&mut workspace, app)?;
        workspace.command(params)?;
    }
    publish_visibility(app, &workspace);
    app.ui.docking = workspace;
    Ok(json!({"docking": app.ui.docking}))
}

struct Content<'a> {
    app: &'a mut crate::CadApp,
    queued: Vec<(&'static str, Value)>,
}
impl DockContent<Panel> for Content<'_> {
    fn body(&mut self, ui: &mut egui::Ui, panel: &Panel) {
        match panel {
            Panel::Canvas => {
                ui.painter().rect_filled(ui.max_rect(), 0.0, crate::theme::Tokens::get().canvas);
                crate::canvas::show(self.app, ui);
            }
            Panel::ToolSets => crate::palettes::toolsets(self.app, ui),
            Panel::QuickCalc => crate::palettes::quickcalc::show(self.app, ui),
            Panel::Layers => {
                egui::ScrollArea::vertical()
                    .id_salt("dock-layers")
                    .auto_shrink([false, false])
                    .show(ui, |ui| crate::palettes::layers_section(self.app, ui));
            }
            Panel::Properties => {
                egui::ScrollArea::vertical().id_salt("dock-properties").auto_shrink([false, false]).show(ui, |ui| {
                    crate::shortcut_menus::palette(self.app, ui, crate::palettes::properties_section, crate::shortcut_menus::properties_entries)
                });
            }
            Panel::ExternalRefs => {
                egui::ScrollArea::both().id_salt("dock-xrefs").auto_shrink([false, false]).show(ui, |ui| crate::xrefs::section(self.app, ui));
            }
        }
    }
    fn panel_menu(&mut self, ui: &mut egui::Ui, panel: &Panel) {
        if *panel == Panel::Canvas {
            return;
        }
        for (label, zone) in [(crate::tl!("Dock Left"), "left"), (crate::tl!("Dock Right"), "right")] {
            if ui.button(label).clicked() {
                self.queued.push(("ui.dock", json!({"operation":"dock","panel":panel.id(),"zone":zone})));
                ui.close();
            }
        }
        let mut enabled = self.app.ui.docking.auto_hide.contains(panel);
        if ui.checkbox(&mut enabled, crate::tl!("Auto-hide")).changed() {
            self.queued.push(("ui.dock", json!({"operation":"autoHide","panel":panel.id(),"enabled":enabled})));
            ui.close();
        }
        if *panel == Panel::ToolSets && ui.button(crate::tl!("Move to Ribbon")).clicked() {
            self.queued.push(("ui.toolsets.ribbon", json!({})));
            ui.close();
        }
        ui.separator();
    }
    fn group_header_width(&self, _panels: &[Panel], active: &Panel, floating: bool) -> f32 {
        if floating && *active != Panel::Canvas { 48.0 } else { 0.0 }
    }
    fn group_header(&mut self, ui: &mut egui::Ui, _panels: &[Panel], active: &Panel, floating: bool) {
        if !floating || *active == Panel::Canvas {
            return;
        }
        ui.horizontal(|ui| {
            for (icon, label, zone) in
                [(crate::icons::Icon::ChevronLeft, "Dock Left", "left"), (crate::icons::Icon::ChevronRight, "Dock Right", "right")]
            {
                if crate::icons::button(ui, icon, 20.0, label, false).clicked() {
                    self.queued.push(("ui.dock", json!({"operation":"dock","panel":active.id(),"zone":zone})));
                }
            }
        });
    }
}

pub fn show(app: &mut crate::CadApp, ui: &mut egui::Ui) {
    let t = crate::theme::Tokens::get();
    let mut workspace = app.ui.docking.clone();
    if let Err(error) = workspace.validate().and_then(|()| sync(&mut workspace, app)) {
        app.set_status(format!("Panel layout could not be restored: {error}"));
        workspace = Workspace::default();
        let _ = sync(&mut workspace, app);
    }
    // Bodies can dispatch commands (including the drawing's typed command line).
    // Keep the canonical workspace accessible while rendering an immutable snapshot.
    let reveal = workspace.reveal_auto_hide.take();
    app.ui.docking = workspace.clone();
    let mut style = DockStyle::from_ui(ui);
    style.min_pane = 96.0;
    style.background = t.panel;
    style.tab_background = t.chrome;
    style.active_background = t.tab_active;
    style.text = t.text;
    style.inactive_text = t.text_dim;
    style.border = egui::Stroke::new(1.0, t.border);
    style.accent = t.accent;
    style.font = crate::theme::body();
    let mut visible_layout = workspace.layout.clone();
    for panel in &workspace.auto_hide {
        if visible_layout.contains(panel) {
            let _ = visible_layout.apply_with_permissions(Action::Close { panel: *panel }, permissions);
        }
    }
    let viewport = ui.available_rect_before_wrap();
    let (output, queued) = {
        let mut content = Content { app, queued: Vec::new() };
        let (canvas_rect, blockers) = auto_hide::show(&mut content, ui, &workspace, viewport, reveal);
        let output = ui
            .scope_builder(egui::UiBuilder::new().max_rect(canvas_rect), |ui| {
                DockArea::new(egui::Id::new("cadcraft-docking")).blocked_rects(&blockers).show_customized(
                    ui,
                    &visible_layout,
                    &style,
                    |panel| crate::i18n::t(panel.label()).into(),
                    permissions,
                    panel_limits,
                    &mut content,
                )
            })
            .inner;
        (output, content.queued)
    };
    for (command, params) in queued {
        let _ = app.run(command, params);
    }
    // Existing panel close buttons and commands still own their visibility switches.
    let rendered = workspace;
    let mut workspace = app.ui.docking.clone();
    let _ = sync(&mut workspace, app);
    let stale = workspace.layout != rendered.layout || workspace.hidden != rendered.hidden || workspace.auto_hide != rendered.auto_hide;
    if stale {
        craft_ui::docking::cancel_drag::<Panel>(ui.ctx(), egui::Id::new("cadcraft-docking"));
    }
    publish_visibility(app, &workspace);
    app.ui.docking = workspace;
    if stale {
        return;
    }
    for action in output.actions {
        let Some(action) = auto_hide::canonical_action(action, &visible_layout, &app.ui.docking.layout) else {
            continue;
        };
        if let Err(error) =
            serde_json::to_value(action).map_err(|error| error.to_string()).and_then(|action| command(app, false, &json!({"action": action})))
        {
            app.set_status(format!("Panel move was not applied: {error}"));
        }
    }
}

fn panel_limits(panel: &Panel) -> PanelLimits {
    let (min, max_width) = match panel {
        Panel::Canvas => (egui::vec2(300.0, 180.0), 1_000_000.0),
        Panel::QuickCalc => (egui::vec2(240.0, 160.0), 600.0),
        Panel::ToolSets => (egui::vec2(220.0, 150.0), 480.0),
        Panel::Layers | Panel::Properties | Panel::ExternalRefs => (egui::vec2(220.0, 100.0), 720.0),
    };
    PanelLimits { min, max: egui::vec2(max_width, 1_000_000.0) }
}
