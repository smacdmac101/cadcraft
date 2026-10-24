use super::*;
use crate::Services;
use cadcraft_engine::Session;
use egui::{Event, Key, Modifiers};
use egui_kittest::Harness;
use egui_kittest::kittest::{By, Queryable};

#[test]
fn selected_block_attributes_are_editable_and_undoable() {
    let mut app = CadApp::new(Session::default(), Services::default());
    app.run("attdef", json!({"tag":"MODEL", "default":"Original model", "at":[0,0], "height":1})).unwrap();
    let handles: Vec<_> = app.session.doc().unwrap().model.iter().map(|e| e.handle.hex()).collect();
    app.run("block", json!({"name":"Equipment", "handles":handles, "base":[0,0], "keep":"delete"})).unwrap();
    let inserted = app.run("insert", json!({"name":"Equipment", "at":[0,0]})).unwrap();
    let handle = inserted["handle"].clone();
    app.run("select", json!({"handles":[handle]})).unwrap();
    let mut h = Harness::builder().with_size(vec2(460.0, 1200.0)).build_ui_state(
        |ui, app: &mut CadApp| {
            app.logic(&ui.ctx().clone());
            properties_section(app, ui);
        },
        app,
    );
    h.run_steps(3);
    h.get(By::new().role(egui::accesskit::Role::TextInput).value("Original model")).click();
    h.run_steps(2);
    h.key_press_modifiers(Modifiers::COMMAND, Key::A);
    h.event(Event::Text("Replacement model".into()));
    h.run_steps(2);
    h.key_press(Key::Enter);
    h.run_steps(3);
    assert!(h.query(By::new().role(egui::accesskit::Role::TextInput).value("Replacement model")).is_some());
    let id = cadcraft_doc::Handle::parse_hex(handle.as_str().unwrap()).unwrap();
    let value = |app: &CadApp| {
        let EntityKind::Insert(i) = &app.session.doc().unwrap().entity(id).unwrap().kind else { panic!("insert") };
        i.attribs[0].text.value.clone()
    };
    assert_eq!(value(h.state()), "Replacement model");
    h.state_mut().run("undo", json!({})).unwrap();
    h.state_mut().run("select", json!({"handles":[handle]})).unwrap();
    h.run_steps(3);
    assert_eq!(value(h.state()), "Original model");
    assert!(h.query(By::new().role(egui::accesskit::Role::TextInput).value("Original model")).is_some());
}
