//! Interaction and recovery checks for the actual ribbon, independent of the classic toolbar.
use cadcraft_engine::Session;
use cadcraft_ui_egui::{CadApp, Services, ribbon};
use egui::{Key, accesskit::Role, vec2};
use egui_kittest::{
    Harness,
    kittest::{NodeT, Queryable},
};
use serde_json::json;

fn harness(width: f32) -> Harness<'static, CadApp> {
    let mut app = CadApp::new(Session::default(), Services::default());
    app.run("new", json!({})).unwrap();
    app.run("ribbon", json!({})).unwrap();
    let mut h = Harness::builder().with_size(vec2(width, 600.0)).build_ui_state(|ui, app| ribbon::show(app, ui), app);
    h.run_steps(3);
    h
}

#[test]
fn selecting_a_tab_changes_tools_and_a_tool_runs_the_registry_command() {
    let mut h = harness(1600.0);
    h.get_by_label("Ribbon Tab: Annotate").click();
    h.run_steps(3);
    assert_eq!(h.state().ui.ribbon.active_tab, "annotate");
    assert!(h.query_by_role_and_label(Role::Button, "Ribbon / Draw: Line").is_none());
    h.get_by_role_and_label(Role::Button, "Ribbon / Dimensions: Linear").click();
    h.run_steps(3);
    assert_eq!(h.state().session.running.as_ref().unwrap().id, "dimlinear");
}

#[test]
fn right_click_on_a_tool_hides_its_panel_without_starting_a_command() {
    let mut h = harness(1600.0);
    h.get_by_role_and_label(Role::Button, "Ribbon / Draw: Line").click_secondary();
    h.run_steps(3);
    assert!(h.state().session.running.is_none());
    h.get_by_label_contains("Show Panels").click();
    h.run_steps(3);
    h.get_by_role_and_label(Role::CheckBox, "Draw").click();
    h.run_steps(3);
    assert!(!h.state().ui.ribbon.panel_visible("home", "draw"));
    assert!(h.query_by_role_and_label(Role::Button, "Ribbon / Draw: Line").is_none());
}

#[test]
fn panel_preferences_are_per_tab_and_restoration_recovers_hidden_everything() {
    let mut h = harness(1600.0);
    h.state_mut().run("ui.ribbon.panel", json!({"tab":"home","panel":"block","visible":false})).unwrap();
    h.run_steps(3);
    assert!(h.query_by_role_and_label(Role::Button, "Ribbon / Block: Create Block").is_none());
    h.get_by_label("Ribbon Tab: Insert").click();
    h.run_steps(3);
    assert!(h.query_by_role_and_label(Role::Button, "Ribbon / Block: Create Block").is_some());
    for tab in ribbon::TABS {
        h.state_mut().run("ui.ribbon.tab", json!({"tab":tab.id,"visible":false})).unwrap();
    }
    h.run_steps(3);
    h.get_by_label("Restore Ribbon").click();
    h.run_steps(3);
    assert_eq!(h.state().ui.ribbon, ribbon::State::default());
    assert!(h.query_by_role_and_label(Role::Button, "Ribbon / Draw: Line").is_some());
}

#[test]
fn minimized_ribbon_and_classic_mode_remain_recoverable() {
    let mut h = harness(520.0);
    assert!(h.get_by_role_and_label(Role::Button, "Ribbon / Draw: Line").rect().max.x < 520.0);
    h.state_mut().run("ui.ribbon.options", json!({"minimized":true})).unwrap();
    h.run_steps(3);
    assert!(h.query_by_role_and_label(Role::Button, "Ribbon / Draw: Line").is_none());
    h.get_by_label("Expand Ribbon").click();
    h.run_steps(3);
    assert!(h.query_by_role_and_label(Role::Button, "Ribbon / Draw: Line").is_some());
    h.state_mut().run("ribbonclose", json!({})).unwrap();
    h.run_steps(3);
    assert!(!h.state().ui.show_ribbon);
    h.state_mut().run("ribbon", json!({})).unwrap();
    h.run_steps(3);
    assert!(h.query_by_role_and_label(Role::Button, "Ribbon / Draw: Line").is_some());
}

#[test]
fn panel_geometry_stays_in_one_row_with_every_home_panel_reachable() {
    let h = harness(1600.0);
    let labels = [
        "Ribbon / Draw: Line",
        "Ribbon / Modify: Move",
        "Ribbon / Annotation: Single Line Text",
        "Ribbon / Layers: Layers",
        "Ribbon / Block: Create Block",
        "Ribbon / Properties: Properties Inspector",
        "Ribbon / Utilities: Measure Distance",
        "Ribbon / Clipboard: Copy",
    ];
    let first = h.get_by_role_and_label(Role::Button, labels[0]).rect();
    for label in labels {
        let rect = h.get_by_role_and_label(Role::Button, label).rect();
        assert!((rect.top() - first.top()).abs() < 1.0, "{label} stepped out of the panel row: {rect:?}");
        assert!(rect.bottom() < 150.0 && rect.right() < 1600.0, "{label} is clipped: {rect:?}");
    }
    assert!(h.get_by_label("Ribbon Panel: Draw").rect().bottom() < 168.0, "panel titles must fit inside the ribbon");
    assert!(h.get_by_label("More: Draw").rect().bottom() < 168.0, "overflow controls must remain visible");
}

#[test]
fn right_clicking_an_inactive_tab_customizes_that_tabs_panels() {
    let mut h = harness(1600.0);
    h.get_by_label("Ribbon Tab: Insert").click_secondary();
    h.run_steps(3);
    assert_eq!(h.state().ui.ribbon.active_tab, "insert");
    h.get_by_label_contains("Show Panels").click();
    h.run_steps(3);
    h.get_by_role_and_label(Role::CheckBox, "Block").click();
    h.run_steps(3);
    assert!(!h.state().ui.ribbon.panel_visible("insert", "block"));
    assert!(h.state().ui.ribbon.panel_visible("home", "block"));
}

#[test]
fn keyboard_tabs_select_the_ribbon_without_starting_drawing_commands() {
    let mut h = harness(1600.0);
    h.key_press(Key::Tab);
    h.run_steps(3);
    assert!(h.get_by_label("Ribbon Tab: Home").is_focused());
    h.key_press(Key::Tab);
    h.run_steps(3);
    assert!(h.get_by_label("Ribbon Tab: Insert").is_focused());
    h.key_press(Key::Enter);
    h.run_steps(3);
    assert_eq!(h.state().ui.ribbon.active_tab, "insert");
    assert!(h.state().session.running.is_none());
}

#[test]
fn invalid_configuration_is_atomic_and_does_not_edit_the_drawing() {
    let mut app = CadApp::new(Session::default(), Services::default());
    app.run("new", json!({})).unwrap();
    let dirty = app.session.state().unwrap().is_dirty();
    let before = app.ui.ribbon.clone();
    for (command, params) in [
        ("ui.ribbon.tab", json!({"tab":"unknown"})),
        ("ui.ribbon.tab", json!({"tab":"home","visible":"yes"})),
        ("ui.ribbon.panel", json!({"tab":"home","panel":"unknown"})),
        ("ui.ribbon.options", json!({"minimized":true,"titles":42})),
    ] {
        assert!(app.run(command, params).is_err());
        assert_eq!(app.ui.ribbon, before);
        assert_eq!(app.session.state().unwrap().is_dirty(), dirty);
    }
    let restored: cadcraft_ui_egui::UiState = serde_json::from_value(json!({})).unwrap();
    assert!(!restored.show_ribbon, "older workspace settings retain the Tool Sets presentation until moved");
    assert_eq!(restored.ribbon, before, "older workspace settings receive a usable default ribbon");
}

#[test]
fn tool_sets_move_between_presentations_without_losing_tool_choices() {
    let mut app = CadApp::new(Session::default(), Services::default());
    app.run("new", json!({})).unwrap();
    app.ui.hidden_tools.push("line".into());
    let drawing = app.session.state().unwrap().uid;
    let dirty = app.session.state().unwrap().is_dirty();
    let mut h = Harness::builder().with_size(vec2(1600.0, 1000.0)).build_ui_state(
        |ui, app| {
            app.logic(ui.ctx());
            app.ui(ui);
        },
        app,
    );
    h.run_steps(5);
    h.state_mut().run("ui.toolsets.ribbon", json!({})).unwrap();
    h.run_steps(5);
    assert!(h.state().ui.show_ribbon);
    assert!(!h.state().ui.show_toolsets);
    h.state_mut().run("ui.dock.reset", json!({})).unwrap();
    h.run_steps(3);
    assert!(!h.state().ui.show_toolsets, "reset keeps the selected ribbon presentation");
    assert!(h.query_by_role_and_label(Role::Button, "Ribbon / Draw: Line").is_none());
    assert!(h.query_by_role_and_label(Role::Button, "Ribbon / Draw: Polyline").is_some());
    h.state_mut().run("ui.toolsets.palette", json!({})).unwrap();
    h.run_steps(5);
    assert!(!h.state().ui.show_ribbon && h.state().ui.show_toolsets);
    assert_eq!(h.state().ui.hidden_tools, ["line"]);
    h.state_mut().ui.toolset_tab = "Modeling".into();
    h.state_mut().run("ui.dock", json!({"operation":"float","panel":"toolSets","rect":[300,180,260,480]})).unwrap();
    h.run_steps(5);
    h.state_mut().run("ui.toolsets.ribbon", json!({})).unwrap();
    h.run_steps(5);
    assert_eq!(h.state().ui.ribbon.active_tab, "modeling");
    assert!(h.get_by_role_and_label(Role::Button, "Ribbon / Primitives: Box").accesskit_node().is_disabled());
    h.state_mut().run("ui.toolsets.palette", json!({})).unwrap();
    h.run_steps(5);
    assert!(
        h.state().ui.docking.layout.floating.iter().any(|group| group.panels.contains(&cadcraft_ui_egui::docking::Panel::ToolSets)),
        "returning restores the previous floating palette placement"
    );
    assert_eq!(h.state().session.state().unwrap().uid, drawing);
    assert_eq!(h.state().session.state().unwrap().is_dirty(), dirty);
}

#[test]
fn floating_tool_sets_have_working_left_and_right_dock_buttons() {
    let mut app = CadApp::new(Session::default(), Services::default());
    app.run("new", json!({})).unwrap();
    app.run("ui.dock", json!({"operation":"float","panel":"toolSets","rect":[300,180,260,480]})).unwrap();
    let mut h = Harness::builder().with_size(vec2(1600.0, 1000.0)).build_ui_state(
        |ui, app| {
            app.logic(ui.ctx());
            app.ui(ui);
        },
        app,
    );
    h.run_steps(5);
    for label in ["Dock Left", "Dock Right"] {
        h.get_by_label(label).click();
        h.run_steps(5);
        assert!(h.state().ui.docking.layout.floating.iter().all(|group| !group.panels.contains(&cadcraft_ui_egui::docking::Panel::ToolSets)));
        assert!(h.state().ui.show_toolsets);
        let location = h.state().ui.docking.layout.location(&cadcraft_ui_egui::docking::Panel::ToolSets).unwrap();
        assert_eq!(
            location.size,
            Some(if label == "Dock Left" { craft_ui::layout::SplitSize::FixedFirst(260.0) } else { craft_ui::layout::SplitSize::FixedSecond(260.0) })
        );
        if label == "Dock Left" {
            h.state_mut().run("ui.dock", json!({"operation":"float","panel":"toolSets","rect":[300,180,260,480]})).unwrap();
            h.run_steps(5);
        }
    }
    let before = serde_json::to_value(&h.state().ui.docking).unwrap();
    assert!(h.state_mut().run("ui.dock", json!({"operation":"dock","panel":"toolSets","zone":"invalid"})).is_err());
    assert_eq!(serde_json::to_value(&h.state().ui.docking).unwrap(), before);
}
