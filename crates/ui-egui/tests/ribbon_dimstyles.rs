use cadcraft_engine::{
    Session,
    doc::{EntityKind, Handle},
};
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
    app.run("dimstyle", json!({"name":"Review","current":false,"textHeight":3.0})).unwrap();
    app.run("ui.ribbon.tab", json!({"tab":"annotate"})).unwrap();
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
    h.get_by_label("Ribbon Dimension Style").click();
    h.run_steps(4);
    h.get_by_label(&format!("Ribbon Dimension Style / {name}")).scroll_to_me();
    h.run_steps(4);
    h.get_by_label(&format!("Ribbon Dimension Style / {name}")).click();
    h.run_steps(4);
}
fn dim(h: &mut Harness<'_, CadApp>, y: i32) -> Handle {
    let r = h.state_mut().run("dimlinear", json!({"p1":[0,y],"p2":[100,y],"at":[50,y+20]})).unwrap();
    Handle::parse_hex(r["handle"].as_str().unwrap()).unwrap()
}
fn style(h: &Harness<'_, CadApp>, handle: Handle) -> String {
    match &h.state().session.doc().unwrap().entity(handle).unwrap().kind {
        EntityKind::Dimension(d) => d.style.clone(),
        _ => panic!("not dimension"),
    }
}
#[test]
fn current_selected_and_mixed_dimension_styles_edit_and_undo_independently() {
    let mut h = harness();
    let a = dim(&mut h, 0);
    pick(&mut h, "Review");
    let b = dim(&mut h, 50);
    pick(&mut h, "Standard");
    h.state_mut().session.set_selection(vec![a, b]);
    h.run_steps(4);
    assert_eq!(h.get_by_label("Ribbon Dimension Style").value().as_deref(), Some(""));
    pick(&mut h, "Review");
    assert_eq!(style(&h, a), "Review");
    assert_eq!(style(&h, b), "Review");
    assert_eq!(h.state().session.doc().unwrap().header.str("DIMSTYLE", ""), "Standard");
    h.state_mut().run("undo", json!({})).unwrap();
    assert_eq!(style(&h, a), "Standard");
    assert_eq!(style(&h, b), "Review");
    h.state_mut().run("redo", json!({})).unwrap();
    assert_eq!(style(&h, a), "Review");
}
#[test]
fn non_dimension_selection_is_blank_and_cannot_assign_a_style_or_change_current() {
    let mut h = harness();
    let r = h.state_mut().run("line", json!({"points":[[0,0],[50,0]]})).unwrap();
    let line = Handle::parse_hex(r["handles"][0].as_str().unwrap()).unwrap();
    h.state_mut().session.set_selection(vec![line]);
    h.run_steps(4);
    assert_eq!(h.get_by_label("Ribbon Dimension Style").value().as_deref(), Some(""));
    h.get_by_label("Ribbon Dimension Style").click();
    h.run_steps(4);
    assert!(h.get_by_label("Ribbon Dimension Style / Review").accesskit_node().is_disabled());
    h.key_press(Key::Escape);
    h.run_steps(4);
    assert_eq!(h.state().session.doc().unwrap().header.str("DIMSTYLE", ""), "Standard");
}
#[test]
fn locked_dimension_guard_and_first_escape_preserve_selection() {
    let mut h = harness();
    let a = dim(&mut h, 0);
    h.state_mut().run("layer.set", json!({"name":"0","locked":true})).unwrap();
    h.state_mut().session.set_selection(vec![a]);
    h.run_steps(4);
    h.get_by_label("Ribbon Dimension Style").click();
    h.run_steps(4);
    assert!(h.get_by_label("Ribbon Dimension Style / Review").accesskit_node().is_disabled());
    h.key_press(Key::Escape);
    h.run_steps(4);
    assert_eq!(h.state().session.selection(), vec![a]);
    assert_eq!(style(&h, a), "Standard");
}
#[test]
fn detached_dimension_style_selector_returns_and_opens_real_manager() {
    let mut h = harness();
    h.get_by_label("Ribbon Dimension Style").click_secondary();
    h.run_steps(4);
    h.get_by_label("Float Panel").click();
    h.run_steps(4);
    pick(&mut h, "Review");
    assert_eq!(h.state().session.doc().unwrap().header.str("DIMSTYLE", ""), "Review");
    h.get_by_label("Return to Ribbon").click();
    h.run_steps(4);
    h.get_by_label("Ribbon Dimension Style").click();
    h.run_steps(4);
    h.get_by_label("Manage Dimension Styles").click();
    h.run_steps(4);
    assert_eq!(h.state().ui.dialog.as_deref(), Some("dimstyle"));
}
