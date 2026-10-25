//! Dimension styles: drawing defaults without selection, object styles with selection.
use crate::{
    CadApp,
    icons::{self, Icon},
    theme::Tokens,
};
use cadcraft_doc::EntityKind;
use egui::{Rect, Sense, vec2};
use serde_json::json;

pub(super) fn show(app: &mut CadApp, ui: &mut egui::Ui, width: f32) -> Option<egui::Response> {
    let doc = app.session.doc().ok()?;
    let selection = app.session.selection();
    let dimensions: Vec<_> = selection
        .iter()
        .filter_map(|h| {
            let e = doc.entity(*h)?;
            if let EntityKind::Dimension(dm) = &e.kind { Some((*h, e, dm)) } else { None }
        })
        .collect();
    let editable =
        selection.is_empty() || (!dimensions.is_empty() && dimensions.iter().all(|(_, e, _)| doc.layer(&e.common.layer).is_none_or(|l| !l.locked)));
    let shown = if selection.is_empty() {
        Some(doc.header.str("DIMSTYLE", "Standard"))
    } else {
        dimensions
            .first()
            .filter(|(_, _, first)| dimensions.iter().all(|(_, _, dm)| dm.style.eq_ignore_ascii_case(&first.style)))
            .map(|(_, _, dm)| dm.style.clone())
    };
    let value = shown.as_deref().unwrap_or_default();
    let t = Tokens::get();
    let (rect, response) = ui.allocate_exact_size(vec2(width, 24.0), Sense::click());
    ui.painter().rect_filled(rect, 3.0, if response.hovered() { t.control_hover } else { t.control });
    let label = crate::tl!("Ribbon Dimension Style");
    response.widget_info(|| {
        let mut info = egui::WidgetInfo::labeled(egui::WidgetType::ComboBox, ui.is_enabled(), label);
        info.current_text_value = Some(value.to_string());
        info
    });
    crate::control::record_widget(ui, &response, label, "combobox", None);
    icons::paint(ui.painter(), Rect::from_min_size(rect.min + vec2(4.0, 4.0), vec2(16.0, 16.0)), Icon::DimLinear, false);
    let mut text = egui::text::LayoutJob::simple(value.to_string(), crate::theme::body(), t.text, (width - 50.0).max(1.0));
    text.wrap.max_rows = 1;
    let text = ui.painter().layout_job(text);
    ui.painter().galley(rect.min + vec2(26.0, (rect.height() - text.size().y) / 2.0), text, t.text);
    icons::paint(ui.painter(), Rect::from_center_size(egui::pos2(rect.right() - 12.0, rect.center().y), vec2(12.0, 12.0)), Icon::ChevronDown, false);
    let mut action = None;
    let mut manage = false;
    egui::Popup::from_toggle_button_response(&response).width(width).close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside).show(|ui| {
        egui::ScrollArea::vertical().max_height((ui.ctx().content_rect().height() - 24.0).clamp(60.0, 320.0)).show(ui, |ui| {
            let mut styles: Vec<_> = doc.dim_styles.iter().collect();
            styles.sort_by_cached_key(|s| s.name.to_lowercase());
            for style in styles {
                let selected = shown.as_ref().is_some_and(|s| s.eq_ignore_ascii_case(&style.name));
                let response = ui.add_enabled(editable, egui::Button::selectable(selected, &style.name).truncate());
                response.widget_info(|| {
                    egui::WidgetInfo::selected(
                        egui::WidgetType::Button,
                        ui.is_enabled() && editable,
                        selected,
                        format!("Ribbon Dimension Style / {}", style.name),
                    )
                });
                crate::control::record_widget(ui, &response, &format!("Ribbon Dimension Style / {}", style.name), "button", Some(selected));
                if response.clicked() {
                    action = Some(if selection.is_empty() {
                        ("dimstyle.current", json!({"name":style.name}))
                    } else {
                        ("properties.set", json!({"handles":dimensions.iter().map(|(h,_,_)| h.hex()).collect::<Vec<_>>(),"dimStyle":style.name}))
                    });
                    ui.close();
                }
            }
            ui.separator();
            if ui.button(crate::tl!("Manage Dimension Styles")).clicked() {
                manage = true;
                ui.close();
            }
        });
    });
    if let Some((command, params)) = action {
        let _ = app.run(command, params);
        app.canvas.list = None;
    }
    if manage {
        app.start("dimstyle");
    }
    Some(response.on_hover_text(shown.as_deref().unwrap_or(crate::tl!("Varies"))))
}
