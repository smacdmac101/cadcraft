//! Selection-aware ribbon color control. Popup contents scroll in small native panels.
use crate::{
    CadApp,
    icons::{self, Icon},
    theme::Tokens,
};
use cadcraft_color::{Color, Rgb};
use egui::{Rect, Sense, vec2};
use serde_json::json;

pub(super) fn show(app: &mut CadApp, ui: &mut egui::Ui, width: f32) -> Option<egui::Response> {
    let doc = app.session.doc().ok()?;
    let selection = app.session.selection();
    let editable = selection.iter().all(|h| doc.entity(*h).is_some_and(|e| doc.layer(&e.common.layer).is_none_or(|l| !l.locked)));
    let mut entities = selection.iter().filter_map(|h| doc.entity(*h));
    let first = entities.next();
    let shown = if selection.is_empty() {
        Some(
            doc.header
                .get("CECOLOR_RGB")
                .and_then(|v| v.as_i64())
                .filter(|v| (0..=0xff_ffff).contains(v))
                .map(|v| Color::True(Rgb::from_u32(v as u32)))
                .unwrap_or_else(|| Color::from_aci(doc.header.i64("CECOLOR", 256) as i16)),
        )
    } else {
        first.map(|e| e.common.color).filter(|c| entities.all(|e| e.common.color == *c))
    };
    let layer = first.map(|e| e.common.layer.clone()).unwrap_or_else(|| doc.header.str("CLAYER", "0"));
    let resolved = shown.map(|c| c.resolve(doc.layer(&layer).map_or(Color::Index(7), |l| l.color), Color::Index(7)));
    let t = Tokens::get();
    let (rect, response) = ui.allocate_exact_size(vec2(width, 24.0), Sense::click());
    ui.painter().rect_filled(rect, 3.0, if response.hovered() { t.control_hover } else { t.control });
    let label = crate::tl!("Ribbon Color");
    let value = shown.map(|c| c.name()).unwrap_or_default();
    response.widget_info(|| {
        let mut info = egui::WidgetInfo::labeled(egui::WidgetType::ComboBox, ui.is_enabled(), label);
        info.current_text_value = Some(value.clone());
        info
    });
    crate::control::record_widget(ui, &response, label, "combobox", None);
    if let Some(Rgb(r, g, b)) = resolved {
        ui.painter().rect_filled(Rect::from_min_size(rect.min + vec2(5.0, 6.0), vec2(12.0, 12.0)), 1.0, egui::Color32::from_rgb(r, g, b));
    }
    let mut text = egui::text::LayoutJob::simple(crate::i18n::t(&value).to_string(), crate::theme::body(), t.text, (width - 45.0).max(1.0));
    text.wrap.max_rows = 1;
    let text = ui.painter().layout_job(text);
    ui.painter().galley(rect.min + vec2(23.0, (rect.height() - text.size().y) / 2.0), text, t.text);
    icons::paint(ui.painter(), Rect::from_center_size(egui::pos2(rect.right() - 12.0, rect.center().y), vec2(12.0, 12.0)), Icon::ChevronDown, false);
    let mut picked = None;
    egui::Popup::from_toggle_button_response(&response).width(width).close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside).show(|ui| {
        let max_height = (ui.ctx().content_rect().height() - 24.0).clamp(60.0, 340.0);
        egui::ScrollArea::vertical().max_height(max_height).show(ui, |ui| {
            ui.add_enabled_ui(editable, |ui| {
                for color in [
                    Color::ByLayer,
                    Color::ByBlock,
                    Color::Index(1),
                    Color::Index(2),
                    Color::Index(3),
                    Color::Index(4),
                    Color::Index(5),
                    Color::Index(6),
                    Color::Index(7),
                ] {
                    ui.horizontal(|ui| {
                        let (swatch, _) = ui.allocate_exact_size(vec2(12.0, 12.0), Sense::hover());
                        crate::layers::swatch(ui.painter(), swatch, color);
                        let response = ui.selectable_label(shown == Some(color), crate::i18n::t(&color.name()));
                        crate::control::record_widget(
                            ui,
                            &response,
                            &format!("Ribbon Color / {}", color.name()),
                            "button",
                            Some(shown == Some(color)),
                        );
                        if response.clicked() {
                            picked = Some(color);
                        }
                    });
                }
                ui.separator();
                // A compact editor keeps all 255 indexed and all RGB colors reachable
                // even when this panel occupies a small independent native viewport.
                let id = ui.id().with("color_input");
                let mut input = ui.data_mut(|d| d.get_temp::<String>(id)).unwrap_or_default();
                ui.label(crate::tl!("Index or RGB color"));
                let edit = ui.add(egui::TextEdit::singleline(&mut input).desired_width((width - 20.0).max(40.0)).hint_text("1–255 / #RRGGBB"));
                edit.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::TextEdit, ui.is_enabled(), crate::tl!("Custom Color")));
                let parsed = Color::parse(input.trim());
                if ui.add_enabled(parsed.is_some(), egui::Button::new(crate::tl!("Apply Color"))).clicked() {
                    picked = parsed;
                }
                ui.data_mut(|d| d.insert_temp(id, input));
            });
        });
        if picked.is_some() {
            ui.close();
        }
    });
    if let Some(color) = picked {
        let (command, params) = if selection.is_empty() {
            ("color", json!({"color":color.name()}))
        } else {
            ("properties.set", json!({"handles":selection.iter().map(|h| h.hex()).collect::<Vec<_>>(),"color":color.name()}))
        };
        let _ = app.run(command, params);
        app.canvas.list = None;
    }
    Some(response.on_hover_text(if value.is_empty() { crate::tl!("Varies").to_string() } else { value }))
}
