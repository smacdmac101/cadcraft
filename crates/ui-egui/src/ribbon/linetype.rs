//! Loaded linetypes in the ribbon, with current/selected-object semantics.
use crate::{
    CadApp,
    icons::{self, Icon},
    theme::Tokens,
};
use egui::{Rect, Sense, vec2};
use serde_json::json;

pub(super) fn show(app: &mut CadApp, ui: &mut egui::Ui, width: f32) -> Option<egui::Response> {
    let doc = app.session.doc().ok()?;
    let selection = app.session.selection();
    let editable = selection.iter().all(|h| doc.entity(*h).is_some_and(|e| doc.layer(&e.common.layer).is_none_or(|l| !l.locked)));
    let mut values = selection.iter().filter_map(|h| doc.entity(*h).map(|e| e.common.linetype.as_str()));
    let shown = if selection.is_empty() {
        Some(doc.header.str("CELTYPE", "ByLayer"))
    } else {
        values.next().filter(|first| values.all(|v| v.eq_ignore_ascii_case(first))).map(str::to_string)
    };
    let value = shown.as_deref().unwrap_or_default();
    let t = Tokens::get();
    let (rect, response) = ui.allocate_exact_size(vec2(width, 24.0), Sense::click());
    ui.painter().rect_filled(rect, 3.0, if response.hovered() { t.control_hover } else { t.control });
    let label = crate::tl!("Ribbon Linetype");
    response.widget_info(|| {
        let mut info = egui::WidgetInfo::labeled(egui::WidgetType::ComboBox, ui.is_enabled(), label);
        info.current_text_value = Some(value.to_string());
        info
    });
    crate::control::record_widget(ui, &response, label, "combobox", None);
    if let Some(name) = &shown {
        crate::layers::lt_sample(ui.painter(), Rect::from_min_size(rect.min + vec2(5.0, 6.0), vec2(30.0, 12.0)), doc.linetype(name), t.text);
    }
    let mut text = egui::text::LayoutJob::simple(crate::i18n::t(value).to_string(), crate::theme::body(), t.text, (width - 62.0).max(1.0));
    text.wrap.max_rows = 1;
    let text = ui.painter().layout_job(text);
    ui.painter().galley(rect.min + vec2(40.0, (rect.height() - text.size().y) / 2.0), text, t.text);
    icons::paint(ui.painter(), Rect::from_center_size(egui::pos2(rect.right() - 12.0, rect.center().y), vec2(12.0, 12.0)), Icon::ChevronDown, false);
    let mut action = None;
    egui::Popup::from_toggle_button_response(&response).width(width).close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside).show(|ui| {
        egui::ScrollArea::vertical().max_height((ui.ctx().content_rect().height() - 24.0).clamp(60.0, 320.0)).show(ui, |ui| {
            let mut names = vec!["ByLayer".to_string(), "ByBlock".to_string()];
            let mut loaded: Vec<_> = doc
                .linetypes
                .iter()
                .filter(|l| !["bylayer", "byblock"].contains(&l.name.to_ascii_lowercase().as_str()))
                .map(|l| l.name.clone())
                .collect();
            loaded.sort_by_cached_key(|n| n.to_lowercase());
            names.extend(loaded);
            for name in names {
                ui.horizontal(|ui| {
                    let (sample, _) = ui.allocate_exact_size(vec2(30.0, 16.0), Sense::hover());
                    crate::layers::lt_sample(ui.painter(), sample, doc.linetype(&name), t.text);
                    let selected = shown.as_ref().is_some_and(|s| s.eq_ignore_ascii_case(&name));
                    let response = ui.add_enabled(editable, egui::Button::selectable(selected, crate::i18n::t(&name)).truncate());
                    crate::control::record_widget(ui, &response, &format!("Ribbon Linetype / {name}"), "button", Some(selected));
                    if response.clicked() {
                        action = Some(if selection.is_empty() {
                            ("linetype", json!({"current":name}))
                        } else {
                            ("properties.set", json!({"handles":selection.iter().map(|h| h.hex()).collect::<Vec<_>>(),"linetype":name}))
                        });
                        ui.close();
                    }
                });
            }
            ui.separator();
            ui.collapsing(crate::tl!("Load Linetype"), |ui| {
                for lt in cadcraft_doc::library::standard_linetypes_for(doc).into_iter().filter(|l| doc.linetype(&l.name).is_none()) {
                    if ui.button(&lt.name).on_hover_text(&lt.description).clicked() {
                        action = Some(("linetype", json!({"load":lt.name})));
                    }
                }
            });
        });
    });
    let response = response.on_hover_text(shown.as_deref().unwrap_or(crate::tl!("Varies")));
    if let Some((command, params)) = action {
        let _ = app.run(command, params);
        app.canvas.list = None;
    }
    Some(response)
}
