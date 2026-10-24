//! Attribute value editing for a selected block. All edits go through ATTEDIT for undo.
use cadcraft_doc::{EntityKind, Handle};
use serde_json::json;

use crate::CadApp;

#[derive(Clone)]
struct State {
    drawing: u64,
    handle: Handle,
    name: String,
    values: Vec<(String, String, String, bool)>,
}

pub(crate) fn open(app: &mut CadApp, handle: Handle) -> bool {
    let has_attributes =
        app.session.doc().ok().and_then(|d| d.entity(handle)).is_some_and(|e| matches!(&e.kind, EntityKind::Insert(i) if !i.attribs.is_empty()));
    if has_attributes {
        app.session.set_selection(vec![handle]);
        app.ui.dialog = Some("attedit".into());
    }
    has_attributes
}

pub(crate) fn open_selected(app: &mut CadApp) -> bool {
    let handle = app.session.state().ok().filter(|s| s.selection.len() == 1).and_then(|s| s.selection.first().copied());
    handle.is_some_and(|h| open(app, h))
}

pub(crate) fn show(app: &mut CadApp, ctx: &egui::Context) {
    let id = egui::Id::new("attribute_editor_state");
    if app.ui.dialog.as_deref() != Some("attedit") {
        ctx.data_mut(|d| d.remove::<State>(id));
        return;
    }
    let drawing = app.session.state().map_or(0, |s| s.uid);
    let selected = app.session.state().ok().and_then(|s| s.selection.first().copied());
    let Some((handle, name, attributes)) = selected.and_then(|h| {
        let EntityKind::Insert(i) = &app.session.doc().ok()?.entity(h)?.kind else { return None };
        Some((h, i.block.clone(), i.attribs.clone()))
    }) else {
        app.ui.dialog = None;
        ctx.data_mut(|d| d.remove::<State>(id));
        return;
    };
    let mut state = ctx.data(|d| d.get_temp::<State>(id)).filter(|s| s.drawing == drawing && s.handle == handle).unwrap_or_else(|| State {
        drawing,
        handle,
        name: app.session.doc().ok().and_then(|d| cadcraft_doc::dynblock::custom_properties(d, handle)).map_or(name, |(name, _)| name),
        values: attributes.iter().map(|a| (a.tag.clone(), a.prompt.clone(), a.text.value.clone(), a.constant)).collect(),
    });
    let mut open = true;
    let mut apply = false;
    let mut close = false;
    egui::Window::new("Edit Attributes").open(&mut open).default_width(520.0).show(ctx, |ui| {
        ui.label(format!("Block: {}", state.name));
        egui::ScrollArea::vertical().max_height(420.0).show(ui, |ui| {
            for (index, (tag, prompt, value, constant)) in state.values.iter_mut().enumerate() {
                ui.push_id(index, |ui| {
                    let label = ui.label(tag.as_str()).on_hover_text(prompt.as_str());
                    ui.add_enabled(!*constant, egui::TextEdit::singleline(value).desired_width(f32::INFINITY)).labelled_by(label.id);
                });
            }
        });
        ui.separator();
        ui.horizontal(|ui| {
            if ui.button("OK").clicked() {
                apply = true;
                close = true;
            }
            if ui.button("Apply").clicked() {
                apply = true;
            }
            if ui.button("Cancel").clicked() {
                close = true;
            }
        });
    });
    if apply {
        let values: serde_json::Map<_, _> = state
            .values
            .iter()
            .filter(|(tag, _, value, constant)| !constant && attributes.iter().any(|a| a.tag == *tag && a.text.value != *value))
            .map(|(tag, _, value, _)| (tag.clone(), json!(value)))
            .collect();
        if !values.is_empty() && app.run("attedit", json!({"handle": handle.hex(), "values": values})).is_err() {
            close = false;
        }
    }
    if !open || close || ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
        app.ui.dialog = None;
        ctx.data_mut(|d| d.remove::<State>(id));
    } else {
        ctx.data_mut(|d| d.insert_temp(id, state));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Services;
    use cadcraft_engine::Session;
    use egui::{Event, Key, Modifiers, vec2};
    use egui_kittest::Harness;
    use egui_kittest::kittest::Queryable;

    fn harness() -> Harness<'static, CadApp> {
        let mut app = CadApp::new(Session::default(), Services::default());
        app.run("attdef", json!({"tag":"DEVICE_TYPE", "default":"POWER ADAPTER", "at":[0,0], "height":1})).unwrap();
        let handles: Vec<_> = app.session.doc().unwrap().model.iter().map(|e| e.handle.hex()).collect();
        app.run("block", json!({"name":"Equipment", "handles":handles, "base":[0,0], "keep":"delete"})).unwrap();
        let inserted = app.run("insert", json!({"name":"Equipment", "at":[0,0]})).unwrap();
        app.run("select", json!({"handles":[inserted["handle"]]})).unwrap();
        app.start("eattedit");
        assert_eq!(app.ui.dialog.as_deref(), Some("attedit"));
        Harness::builder().with_size(vec2(800.0, 600.0)).build_ui_state(|ui, app| show(app, ui.ctx()), app)
    }

    fn value(app: &CadApp) -> String {
        let s = app.session.state().unwrap();
        let EntityKind::Insert(i) = &s.doc.model.iter().find(|e| matches!(e.kind, EntityKind::Insert(_))).unwrap().kind else { panic!("insert") };
        i.attribs[0].text.value.clone()
    }

    #[test]
    fn attribute_dialog_applies_values_and_undo_restores_them() {
        let mut h = harness();
        h.run_steps(3);
        h.get_by_role_and_label(egui::accesskit::Role::TextInput, "DEVICE_TYPE").click();
        h.run_steps(2);
        h.key_press_modifiers(Modifiers::COMMAND, Key::A);
        h.event(Event::Text("Replacement".into()));
        h.run_steps(2);
        h.get_by_label("Apply").click();
        h.run_steps(3);
        assert_eq!(value(h.state()), "Replacement");
        h.get_by_label("Cancel").click();
        h.run_steps(2);
        h.state_mut().run("undo", json!({})).unwrap();
        assert_eq!(value(h.state()), "POWER ADAPTER");
    }

    #[test]
    fn attribute_dialog_cancel_discards_unapplied_values() {
        let mut h = harness();
        h.run_steps(3);
        h.get_by_role_and_label(egui::accesskit::Role::TextInput, "DEVICE_TYPE").click();
        h.run_steps(2);
        h.key_press_modifiers(Modifiers::COMMAND, Key::A);
        h.event(Event::Text("Do not keep".into()));
        h.run_steps(2);
        h.get_by_label("Cancel").click();
        h.run_steps(2);
        assert_eq!(value(h.state()), "POWER ADAPTER");
        h.state_mut().run("attedit", json!({"values":{"DEVICE_TYPE":"Scripted"}})).unwrap();
        assert_eq!(value(h.state()), "Scripted");
        assert!(h.state().ui.dialog.is_none(), "JSON edits never open a dialog");
    }
}
