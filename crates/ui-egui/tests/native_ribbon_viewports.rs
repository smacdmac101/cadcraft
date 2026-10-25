//! Exercise the actual immediate-viewport callback without depending on a particular OS.
use cadcraft_engine::Session;
use cadcraft_ui_egui::{
    CadApp, Services,
    control::{self, ControlRequest, Outcome},
    ribbon,
};
use egui::{Context, Event, Pos2, RawInput, Rect, ViewportCommand, ViewportEvent, ViewportId, ViewportInfo, vec2};
use serde_json::{Value, json};
use std::{cell::RefCell, rc::Rc};

#[derive(Default)]
struct Host {
    viewports: egui::ViewportIdMap<ViewportInfo>,
    sizes: egui::ViewportIdMap<egui::Vec2>,
    modifiers: egui::Modifiers,
    resize_delay: u8,
    position_delay: u8,
    events: Vec<Event>,
    close: bool,
    commands: Vec<ViewportCommand>,
    titles: Vec<String>,
    position: Pos2,
}

fn host() -> (Context, Rc<RefCell<Host>>) {
    let ctx = Context::default();
    ctx.set_embed_viewports(false);
    let host = Rc::new(RefCell::new(Host::default()));
    host.borrow_mut().viewports.insert(
        ViewportId::ROOT,
        ViewportInfo {
            native_pixels_per_point: Some(1.0),
            inner_rect: Some(Rect::from_min_size(egui::pos2(100.0, 100.0), vec2(1200.0, 800.0))),
            ..Default::default()
        },
    );
    let copy = host.clone();
    Context::set_immediate_viewport_renderer(move |ctx, mut viewport| {
        let mut h = copy.borrow_mut();
        h.titles.push(viewport.builder.title.clone().unwrap_or_default());
        let close = h.close;
        h.close = false;
        let size = *h.sizes.entry(viewport.ids.this).or_insert(viewport.builder.inner_size.unwrap_or(vec2(204.0, 154.0)));
        let info = ViewportInfo {
            parent: Some(ViewportId::ROOT),
            native_pixels_per_point: Some(1.0),
            outer_rect: Some(Rect::from_min_size(h.position, vec2(220.0, 190.0))),
            inner_rect: Some(Rect::from_min_size(h.position, size)),
            events: if close { vec![ViewportEvent::Close] } else { vec![] },
            ..Default::default()
        };
        h.viewports.insert(viewport.ids.this, info);
        let modifiers = h.modifiers;
        h.events.insert(0, Event::ModifiersChanged(modifiers));
        let raw = RawInput {
            viewport_id: viewport.ids.this,
            viewports: h.viewports.clone(),
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, size)),
            events: std::mem::take(&mut h.events),
            ..Default::default()
        };
        drop(h);
        let mut output = ctx.run_ui(raw, |ui| (viewport.viewport_ui_cb)(ui));
        output.textures_delta.clear();
        let commands = output.viewport_output.get(&viewport.ids.this).unwrap().commands.clone();
        let mut h = copy.borrow_mut();
        for command in &commands {
            if let ViewportCommand::OuterPosition(pos) = command {
                if h.position_delay > 0 {
                    h.position_delay -= 1;
                } else {
                    h.position = *pos;
                }
            }
            if let ViewportCommand::InnerSize(size) = command {
                if h.resize_delay > 0 {
                    h.resize_delay -= 1;
                } else {
                    h.sizes.insert(viewport.ids.this, *size);
                }
            }
        }
        h.commands.extend(commands);
    });
    (ctx, host)
}

fn frame(ctx: &Context, host: &Rc<RefCell<Host>>, app: &mut CadApp) {
    let input = RawInput {
        viewports: host.borrow().viewports.clone(),
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(1200.0, 800.0))),
        ..Default::default()
    };
    let mut output = ctx.run_ui(input, |ui| {
        ribbon::sync_native_desktop(ui.ctx(), Vec::new());
        ribbon::show(app, ui);
    });
    output.textures_delta.clear();
    let mut h = host.borrow_mut();
    for (id, viewport) in &output.viewport_output {
        if *id != ViewportId::ROOT {
            for command in &viewport.commands {
                if let ViewportCommand::OuterPosition(pos) = command {
                    if h.position_delay > 0 {
                        h.position_delay -= 1;
                    } else {
                        h.position = *pos;
                    }
                }
                if let ViewportCommand::InnerSize(size) = command {
                    if h.resize_delay > 0 {
                        h.resize_delay -= 1;
                    } else {
                        h.sizes.insert(*id, *size);
                    }
                }
            }
            h.commands.extend(viewport.commands.clone());
        }
    }
}

fn app() -> CadApp {
    let mut app = CadApp::new(Session::default(), Services::default());
    app.run("ribbon", json!({})).unwrap();
    app.run("ui.ribbon.float", json!({"panel":"draw","desktopPosition":[-1200,200]})).unwrap();
    app
}

fn elements(app: &mut CadApp, ctx: &Context, label: &str) -> Value {
    let (request, _) = ControlRequest::new("ui.elements", json!({"label":label}));
    let Outcome::Done(value) = control::handle(app, ctx, &request) else { panic!("Expected elements") };
    value["result"][0].clone()
}

#[test]
fn native_properties_panel_color_popup_remains_inside_small_viewport_and_routes_edits() {
    let (ctx, host) = host();
    let mut app = CadApp::new(Session::default(), Services::default());
    app.run("new", json!({})).unwrap();
    app.run("ribbon", json!({})).unwrap();
    app.run("ui.ribbon.float", json!({"tab":"home","panel":"properties"})).unwrap();
    for _ in 0..5 {
        frame(&ctx, &host, &mut app);
    }
    for label in ["Ribbon Color", "Ribbon Color / Red"] {
        let element = elements(&mut app, &ctx, label);
        assert_ne!(element["viewport"], format!("{:016x}", ViewportId::ROOT.0.value()));
        let r = element["rect"].as_array().unwrap();
        let x = r[0].as_f64().unwrap() as f32;
        let y = r[1].as_f64().unwrap() as f32;
        assert!(x >= 0.0 && y >= 0.0 && x + r[2].as_f64().unwrap() as f32 <= 204.0 && y + r[3].as_f64().unwrap() as f32 <= 154.0, "{label}: {r:?}");
        let pos = egui::pos2(x + 5.0, y + 5.0);
        for pressed in [true, false] {
            host.borrow_mut().events = vec![
                Event::PointerMoved(pos),
                Event::PointerButton { pos, button: egui::PointerButton::Primary, pressed, modifiers: egui::Modifiers::NONE },
            ];
            frame(&ctx, &host, &mut app);
        }
        for _ in 0..4 {
            frame(&ctx, &host, &mut app);
        }
    }
    assert_eq!(app.session.doc().unwrap().header.i64("CECOLOR", 256), 1);
}

#[test]
fn native_linetype_popup_fits_and_sets_current_without_using_the_canvas() {
    let (ctx, host) = host();
    let mut app = CadApp::new(Session::default(), Services::default());
    app.run("new", json!({})).unwrap();
    app.run("ribbon", json!({})).unwrap();
    app.run("ui.ribbon.float", json!({"tab":"home","panel":"properties"})).unwrap();
    for _ in 0..5 {
        frame(&ctx, &host, &mut app);
    }
    for label in ["Ribbon Linetype", "Ribbon Linetype / ByBlock"] {
        let element = elements(&mut app, &ctx, label);
        assert_ne!(element["viewport"], format!("{:016x}", ViewportId::ROOT.0.value()));
        let r = element["rect"].as_array().unwrap();
        let x = r[0].as_f64().unwrap() as f32;
        let y = r[1].as_f64().unwrap() as f32;
        assert!(x >= 0.0 && y >= 0.0 && x + r[2].as_f64().unwrap() as f32 <= 204.0 && y + r[3].as_f64().unwrap() as f32 <= 154.0, "{label}: {r:?}");
        let pos = egui::pos2(x + 5.0, y + 5.0);
        for pressed in [true, false] {
            host.borrow_mut().events = vec![
                Event::PointerMoved(pos),
                Event::PointerButton { pos, button: egui::PointerButton::Primary, pressed, modifiers: egui::Modifiers::NONE },
            ];
            frame(&ctx, &host, &mut app);
        }
        for _ in 0..4 {
            frame(&ctx, &host, &mut app);
        }
    }
    assert_eq!(app.session.doc().unwrap().header.str("CELTYPE", ""), "ByBlock");
    assert!(app.session.doc().unwrap().model.is_empty());
}

#[test]
fn native_callback_uses_desktop_position_routes_tool_input_and_closes_only_the_panel() {
    let (ctx, host) = host();
    host.borrow_mut().position_delay = 12;
    let mut app = app();
    let documents = app.session.docs.len();
    for _ in 0..32 {
        frame(&ctx, &host, &mut app);
    }
    assert_eq!(host.borrow().position, egui::pos2(-1200.0, 200.0));
    assert!(host.borrow().titles.iter().any(|t| t == "CADCraft — Home / Draw"));
    assert!(host.borrow().commands.iter().any(|c| matches!(c,ViewportCommand::OuterPosition(p) if *p==egui::pos2(-1200.0,200.0))));
    let tool = elements(&mut app, &ctx, "Ribbon / Draw: Line");
    assert_ne!(tool["viewport"], format!("{:016x}", ViewportId::ROOT.0.value()));
    let r = tool["rect"].as_array().unwrap();
    let pos = egui::pos2(r[0].as_f64().unwrap() as f32 + 20.0, r[1].as_f64().unwrap() as f32 + 20.0);
    for pressed in [true, false] {
        host.borrow_mut().events = vec![
            Event::PointerMoved(pos),
            Event::PointerButton { pos, button: egui::PointerButton::Primary, pressed, modifiers: egui::Modifiers::NONE },
        ];
        frame(&ctx, &host, &mut app);
    }
    assert!(app.session.prompt_text().starts_with("LINE Specify first point"));
    host.borrow_mut().close = true;
    frame(&ctx, &host, &mut app);
    assert!(app.ui.ribbon.floating_panels.is_empty());
    assert_eq!(app.session.docs.len(), documents);
    assert!(!app.quit_requested);
}

#[test]
fn multiple_native_panels_have_distinct_viewports_and_hide_with_the_ribbon() {
    let (ctx, host) = host();
    let mut app = app();
    app.run("ui.ribbon.float", json!({"panel":"modify"})).unwrap();
    frame(&ctx, &host, &mut app);
    let draw = elements(&mut app, &ctx, "Ribbon / Draw: Line");
    let modify = elements(&mut app, &ctx, "Ribbon / Modify: Move");
    assert_ne!(draw["viewport"], modify["viewport"]);
    let count = host.borrow().titles.len();
    app.run("ribbonclose", json!({})).unwrap();
    frame(&ctx, &host, &mut app);
    assert_eq!(host.borrow().titles.len(), count);
    assert!(ribbon::inspect_native_windows(&ctx)["windows"].as_array().unwrap().is_empty());
    app.run("ribbon", json!({})).unwrap();
    frame(&ctx, &host, &mut app);
    assert!(host.borrow().titles.len() > count);
}

#[test]
fn native_panels_follow_main_window_minimization_and_application_focus() {
    let (ctx, host) = host();
    let mut app = app();
    host.borrow_mut().viewports.get_mut(&ViewportId::ROOT).unwrap().focused = Some(true);
    frame(&ctx, &host, &mut app);
    host.borrow_mut().commands.clear();
    {
        let mut h = host.borrow_mut();
        let root = h.viewports.get_mut(&ViewportId::ROOT).unwrap();
        root.focused = Some(false);
        root.minimized = Some(true);
    }
    frame(&ctx, &host, &mut app);
    assert!(host.borrow().commands.iter().any(|c| matches!(c, ViewportCommand::Visible(false))));
    assert!(host.borrow().commands.iter().any(|c| matches!(c, ViewportCommand::WindowLevel(egui::WindowLevel::Normal))));
    host.borrow_mut().commands.clear();
    {
        let mut h = host.borrow_mut();
        let root = h.viewports.get_mut(&ViewportId::ROOT).unwrap();
        root.focused = Some(true);
        root.minimized = Some(false);
    }
    frame(&ctx, &host, &mut app);
    assert!(host.borrow().commands.iter().any(|c| matches!(c, ViewportCommand::Visible(true))));
    assert!(host.borrow().commands.iter().any(|c| matches!(c, ViewportCommand::WindowLevel(egui::WindowLevel::AlwaysOnTop))));
}

#[test]
fn whole_ribbon_native_window_closes_without_docking_or_quitting_and_reopens() {
    let (ctx, host) = host();
    let mut app = CadApp::new(Session::default(), Services::default());
    app.run("ribbon", json!({})).unwrap();
    app.run("ui.ribbon.window", json!({"floating":true,"desktopPosition":[-1000,150]})).unwrap();
    for _ in 0..6 {
        frame(&ctx, &host, &mut app);
    }
    assert!(host.borrow().titles.iter().any(|t| t == "CADCraft — Ribbon"));
    assert!(app.ui.ribbon.window.floating);
    host.borrow_mut().close = true;
    frame(&ctx, &host, &mut app);
    assert!(!app.ui.show_ribbon);
    assert!(app.ui.ribbon.window.floating);
    assert!(!app.quit_requested);
    app.run("ribbon", json!({})).unwrap();
    frame(&ctx, &host, &mut app);
    assert!(app.ui.show_ribbon);
    assert!(app.ui.ribbon.window.floating);
    app.run("ui.ribbon.window", json!({"floating":false})).unwrap();
    frame(&ctx, &host, &mut app);
    assert!(ribbon::inspect_native_windows(&ctx)["windows"].as_array().unwrap().is_empty());
}

#[test]
fn whole_ribbon_and_individual_panels_have_independent_native_identities() {
    let (ctx, host) = host();
    let mut app = app();
    app.run("ui.ribbon.window", json!({"floating":true})).unwrap();
    frame(&ctx, &host, &mut app);
    // The host reports new viewport metadata on the next root input frame.
    frame(&ctx, &host, &mut app);
    assert!(host.borrow().titles.iter().any(|t| t == "CADCraft — Ribbon"));
    assert!(host.borrow().titles.iter().any(|t| t == "CADCraft — Home / Draw"));
    assert_eq!(ribbon::inspect_native_windows(&ctx)["windows"].as_array().unwrap().len(), 2);
}

#[test]
fn native_return_button_does_not_redetach_a_panel_when_geometry_changes() {
    let (ctx, host) = host();
    let mut app = app();
    for _ in 0..6 {
        frame(&ctx, &host, &mut app);
    }
    let button = elements(&mut app, &ctx, "Return to Ribbon");
    let r = button["rect"].as_array().unwrap();
    let pos = egui::pos2(r[0].as_f64().unwrap() as f32 + 10.0, r[1].as_f64().unwrap() as f32 + 10.0);
    for pressed in [true, false] {
        host.borrow_mut().position = egui::pos2(400.0, if pressed { 300.0 } else { 350.0 });
        host.borrow_mut().events = vec![
            Event::PointerMoved(pos),
            Event::PointerButton { pos, button: egui::PointerButton::Primary, pressed, modifiers: egui::Modifiers::NONE },
        ];
        frame(&ctx, &host, &mut app);
    }
    assert!(app.ui.ribbon.floating_panels.is_empty());
    assert!(!app.quit_requested);
}

#[test]
fn whole_ribbon_waits_for_native_resize_and_then_persists_user_resizing() {
    let (ctx, host) = host();
    let mut app = CadApp::new(Session::default(), Services::default());
    app.run("ribbon", json!({})).unwrap();
    app.run("ui.ribbon.window", json!({"floating":true})).unwrap();
    for _ in 0..6 {
        frame(&ctx, &host, &mut app);
    }
    let id = egui::ViewportId::from_hash_of("cadcraft-whole-ribbon");
    host.borrow_mut().resize_delay = 12;
    app.run("ui.ribbon.window", json!({"size":[1000,260]})).unwrap();
    frame(&ctx, &host, &mut app);
    assert_eq!(app.ui.ribbon.window.size, [1000.0, 260.0]);
    for _ in 0..32 {
        frame(&ctx, &host, &mut app);
    }
    assert_eq!(host.borrow().sizes[&id], vec2(1000.0, 260.0));
    assert_eq!(app.ui.ribbon.window.size, [1000.0, 260.0]);
    host.borrow_mut().sizes.insert(id, vec2(800.0, 300.0));
    frame(&ctx, &host, &mut app);
    assert_eq!(app.ui.ribbon.window.size, [800.0, 300.0]);
}

#[test]
fn native_ribbon_programmatic_placement_does_not_dock_but_user_move_does() {
    for allow in [true, false] {
        let (ctx, host) = host();
        let mut app = CadApp::new(Session::default(), Services::default());
        app.run("ribbon", json!({})).unwrap();
        app.run("ui.ribbon.window", json!({"floating":true,"desktopPosition":[450,100],"allowDocking":allow})).unwrap();
        for _ in 0..40 {
            frame(&ctx, &host, &mut app);
        }
        assert!(app.ui.ribbon.window.floating, "saved edge placement must not dock");
        host.borrow_mut().position = egui::pos2(500.0, 110.0);
        for _ in 0..60 {
            frame(&ctx, &host, &mut app);
        }
        assert_eq!(app.ui.ribbon.window.floating, !allow, "user drag docking toggle");
        assert!(app.ui.ribbon.window.anchor.is_none());
    }
}

#[test]
fn native_floating_auto_hide_retains_expanded_size_and_can_reveal_on_hover() {
    let (ctx, host) = host();
    let mut app = CadApp::new(Session::default(), Services::default());
    app.run("ribbon", json!({})).unwrap();
    app.run("ui.ribbon.window", json!({"floating":true,"autoHide":true,"desktopPosition":[2000,250]})).unwrap();
    for _ in 0..40 {
        frame(&ctx, &host, &mut app);
    }
    let id = ViewportId::from_hash_of("cadcraft-whole-ribbon");
    assert_eq!(host.borrow().sizes[&id].x, 32.0);
    assert_eq!(app.ui.ribbon.window.size, [360.0, 520.0]);
    host.borrow_mut().events = vec![Event::PointerMoved(egui::pos2(16.0, 100.0))];
    for _ in 0..8 {
        frame(&ctx, &host, &mut app);
    }
    assert_eq!(host.borrow().sizes[&id].x, 360.0);
    assert_eq!(app.ui.ribbon.window.size, [360.0, 520.0]);
    host.borrow_mut().events = vec![Event::PointerGone];
    for _ in 0..60 {
        frame(&ctx, &host, &mut app);
    }
    assert_eq!(host.borrow().sizes[&id].x, 32.0);
    assert_eq!(app.ui.ribbon.window.size, [360.0, 520.0]);
    assert_eq!(app.ui.ribbon.window.desktop_position, Some([2000.0, 250.0]));
}

#[test]
fn native_ribbon_control_drag_bypasses_edge_docking() {
    let (ctx, host) = host();
    let mut app = CadApp::new(Session::default(), Services::default());
    app.run("ribbon", json!({})).unwrap();
    app.run("ui.ribbon.window", json!({"floating":true,"desktopPosition":[500,400]})).unwrap();
    for _ in 0..10 {
        frame(&ctx, &host, &mut app);
    }
    host.borrow_mut().modifiers = egui::Modifiers::CTRL;
    host.borrow_mut().position = egui::pos2(500.0, 110.0);
    for _ in 0..60 {
        frame(&ctx, &host, &mut app);
    }
    assert!(app.ui.ribbon.window.floating);
    host.borrow_mut().modifiers = egui::Modifiers::NONE;
    for _ in 0..60 {
        frame(&ctx, &host, &mut app);
    }
    assert!(app.ui.ribbon.window.floating, "releasing Ctrl after the move must not dock later");
}
