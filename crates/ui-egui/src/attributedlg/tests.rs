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

fn edit_field(h: &mut Harness<'static, CadApp>, label: &str, text: &str) {
    h.get_by_role_and_label(egui::accesskit::Role::TextInput, label).click();
    h.run_steps(2);
    h.key_press_modifiers(Modifiers::COMMAND, Key::A);
    h.event(Event::Text(text.into()));
    h.run_steps(2);
}

fn text(app: &CadApp) -> cadcraft_doc::Text {
    let EntityKind::Insert(i) = &app.session.doc().unwrap().model.iter().find(|e| matches!(e.kind, EntityKind::Insert(_))).unwrap().kind else {
        panic!("insert")
    };
    i.attribs[0].text.clone()
}

#[test]
fn text_options_and_values_apply_together_without_replaying_unchanged_drafts() {
    let mut h = harness();
    h.run_steps(3);
    let original = text(h.state());
    edit_field(&mut h, "DEVICE_TYPE", "Replacement");
    h.get_by_label("Text Options").click();
    h.run_steps(3);
    edit_field(&mut h, "Height", "4");
    edit_field(&mut h, "Rotation (degrees)", "15");
    h.get_by_label("Apply").click();
    h.run_steps(3);
    let changed = text(h.state());
    assert_eq!((changed.value.as_str(), changed.height, changed.rotation), ("Replacement", 4.0, 15_f64.to_radians()));
    assert_eq!(h.state().ui.dialog.as_deref(), Some("attedit"));
    let handle = h.state().session.selection()[0];
    h.state_mut().run("attedit", json!({"handle":handle.hex(),"values":{"DEVICE_TYPE":"External"},"formats":{"DEVICE_TYPE":{"height":2}}})).unwrap();
    let external = text(h.state());
    h.run_steps(3);
    h.get_by_label("Apply").click();
    h.run_steps(3);
    assert_eq!(text(h.state()), external, "unchanged drafts must not replay applied edits after an external edit");
    edit_field(&mut h, "Height", "6");
    h.get_by_label("Apply").click();
    h.run_steps(3);
    assert_eq!(text(h.state()).height, 6.0);
    assert_eq!(text(h.state()).value, external.value);
    assert_eq!(text(h.state()).rotation, external.rotation);
    h.get_by_label("Cancel").click();
    h.run_steps(3);
    h.state_mut().run("undo", json!({})).unwrap();
    assert_eq!(text(h.state()), external);
    h.state_mut().run("undo", json!({})).unwrap();
    assert_eq!(text(h.state()), changed, "no-op Apply must not add an undo record");
    h.state_mut().run("undo", json!({})).unwrap();
    assert_eq!(text(h.state()), original, "one Undo restores the initial combined value/format edit");
}

#[test]
fn cancel_discards_unapplied_text_options() {
    let mut h = harness();
    h.run_steps(3);
    let original = text(h.state());
    h.get_by_label("Text Options").click();
    h.run_steps(3);
    edit_field(&mut h, "Height", "9");
    h.get_by_label("Cancel").click();
    h.run_steps(3);
    assert_eq!(text(h.state()), original);
    assert!(h.state().ui.dialog.is_none());
}

#[test]
fn invalid_text_options_leave_the_editor_open_without_applying_values() {
    let mut h = harness();
    h.run_steps(3);
    let original = text(h.state());
    edit_field(&mut h, "DEVICE_TYPE", "Must not change");
    h.get_by_label("Text Options").click();
    h.run_steps(3);
    edit_field(&mut h, "Height", "NaN");
    h.get_by_label("OK").click();
    h.run_steps(3);
    assert_eq!(text(h.state()), original);
    assert_eq!(h.state().ui.dialog.as_deref(), Some("attedit"));
    assert!(h.query_by_label("DEVICE_TYPE: height must be a finite number").is_some());
    edit_field(&mut h, "Height", "5");
    h.get_by_label("OK").click();
    h.run_steps(3);
    assert_eq!(text(h.state()).height, 5.0);
    assert_eq!(value(h.state()), "Must not change");
    assert!(h.state().ui.dialog.is_none());
}
