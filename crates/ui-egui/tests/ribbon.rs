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
fn detached_panel_stays_across_tabs_and_returns_to_its_original_place() {
    let mut h = harness(1600.0);
    let original = h.get_by_label("Ribbon / Draw: Line").rect();
    h.get_by_label("Ribbon / Draw: Line").click_secondary();
    h.run_steps(3);
    h.get_by_label("Float Panel").click();
    h.run_steps(5);
    assert_eq!(h.state().ui.ribbon.floating_panels.len(), 1);
    h.get_by_label("Ribbon Tab: Annotate").click();
    h.run_steps(4);
    assert!(h.query_by_label("Ribbon / Draw: Line").is_some());
    h.get_by_label("Return to Ribbon").click();
    h.run_steps(4);
    assert!(h.state().ui.ribbon.floating_panels.is_empty());
    assert!(h.query_by_label("Ribbon / Draw: Line").is_none());
    h.get_by_label("Ribbon Tab: Home").click();
    h.run_steps(4);
    assert_eq!(h.get_by_label("Ribbon / Draw: Line").rect(), original);
}

#[test]
fn detached_panel_configuration_is_atomic_unique_and_restorable() {
    let mut h = harness(1600.0);
    let initial = h.state().ui.ribbon.clone();
    for params in [
        json!({"tab":"bad","panel":"draw"}),
        json!({"panel":"missing"}),
        json!({"panel":"draw","position":[1]}),
        json!({"panel":"draw","position":[1e200,0]}),
    ] {
        assert!(h.state_mut().run("ui.ribbon.float", params).is_err());
        assert_eq!(h.state().ui.ribbon, initial);
    }
    for x in [300, 400] {
        h.state_mut().run("ui.ribbon.float", json!({"panel":"draw","position":[x,300]})).unwrap();
    }
    assert_eq!(h.state().ui.ribbon.floating_panels.len(), 1);
    let json = serde_json::to_value(&h.state().ui.ribbon).unwrap();
    assert_eq!(serde_json::from_value::<ribbon::State>(json).unwrap(), h.state().ui.ribbon);
    h.state_mut().run("ui.ribbon.return", json!({"all":true})).unwrap();
    assert_eq!(h.state().ui.ribbon, initial);
}

#[test]
fn native_desktop_positions_are_atomic_distinct_from_embedded_coordinates_and_compatible() {
    let mut h = harness(1600.0);
    h.state_mut().run("ui.ribbon.float", json!({"panel":"draw","position":[90000,90000],"desktopPosition":[-2200,140]})).unwrap();
    let before = h.state().ui.ribbon.clone();
    let saved = serde_json::to_value(&before).unwrap();
    assert_eq!(saved["floatingPanels"][0]["desktopPosition"], json!([-2200.0, 140.0]));
    for value in [json!([1]), json!([1e200, 0]), json!([1_000_001, 0]), json!("left")] {
        assert!(h.state_mut().run("ui.ribbon.float", json!({"panel":"draw","desktopPosition":value})).is_err());
        assert_eq!(h.state().ui.ribbon, before);
    }
    // A browser fallback can constrain its own position without replacing the desktop position.
    h.run_steps(5);
    assert_eq!(
        serde_json::to_value(&h.state().ui.ribbon).unwrap()["floatingPanels"][0]["desktopPosition"],
        saved["floatingPanels"][0]["desktopPosition"]
    );
    h.state_mut().run("ui.ribbon.float", json!({"panel":"draw","position":[100,100]})).unwrap();
    assert!(h.state().ui.ribbon.floating_panels[0].desktop_position.is_none());
    let mut legacy = saved;
    legacy["floatingPanels"][0].as_object_mut().unwrap().remove("desktopPosition");
    let loaded: ribbon::State = serde_json::from_value(legacy).unwrap();
    assert!(loaded.floating_panels[0].desktop_position.is_none());
    assert!(loaded.validate().is_ok());
}

fn drag_ribbon(h: &mut Harness<'static, CadApp>, from: egui::Pos2, to: egui::Pos2) {
    h.event(egui::Event::PointerMoved(from));
    h.run_steps(2);
    h.event(egui::Event::PointerButton { pos: from, button: egui::PointerButton::Primary, pressed: true, modifiers: egui::Modifiers::NONE });
    h.run_steps(2);
    for step in 1..=8 {
        h.event(egui::Event::PointerMoved(from.lerp(to, step as f32 / 8.0)));
        h.run_steps(1);
    }
    h.event(egui::Event::PointerButton { pos: to, button: egui::PointerButton::Primary, pressed: false, modifiers: egui::Modifiers::NONE });
    h.run_steps(4);
}

#[test]
fn title_drag_detaches_and_floating_header_drag_saves_its_position() {
    let mut h = harness(1600.0);
    let from = h.get_by_label("Ribbon Panel: Draw").rect().center();
    drag_ribbon(&mut h, from, egui::pos2(400.0, 300.0));
    assert_eq!(h.state().ui.ribbon.floating_panels.len(), 1);
    let before = h.state().ui.ribbon.floating_panels[0].position;
    let from = egui::pos2(before[0] + 70.0, before[1] + 12.0);
    drag_ribbon(&mut h, from, from + vec2(200.0, 50.0));
    let after = h.state().ui.ribbon.floating_panels[0].position;
    assert!(after[0] > before[0] + 150.0, "{before:?} -> {after:?}");
}

#[test]
fn a_floating_tool_click_does_not_place_a_canvas_point() {
    let mut app = CadApp::new(Session::default(), Services::default());
    app.run("new", json!({})).unwrap();
    app.run("ribbon", json!({})).unwrap();
    app.run("ui.ribbon.float", json!({"panel":"draw","position":[400,300]})).unwrap();
    let dirty = app.session.state().unwrap().is_dirty();
    let mut h = Harness::builder().with_size(vec2(1600.0, 900.0)).build_ui_state(
        |ui, app: &mut CadApp| {
            app.logic(ui.ctx());
            app.ui(ui);
        },
        app,
    );
    h.run_steps(5);
    h.get_by_label("Ribbon / Draw: Line").click();
    h.run_steps(5);
    assert!(h.state().session.prompt_text().starts_with("LINE Specify first point"));
    assert_eq!(h.state().session.state().unwrap().is_dirty(), dirty);
}

fn select_context_entity(h: &mut Harness<'static, CadApp>, kind: &str) -> cadcraft_engine::doc::Handle {
    let result = if kind == "hatch" {
        h.state_mut().run("circle", json!({"center":[0,0],"radius":5})).unwrap();
        h.state_mut().run("hatch", json!({"points":[[0,0]],"pattern":"SOLID"})).unwrap()
    } else {
        h.state_mut().run("table", json!({"at":[0,0],"rows":2,"cols":2})).unwrap()
    };
    let handle = cadcraft_engine::doc::Handle::parse_hex(result["handle"].as_str().unwrap()).unwrap();
    h.state_mut().session.set_selection(vec![handle]);
    h.run_steps(4);
    handle
}

#[test]
fn selection_context_activates_tools_and_restores_the_last_regular_tab() {
    let mut h = harness(1600.0);
    h.get_by_label("Ribbon Tab: Annotate").click();
    h.run_steps(3);
    select_context_entity(&mut h, "hatch");
    assert!(h.query_by_label("Ribbon Tab: Hatch Editor").is_some());
    assert!(h.query_by_label("Ribbon / Hatch: Hatch Edit").is_some());
    assert_eq!(h.state().ui.ribbon.active_tab, "annotate");
    h.get_by_label("Ribbon / Hatch: Hatch Edit").click();
    h.run_steps(3);
    assert!(h.state().hatch_dlg.is_some());
    h.state_mut().hatch_dlg = None;
    h.state_mut().session.set_selection(vec![]);
    h.run_steps(3);
    assert!(h.query_by_label("Ribbon Tab: Hatch Editor").is_none());
    assert!(h.query_by_label("Ribbon / Dimensions: Linear").is_some());
}

#[test]
fn contextual_tabs_allow_regular_tab_choice_and_reject_mixed_selection() {
    let mut h = harness(1600.0);
    let hatch = select_context_entity(&mut h, "hatch");
    h.state_mut().run("ui.ribbon.tab", json!({"tab":"output","visible":false})).unwrap();
    h.run_steps(3);
    assert!(h.query_by_label("Ribbon / Hatch: Hatch Edit").is_some(), "visibility changes must not switch the active context");
    h.get_by_label("Ribbon Tab: Home").click();
    h.run_steps(4);
    assert!(h.query_by_label("Ribbon / Draw: Line").is_some());
    assert!(h.query_by_label("Ribbon Tab: Hatch Editor").is_some());
    let table = select_context_entity(&mut h, "table");
    assert!(h.query_by_label("Ribbon / Table Cells: Edit Table Cell").is_some());
    h.state_mut().session.set_selection(vec![hatch, table]);
    h.run_steps(3);
    assert!(h.query_by_label("Ribbon Tab: Table").is_none());
    assert!(h.query_by_label("Ribbon Tab: Hatch Editor").is_none());
    assert!(h.state_mut().run("ui.ribbon.tab", json!({"tab":"hatchEditor"})).is_err());
    h.state_mut().session.set_selection(vec![table]);
    h.run_steps(3);
    h.state_mut().run("ui.ribbon.options", json!({"contextualTabs":false})).unwrap();
    h.run_steps(3);
    assert!(h.query_by_label("Ribbon Tab: Table").is_none());
    assert!(h.query_by_label("Ribbon / Draw: Line").is_some());
}

#[test]
fn block_editor_context_tracks_editor_lifetime_and_document_switching() {
    let mut h = harness(1600.0);
    let editing_document = h.state().session.active;
    h.state_mut().run("bedit", json!({"name":"<Current Drawing>"})).unwrap();
    h.run_steps(4);
    assert!(h.query_by_label("Ribbon Tab: Block Editor").is_some());
    assert!(h.query_by_label("Ribbon / Block Definition: Close Block Editor").is_some());
    h.state_mut().run("new", json!({})).unwrap();
    h.run_steps(3);
    assert!(h.query_by_label("Ribbon Tab: Block Editor").is_none());
    h.state_mut().session.active = editing_document;
    h.run_steps(3);
    assert!(h.query_by_label("Ribbon Tab: Block Editor").is_some());
    h.get_by_label("Ribbon / Block Definition: Close Block Editor").click();
    h.run_steps(4);
    assert!(h.query_by_label("Ribbon Tab: Block Editor").is_none());
    assert!(h.query_by_label("Ribbon / Draw: Line").is_some());
}

#[test]
fn contextual_floating_panels_follow_context_availability_without_losing_their_position() {
    let mut h = harness(1600.0);
    let hatch = select_context_entity(&mut h, "hatch");
    h.state_mut().run("ui.ribbon.float", json!({"tab":"hatchEditor","panel":"edit","position":[600,300]})).unwrap();
    h.run_steps(4);
    assert!(h.query_by_label("Return to Ribbon").is_some());
    let saved = h.state().ui.ribbon.floating_panels.clone();
    h.state_mut().session.set_selection(vec![]);
    h.run_steps(4);
    assert!(h.query_by_label("Return to Ribbon").is_none());
    assert_eq!(h.state().ui.ribbon.floating_panels, saved);
    h.state_mut().session.set_selection(vec![hatch]);
    h.run_steps(4);
    assert!(h.query_by_label("Return to Ribbon").is_some());
}

#[test]
fn offscreen_panels_are_constrained_and_duplicate_saved_state_is_rejected() {
    let mut h = harness(520.0);
    h.state_mut().run("ui.ribbon.float", json!({"panel":"draw","position":[90000,90000]})).unwrap();
    h.run_steps(6);
    let rect = h.get_by_label("Return to Ribbon").rect();
    assert!(rect.right() <= 520.0 && rect.bottom() <= 600.0, "{rect:?}");
    let mut invalid = h.state().ui.ribbon.clone();
    invalid.floating_panels.push(invalid.floating_panels[0].clone());
    assert!(invalid.validate().is_err());
    h.state_mut().run("ribbonclose", json!({})).unwrap();
    h.run_steps(3);
    assert!(h.query_by_label("Return to Ribbon").is_none());
    h.state_mut().run("ribbon", json!({})).unwrap();
    h.run_steps(3);
    assert!(h.query_by_label("Return to Ribbon").is_some());
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
fn minimized_tabs_open_temporary_tools_and_dispatch_without_expanding_the_ribbon() {
    let mut h = harness(1600.0);
    h.state_mut().run("ui.ribbon.options", json!({"minimized":true})).unwrap();
    h.run_steps(3);
    h.get_by_label("Ribbon Tab: Annotate").click();
    h.run_steps(5);
    h.get_by_label("Ribbon / Dimensions: Linear").click();
    h.run_steps(5);
    assert_eq!(h.state().session.running.as_ref().unwrap().id, "dimlinear");
    assert!(h.state().ui.ribbon.minimized);
    assert!(h.query_by_label("Ribbon / Dimensions: Linear").is_none());
}

#[test]
fn compact_panel_titles_and_buttons_open_only_their_panel_and_escape_dismisses() {
    for mode in ["panelTitles", "panelButtons"] {
        let mut h = harness(700.0);
        h.state_mut().run("ui.ribbon.options", json!({"minimized":true,"minimizeMode":mode})).unwrap();
        h.run_steps(3);
        h.get_by_label("Ribbon Panel: Draw").click();
        h.run_steps(5);
        h.get_by_label("Ribbon / Draw: Line");
        assert!(h.query_by_label("Ribbon / Modify: Move").is_none());
        h.key_press(Key::Escape);
        h.run_steps(3);
        assert!(h.query_by_label("Ribbon / Draw: Line").is_none());
        assert!(h.state().ui.ribbon.minimized);
    }
}

#[test]
fn ribbon_display_cycle_persists_and_preserves_hidden_panels_and_drawing() {
    let mut h = harness(1600.0);
    h.state_mut().run("ui.ribbon.options", json!({"minimizeMode":"cycle"})).unwrap();
    h.state_mut().run("ui.ribbon.panel", json!({"tab":"home","panel":"block","visible":false})).unwrap();
    let dirty = h.state().session.state().unwrap().is_dirty();
    for (minimized, position) in [(true, 0), (true, 1), (true, 2), (false, 0)] {
        h.state_mut().run("ui.ribbon.cycle", json!({})).unwrap();
        h.run_steps(3);
        assert_eq!(h.state().ui.ribbon.minimized, minimized);
        assert_eq!(h.state().ui.ribbon.cycle_position, position);
        assert!(!h.state().ui.ribbon.panel_visible("home", "block"));
        let saved = serde_json::to_value(&h.state().ui).unwrap();
        let restored: cadcraft_ui_egui::UiState = serde_json::from_value(saved).unwrap();
        assert_eq!(restored.ribbon, h.state().ui.ribbon);
    }
    assert_eq!(h.state().session.state().unwrap().is_dirty(), dirty);
}

#[test]
fn malformed_display_and_panel_tab_parameters_do_not_change_preferences() {
    let mut h = harness(1600.0);
    let before = h.state().ui.ribbon.clone();
    for (command, params) in [
        ("ui.ribbon.options", json!({"minimizeMode":"unknown","minimized":true})),
        ("ui.ribbon.options", json!({"minimizeMode":123})),
        ("ui.ribbon.panel", json!({"tab":"missing","panel":"draw","visible":false})),
        ("ui.ribbon.panel", json!({"tab":123,"panel":"draw","visible":false})),
    ] {
        assert!(h.state_mut().run(command, params).is_err());
        assert_eq!(h.state().ui.ribbon, before);
    }
    let mut invalid = before;
    invalid.cycle_position = 3;
    assert!(invalid.validate().is_err());
}

#[test]
fn display_menu_selects_compact_titles_and_outside_click_dismisses_the_flyout() {
    let mut h = harness(1600.0);
    h.get_by_label("Ribbon Display").click();
    h.run_steps(3);
    h.get_by_label("Minimize to Panel Titles").click();
    h.run_steps(3);
    assert_eq!(h.state().ui.ribbon.minimize_mode, ribbon::MinimizeMode::PanelTitles);
    assert!(h.state().ui.ribbon.minimized);
    h.get_by_label("Ribbon Panel: Draw").click();
    h.run_steps(3);
    h.get_by_label("Ribbon / Draw: Line");
    let pos = egui::pos2(1200.0, 500.0);
    h.event(egui::Event::PointerMoved(pos));
    for pressed in [true, false] {
        h.event(egui::Event::PointerButton { pos, button: egui::PointerButton::Primary, pressed, modifiers: egui::Modifiers::NONE });
        h.run_steps(3);
    }
    assert!(h.query_by_label("Ribbon / Draw: Line").is_none());
    assert!(h.state().session.running.is_none());
}

#[test]
fn a_temporary_tool_click_does_not_pick_a_point_in_the_canvas_underneath() {
    let mut app = CadApp::new(Session::default(), Services::default());
    app.run("new", json!({})).unwrap();
    app.run("ribbon", json!({})).unwrap();
    app.run("ui.ribbon.options", json!({"minimized":true})).unwrap();
    let dirty = app.session.state().unwrap().is_dirty();
    let mut h = Harness::builder().with_size(vec2(1600.0, 1000.0)).build_ui_state(
        |ui, app| {
            app.logic(ui.ctx());
            app.ui(ui);
        },
        app,
    );
    h.run_steps(5);
    h.get_by_label("Ribbon Tab: Home").click();
    h.run_steps(5);
    h.get_by_label("Ribbon / Draw: Line").click();
    h.run_steps(5);
    assert!(h.state().session.prompt_text().starts_with("LINE Specify first point"), "{}", h.state().session.prompt_text());
    assert_eq!(h.state().session.state().unwrap().is_dirty(), dirty);
    assert!(h.state().ui.ribbon.minimized);
}

#[test]
fn a_full_tab_flyout_uses_available_width_instead_of_the_generic_popup_width() {
    let mut h = harness(1600.0);
    h.state_mut().run("ui.ribbon.options", json!({"minimized":true})).unwrap();
    h.run_steps(3);
    h.get_by_label("Ribbon Tab: Home").click();
    h.run_steps(5);
    let first = h.get_by_label("Ribbon / Draw: Line").rect();
    let last = h.get_by_label("Ribbon / Clipboard: Copy").rect();
    assert!(last.center().x > 1200.0 && last.right() <= 1600.0, "{last:?}");
    assert!((first.top() - last.top()).abs() < 1.0);
}

#[test]
fn panel_geometry_stays_in_one_row_with_every_home_panel_reachable() {
    let h = harness(1600.0);
    let labels = [
        "Ribbon / Draw: Line",
        "Ribbon / Modify: Move",
        "Ribbon / Annotation: Single Line Text",
        "Ribbon / Block: Create Block",
        "Ribbon / Properties: Properties Inspector",
        "Ribbon / Utilities: Measure Distance",
        "Ribbon / Clipboard: Copy",
    ];
    let first = h.get_by_role_and_label(Role::Button, labels[0]).rect();
    let layer = h.get_by_label("Ribbon Layer").rect();
    assert!((layer.top() - first.top()).abs() < 1.0, "layer selector stepped out of the panel row");
    let layer_action = h.get_by_label("Ribbon / Layers: Layers").rect();
    assert!(layer_action.top() > layer.bottom() && layer_action.bottom() < 150.0);
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

#[test]
fn whole_ribbon_window_settings_are_atomic_persistent_and_independent_of_panels() {
    let mut app = CadApp::new(Session::default(), Services::default());
    let legacy: ribbon::State = serde_json::from_value(json!({"activeTab":"insert"})).unwrap();
    assert!(!legacy.window.floating);
    let before = app.ui.ribbon.clone();
    for params in [
        json!({"floating":"yes"}),
        json!({"floating":true,"size":[20,10]}),
        json!({"desktopPosition":[1e100,0]}),
        json!({"size":[500]}),
        json!({"size":[9000,400]}),
    ] {
        assert!(app.run("ui.ribbon.window", params).is_err());
        assert_eq!(app.ui.ribbon, before);
    }
    app.run("ui.ribbon.float", json!({"panel":"draw"})).unwrap();
    app.run("ui.ribbon.window", json!({"floating":true,"desktopPosition":[-1200,200],"size":[360,520]})).unwrap();
    let saved = app.ui.ribbon.clone();
    let restored: ribbon::State = serde_json::from_value(serde_json::to_value(&saved).unwrap()).unwrap();
    assert_eq!(restored, saved);
    app.run("ui.ribbon.window", json!({"floating":false})).unwrap();
    assert_eq!(app.ui.ribbon.window.desktop_position, Some([-1200.0, 200.0]));
    assert_eq!(app.ui.ribbon.floating_panels.len(), 1);
    app.run("ui.ribbon.reset", json!({})).unwrap();
    assert!(!app.ui.ribbon.window.floating);
}

#[test]
fn whole_ribbon_embedded_palette_switches_tabs_and_docks_back() {
    let mut h = harness(1600.0);
    h.ctx.set_embed_viewports(true);
    h.state_mut().run("ui.ribbon.window", json!({"floating":true})).unwrap();
    h.run_steps(5);
    assert!(h.query_by_label("Dock Ribbon").is_some());
    assert!(
        h.query_by_label("Ribbon / Draw: Line").is_none(),
        "line={:?}, dock={:?}, window={:?}",
        h.query_by_label("Ribbon / Draw: Line").map(|n| n.rect()),
        h.get_by_label("Dock Ribbon").rect(),
        h.state().ui.ribbon.window
    );
    h.get_by_label("Insert").click_secondary();
    h.run_steps(4);
    assert_eq!(h.state().ui.ribbon.active_tab, "insert");
    h.key_press(Key::Escape);
    h.run_steps(3);
    h.get_by_label("Dock Ribbon").click();
    h.run_steps(4);
    assert!(!h.state().ui.ribbon.window.floating);
    assert!(h.query_by_label("Ribbon Tab: Insert").is_some());
}

#[test]
fn wide_floating_ribbon_preserves_minimized_tab_flyouts_and_command_dispatch() {
    let mut h = harness(1600.0);
    h.ctx.set_embed_viewports(true);
    h.state_mut().run("ui.ribbon.window", json!({"floating":true,"size":[1000,300]})).unwrap();
    h.state_mut().run("ui.ribbon.options", json!({"minimized":true,"minimizeMode":"tabs"})).unwrap();
    h.run_steps(5);
    assert!(h.query_by_label("Ribbon / Draw: Line").is_none());
    h.get_by_label("Home").click();
    h.run_steps(4);
    h.get_by_label("Ribbon / Draw: Line").click();
    h.run_steps(4);
    assert!(h.state().session.prompt_text().starts_with("LINE Specify first point"));
    assert!(h.state().ui.ribbon.minimized);
    assert!(h.state().ui.ribbon.window.floating);
}

#[test]
fn ribbon_anchors_validate_atomically_and_preserve_floating_geometry() {
    let mut app = CadApp::new(Session::default(), Services::default());
    app.run("ui.ribbon.window", json!({"floating":true,"desktopPosition":[4200,250],"size":[360,520]})).unwrap();
    app.run("ui.ribbon.window", json!({"anchor":"left"})).unwrap();
    assert!(!app.ui.ribbon.window.floating);
    assert!(app.ui.ribbon.window.auto_hide);
    let saved = app.ui.ribbon.clone();
    for invalid in [
        json!({"anchor":"bottom"}),
        json!({"anchor":"left","floating":true}),
        json!({"autoHide":"yes"}),
        json!({"allowDocking":2}),
        json!({"anchor":"right","size":[2,2]}),
    ] {
        assert!(app.run("ui.ribbon.window", invalid).is_err());
        assert_eq!(app.ui.ribbon, saved);
    }
    let restored: ribbon::State = serde_json::from_value(serde_json::to_value(&saved).unwrap()).unwrap();
    assert_eq!(restored, saved);
    app.run("ui.ribbon.window", json!({"floating":true})).unwrap();
    assert!(app.ui.ribbon.window.anchor.is_none());
    assert_eq!(app.ui.ribbon.window.desktop_position, Some([4200.0, 250.0]));
    assert_eq!(app.ui.ribbon.window.size, [360.0, 520.0]);
    assert!(app.ui.ribbon.window.auto_hide);
    let legacy: ribbon::State = serde_json::from_value(json!({"window":{"floating":true}})).unwrap();
    assert!(legacy.window.allow_docking && !legacy.window.auto_hide && legacy.window.anchor.is_none());
}

#[test]
fn whole_ribbon_anchors_hover_rollup_pin_and_switch_sides_without_drawing_changes() {
    for side in ["left", "right"] {
        let mut h = harness(1200.0);
        h.state_mut().run("ui.ribbon.window", json!({"anchor":side})).unwrap();
        let saved = h.state().ui.ribbon.clone();
        let dirty = h.state().session.state().unwrap().is_dirty();
        h.event(egui::Event::PointerMoved(egui::pos2(600.0, 550.0)));
        h.run_steps(60);
        assert!(h.query_by_label("Keep Open").is_none());
        let rail = h.get_by_label("Ribbon Auto-hide").rect();
        h.event(egui::Event::PointerMoved(rail.center()));
        h.run_steps(6);
        assert!(h.query_by_label("Keep Open").is_some(), "hover {side}");
        assert_eq!(h.state().ui.ribbon, saved);
        h.event(egui::Event::PointerMoved(egui::pos2(600.0, 550.0)));
        h.run_steps(60);
        assert!(h.query_by_label("Keep Open").is_none(), "rollup {side}");
        h.event(egui::Event::PointerMoved(rail.center()));
        h.run_steps(6);
        h.get_by_label("Keep Open").click();
        h.run_steps(6);
        assert!(!h.state().ui.ribbon.window.auto_hide, "pin {side}");
        assert_eq!(h.state().session.state().unwrap().is_dirty(), dirty);
    }
}

#[test]
fn anchored_ribbon_menu_prevents_rollup_until_dismissed() {
    let mut h = harness(1200.0);
    h.state_mut().run("ui.ribbon.window", json!({"anchor":"left"})).unwrap();
    h.run_steps(60);
    let rail = h.get_by_label("Ribbon Auto-hide").rect();
    h.event(egui::Event::PointerMoved(rail.center()));
    h.run_steps(6);
    h.get_by_label("Ribbon Options").click();
    h.run_steps(5);
    h.event(egui::Event::PointerMoved(egui::pos2(800.0, 550.0)));
    h.run_steps(60);
    assert!(h.query_by_label("Keep Open").is_some());
    assert!(h.query_by_label("Anchor Right").is_some());
    h.key_press(Key::Escape);
    h.run_steps(60);
    assert!(h.query_by_label("Keep Open").is_none());
}

#[test]
fn anchored_overlay_pin_does_not_pass_a_click_into_a_running_drawing_command() {
    let mut app = CadApp::new(Session::default(), Services::default());
    app.run("new", json!({})).unwrap();
    app.run("ribbon", json!({})).unwrap();
    app.run("ui.ribbon.window", json!({"anchor":"left"})).unwrap();
    let dirty = app.session.state().unwrap().is_dirty();
    let mut h = Harness::builder().with_size(vec2(1600.0, 900.0)).build_ui_state(
        |ui, app: &mut CadApp| {
            app.logic(ui.ctx());
            app.ui(ui);
        },
        app,
    );
    h.run_steps(60);
    h.state_mut().start("line");
    let rail = h.get_by_label("Ribbon Auto-hide").rect();
    h.event(egui::Event::PointerMoved(rail.center()));
    h.run_steps(6);
    h.get_by_label("Keep Open").click();
    h.run_steps(5);
    assert!(h.state().session.prompt_text().starts_with("LINE Specify first point"));
    assert_eq!(h.state().session.state().unwrap().is_dirty(), dirty);
}

#[test]
fn anchored_undock_uses_native_root_geometry_without_reentering_context_lock() {
    let mut h = harness(1200.0);
    h.input_mut().viewports.get_mut(&egui::ViewportId::ROOT).unwrap().inner_rect =
        Some(egui::Rect::from_min_size(egui::pos2(100.0, 100.0), vec2(1200.0, 600.0)));
    h.state_mut().run("ui.ribbon.window", json!({"anchor":"right","autoHide":false})).unwrap();
    h.run_steps(6);
    h.get_by_label("Undock Ribbon").click();
    h.run_steps(5);
    assert!(h.state().ui.ribbon.window.floating);
    assert!(h.state().ui.ribbon.window.anchor.is_none());
    assert!(h.state().ui.ribbon.window.desktop_position.is_some());
}

#[test]
fn layout_context_tracks_paper_tabs_preserves_regular_choice_and_dispatches_viewports() {
    let mut h = harness(1600.0);
    h.state_mut().run("ui.ribbon.tab", json!({"tab":"insert"})).unwrap();
    let result = h.state_mut().run("layout.new", json!({"name":"Sheet","viewport":false})).unwrap();
    assert_eq!(result["name"], "Sheet");
    h.state_mut().run("layout.set", json!({"name":"Sheet"})).unwrap();
    h.run_steps(6);
    assert!(h.query_by_label("Ribbon Tab: Layout").is_some());
    assert_eq!(ribbon::context::active(h.state()).unwrap().id, "layout");
    assert_eq!(h.state().ui.ribbon.active_tab, "insert");
    h.get_by_label("Ribbon / Layout Viewports: Viewports").click();
    h.run_steps(4);
    assert!(h.state().session.prompt_text().to_ascii_uppercase().starts_with("MVIEW"));
    h.key_press(Key::Escape);
    h.run_steps(3);
    h.get_by_label("Ribbon Tab: Home").click();
    h.run_steps(3);
    assert_eq!(ribbon::context::active(h.state()).unwrap().id, "home");
    h.get_by_label("Ribbon Tab: Layout").click();
    h.run_steps(3);
    h.state_mut().run("layout.set", json!({"name":"Model"})).unwrap();
    h.run_steps(4);
    assert!(h.query_by_label("Ribbon Tab: Layout").is_none());
    assert_eq!(h.state().ui.ribbon.active_tab, "home");
    assert!(h.state_mut().run("ui.ribbon.tab", json!({"tab":"layout"})).is_err());
}

#[test]
fn paper_layout_remains_available_alongside_selection_context_and_in_floating_ribbon() {
    let mut h = harness(1600.0);
    h.state_mut().run("layout.new", json!({"name":"Sheet","viewport":false})).unwrap();
    h.state_mut().run("layout.set", json!({"name":"Sheet"})).unwrap();
    h.run_steps(4);
    h.state_mut().run("mview", json!({"p1":[0,0],"p2":[100,80]})).unwrap();
    h.state_mut().run("mspace", json!({})).unwrap();
    h.run_steps(4);
    assert!(ribbon::context::is_available(h.state(), "layout"));
    h.state_mut().run("pspace", json!({})).unwrap();
    h.run_steps(3);
    select_context_entity(&mut h, "table");
    assert_eq!(ribbon::context::available(h.state()).unwrap().id, "tableEditor");
    assert!(ribbon::context::is_available(h.state(), "layout"));
    h.get_by_label("Ribbon Tab: Layout").click();
    h.run_steps(4);
    assert_eq!(ribbon::context::active(h.state()).unwrap().id, "layout");
    h.state_mut().run("ui.ribbon.float", json!({"tab":"layout","panel":"viewports"})).unwrap();
    h.run_steps(4);
    assert!(h.query_by_label("Return to Ribbon").is_some());
    h.state_mut().run("ui.ribbon.window", json!({"floating":true,"size":[1000,300]})).unwrap();
    h.run_steps(5);
    assert!(h.query_by_label("Layout").is_some());
    assert!(h.query_by_label("Return to Ribbon").is_some());
    h.state_mut().run("ui.ribbon.options", json!({"contextualTabs":false})).unwrap();
    h.run_steps(5);
    assert!(!ribbon::context::is_available(h.state(), "layout"));
    assert!(h.query_by_label("Return to Ribbon").is_none());
}
