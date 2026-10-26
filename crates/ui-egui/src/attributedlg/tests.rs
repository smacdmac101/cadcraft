use super::*;
use crate::Services;
use cadcraft_engine::Session;
use egui::{Event, Key, Modifiers, vec2};
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT, Queryable};

fn app() -> CadApp {
    let mut app = CadApp::new(Session::default(), Services::default());
    app.run("attdef", json!({"tag":"DEVICE_TYPE", "default":"POWER ADAPTER", "at":[0,0], "height":1})).unwrap();
    let handles: Vec<_> = app.session.doc().unwrap().model.iter().map(|e| e.handle.hex()).collect();
    app.run("block", json!({"name":"Equipment", "handles":handles, "base":[0,0], "keep":"delete"})).unwrap();
    let inserted = app.run("insert", json!({"name":"Equipment", "at":[0,0]})).unwrap();
    app.run("select", json!({"handles":[inserted["handle"]]})).unwrap();
    app.start("eattedit");
    assert_eq!(app.ui.dialog.as_deref(), Some("attedit"));
    app
}

fn harness() -> Harness<'static, CadApp> {
    Harness::builder().with_size(vec2(800.0, 600.0)).build_ui_state(|ui, app| show(app, ui.ctx()), app())
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

fn full_harness() -> Harness<'static, CadApp> {
    let mut app = app();
    app.run("ui.toolsets.ribbon", json!({})).unwrap();
    app.run("view.set", json!({"center":[50,50],"height":100})).unwrap();
    app.canvas.zoom_pending = false;
    let mut h = Harness::builder().with_size(vec2(1600.0, 1000.0)).with_step_dt(1.0 / 60.0).build_ui_state(
        |ui, app: &mut CadApp| {
            app.logic(ui.ctx());
            app.ui(ui);
        },
        app,
    );
    h.run_steps(5);
    h
}

fn click_at(h: &mut Harness<'_, CadApp>, pos: egui::Pos2) {
    h.event(Event::PointerMoved(pos));
    h.run_steps(2);
    for pressed in [true, false] {
        h.event(Event::PointerButton { pos, button: egui::PointerButton::Primary, pressed, modifiers: Modifiers::NONE });
        h.run_steps(1);
    }
    h.run_steps(2);
}

#[test]
fn attribute_editor_outside_click_keeps_canvas_selection_idle() {
    let mut h = full_harness();
    let selection = h.state().session.selection();
    let original = text(h.state());
    let outside = h.state().canvas.rect.unwrap().max - vec2(90.0, 180.0);
    assert!(h.state().session.pending_window.is_none());
    click_at(&mut h, outside);
    assert!(h.state().session.pending_window.is_none(), "an outside click must not start a canvas selection under the editor");
    assert_eq!(h.state().ui.dialog.as_deref(), Some("attedit"));
    assert_eq!(h.state().session.selection(), selection);
    assert_eq!(text(h.state()), original);
    let cancel = h.get_by_label("Cancel").rect().center();
    click_at(&mut h, cancel);
    assert!(h.state().ui.dialog.is_none(), "the shield must not cover editor controls after an outside click");
}

#[test]
fn attribute_editor_cancel_over_canvas_does_not_start_selection() {
    let mut h = full_harness();
    let selection = h.state().session.selection();
    let original = text(h.state());
    h.get_by_label("Text Options").click();
    h.run_steps(5);
    let cancel = h.get_by_label("Cancel").rect().center();
    assert!(h.state().canvas.rect.unwrap().contains(cancel), "fixture must put Cancel above the canvas");
    click_at(&mut h, cancel);
    assert!(h.state().ui.dialog.is_none());
    assert!(h.state().session.pending_window.is_none());
    assert_eq!(h.state().session.selection(), selection);
    assert_eq!(text(h.state()), original);
}

#[test]
fn attribute_editor_blocks_drawing_shortcuts_and_command_text() {
    let mut h = full_harness();
    let selection = h.state().session.selection();
    let original = text(h.state());
    let grid = h.state().session.settings.gridmode;
    h.key_press(Key::Delete);
    h.run_steps(2);
    assert_eq!(h.state().session.selection(), selection);
    assert_eq!(text(h.state()), original);
    h.key_press(Key::F7);
    h.event(Event::Text("line".into()));
    h.run_steps(2);
    assert_eq!(h.state().session.settings.gridmode, grid);
    assert!(h.state().cmd.buffer.is_empty());
    assert!(h.state().session.running.is_none());
    h.get_by_label("Cancel").click();
    h.run_steps(3);
    h.event(Event::Text("line".into()));
    h.run_steps(2);
    assert_eq!(h.state().cmd.buffer, "line", "command typing resumes after closing the editor");
}

#[test]
fn attribute_editor_outside_click_does_not_feed_a_running_line() {
    let mut h = full_harness();
    let selection = h.state().session.selection();
    h.state_mut().session.start("line").unwrap();
    h.state_mut().session.input(cadcraft_engine::Input::Point(cadcraft_engine::geom::Vec2::new(25.0, 25.0))).unwrap();
    h.state_mut().session.set_selection(selection);
    h.run_steps(2);
    let count = h.state().session.doc().unwrap().entity_count();
    let prompt = h.state().session.prompt_text();
    let outside = h.state().canvas.rect.unwrap().max - vec2(90.0, 180.0);
    click_at(&mut h, outside);
    assert_eq!(h.state().session.doc().unwrap().entity_count(), count);
    assert_eq!(h.state().session.prompt_text(), prompt);
    assert_eq!(h.state().ui.dialog.as_deref(), Some("attedit"));
}

#[test]
fn modal_tab_navigation_is_not_rewritten_as_command_text() {
    for modal in [false, true] {
        let ctx = egui::Context::default();
        let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
            if modal {
                egui::Modal::new(egui::Id::new("test_modal_tab")).show(ui.ctx(), |ui| {
                    ui.label("Modal");
                });
            }
        });
        output.textures_delta.clear();
        let mut raw = egui::RawInput::default();
        raw.events.push(Event::Key { key: Key::Tab, physical_key: Some(Key::Tab), pressed: true, repeat: false, modifiers: Modifiers::NONE });
        crate::cmdline::capture_tab(&ctx, &mut raw);
        if modal {
            assert!(matches!(raw.events.first(), Some(Event::Key { key: Key::Tab, .. })));
        } else {
            assert!(matches!(raw.events.first(),Some(Event::Text(t)) if t == "\t"));
        }
    }
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

fn attribute_props(app: &CadApp) -> cadcraft_doc::AttribProps {
    let h = app.session.selection()[0];
    let EntityKind::Insert(i) = &app.session.doc().unwrap().entity(h).unwrap().kind else { panic!("insert") };
    i.attribs[0].props.clone()
}

#[test]
fn attribute_properties_controls_apply_loaded_choices_and_cancel_discards_drafts() {
    let mut h = harness();
    h.state_mut().run("layer.new", json!({"name":"AttributeTest","color":"cyan","current":false})).unwrap();
    h.state_mut().run("linetype", json!({"load":"*"})).unwrap();
    h.run_steps(3);
    let original = attribute_props(h.state());
    h.get_by_label("Properties").click();
    h.run_steps(3);
    h.get_by_label("Apply").click();
    h.run_steps(3);
    assert_eq!(attribute_props(h.state()), original, "no-op Apply preserves inherited overrides");
    for (label, choice) in [("Layer", "AttributeTest"), ("Linetype", "DASHED"), ("Lineweight", "0.35 mm")] {
        h.get_by_label(label).click();
        h.run_steps(3);
        h.get_by_label(choice).scroll_to_me();
        h.run_steps(3);
        h.get_by_label(choice).click();
        h.run_steps(3);
    }
    edit_field(&mut h, "Color", "Red");
    h.get_by_label("Apply").click();
    h.run_steps(3);
    let edited = attribute_props(h.state());
    assert_eq!(edited.layer.as_deref(), Some("AttributeTest"));
    assert_eq!(edited.linetype.as_deref(), Some("DASHED"));
    assert_eq!(edited.color, Some(cadcraft_doc::color::Color::Index(1)));
    assert_eq!(edited.lineweight, Some(cadcraft_doc::Lineweight::Mm100(35)));
    edit_field(&mut h, "Color", "Blue");
    h.get_by_label("Cancel").click();
    h.run_steps(3);
    assert_eq!(attribute_props(h.state()), edited);
    h.state_mut().run("undo", json!({})).unwrap();
    let EntityKind::Insert(i) = &h.state().session.doc().unwrap().model.iter().find(|e| matches!(e.kind, EntityKind::Insert(_))).unwrap().kind else {
        panic!("insert")
    };
    assert_eq!(i.attribs[0].props, original);
}

#[test]
fn attribute_properties_errors_keep_all_tabs_unapplied_and_editor_open() {
    let mut h = harness();
    h.run_steps(3);
    let original = text(h.state());
    let original_props = attribute_props(h.state());
    edit_field(&mut h, "DEVICE_TYPE", "Must not change");
    h.get_by_label("Text Options").click();
    h.run_steps(3);
    edit_field(&mut h, "Height", "9");
    h.get_by_label("Properties").click();
    h.run_steps(3);
    edit_field(&mut h, "Color", "Invalid color");
    h.get_by_label("OK").click();
    h.run_steps(3);
    assert_eq!(text(h.state()), original);
    assert_eq!(attribute_props(h.state()), original_props);
    assert_eq!(h.state().ui.dialog.as_deref(), Some("attedit"));
    assert!(h.query_by_label("invalid parameters for `attedit`: invalid attribute color").is_some());
    h.get_by_label("Cancel").click();
    h.run_steps(3);
    assert_eq!(text(h.state()), original);
}

#[test]
fn attribute_properties_noop_apply_does_not_replay_an_applied_draft_over_external_edits() {
    let mut h = harness();
    h.run_steps(3);
    h.get_by_label("Properties").click();
    h.run_steps(3);
    edit_field(&mut h, "Color", "Red");
    h.get_by_label("Apply").click();
    h.run_steps(3);
    let handle = h.state().session.selection()[0];
    h.state_mut().run("attedit", json!({"handle":handle.hex(),"properties":{"DEVICE_TYPE":{"color":"Blue","lineweight":0.5}}})).unwrap();
    let external = attribute_props(h.state());
    h.get_by_label("Apply").click();
    h.run_steps(3);
    assert_eq!(attribute_props(h.state()), external);
    edit_field(&mut h, "Color", "Cyan");
    h.get_by_label("Apply").click();
    h.run_steps(3);
    assert_eq!(attribute_props(h.state()).color, Some(cadcraft_doc::color::Color::Index(4)));
    assert_eq!(attribute_props(h.state()).lineweight, external.lineweight);
}

#[test]
fn attribute_properties_show_definition_defaults_without_creating_overrides() {
    let mut fixture = app();
    fixture.run("linetype", json!({"load":"*"})).unwrap();
    let block = std::sync::Arc::make_mut(fixture.session.doc_mut().unwrap().blocks.get_mut("Equipment").unwrap());
    let definition = block.entities.iter().find(|e| matches!(e.kind, EntityKind::AttDef(_))).unwrap().handle;
    assert!(block.entities.modify(definition, |definition| {
        definition.common.color = cadcraft_doc::color::Color::Index(4);
        definition.common.linetype = "DASHED".into();
        definition.common.lineweight = cadcraft_doc::Lineweight::Mm100(50);
    }));
    let target = fixture.session.selection()[0];
    fixture
        .session
        .doc_mut()
        .unwrap()
        .modify_entity(target, |e| {
            let EntityKind::Insert(i) = &mut e.kind else { panic!("insert") };
            i.attribs[0].props = cadcraft_doc::AttribProps::default();
        })
        .unwrap();
    let mut h = Harness::builder().with_size(vec2(800.0, 600.0)).build_ui_state(|ui, app| show(app, ui.ctx()), fixture);
    h.run_steps(3);
    h.get_by_label("Properties").click();
    h.run_steps(3);
    assert_eq!(h.get_by_role_and_label(egui::accesskit::Role::TextInput, "Color").accesskit_node().value().as_deref(), Some("Cyan"));
    h.get_by_label("Apply").click();
    h.run_steps(3);
    assert!(attribute_props(h.state()).is_empty(), "displaying inherited definition defaults must not materialize per-reference overrides");
}
