//! Transient contextual tabs: selection/editor state never becomes a saved active tab.
use super::*;
use cadcraft_engine::doc::EntityKind;

pub const TABS: &[Tab] = &[
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
}

pub fn available(app: &CadApp) -> Option<&'static Tab> {
    if !app.ui.ribbon.contextual_tabs || app.ui.start_tab {
        return None;
    }
    let state = app.session.state().ok()?;
    let id = if state.block_edit.is_some() {
        "blockEditor"
    } else {
        let selection = app.session.selection();
        let first = state.doc.entity(*selection.first()?)?;
        let id = match first.kind {
            EntityKind::Hatch(_) => "hatchEditor",
            EntityKind::Table(_) => "tableEditor",
            _ => return None,
        };
        if !selection.iter().all(|h| {
            state.doc.entity(*h).is_some_and(|e| {
                matches!((&first.kind, &e.kind), (EntityKind::Hatch(_), EntityKind::Hatch(_)) | (EntityKind::Table(_), EntityKind::Table(_)))
            })
        }) {
            return None;
        }
        id
    };
    TABS.iter().find(|t| t.id == id)
}

pub fn sync(app: &mut CadApp) {
    let key = available(app).and_then(|t| app.session.state().ok().map(|s| (s.uid, t.id)));
    if key != app.ribbon_context.key {
        app.ribbon_context.active = key.is_some();
        app.ribbon_context.key = key;
    }
}

pub fn active(app: &CadApp) -> Option<&'static Tab> {
    if app.ribbon_context.active { available(app).or_else(|| app.ui.ribbon.active()) } else { app.ui.ribbon.active() }
}
