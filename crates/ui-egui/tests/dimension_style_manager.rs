//! The dimension manager routes aliases, creates styles and changes the current style.
use cadcraft_engine::Session;
use cadcraft_ui_egui::{CadApp, Services, dialogs};
use egui::vec2;
use egui_kittest::{Harness, kittest::Queryable};
use serde_json::json;
#[test]
fn dimension_manager_creates_sets_current_and_undoes_without_opening_from_json() {
    let mut app = CadApp::new(Session::default(), Services::default());
    app.run("new", json!({})).unwrap();
    app.run("dimstyle", json!({"name":"Scripted","current":false})).unwrap();
    assert!(app.ui.dialog.is_none());
    for alias in ["dimstyle", "dimstyle.dimension", "d", "dst", "ddim"] {
        app.start(alias);
        assert_eq!(app.ui.dialog.as_deref(), Some("dimstyle"));
        app.ui.dialog = None;
    }
    app.start("dimstyle");
    let mut h = Harness::builder().with_size(vec2(1000.0, 800.0)).build_ui_state(|ui, app: &mut CadApp| dialogs::show(app, ui.ctx()), app);
    h.run_steps(4);
    h.get_by_label("Scripted").click();
    h.run_steps(4);
    h.get_by_label("Set Current").click();
    h.run_steps(4);
    assert_eq!(h.state().session.doc().unwrap().header.str("DIMSTYLE", ""), "Scripted");
    h.get_all_by_role(egui::accesskit::Role::TextInput).next().unwrap().click();
    h.run_steps(2);
    h.get_all_by_role(egui::accesskit::Role::TextInput).next().unwrap().type_text("CreatedInManager");
    h.run_steps(2);
    h.get_by_label("New").click();
    h.run_steps(4);
    assert!(h.state().session.doc().unwrap().dim_style("CreatedInManager").is_some());
    assert_eq!(h.state().session.doc().unwrap().header.str("DIMSTYLE", ""), "Scripted");
    h.state_mut().run("undo", json!({})).unwrap();
    assert!(h.state().session.doc().unwrap().dim_style("CreatedInManager").is_none());
    h.state_mut().run("undo", json!({})).unwrap();
    assert_eq!(h.state().session.doc().unwrap().header.str("DIMSTYLE", ""), "Standard");
}
