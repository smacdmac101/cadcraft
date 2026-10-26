//! Validate per-reference attribute property overrides before any edits are applied.
use cadcraft_doc::{AttribProps, Drawing, EntityKind, Handle, Lineweight, color::Color};
use serde_json::Value;
use std::collections::BTreeMap;

use crate::{Result, cmd::bad};

pub(super) fn prepare(d: &Drawing, h: Handle, input: Option<&Value>) -> Result<BTreeMap<usize, AttribProps>> {
    let Some(input) = input else { return Ok(BTreeMap::new()) };
    let properties = input.as_object().ok_or_else(|| bad("attedit", "properties must be an object keyed by attribute tag"))?;
    if properties.len() > 4096 {
        return Err(bad("attedit", "too many attribute properties"));
    }
    if properties.is_empty() {
        return Ok(BTreeMap::new());
    }
    let Some(EntityKind::Insert(ins)) = d.entity(h).map(|e| &e.kind) else {
        return Err(bad("attedit", "properties require a block reference"));
    };
    let mut positions = BTreeMap::new();
    for tag in properties.keys() {
        if tag.len() > 4096 || positions.insert(tag.to_ascii_lowercase(), None).is_some() {
            return Err(bad("attedit", "duplicate or oversized attribute property tag"));
        }
    }
    for (index, attribute) in ins.attribs.iter().enumerate() {
        if let Some(position) = positions.get_mut(&attribute.tag.to_ascii_lowercase()) {
            if position.is_some() {
                return Err(bad("attedit", format!("attribute tag `{}` is ambiguous", attribute.tag)));
            }
            *position = Some(index);
        }
    }
    let mut edits = BTreeMap::new();
    for (tag, input) in properties {
        let index =
            positions.get(&tag.to_ascii_lowercase()).copied().flatten().ok_or_else(|| bad("attedit", format!("attribute `{tag}` not found")))?;
        let attribute = ins.attribs.get(index).ok_or_else(|| bad("attedit", "attribute no longer exists"))?;
        if attribute.constant {
            return Err(bad("attedit", "properties require a nonconstant attribute"));
        }
        let fields = input.as_object().ok_or_else(|| bad("attedit", "each attribute property entry must be an object"))?;
        let mut next = attribute.props.clone();
        for (key, value) in fields {
            match key.as_str() {
                "layer" => {
                    let name = table_name(value)?;
                    next.layer = Some(d.layer(name).ok_or_else(|| bad("attedit", format!("layer `{name}` not found")))?.name.clone());
                }
                "linetype" => {
                    let name = table_name(value)?;
                    next.linetype = Some(if name.eq_ignore_ascii_case("ByLayer") {
                        "ByLayer".into()
                    } else if name.eq_ignore_ascii_case("ByBlock") {
                        "ByBlock".into()
                    } else {
                        d.linetype(name).ok_or_else(|| bad("attedit", format!("linetype `{name}` is not loaded")))?.name.clone()
                    });
                }
                "color" => next.color = Some(color(value).ok_or_else(|| bad("attedit", "invalid attribute color"))?),
                "lineweight" => {
                    next.lineweight = Some(
                        lineweight(value)
                            .ok_or_else(|| bad("attedit", "lineweight must be ByLayer, ByBlock, Default or a standard millimetre value"))?,
                    )
                }
                _ => return Err(bad("attedit", format!("unknown attribute property `{key:.64}`"))),
            }
        }
        if next != attribute.props {
            edits.insert(index, next);
        }
    }
    Ok(edits)
}

fn table_name(value: &Value) -> Result<&str> {
    value
        .as_str()
        .filter(|s| !s.is_empty() && s.len() <= 4096)
        .ok_or_else(|| bad("attedit", "property table names must be nonempty strings of at most 4096 bytes"))
}

fn color(value: &Value) -> Option<Color> {
    if let Some(name) = value.as_str().filter(|s| s.len() <= 128) {
        return Color::parse(name);
    }
    let index = value.as_u64().filter(|n| *n <= 256)?;
    Color::parse(&index.to_string())
}

fn lineweight(value: &Value) -> Option<Lineweight> {
    let number = if let Some(s) = value.as_str().filter(|s| s.len() <= 64) {
        match s.trim().to_ascii_lowercase().as_str() {
            "bylayer" => return Some(Lineweight::ByLayer),
            "byblock" => return Some(Lineweight::ByBlock),
            "default" => return Some(Lineweight::Default),
            s => s.trim_end_matches("mm").trim().parse::<f64>().ok()?,
        }
    } else {
        value.as_f64()?
    };
    if !number.is_finite() || !(0.0..=2.11).contains(&number) {
        return None;
    }
    let hundredths = number * 100.0;
    Lineweight::STANDARD.into_iter().find(|v| (f64::from(*v) - hundredths).abs() < 1e-7).map(Lineweight::Mm100)
}
