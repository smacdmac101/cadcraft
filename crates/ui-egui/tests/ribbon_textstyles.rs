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
    app.run("style", json!({"name":"Review","current":false,"widthFactor":0.8})).unwrap();
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
    h.get_by_label("Ribbon Text Style").click();
    h.run_steps(4);
    h.get_by_label(&format!("Ribbon Text Style / {name}")).scroll_to_me();
    h.run_steps(4);
    h.get_by_label(&format!("Ribbon Text Style / {name}")).click();
    h.run_steps(4);
}
fn dim(h: &mut Harness<'_, CadApp>, y: i32) -> Handle {
    let command = if y == 0 { "text" } else { "mtext" };
    let r = h.state_mut().run(command, json!({"at":[0,y],"text":"Style sample","height":3})).unwrap();
    Handle::parse_hex(r["handle"].as_str().unwrap()).unwrap()
}
fn style(h: &Harness<'_, CadApp>, handle: Handle) -> String {
    match &h.state().session.doc().unwrap().entity(handle).unwrap().kind {
        EntityKind::Text(d) => d.style.clone(),
        EntityKind::MText(d) => d.style.clone(),
        _ => panic!("not text"),
    }
}
#[test]
fn current_selected_and_mixed_text_styles_edit_and_undo_independently() {
    let mut h = harness();
    let a = dim(&mut h, 0);
    pick(&mut h, "Review");
    let b = dim(&mut h, 50);
    pick(&mut h, "Standard");
    h.state_mut().session.set_selection(vec![a, b]);
    h.run_steps(4);
    assert_eq!(h.get_by_label("Ribbon Text Style").value().as_deref(), Some(""));
    pick(&mut h, "Review");
    assert_eq!(style(&h, a), "Review");
    assert_eq!(style(&h, b), "Review");
    assert_eq!(h.state().session.doc().unwrap().header.str("TEXTSTYLE", ""), "Standard");
    h.state_mut().run("undo", json!({})).unwrap();
    assert_eq!(style(&h, a), "Standard");
    assert_eq!(style(&h, b), "Review");
    h.state_mut().run("redo", json!({})).unwrap();
    assert_eq!(style(&h, a), "Review");
}
#[test]
fn non_text_selection_is_blank_and_cannot_assign_a_style_or_change_current() {
    let mut h = harness();
    let r = h.state_mut().run("line", json!({"points":[[0,0],[50,0]]})).unwrap();
    let line = Handle::parse_hex(r["handles"][0].as_str().unwrap()).unwrap();
    h.state_mut().session.set_selection(vec![line]);
    h.run_steps(4);
    assert_eq!(h.get_by_label("Ribbon Text Style").value().as_deref(), Some(""));
    h.get_by_label("Ribbon Text Style").click();
    h.run_steps(4);
    assert!(h.get_by_label("Ribbon Text Style / Review").accesskit_node().is_disabled());
    h.key_press(Key::Escape);
    h.run_steps(4);
    assert_eq!(h.state().session.doc().unwrap().header.str("TEXTSTYLE", ""), "Standard");
}
#[test]
fn locked_text_guard_and_first_escape_preserve_selection() {
    let mut h = harness();
    let a = dim(&mut h, 0);
    h.state_mut().run("layer.set", json!({"name":"0","locked":true})).unwrap();
    h.state_mut().session.set_selection(vec![a]);
    h.run_steps(4);
    h.get_by_label("Ribbon Text Style").click();
    h.run_steps(4);
    assert!(h.get_by_label("Ribbon Text Style / Review").accesskit_node().is_disabled());
    h.key_press(Key::Escape);
    h.run_steps(4);
    assert_eq!(h.state().session.selection(), vec![a]);
    assert_eq!(style(&h, a), "Standard");
}
#[test]
fn detached_text_style_selector_returns_and_opens_real_manager() {
    let mut h = harness();
    h.get_by_label("Ribbon Text Style").click_secondary();
    h.run_steps(4);
    h.get_by_label("Float Panel").click();
    h.run_steps(4);
    pick(&mut h, "Review");
    assert_eq!(h.state().session.doc().unwrap().header.str("TEXTSTYLE", ""), "Review");
    h.get_by_label("Return to Ribbon").click();
    h.run_steps(4);
    h.get_by_label("Ribbon Text Style").click();
    h.run_steps(4);
    h.get_by_label("Manage Text Styles").click();
    h.run_steps(4);
    assert_eq!(h.state().ui.dialog.as_deref(), Some("textstyle"));
}
