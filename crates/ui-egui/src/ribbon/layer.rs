//! Ribbon layer control: current layer without a selection, object layers with one.
use crate::{
    CadApp,
    icons::{self, Icon},
    theme::Tokens,
};
use egui::{Rect, Sense, pos2, vec2};
use serde_json::json;

pub(super) fn show(app: &mut CadApp, ui: &mut egui::Ui, width: f32) -> Option<egui::Response> {
    let doc = app.session.doc().ok()?;
    let selection = app.session.selection();
    let editable = selection.iter().all(|h| doc.entity(*h).is_some_and(|e| doc.layer(&e.common.layer).is_none_or(|l| !l.locked)));
    let mut names = selection.iter().filter_map(|h| doc.entity(*h).map(|e| e.common.layer.as_str()));
    let shown = if selection.is_empty() {
        Some(doc.header.str("CLAYER", "0"))
    } else {
        names.next().filter(|first| names.all(|name| name.eq_ignore_ascii_case(first))).map(str::to_string)
    };
    let info = shown.as_ref().and_then(|name| doc.layer(name)).cloned();
    let t = Tokens::get();
    let (rect, response) = ui.allocate_exact_size(vec2(width, 24.0), Sense::click());
    ui.painter().rect_filled(rect, 3.0, if response.hovered() { t.control_hover } else { t.control });
    let label = crate::tl!("Ribbon Layer");
    response.widget_info(|| {
        let mut info = egui::WidgetInfo::labeled(egui::WidgetType::ComboBox, ui.is_enabled(), label);
        info.current_text_value = Some(shown.clone().unwrap_or_default());
        info
    });
    crate::control::record_widget(ui, &response, label, "combobox", None);
    let start = if let Some(layer) = info {
        for (index, icon) in [
            if layer.on { Icon::Bulb } else { Icon::BulbOff },
            if layer.frozen { Icon::Snowflake } else { Icon::Sun },
            if layer.locked { Icon::Lock } else { Icon::Unlock },
        ]
        .iter()
        .enumerate()
        {
            icons::paint(ui.painter(), Rect::from_min_size(rect.min + vec2(4.0 + index as f32 * 18.0, 4.0), vec2(16.0, 16.0)), *icon, false);
        }
        crate::layers::swatch(ui.painter(), Rect::from_min_size(rect.min + vec2(60.0, 7.0), vec2(10.0, 10.0)), layer.color);
        76.0
    } else {
        6.0
    };
    let mut text = egui::text::LayoutJob::simple(shown.clone().unwrap_or_default(), crate::theme::body(), t.text, (width - start - 22.0).max(1.0));
    text.wrap.max_rows = 1;
    let text = ui.painter().layout_job(text);
    ui.painter().galley(pos2(rect.left() + start, rect.center().y - text.size().y / 2.0), text, t.text);
    icons::paint(ui.painter(), Rect::from_center_size(pos2(rect.right() - 12.0, rect.center().y), vec2(12.0, 12.0)), Icon::ChevronDown, false);
    let mut action = None;
    egui::Popup::from_toggle_button_response(&response).width(width).close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside).show(|ui| {
        let mut layers = doc.layers.clone();
        layers.sort_by_cached_key(|l| l.name.to_lowercase());
        egui::ScrollArea::vertical().max_height(320.0).show(ui, |ui| {
            for layer in &layers {
                ui.push_id(&layer.name, |ui| {
                    ui.horizontal(|ui| {
                        for (key, icon, enabled, value, label) in [
                            ("on", if layer.on { Icon::Bulb } else { Icon::BulbOff }, true, !layer.on, "On/Off"),
                            (
                                "frozen",
                                if layer.frozen { Icon::Snowflake } else { Icon::Sun },
                                !doc.header.str("CLAYER", "0").eq_ignore_ascii_case(&layer.name),
                                !layer.frozen,
                                "Freeze/Thaw",
                            ),
                            ("locked", if layer.locked { Icon::Lock } else { Icon::Unlock }, true, !layer.locked, "Lock/Unlock"),
                        ] {
                            if ui.add_enabled_ui(enabled, |ui| icons::button(ui, icon, 18.0, label, false)).inner.clicked() {
                                action = Some(("layer.set", json!({"name":layer.name,key:value})));
                            }
                        }
                        let (swatch, _) = ui.allocate_exact_size(vec2(12.0, 12.0), Sense::hover());
                        crate::layers::swatch(ui.painter(), swatch, layer.color);
                        let selected = shown.as_ref().is_some_and(|n| n.eq_ignore_ascii_case(&layer.name));
                        if ui
                            .add_enabled_ui(editable, |ui| {
                                ui.add_sized(vec2((width - 100.0).max(40.0), 22.0), egui::Button::selectable(selected, &layer.name).truncate())
                            })
                            .inner
                            .on_hover_text(&layer.name)
                            .clicked()
                        {
                            action = Some(if selection.is_empty() {
                                ("layer.current", json!({"name":layer.name}))
                            } else {
                                ("properties.set", json!({"handles":selection.iter().map(|h| h.hex()).collect::<Vec<_>>(),"layer":layer.name}))
                            });
                            ui.close();
                        }
                    })
                });
            }
        });
    });
    let response = response.on_hover_text(shown.as_deref().unwrap_or(crate::tl!("Varies")));
    if let Some((command, params)) = action {
        let _ = app.run(command, params);
        app.canvas.list = None;
    }
    Some(response)
}
