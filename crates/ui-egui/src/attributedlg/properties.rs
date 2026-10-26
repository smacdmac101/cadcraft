//! Per-attribute property drafts, preserving inherited defaults until explicitly edited.
use cadcraft_doc::{Attrib, Drawing, EntityKind, Lineweight};
use serde_json::{Map, Value, json};

#[derive(Clone)]
pub(super) struct Properties {
    pub tag: String,
    constant: bool,
    initial: [String; 4],
    draft: [String; 4],
}

impl Properties {
    pub fn new(drawing: Option<&Drawing>, block: &str, attribute: &Attrib) -> Self {
        let definition = drawing
            .and_then(|d| d.block(block))
            .and_then(|b| b.entities.iter().find(|e| matches!(&e.kind, EntityKind::AttDef(a) if a.tag.eq_ignore_ascii_case(&attribute.tag))));
        let effective = attribute.props.over(&definition.map(|e| e.common.clone()).unwrap_or_default());
        let initial = [effective.layer, effective.color.name(), effective.linetype, effective.lineweight.name()];
        Self { tag: attribute.tag.clone(), constant: attribute.constant, draft: initial.clone(), initial }
    }

    pub fn pending(&self) -> Map<String, Value> {
        let mut fields = Map::new();
        if !self.constant {
            for (index, key) in ["layer", "color", "linetype", "lineweight"].into_iter().enumerate() {
                if self.draft[index] != self.initial[index] {
                    fields.insert(key.into(), json!(self.draft[index]));
                }
            }
        }
        fields
    }

    pub fn applied(&mut self) {
        self.initial.clone_from(&self.draft);
    }

    pub fn show(&mut self, ui: &mut egui::Ui, layers: &[String], linetypes: &[String]) {
        if self.constant {
            ui.label("Constant attribute properties are read-only.");
            return;
        }
        egui::Grid::new("attribute_properties").num_columns(2).spacing([16.0, 8.0]).show(ui, |ui| {
            choice(ui, "Layer", &mut self.draft[0], layers.iter().map(String::as_str));
            ui.end_row();
            choice(ui, "Linetype", &mut self.draft[2], ["ByLayer", "ByBlock"].into_iter().chain(linetypes.iter().map(String::as_str)));
            ui.end_row();
            let label = ui.label("Color");
            ui.horizontal(|ui| {
                ui.add(egui::TextEdit::singleline(&mut self.draft[1]).desired_width(120.0).char_limit(128)).labelled_by(label.id);
                egui::ComboBox::from_id_salt("attribute_color_choices").selected_text("Choose").show_ui(ui, |ui| {
                    for color in ["ByLayer", "ByBlock", "Red", "Yellow", "Green", "Cyan", "Blue", "Magenta", "White"] {
                        ui.selectable_value(&mut self.draft[1], color.into(), color);
                    }
                });
            });
            ui.end_row();
            let weights: Vec<_> = Lineweight::STANDARD.into_iter().map(|v| Lineweight::Mm100(v).name()).collect();
            choice(ui, "Lineweight", &mut self.draft[3], ["ByLayer", "ByBlock"].into_iter().chain(weights.iter().map(String::as_str)));
            ui.end_row();
        });
        ui.add_space(8.0);
        ui.label("Color accepts a name, index, #RRGGBB or R,G,B.");
        ui.label("Properties apply to this attribute in this block reference only.");
    }
}

fn choice<'a>(ui: &mut egui::Ui, label: &str, value: &mut String, choices: impl Iterator<Item = &'a str>) {
    let label = ui.label(label);
    egui::ComboBox::from_id_salt(label.id)
        .selected_text(value.as_str())
        .show_ui(ui, |ui| {
            for choice in choices {
                ui.selectable_value(value, choice.into(), choice);
            }
        })
        .response
        .labelled_by(label.id);
}
