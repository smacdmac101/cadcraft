//! Transient contextual tabs: selection/editor state never becomes a saved active tab.
use super::*;
use cadcraft_engine::doc::EntityKind;

pub const TABS: &[Tab] = &[
    Tab {
        id: "layout",
        label: "Layout",
        panels: &[
            panel("layout", "Layout", &[(Icon::New, "layout.new", "New Layout"), (Icon::PageSetup, "pagesetup", "Page Setup Manager")], None),
            panel(
                "viewports",
                "Layout Viewports",
                &[
                    (Icon::Rectangle, "mview", "Viewports"),
                    (Icon::Polygon, "mview.polygonal", "Polygonal Viewport"),
                    (Icon::Trim, "vpclip", "Clip Viewport"),
                ],
                None,
            ),
            panel("space", "Space", &[(Icon::ZoomWindow, "mspace", "Model Space (in viewport)"), (Icon::PageSetup, "pspace", "Paper Space")], None),
            panel(
                "plot",
                "Plot",
                &[(Icon::Plot, "plot", "Print"), (Icon::Preview, "preview", "Plot Preview"), (Icon::Publish, "publish", "Batch Publish")],
                None,
            ),
        ],
    },
    Tab {
        id: "hatchEditor",
        label: "Hatch Editor",
        panels: &[panel(
            "edit",
            "Hatch",
            &[
                (Icon::Hatch, "hatchedit", "Hatch Edit"),
                (Icon::Properties, "ui.panel.properties", "Properties Inspector"),
                (Icon::MatchProp, "matchprop", "Match Properties"),
            ],
            None,
        )],
    },
    Tab {
        id: "tableEditor",
        label: "Table",
        panels: &[panel(
            "cells",
            "Table Cells",
            &[
                (Icon::Table, "tabledit", "Edit Table Cell"),
                (Icon::Export, "tableexport", "Export Table"),
                (Icon::Properties, "ui.panel.properties", "Properties Inspector"),
            ],
            None,
        )],
    },
    Tab {
        id: "blockEditor",
        label: "Block Editor",
        panels: &[
            palette("draw", "Draw", "Draw"),
            palette("modify", "Modify", "Modify"),
            panel(
                "definition",
                "Block Definition",
                &[(Icon::Save, "bsave", "Save Block"), (Icon::SaveAs, "bsaveas", "Save Block As"), (Icon::Close, "bclose", "Close Block Editor")],
                None,
            ),
        ],
    },
];

#[derive(Default)]
pub struct Runtime {
    key: Option<(u64, &'static str)>,
    pub active: bool,
    pub selected: Option<&'static str>,
}

pub fn available(app: &CadApp) -> Option<&'static Tab> {
    if !app.ui.ribbon.contextual_tabs || app.ui.start_tab {
        return None;
    }
    let state = app.session.state().ok()?;
    let selection_tab = || {
        let selection = app.session.selection();
        let first = state.doc.entity(*selection.first()?)?;
        let id = match first.kind {
            EntityKind::Hatch(_) => "hatchEditor",
            EntityKind::Table(_) => "tableEditor",
            _ => return None,
        };
        selection
            .iter()
            .all(|h| {
                state.doc.entity(*h).is_some_and(|e| {
                    matches!((&first.kind, &e.kind), (EntityKind::Hatch(_), EntityKind::Hatch(_)) | (EntityKind::Table(_), EntityKind::Table(_)))
                })
            })
            .then_some(id)
    };
    let id = if state.block_edit.is_some() {
        "blockEditor"
    } else if let Some(id) = selection_tab() {
        id
    } else if matches!(app.session.layout_space(), cadcraft_doc::Space::Paper(_)) {
        "layout"
    } else {
        return None;
    };
    TABS.iter().find(|t| t.id == id)
}

pub fn available_tabs(app: &CadApp) -> impl Iterator<Item = &'static Tab> {
    let primary = available(app);
    let layout = (primary.is_some_and(|t| t.id != "layout" && t.id != "blockEditor")
        && matches!(app.session.layout_space(), cadcraft_doc::Space::Paper(_)))
    .then(|| TABS.iter().find(|t| t.id == "layout"))
    .flatten();
    primary.into_iter().chain(layout)
}

pub fn is_available(app: &CadApp, id: &str) -> bool {
    available_tabs(app).any(|t| t.id == id)
}

pub fn sync(app: &mut CadApp) {
    let key = available(app).and_then(|t| app.session.state().ok().map(|s| (s.uid, t.id)));
    if key != app.ribbon_context.key {
        app.ribbon_context.active = key.is_some();
        app.ribbon_context.key = key;
        app.ribbon_context.selected = None;
    }
}

pub fn active(app: &CadApp) -> Option<&'static Tab> {
    if app.ribbon_context.active {
        available_tabs(app).find(|t| Some(t.id) == app.ribbon_context.selected).or_else(|| available(app)).or_else(|| app.ui.ribbon.active())
    } else {
        app.ui.ribbon.active()
    }
}
