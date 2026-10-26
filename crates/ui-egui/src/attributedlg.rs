//! Attribute value editing for a selected block. All edits go through ATTEDIT for undo.
use cadcraft_doc::{EntityKind, Handle};
use serde_json::json;

use crate::CadApp;
mod properties;
mod text_options;

#[derive(Clone, Copy, PartialEq)]
enum Tab {
    Attribute,
    TextOptions,
    Properties,
}

#[derive(Clone)]
struct State {
    drawing: u64,
    handle: Handle,
    name: String,
    values: Vec<(String, String, String, bool)>,
    initial_values: Vec<String>,
    formats: Vec<text_options::Format>,
    properties: Vec<properties::Properties>,
    tab: Tab,
    selected_attribute: usize,
    error: Option<String>,
}

pub(crate) fn open(app: &mut CadApp, handle: Handle) -> bool {
    let has_attributes =
        app.session.doc().ok().and_then(|d| d.entity(handle)).is_some_and(|e| matches!(&e.kind, EntityKind::Insert(i) if !i.attribs.is_empty()));
    if has_attributes {
        app.session.set_selection(vec![handle]);
        app.ui.dialog = Some("attedit".into());
    }
    has_attributes
}

pub(crate) fn open_selected(app: &mut CadApp) -> bool {
    let handle = app.session.state().ok().filter(|s| s.selection.len() == 1).and_then(|s| s.selection.first().copied());
    handle.is_some_and(|h| open(app, h))
}

pub(crate) fn show(app: &mut CadApp, ctx: &egui::Context) {
    let id = egui::Id::new("attribute_editor_state");
    if app.ui.dialog.as_deref() != Some("attedit") {
        ctx.data_mut(|d| d.remove::<State>(id));
        return;
    }
    let drawing = app.session.state().map_or(0, |s| s.uid);
    let selected = app.session.state().ok().and_then(|s| s.selection.first().copied());
    let Some((handle, name, attributes)) = selected.and_then(|h| {
        let EntityKind::Insert(i) = &app.session.doc().ok()?.entity(h)?.kind else {
            return None;
        };
        Some((h, i.block.clone(), i.attribs.clone()))
    }) else {
        app.ui.dialog = None;
        ctx.data_mut(|d| d.remove::<State>(id));
        return;
    };
    let mut state = ctx.data(|d| d.get_temp::<State>(id)).filter(|s| s.drawing == drawing && s.handle == handle).unwrap_or_else(|| State {
        drawing,
        handle,
        properties: attributes.iter().map(|a| properties::Properties::new(app.session.doc().ok(), &name, a)).collect(),
        name: app.session.doc().ok().and_then(|d| cadcraft_doc::dynblock::custom_properties(d, handle)).map_or(name, |(name, _)| name),
        values: attributes.iter().map(|a| (a.tag.clone(), a.prompt.clone(), a.text.value.clone(), a.constant)).collect(),
        initial_values: attributes.iter().map(|a| a.text.value.clone()).collect(),
        formats: attributes.iter().map(text_options::Format::new).collect(),
        tab: Tab::Attribute,
        selected_attribute: 0,
        error: None,
    });
    let locked = app.session.doc().ok().is_some_and(|d| d.entity(handle).and_then(|e| d.layer(&e.common.layer)).is_some_and(|l| l.locked));
    let mut styles: Vec<_> = app.session.doc().map(|d| d.text_styles.iter().map(|s| s.name.clone()).collect()).unwrap_or_default();
    styles.sort_by_cached_key(|name| name.to_lowercase());
    let mut layers: Vec<_> = app.session.doc().map(|d| d.layers.iter().map(|l| l.name.clone()).collect()).unwrap_or_default();
    layers.sort_by_cached_key(|name| name.to_lowercase());
    let mut linetypes: Vec<_> = app
        .session
        .doc()
        .map(|d| {
            d.linetypes.iter().filter(|l| !["bylayer", "byblock"].contains(&l.name.to_ascii_lowercase().as_str())).map(|l| l.name.clone()).collect()
        })
        .unwrap_or_default();
    linetypes.sort_by_cached_key(|name| name.to_lowercase());
    let mut open = true;
    let mut apply = false;
    let mut close = false;
    // Keep the draggable editor above a modal input shield. Drawing/ribbon clicks must
    // not select objects or feed a running command while its attribute drafts are open.
    egui::Modal::new(egui::Id::new("attribute_editor_modal")).frame(egui::Frame::NONE).show(ctx, |modal_ui| {
        let editor_id = egui::Id::new("Edit Attributes");
        // Keep the window above its shield even when an outside click raises the modal.
        ctx.set_sublayer(modal_ui.layer_id(), egui::LayerId::new(egui::Order::Foreground, editor_id));
        egui::Window::new("Edit Attributes").id(editor_id).order(egui::Order::Foreground).open(&mut open).default_width(520.0).show(ctx, |ui| {
            ui.label(format!("Block: {}", state.name));
            ui.horizontal(|ui| {
                ui.selectable_value(&mut state.tab, Tab::Attribute, "Attribute");
                ui.selectable_value(&mut state.tab, Tab::TextOptions, "Text Options");
                ui.selectable_value(&mut state.tab, Tab::Properties, "Properties");
            });
            ui.separator();
            ui.add_enabled_ui(!locked, |ui| {
                if state.tab != Tab::Attribute {
                    let shown = state.formats.get(state.selected_attribute).map_or("", |f| f.tag.as_str());
                    egui::ComboBox::from_id_salt("attribute_text_target").selected_text(shown).show_ui(ui, |ui| {
                        for (index, format) in state.formats.iter().enumerate() {
                            ui.selectable_value(&mut state.selected_attribute, index, &format.tag);
                        }
                    });
                    ui.add_space(8.0);
                    if state.tab == Tab::TextOptions {
                        if let Some(format) = state.formats.get_mut(state.selected_attribute) {
                            format.show(ui, &styles);
                        }
                    } else if let Some(properties) = state.properties.get_mut(state.selected_attribute) {
                        properties.show(ui, &layers, &linetypes);
                    }
                } else {
                    egui::ScrollArea::vertical().max_height(360.0).show(ui, |ui| {
                        for (index, (tag, prompt, value, constant)) in state.values.iter_mut().enumerate() {
                            ui.push_id(index, |ui| {
                                let label = ui.label(tag.as_str()).on_hover_text(prompt.as_str());
                                ui.add_enabled(!*constant, egui::TextEdit::singleline(value).desired_width(f32::INFINITY)).labelled_by(label.id);
                            });
                        }
                    });
                }
            });
            if locked {
                ui.label("This block reference is on a locked layer.");
            }
            if let Some(error) = &state.error {
                ui.colored_label(crate::theme::Tokens::get().text, error);
            }
            ui.separator();
            ui.horizontal(|ui| {
                if ui.add_enabled(!locked, egui::Button::new("OK")).clicked() {
                    apply = true;
                    close = true;
                }
                if ui.add_enabled(!locked, egui::Button::new("Apply")).clicked() {
                    apply = true;
                }
                if ui.button("Cancel").clicked() {
                    close = true;
                }
            });
        });
    });
    if apply {
        let values: serde_json::Map<_, _> = state
            .values
            .iter()
            .enumerate()
            .filter(|(index, (tag, _, value, constant))| {
                !constant && state.initial_values.get(*index) != Some(value) && attributes.iter().any(|a| a.tag == *tag && a.text.value != *value)
            })
            .map(|(_, (tag, _, value, _))| (tag.clone(), json!(value)))
            .collect();
        let formats = state.formats.iter().try_fold(serde_json::Map::new(), |mut out, format| {
            let fields = format.pending()?;
            if !fields.is_empty() {
                out.insert(format.tag.clone(), json!(fields));
            }
            Ok::<_, String>(out)
        });
        state.error = match formats {
            Err(error) => Some(error),
            Ok(formats) => {
                let properties: serde_json::Map<_, _> = state
                    .properties
                    .iter()
                    .filter_map(|property| {
                        let fields = property.pending();
                        (!fields.is_empty()).then(|| (property.tag.clone(), json!(fields)))
                    })
                    .collect();
                if values.is_empty() && formats.is_empty() && properties.is_empty() {
                    None
                } else {
                    app.run("attedit", json!({"handle": handle.hex(), "values": values, "formats":formats, "properties":properties})).err()
                }
            }
        };
        if state.error.is_some() {
            close = false;
        } else {
            state.initial_values = state.values.iter().map(|(_, _, value, _)| value.clone()).collect();
            for format in &mut state.formats {
                format.applied();
            }
            for property in &mut state.properties {
                property.applied();
            }
        }
    }
    if !open || close || ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
        app.ui.dialog = None;
        ctx.data_mut(|d| d.remove::<State>(id));
    } else {
        ctx.data_mut(|d| d.insert_temp(id, state));
    }
}

#[cfg(test)]
mod tests;
