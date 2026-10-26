//! Draft attribute text options; only explicit edits become command parameters.
use cadcraft_doc::{Attrib, HAlign, VAlign};
use serde_json::{Map, Value, json};

const FIELDS: [&str; 6] = ["textStyle", "justify", "height", "widthFactor", "rotation", "oblique"];
const JUSTIFICATIONS: [(&str, &str); 15] = [
    ("L", "Left"),
    ("A", "Align"),
    ("F", "Fit"),
    ("C", "Center"),
    ("M", "Middle"),
    ("R", "Right"),
    ("TL", "Top left"),
    ("TC", "Top center"),
    ("TR", "Top right"),
    ("ML", "Middle left"),
    ("MC", "Middle center"),
    ("MR", "Middle right"),
    ("BL", "Bottom left"),
    ("BC", "Bottom center"),
    ("BR", "Bottom right"),
];

#[derive(Clone)]
pub(super) struct Format {
    pub tag: String,
    supported: bool,
    initial: [String; 6],
    draft: [String; 6],
}

impl Format {
    pub fn new(a: &Attrib) -> Self {
        let t = &a.text;
        let code = match (t.halign, t.valign) {
            (HAlign::Left, VAlign::Baseline) => "L",
            (HAlign::Center, VAlign::Baseline) => "C",
            (HAlign::Right, VAlign::Baseline) => "R",
            (HAlign::Middle, VAlign::Baseline) => "M",
            (HAlign::Aligned, VAlign::Baseline) => "A",
            (HAlign::Fit, VAlign::Baseline) => "F",
            (HAlign::Left, VAlign::Top) => "TL",
            (HAlign::Center, VAlign::Top) => "TC",
            (HAlign::Right, VAlign::Top) => "TR",
            (HAlign::Left, VAlign::Middle) => "ML",
            (HAlign::Center, VAlign::Middle) => "MC",
            (HAlign::Right, VAlign::Middle) => "MR",
            (HAlign::Left, VAlign::Bottom) => "BL",
            (HAlign::Center, VAlign::Bottom) => "BC",
            (HAlign::Right, VAlign::Bottom) => "BR",
            _ => "Other (preserved)",
        };
        let initial = [
            t.style.clone(),
            code.into(),
            t.height.to_string(),
            t.width_factor.to_string(),
            t.rotation.to_degrees().to_string(),
            t.oblique.to_degrees().to_string(),
        ];
        Self { tag: a.tag.clone(), supported: !a.constant && a.multiline.is_none(), draft: initial.clone(), initial }
    }

    pub fn pending(&self) -> Result<Map<String, Value>, String> {
        let mut fields = Map::new();
        if self.supported {
            for (index, key) in FIELDS.iter().enumerate() {
                if self.draft[index] != self.initial[index] {
                    let value = if index < 2 {
                        json!(self.draft[index])
                    } else {
                        let n = self.draft[index]
                            .parse::<f64>()
                            .ok()
                            .filter(|n| n.is_finite())
                            .ok_or_else(|| format!("{}: {key} must be a finite number", self.tag))?;
                        json!(n)
                    };
                    fields.insert((*key).into(), value);
                }
            }
        }
        Ok(fields)
    }

    pub fn applied(&mut self) {
        self.initial.clone_from(&self.draft);
    }

    pub fn show(&mut self, ui: &mut egui::Ui, styles: &[String]) {
        if !self.supported {
            ui.label("Text options are currently available for nonconstant single-line attributes.");
            ui.label("Existing multiline formatting is preserved.");
            return;
        }
        egui::Grid::new("attribute_text_options").num_columns(2).spacing([16.0, 8.0]).show(ui, |ui| {
            let label = ui.label("Text style");
            egui::ComboBox::from_id_salt("attribute_text_style")
                .selected_text(&self.draft[0])
                .show_ui(ui, |ui| {
                    for style in styles {
                        ui.selectable_value(&mut self.draft[0], style.clone(), style);
                    }
                })
                .response
                .labelled_by(label.id);
            ui.end_row();
            let label = ui.label("Justification");
            let shown = JUSTIFICATIONS.iter().find(|(code, _)| *code == self.draft[1]).map_or(self.draft[1].as_str(), |(_, label)| *label);
            egui::ComboBox::from_id_salt("attribute_justification")
                .selected_text(shown)
                .show_ui(ui, |ui| {
                    for (code, label) in JUSTIFICATIONS {
                        let enabled = !matches!(code, "A" | "F") || self.initial[1] == code;
                        ui.add_enabled_ui(enabled, |ui| {
                            ui.selectable_value(&mut self.draft[1], code.into(), label);
                        });
                    }
                })
                .response
                .labelled_by(label.id);
            ui.end_row();
            for (index, label) in [(2, "Height"), (3, "Width factor"), (4, "Rotation (degrees)"), (5, "Oblique angle (degrees)")] {
                let label = ui.label(label);
                ui.add(egui::TextEdit::singleline(&mut self.draft[index]).desired_width(160.0)).labelled_by(label.id);
                ui.end_row();
            }
        });
        ui.add_space(8.0);
        ui.label("Align/Fit need two placement points. Text options apply to this block reference only.");
    }
}
