//! Validate per-reference attribute formatting before any value or geometry mutation.
use cadcraft_doc::{Drawing, EntityKind, HAlign, Handle, Text, VAlign};
use serde_json::Value;
use std::collections::BTreeMap;

use crate::Result;
use crate::cmd::{bad, draw::justify};

pub(super) fn prepare(d: &Drawing, h: Handle, input: Option<&Value>) -> Result<BTreeMap<usize, Text>> {
    let Some(input) = input else {
        return Ok(BTreeMap::new());
    };
    let formats = input.as_object().ok_or_else(|| bad("attedit", "formats must be an object keyed by attribute tag"))?;
    if formats.len() > 4096 {
        return Err(bad("attedit", "too many attribute formats"));
    }
    if formats.is_empty() {
        return Ok(BTreeMap::new());
    }
    let Some(EntityKind::Insert(ins)) = d.entity(h).map(|e| &e.kind) else {
        return Err(bad("attedit", "formats require a block reference"));
    };
    // Index only requested tags; scan large references once, rather than once per edit.
    let mut positions = BTreeMap::new();
    for tag in formats.keys() {
        if positions.insert(tag.to_ascii_lowercase(), None).is_some() {
            return Err(bad("attedit", format!("duplicate attribute format `{tag}`")));
        }
    }
    for (index, a) in ins.attribs.iter().enumerate() {
        if let Some(position) = positions.get_mut(&a.tag.to_ascii_lowercase()) {
            if position.is_some() {
                return Err(bad("attedit", format!("attribute tag `{}` is ambiguous", a.tag)));
            }
            *position = Some(index);
        }
    }
    let mut edits = BTreeMap::new();
    for (tag, format) in formats {
        let index =
            positions.get(&tag.to_ascii_lowercase()).copied().flatten().ok_or_else(|| bad("attedit", format!("attribute `{tag}` not found")))?;
        let a = ins.attribs.get(index).ok_or_else(|| bad("attedit", "attribute no longer exists"))?;
        if a.constant || a.multiline.is_some() {
            return Err(bad("attedit", "text options require a nonconstant single-line attribute"));
        }
        let fields = format.as_object().ok_or_else(|| bad("attedit", "each attribute format must be an object"))?;
        for key in fields.keys() {
            if !matches!(key.as_str(), "textStyle" | "height" | "widthFactor" | "rotation" | "oblique" | "justify") {
                return Err(bad("attedit", format!("unknown attribute text option `{key}`")));
            }
        }
        let mut text = a.text.clone();
        if let Some(v) = fields.get("textStyle") {
            let name = v.as_str().ok_or_else(|| bad("attedit", "textStyle must be a loaded style name"))?;
            text.style = d.text_style(name).ok_or_else(|| bad("attedit", format!("text style `{name}` not found")))?.name.clone();
        }
        for (key, min, max, target, angle) in [
            ("height", f64::MIN_POSITIVE, 1.0e12, &mut text.height, false),
            ("widthFactor", f64::MIN_POSITIVE, 100.0, &mut text.width_factor, false),
            ("rotation", -1.0e9, 1.0e9, &mut text.rotation, true),
            ("oblique", -85.0, 85.0, &mut text.oblique, true),
        ] {
            if let Some(v) = fields.get(key) {
                let n = v
                    .as_f64()
                    .filter(|n| n.is_finite() && *n >= min && *n <= max)
                    .ok_or_else(|| bad("attedit", format!("{key} must be a finite number in {min}..={max}")))?;
                *target = if angle { n.to_radians() } else { n };
            }
        }
        if let Some(v) = fields.get("justify") {
            let (horizontal, vertical) = v.as_str().and_then(justify).ok_or_else(|| bad("attedit", "invalid attribute justification"))?;
            if (horizontal, vertical) != (text.halign, text.valign) {
                // Aligned/Fit require two independently chosen points. Do not invent one.
                if matches!(horizontal, HAlign::Aligned | HAlign::Fit) {
                    return Err(bad("attedit", "creating Aligned/Fit justification requires two placement points"));
                }
                let anchor = match (text.halign, text.valign) {
                    (HAlign::Left, VAlign::Baseline) | (HAlign::Aligned | HAlign::Fit, _) => text.insert,
                    _ => text.align_pt.unwrap_or(text.insert),
                };
                text.halign = horizontal;
                text.valign = vertical;
                if (horizontal, vertical) == (HAlign::Left, VAlign::Baseline) {
                    text.insert = anchor;
                    text.align_pt = None;
                } else {
                    text.align_pt = Some(anchor);
                }
            }
        }
        if text != a.text
            && !matches!((text.halign, text.valign), (HAlign::Left, VAlign::Baseline) | (HAlign::Aligned | HAlign::Fit, _))
            && let Some(anchor) = text.align_pt
        {
            // An imported group-10 baseline can be close enough to pass the writer's
            // font-substitution tolerance even after a small metric edit. Invalidate
            // that derived XY cache so DXF/DWG export recomputes it from group 11.
            text.insert.x = anchor.x;
            text.insert.y = anchor.y;
        }
        edits.insert(index, text);
    }
    Ok(edits)
}
