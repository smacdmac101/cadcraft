//! Current/selected/mixed linetypes and loaded-library actions through real ribbon widgets.
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
    app.run("linetype", json!({"load":"DASHED"})).unwrap();
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
fn pick(h: &mut Harness<'_, CadApp>, name: &str) {
    h.get_by_label("Ribbon Linetype").click();
    h.run_steps(4);
    h.get_by_label(name).scroll_to_me();
    h.run_steps(4);
    h.get_by_label(name).click();
    h.run_steps(4);
}
fn line(h: &mut Harness<'_, CadApp>, y: i32) -> Handle {
    let result = h.state_mut().run("line", json!({"points":[[0,y],[100,y]]})).unwrap();
    Handle::parse_hex(result["handles"][0].as_str().unwrap()).unwrap()
}
#[test]
fn current_selected_and_mixed_linetypes_are_distinct_and_undoable() {
    let mut h = harness();
    let a = line(&mut h, 0);
    pick(&mut h, "DASHED");
    let b = line(&mut h, 40);
    pick(&mut h, "Continuous");
    h.state_mut().session.set_selection(vec![a, b]);
    h.run_steps(4);
    assert_eq!(h.get_by_label("Ribbon Linetype").accesskit_node().value().as_deref(), Some(""));
    pick(&mut h, "ByBlock");
    for handle in [a, b] {
        assert_eq!(h.state().session.doc().unwrap().entity(handle).unwrap().common.linetype, "ByBlock");
    }
    assert_eq!(h.state().session.doc().unwrap().header.str("CELTYPE", ""), "Continuous");
    h.state_mut().run("undo", json!({})).unwrap();
    assert_eq!(h.state().session.doc().unwrap().entity(a).unwrap().common.linetype, "ByLayer");
    assert_eq!(h.state().session.doc().unwrap().entity(b).unwrap().common.linetype, "DASHED");
    h.state_mut().run("redo", json!({})).unwrap();
    assert_eq!(h.state().session.doc().unwrap().entity(b).unwrap().common.linetype, "ByBlock");
}
#[test]
fn current_linetype_can_change_during_line_without_answering_its_point_prompt() {
    let mut h = harness();
    h.state_mut().session.start("line").unwrap();
    h.state_mut().session.input(Input::Point(Vec2::new(0.0, 0.0))).unwrap();
    h.run_steps(4);
    pick(&mut h, "DASHED");
    assert_eq!(h.state().session.running.as_ref().unwrap().id, "line");
    h.state_mut().session.input(Input::Point(Vec2::new(100.0, 0.0))).unwrap();
    assert_eq!(h.state().session.doc().unwrap().space(&cadcraft_engine::doc::Space::Model).unwrap().iter().next().unwrap().common.linetype, "DASHED");
    h.get_by_label("Ribbon Linetype").click();
    h.run_steps(4);
    h.key_press(Key::Escape);
    h.run_steps(4);
    assert_eq!(h.state().session.running.as_ref().unwrap().id, "line");
}
#[test]
fn loading_a_library_linetype_does_not_silently_set_it_current_or_edit_selection() {
    let mut h = harness();
    let a = line(&mut h, 0);
    h.state_mut().session.set_selection(vec![a]);
    h.run_steps(4);
    h.get_by_label("Ribbon Linetype").click();
    h.run_steps(4);
    h.get_by_label("Load Linetype").click();
    h.run_steps(4);
    h.get_by_label("CENTER").scroll_to_me();
    h.run_steps(4);
    h.get_by_label("CENTER").click();
    h.run_steps(4);
    assert!(h.state().session.doc().unwrap().linetype("CENTER").is_some());
    assert_eq!(h.state().session.doc().unwrap().entity(a).unwrap().common.linetype, "ByLayer");
    assert_eq!(h.state().session.doc().unwrap().header.str("CELTYPE", ""), "ByLayer");
    h.key_press(Key::Escape);
    h.run_steps(4);
    pick(&mut h, "CENTER");
    assert_eq!(h.state().session.doc().unwrap().entity(a).unwrap().common.linetype, "CENTER");
}
#[test]
fn detached_linetype_selector_has_context_menu_and_locked_selection_guard() {
    let mut h = harness();
    h.get_by_label("Ribbon Linetype").click_secondary();
    h.run_steps(4);
    h.get_by_label("Float Panel").click();
    h.run_steps(4);
    pick(&mut h, "DASHED");
    let a = line(&mut h, 0);
    h.state_mut().run("layer.set", json!({"name":"0","locked":true})).unwrap();
    h.state_mut().session.set_selection(vec![a]);
    h.run_steps(4);
    h.get_by_label("Ribbon Linetype").click();
    h.run_steps(4);
    assert!(h.get_by_label("ByBlock").accesskit_node().is_disabled());
    h.key_press(Key::Escape);
    h.run_steps(4);
    h.state_mut().run("ui.ribbon.return", json!({"tab":"home","panel":"properties"})).unwrap();
    h.run_steps(4);
    assert!(h.query_by_label("Ribbon Linetype").is_some());
    h.get_by_label("More: Properties").click();
    h.run_steps(4);
    assert!(h.query_by_label("Quick Select...").is_some());
}
