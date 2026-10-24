//! Discoverable visibility controls, shared by the menu bar and toolbar context menu.
use crate::{CadApp, docking::Panel};
use serde_json::json;

pub(crate) fn show(app: &mut CadApp, ui: &mut egui::Ui) {
    ui.menu_button(crate::tl!("Ribbon Options"), |ui| crate::ribbon::customize(app, ui));
    ui.separator();
    ui.label(crate::tl!("Tool Groups"));
    for (id, label) in crate::chrome::TOOLBAR_GROUPS {
        let mut visible = !app.ui.hidden_toolbar_groups.iter().any(|hidden| hidden == id);
        if ui.checkbox(&mut visible, crate::i18n::t(label)).changed() {
            let _ = app.run("ui.toolbar.group", json!({"group":id, "visible":visible}));
        }
    }
    if ui.button(crate::tl!("Show All Tool Groups")).clicked() {
        let _ = app.run("ui.toolbar.reset", json!({}));
    }
    ui.separator();
    ui.label(crate::tl!("Side Panels"));
    for (panel, label) in [
        (Panel::ToolSets, "Tool Sets"),
        (Panel::Layers, "Layers"),
        (Panel::Properties, "Properties"),
        (Panel::ExternalRefs, "External References"),
        (Panel::QuickCalc, "QuickCalc"),
    ] {
        let mut visible = app.ui.docking.layout.contains(&panel);
        if panel == Panel::ToolSets {
            visible = app.ui.show_toolsets;
        } else if matches!(panel, Panel::Layers | Panel::Properties) {
            visible &= app.ui.show_palettes;
        }
        if ui.checkbox(&mut visible, crate::i18n::t(label)).changed() {
            let _ = app.run("ui.dock", json!({"operation": if visible {"open"} else {"close"}, "panel":panel.id()}));
        }
    }
    ui.separator();
    for (command, label, current) in [
        ("ui.toggle.toolbar", "Tool Bar", app.ui.show_toolbar),
        ("ui.toggle.cmdline", "Command Line", app.ui.show_command_line),
        ("ui.toggle.filetabs", "File Tabs", app.ui.show_file_tabs),
        ("ui.toggle.statusbar", "Status Bar", app.ui.show_status_bar),
    ] {
        let mut visible = current;
        if ui.checkbox(&mut visible, crate::i18n::t(label)).changed() {
            let _ = app.run(command, json!({"on":visible}));
        }
    }
    ui.separator();
    if ui.button(crate::tl!("Reset Panel Layout")).clicked() {
        let _ = app.run("ui.dock.reset", json!({}));
        ui.close();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Services;
    use cadcraft_engine::Session;
    use egui::{Event, Modifiers, PointerButton, pos2, vec2};
    use egui_kittest::{Harness, kittest::Queryable};

    #[test]
    fn toolbar_context_menu_works_on_buttons_and_empty_space() {
        for x in [25.0, 1500.0] {
            let mut app = CadApp::new(Session::default(), Services::default());
            app.ui.show_ribbon = false; // This test exercises the full classic toolbar.
            let mut h = Harness::builder().with_size(vec2(1600.0, 700.0)).build_ui_state(|ui, app| crate::chrome::title_and_toolbar(app, ui), app);
            h.run_steps(3);
            let pos = pos2(x, 17.0);
            h.event(Event::PointerMoved(pos));
            h.event(Event::PointerButton { pos, button: PointerButton::Secondary, pressed: true, modifiers: Modifiers::NONE });
            h.run_steps(2);
            h.event(Event::PointerButton { pos, button: PointerButton::Secondary, pressed: false, modifiers: Modifiers::NONE });
            h.run_steps(3);
            h.get_by_role_and_label(egui::accesskit::Role::CheckBox, "Help").click();
            h.run_steps(3);
            assert!(h.state().ui.hidden_toolbar_groups.iter().any(|id| id == "help"));
            h.key_press(egui::Key::Escape);
            h.run_steps(3);
            assert!(h.query_by_role_and_label(egui::accesskit::Role::Button, "Help").is_none());
            let saved = serde_json::to_value(&h.state().ui).unwrap();
            let restored: crate::UiState = serde_json::from_value(saved).unwrap();
            assert!(restored.hidden_toolbar_groups.iter().any(|id| id == "help"));
            h.state_mut().run("ui.toolbar.group", json!({"group":"help","visible":true})).unwrap();
            h.run_steps(3);
            h.get_by_role_and_label(egui::accesskit::Role::Button, "Help").click();
            h.run_steps(3);
            assert_eq!(h.state().ui.dialog.as_deref(), Some("about"), "normal toolbar clicks still run commands");
        }
    }

    #[test]
    fn menu_bar_can_restore_every_hidden_toolbar_group() {
        let mut app = CadApp::new(Session::default(), Services::default());
        for (id, _) in crate::chrome::TOOLBAR_GROUPS {
            app.run("ui.toolbar.group", json!({"group":id,"visible":false})).unwrap();
        }
        app.run("ui.toggle.toolbar", json!({"on":false})).unwrap();
        let mut h = Harness::builder().with_size(vec2(1600.0, 700.0)).build_ui_state(|ui, app| crate::menus::menu_bar(app, ui), app);
        h.run_steps(3);
        h.get_by_label("Panels").click();
        h.run_steps(3);
        h.get_by_label("Show All Tool Groups").click();
        h.run_steps(3);
        // A menu-bar action closes its popup; reopen it to restore the toolbar itself.
        h.get_by_label("Panels").click();
        h.run_steps(3);
        h.get_by_role_and_label(egui::accesskit::Role::CheckBox, "Tool Bar").click();
        h.run_steps(3);
        assert!(h.state().ui.show_toolbar);
        assert!(h.state().ui.hidden_toolbar_groups.is_empty());
    }
}
