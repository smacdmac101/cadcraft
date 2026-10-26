//! A nested color draft. Accepting it does not modify the drawing.
use cadcraft_color::Color;

#[derive(Clone)]
pub(super) struct Picker {
    pub target: usize,
    value: String,
    original: String,
    id: egui::Id,
}

pub(super) enum Action {
    Pending,
    Cancel,
    Accept(Color),
}

impl Picker {
    pub fn new(ctx: &egui::Context, target: usize, value: &str) -> Self {
        let id = egui::Id::new("attribute_color_palette");
        // Each opening starts from its target's current draft, never another tag's RGB.
        ctx.data_mut(|d| {
            d.remove::<[u8; 3]>(id.with("rgb"));
            d.remove::<u8>(id.with("tab"));
        });
        Self { target, value: value.into(), original: value.into(), id }
    }

    pub fn show(&mut self, ctx: &egui::Context) -> Action {
        let mut action = Action::Pending;
        egui::Modal::new(egui::Id::new("attribute_select_color_modal")).show(ctx, |ui| {
            ui.set_width(330.0);
            ui.heading("Select Color");
            ui.separator();
            let current = Color::parse(&self.value).unwrap_or(Color::ByLayer);
            if let Some(color) = crate::layers::color_palette_draft(ui, self.id, current) {
                self.value = color.name();
            }
            ui.separator();
            ui.horizontal(|ui| {
                let label = ui.label("Color value");
                if ui.add(egui::TextEdit::singleline(&mut self.value).desired_width(200.0).char_limit(128)).labelled_by(label.id).changed() {
                    ui.data_mut(|d| d.remove::<[u8; 3]>(self.id.with("rgb")));
                }
            });
            let parsed = Color::parse(&self.value);
            ui.horizontal(|ui| {
                for (label, value) in [("Old", self.original.as_str()), ("New", self.value.as_str())] {
                    ui.label(label);
                    if let Some(color @ (Color::Index(_) | Color::True(_))) = Color::parse(value) {
                        let (rect, _) = ui.allocate_exact_size(egui::vec2(28.0, 18.0), egui::Sense::hover());
                        crate::layers::swatch(ui.painter(), rect, color);
                    }
                    ui.label(value);
                }
            });
            if parsed.is_none() {
                ui.label("Enter a color name, index 0–256, #RRGGBB or R,G,B.");
            }
            ui.horizontal(|ui| {
                if ui.add_enabled(parsed.is_some(), egui::Button::new("Use Color")).clicked()
                    && let Some(color) = parsed
                {
                    action = Action::Accept(color);
                }
                if ui.button("Cancel Color").clicked() {
                    action = Action::Cancel;
                }
            });
        });
        // Consume before the parent checks Escape. An outside click only meets the shield.
        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Escape)) {
            action = Action::Cancel;
        }
        action
    }
}
