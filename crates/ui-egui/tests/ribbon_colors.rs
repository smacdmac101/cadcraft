//! Current, selected, mixed and inherited ribbon color semantics.
use cadcraft_color::{Color, Rgb};
use cadcraft_engine::{Input, Session, doc::Handle, geom::Vec2};
use cadcraft_ui_egui::{CadApp, Services};
use egui::{Key, vec2};
use egui_kittest::{
    Harness,
    kittest::{NodeT, Queryable},
};
use serde_json::json;

fn harness() -> Harness<'static, CadApp> {
    let mut app = CadApp::new(Session::default(), Services::default());
    app.run("new", json!({})).unwrap();
    app.run("ribbon", json!({})).unwrap();
    let mut h = Harness::builder().with_size(vec2(1800.0, 1000.0)).build_ui_state(
        |ui, app: &mut CadApp| {
            app.logic(ui.ctx());
            app.ui(ui);
        },
        app,
    );
    h.run_steps(4);
    h
}
fn pick(h: &mut Harness<'_, CadApp>, label: &str) {
    h.get_by_label("Ribbon Color").click();
    h.run_steps(4);
    h.get_by_label(label).click();
    h.run_steps(4);
}
fn line(h: &mut Harness<'_, CadApp>, y: i32) -> Handle {
    let result = h.state_mut().run("line", json!({"points":[[0,y],[100,y]]})).unwrap();
    Handle::parse_hex(result["handles"][0].as_str().unwrap()).unwrap()
}
#[test]
fn current_color_and_selected_colors_are_independent_undoable_and_mixed() {
    let mut h = harness();
    pick(&mut h, "Red");
    let a = line(&mut h, 0);
    pick(&mut h, "Green");
    let b = line(&mut h, 40);
    pick(&mut h, "Blue");
    h.state_mut().session.set_selection(vec![a, b]);
    h.run_steps(4);
    assert_eq!(h.get_by_label("Ribbon Color").accesskit_node().value().as_deref(), Some(""));
    pick(&mut h, "Yellow");
    for handle in [a, b] {
        assert_eq!(h.state().session.doc().unwrap().entity(handle).unwrap().common.color, Color::Index(2));
    }
    assert_eq!(h.state().session.doc().unwrap().header.i64("CECOLOR", 256), 5);
    h.state_mut().run("undo", json!({})).unwrap();
    assert_eq!(h.state().session.doc().unwrap().entity(a).unwrap().common.color, Color::Index(1));
    assert_eq!(h.state().session.doc().unwrap().entity(b).unwrap().common.color, Color::Index(3));
    h.state_mut().run("redo", json!({})).unwrap();
    assert_eq!(h.state().session.doc().unwrap().entity(a).unwrap().common.color, Color::Index(2));
}
#[test]
fn inherited_colors_on_different_layers_keep_logical_bylayer_and_current_rgb_is_exact() {
    let mut h = harness();
    h.state_mut().run("layer.new", json!({"name":"A","color":1})).unwrap();
    h.state_mut().run("layer.current", json!({"name":"A"})).unwrap();
    let a = line(&mut h, 0);
    h.state_mut().run("layer.new", json!({"name":"B","color":3})).unwrap();
    h.state_mut().run("layer.current", json!({"name":"B"})).unwrap();
    let b = line(&mut h, 40);
    h.state_mut().session.set_selection(vec![a, b]);
    h.run_steps(4);
    assert_eq!(h.get_by_label("Ribbon Color").accesskit_node().value().as_deref(), Some("ByLayer"));
    h.state_mut().session.set_selection(Vec::new());
    h.state_mut().run("color", json!({"color":"12,34,56"})).unwrap();
    h.run_steps(4);
    assert_eq!(h.get_by_label("Ribbon Color").accesskit_node().value().as_deref(), Some("12,34,56"));
    let c = line(&mut h, 80);
    assert_eq!(h.state().session.doc().unwrap().entity(c).unwrap().common.color, Color::True(Rgb(12, 34, 56)));
    pick(&mut h, "ByBlock");
    assert!(h.state().session.doc().unwrap().header.get("CECOLOR_RGB").is_none());
}
#[test]
fn color_picker_preserves_active_command_and_first_escape_only_closes_popup() {
    let mut h = harness();
    h.state_mut().session.start("line").unwrap();
    h.state_mut().session.input(Input::Point(Vec2::new(0.0, 0.0))).unwrap();
    h.run_steps(4);
    pick(&mut h, "Green");
    assert_eq!(h.state().session.running.as_ref().unwrap().id, "line");
    h.state_mut().session.input(Input::Point(Vec2::new(100.0, 0.0))).unwrap();
    let doc = h.state().session.doc().unwrap();
    assert_eq!(doc.space(&cadcraft_engine::doc::Space::Model).unwrap().iter().next().unwrap().common.color, Color::Index(3));
    h.get_by_label("Ribbon Color").click();
    h.run_steps(4);
    h.key_press(Key::Escape);
    h.run_steps(4);
    assert_eq!(h.state().session.running.as_ref().unwrap().id, "line");
}
#[test]
fn color_selector_supports_panel_context_menu_and_protects_locked_selection() {
    let mut h = harness();
    h.get_by_label("Ribbon Color").click_secondary();
    h.run_steps(4);
    h.get_by_label("Float Panel").click();
    h.run_steps(5);
    pick(&mut h, "Red");
    let a = line(&mut h, 0);
    h.state_mut().run("layer.set", json!({"name":"0","locked":true})).unwrap();
    h.state_mut().session.set_selection(vec![a]);
    h.run_steps(4);
    h.get_by_label("Ribbon Color").click();
    h.run_steps(4);
    assert!(h.get_by_label("Green").accesskit_node().is_disabled());
    h.key_press(Key::Escape);
    h.run_steps(4);
    h.state_mut().run("ui.ribbon.return", json!({"tab":"home","panel":"properties"})).unwrap();
    h.run_steps(5);
    assert!(h.query_by_label("Ribbon Color").is_some());
}

#[test]
fn custom_editor_applies_rgb_and_index_and_rejects_invalid_input() {
    let mut h = harness();
    for (input, expected) in [("#0c2238", Color::True(Rgb(12, 34, 56))), ("153", Color::Index(153))] {
        h.get_by_label("Ribbon Color").click();
        h.run_steps(4);
        h.get_by_label("Custom Color").click();
        h.run_steps(3);
        h.key_press_modifiers(egui::Modifiers::COMMAND, Key::A);
        h.run_steps(1);
        h.get_by_label("Custom Color").type_text(input);
        h.run_steps(4);
        h.get_by_label("Apply Color").click();
        h.run_steps(4);
        let handle = line(&mut h, 0);
        assert_eq!(h.state().session.doc().unwrap().entity(handle).unwrap().common.color, expected);
    }
    h.get_by_label("Ribbon Color").click();
    h.run_steps(4);
    h.get_by_label("Custom Color").click();
    h.run_steps(3);
    h.key_press_modifiers(egui::Modifiers::COMMAND, Key::A);
    h.run_steps(1);
    h.get_by_label("Custom Color").type_text("invalid color");
    h.run_steps(4);
    assert!(h.get_by_label("Apply Color").accesskit_node().is_disabled());
}
