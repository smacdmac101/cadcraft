//! The drafting toggles on the status bar are transparent, as F7-F12 are: clicking Grid or Snap
//! while drawing a line changes the setting and leaves LINE running (#1108).

use cadcraft_engine::doc::{Drawing, EntityKind, Space};
use cadcraft_engine::geom::Vec2;
use cadcraft_engine::{Input, Session};
use cadcraft_ui_egui::{CadApp, Services};
use egui::vec2;
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;

fn harness() -> Harness<'static, CadApp> {
    let mut session = Session::empty();
    session.open_drawing(Drawing::new_imperial(), "Status bar toggles", None);
    let app = CadApp::new(session, Services::default());
    let mut h = Harness::builder().with_size(vec2(1600.0, 1000.0)).build_ui_state(
        |ui, app: &mut CadApp| {
            app.logic(ui.ctx());
            app.ui(ui);
        },
        app,
    );
    h.run_steps(3);
    h
}

fn running(h: &Harness<'_, CadApp>) -> Option<String> {
    h.state().session.running.as_ref().map(|r| r.id.clone())
}

#[test]
fn clicking_grid_and_snap_on_the_status_bar_keeps_line_running() {
    let mut h = harness();
    let (grid, snap) = (h.state().session.settings.gridmode, h.state().session.settings.snapmode);
    {
        let s = &mut h.state_mut().session;
        s.start("line").unwrap();
        s.input(Input::Point(Vec2::new(0.0, 0.0))).unwrap();
    }
    h.run_steps(2);
    h.get_by_label("Grid (F7)").click();
    h.run_steps(2);
    h.get_by_label("Snap Mode (F9)").click();
    h.run_steps(2);
    assert_eq!(h.state().session.settings.gridmode, !grid);
    assert_eq!(h.state().session.settings.snapmode, !snap);
    assert_eq!(running(&h).as_deref(), Some("line"));
    h.state_mut().session.input(Input::Point(Vec2::new(10.0, 0.0))).unwrap();
    let lines: Vec<_> = h
        .state()
        .session
        .state()
        .unwrap()
        .doc
        .space(&Space::Model)
        .unwrap()
        .iter()
        .filter_map(|e| match &e.kind {
            EntityKind::Line(l) => Some((l.a.xy(), l.b.xy())),
            _ => None,
        })
        .collect();
    assert_eq!(lines, vec![(Vec2::new(0.0, 0.0), Vec2::new(10.0, 0.0))]);
}

#[test]
fn status_visibility_is_atomic_persistent_and_separate_from_drafting_settings() {
    use serde_json::json;
    let mut h = harness();
    let settings = h.state().session.settings.clone();
    let dirty = h.state().session.state().unwrap().is_dirty();
    for (item, _) in cadcraft_ui_egui::chrome::status::ITEMS {
        h.state_mut().run("ui.status.item", json!({"item":item,"visible":false})).unwrap();
    }
    let before = h.state().ui.hidden_status_items.clone();
    for params in [json!({"item":"unknown"}), json!({"item":"grid","visible":"no"}), json!({"visible":true})] {
        assert!(h.state_mut().run("ui.status.item", params).is_err());
        assert_eq!(h.state().ui.hidden_status_items, before);
    }
    let restored: cadcraft_ui_egui::UiState = serde_json::from_value(serde_json::to_value(&h.state().ui).unwrap()).unwrap();
    assert_eq!(restored.hidden_status_items, before);
    assert!(serde_json::from_value::<cadcraft_ui_egui::UiState>(json!({})).unwrap().hidden_status_items.is_empty());
    assert_eq!(h.state().session.settings.gridmode, settings.gridmode);
    assert_eq!(h.state().session.settings.snapmode, settings.snapmode);
    assert_eq!(h.state().session.state().unwrap().is_dirty(), dirty);
    h.run_steps(4);
    assert!(h.query_by_label("Grid (F7)").is_none());
    h.get_by_label("Status Bar Customization").click();
    h.run_steps(4);
    h.get_by_label("Show All Status Items").click();
    h.run_steps(4);
    assert!(h.state().ui.hidden_status_items.is_empty());
}

#[test]
fn status_customization_does_not_cancel_or_feed_a_point_to_a_running_line() {
    use egui::accesskit::Role;
    let mut h = harness();
    h.state_mut().session.start("line").unwrap();
    h.state_mut().session.input(Input::Point(Vec2::new(0.0, 0.0))).unwrap();
    h.run_steps(3);
    let grid = h.state().session.settings.gridmode;
    h.get_by_label("Status Bar Customization").click();
    h.run_steps(4);
    h.get_by_role_and_label(Role::CheckBox, "Grid (F7)").click();
    h.run_steps(4);
    assert!(h.state().ui.hidden_status_items.iter().any(|id| id == "grid"));
    assert_eq!(h.state().session.settings.gridmode, grid);
    assert_eq!(running(&h).as_deref(), Some("line"));
    h.key_press(egui::Key::Escape);
    h.run_steps(3);
    assert_eq!(running(&h).as_deref(), Some("line"));
    h.state_mut().session.input(Input::Point(Vec2::new(10.0, 0.0))).unwrap();
    let count = h.state().session.doc().unwrap().space(&Space::Model).unwrap().iter().filter(|e| matches!(e.kind, EntityKind::Line(_))).count();
    assert_eq!(count, 1);
    h.key_press(egui::Key::Escape);
    h.run_steps(3);
    assert!(running(&h).is_none(), "a later Escape still cancels the command");
}
