use cadcraft_engine::Session;
use cadcraft_ui_egui::{
    CadApp, Services,
    docking::{self, Panel},
};
use egui::{Event, Modifiers, PointerButton, accesskit::Role, pos2, vec2};
use egui_kittest::{Harness, kittest::Queryable};
use serde_json::json;

fn harness() -> Harness<'static, CadApp> {
    let mut app = CadApp::new(Session::default(), Services::default());
    app.run("new", json!({})).unwrap();
    let mut h = Harness::builder().with_size(vec2(1000.0, 700.0)).build_ui_state(|ui, app| docking::show(app, ui), app);
    h.run_steps(5);
    h
}

fn right_click(h: &mut Harness<'_, CadApp>, x: f32, y: f32) {
    let pos = pos2(x, y);
    h.event(Event::PointerMoved(pos));
    h.event(Event::PointerButton { pos, button: PointerButton::Secondary, pressed: true, modifiers: Modifiers::NONE });
    h.run_steps(2);
    h.event(Event::PointerButton { pos, button: PointerButton::Secondary, pressed: false, modifiers: Modifiers::NONE });
    h.run_steps(3);
}

#[test]
fn all_palette_title_menus_share_docking_float_and_auto_hide_only_tool_sets_moves_to_ribbon() {
    for panel in [Panel::ToolSets, Panel::Layers, Panel::Properties, Panel::ExternalRefs, Panel::QuickCalc] {
        let mut h = harness();
        h.state_mut().run("ui.dock", json!({"operation":"open","panel":panel.id()})).unwrap();
        h.state_mut().run("ui.dock", json!({"operation":"float","panel":panel.id(),"rect":[300,160,300,400]})).unwrap();
        h.run_steps(5);
        right_click(&mut h, 340.0, 170.0);
        assert!(h.query_all_by_label("Dock Left").count() >= 2, "header button and menu action");
        assert!(h.query_all_by_label("Dock Right").count() >= 2, "header button and menu action");
        h.get_by_label("Float panel");
        h.get_by_role_and_label(Role::CheckBox, "Auto-hide");
        assert_eq!(h.query_by_label("Move to Ribbon").is_some(), panel == Panel::ToolSets);
        if panel == Panel::ToolSets {
            h.get_by_label("Move to Ribbon").click();
            h.run_steps(5);
            assert!(h.state().ui.show_ribbon && !h.state().ui.show_toolsets);
        }
    }
}

#[test]
fn every_palette_auto_hides_on_both_sides_and_while_floating_and_can_be_pinned_open() {
    for panel in [Panel::ToolSets, Panel::Layers, Panel::Properties, Panel::ExternalRefs, Panel::QuickCalc] {
        for place in ["left", "right", "floating"] {
            let mut h = harness();
            h.state_mut().run("ui.dock", json!({"operation":"open","panel":panel.id()})).unwrap();
            let params = if place == "floating" {
                json!({"operation":"float","panel":panel.id(),"rect":[300,160,300,400]})
            } else {
                json!({"operation":"dock","panel":panel.id(),"zone":place})
            };
            h.state_mut().run("ui.dock", params).unwrap();
            let saved = h.state().ui.docking.layout.clone();
            let drawing = h.state().session.state().unwrap().uid;
            let dirty = h.state().session.state().unwrap().is_dirty();
            h.state_mut().run("ui.dock", json!({"operation":"autoHide","panel":panel.id(),"enabled":true})).unwrap();
            h.event(Event::PointerMoved(pos2(500.0, 650.0)));
            h.run_steps(5);
            let label = match panel {
                Panel::ToolSets => "Tool Sets",
                Panel::Layers => "Layers",
                Panel::Properties => "Properties",
                Panel::ExternalRefs => "External References",
                Panel::QuickCalc => "QuickCalc",
                Panel::Canvas => unreachable!(),
            };
            assert!(h.query_by_label("Keep Open").is_none());
            let tab = h.get_by_label(&format!("Auto-hide: {label}")).rect();
            h.event(Event::PointerMoved(tab.center()));
            h.run_steps(5);
            assert!(h.query_by_label("Keep Open").is_some(), "hover did not open {label} at {place}, tab {tab:?}");
            assert_eq!(h.state().ui.docking.layout, saved, "hover cannot change saved placement");
            h.event(Event::PointerMoved(pos2(500.0, 650.0)));
            h.run_steps(60);
            assert!(h.query_by_label("Keep Open").is_none(), "{label} at {place} did not roll closed");
            h.event(Event::PointerMoved(tab.center()));
            h.run_steps(5);
            h.get_by_label("Keep Open").click();
            h.run_steps(5);
            assert!(!h.state().ui.docking.auto_hide.contains(&panel));
            assert_eq!(h.state().ui.docking.layout, saved);
            assert_eq!(h.state().session.state().unwrap().uid, drawing);
            assert_eq!(h.state().session.state().unwrap().is_dirty(), dirty);
        }
    }
}

#[test]
fn auto_hide_is_persistent_bounded_and_separate_from_closed_visibility() {
    let mut h = harness();
    h.state_mut().run("ui.dock", json!({"operation":"autoHide","panel":"toolSets","enabled":true})).unwrap();
    h.state_mut().run("ui.dock", json!({"operation":"close","panel":"toolSets"})).unwrap();
    h.run_steps(3);
    assert!(h.query_by_label("Auto-hide: Tool Sets").is_none());
    h.state_mut().run("ui.dock", json!({"operation":"open","panel":"toolSets"})).unwrap();
    h.run_steps(3);
    h.get_by_label("Auto-hide: Tool Sets");
    h.run_steps(60);
    h.get_by_label("Keep Open"); // An explicit open stays usable until entered or dismissed.
    h.key_press(egui::Key::Escape);
    h.run_steps(5);
    assert!(h.query_by_label("Keep Open").is_none());
    let before = serde_json::to_value(&h.state().ui.docking).unwrap();
    for params in [json!({"operation":"autoHide","panel":"canvas","enabled":true}), json!({"operation":"autoHide","panel":"layers","enabled":"yes"})]
    {
        assert!(h.state_mut().run("ui.dock", params).is_err());
        assert_eq!(serde_json::to_value(&h.state().ui.docking).unwrap(), before);
    }
    let restored: docking::Workspace = serde_json::from_value(before).unwrap();
    assert_eq!(restored.auto_hide, [Panel::ToolSets]);
    restored.validate().unwrap();
    let mut invalid = restored;
    invalid.auto_hide.push(Panel::ToolSets);
    assert!(invalid.validate().is_err());
}

#[test]
fn a_hover_palette_stays_open_while_its_context_menu_is_in_use() {
    let mut h = harness();
    h.state_mut().run("ui.dock", json!({"operation":"autoHide","panel":"toolSets","enabled":true})).unwrap();
    h.run_steps(5);
    h.get_by_label("Auto-hide: Tool Sets").click_secondary();
    h.run_steps(5);
    h.event(Event::PointerMoved(pos2(500.0, 650.0)));
    h.run_steps(60);
    h.get_by_label("Keep Open");
    h.get_by_role_and_label(Role::CheckBox, "Auto-hide").click();
    h.run_steps(5);
    assert!(!h.state().ui.docking.auto_hide.contains(&Panel::ToolSets));
    assert!(h.state().ui.docking.layout.contains(&Panel::ToolSets));
}

#[test]
fn an_expanded_palette_can_float_and_roll_closed_without_retaining_its_old_docked_bounds() {
    let mut h = harness();
    h.state_mut().run("ui.dock", json!({"operation":"autoHide","panel":"toolSets","enabled":true})).unwrap();
    h.run_steps(5);
    let tab = h.get_by_label("Auto-hide: Tool Sets").rect();
    h.event(Event::PointerMoved(tab.center()));
    h.run_steps(5);
    h.get_by_label("Keep Open");
    h.state_mut().run("ui.dock", json!({"operation":"float","panel":"toolSets","rect":[350,180,280,480]})).unwrap();
    h.event(Event::PointerMoved(pos2(850.0, 650.0)));
    h.run_steps(60);
    assert!(h.query_by_label("Keep Open").is_none());
    let tab = h.get_by_label("Auto-hide: Tool Sets").rect();
    assert_eq!(tab.height(), 26.0);
    h.event(Event::PointerMoved(tab.center()));
    h.run_steps(5);
    h.get_by_label("Keep Open").click();
    h.run_steps(5);
    assert!(!h.state().ui.docking.auto_hide.contains(&Panel::ToolSets));
}

#[test]
fn idle_auto_hide_schedules_the_frame_that_removes_the_flyout() {
    let mut h = harness();
    h.state_mut().run("ui.dock", json!({"operation":"autoHide","panel":"toolSets","enabled":true})).unwrap();
    h.run_steps(5);
    let tab = h.get_by_label("Auto-hide: Tool Sets").rect();
    h.event(Event::PointerMoved(tab.center()));
    h.run_steps(5);
    h.get_by_label("Keep Open");
    h.event(Event::PointerMoved(pos2(850.0, 650.0)));
    let mut previous_delay = std::time::Duration::MAX;
    for _ in 0..60 {
        h.step();
        if h.query_by_label("Keep Open").is_none() {
            assert!(previous_delay.is_zero(), "the last visible frame must schedule an immediate removal frame for an idle native window");
            return;
        }
        previous_delay = h.output().viewport_output.get(&egui::ViewportId::ROOT).unwrap().repaint_delay;
    }
    panic!("auto-hide did not close");
}
