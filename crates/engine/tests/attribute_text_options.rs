use cadcraft_engine::{
    Session,
    doc::{Attrib, EntityKind, HAlign, Handle, VAlign},
};
use serde_json::{Value, json};

fn setup() -> (Session, Handle, Handle) {
    let mut s = Session::new();
    for (tag, y) in [("T", 1), ("U", 5)] {
        s.execute("attdef", &json!({"tag":tag,"default":tag,"at":[2,y],"height":3})).unwrap();
    }
    let handles: Vec<_> = s.doc().unwrap().model.iter().map(|e| e.handle.hex()).collect();
    s.execute("block", &json!({"name":"Equipment","handles":handles,"base":[0,0],"keep":"delete"})).unwrap();
    let mut insert = |at| {
        let r = s.execute("insert", &json!({"name":"Equipment","at":at,"scale":2,"rotation":30})).unwrap();
        Handle::parse_hex(r["handle"].as_str().unwrap()).unwrap()
    };
    let a = insert([100, 200]);
    let b = insert([300, 200]);
    (s, a, b)
}

fn attributes(s: &Session, h: Handle) -> Vec<Attrib> {
    let EntityKind::Insert(i) = &s.doc().unwrap().entity(h).unwrap().kind else { panic!("insert") };
    i.attribs.clone()
}

#[test]
fn formatting_and_values_are_one_undoable_reference_edit_and_survive_dxf() {
    let (mut s, a, b) = setup();
    s.execute("style", &json!({"name":"Notes","current":false})).unwrap();
    let before = attributes(&s, a);
    let other = attributes(&s, b);
    let definition = s.doc().unwrap().block("Equipment").unwrap().clone();
    let anchor = before[0].text.justify_point();
    let changed = s
        .execute(
            "attedit",
            &json!({"handle":a.hex(),"values":{"t":"Edited"},"formats":{"t":{
                "textStyle":"notes","height":5,"widthFactor":0.7,"rotation":45,"oblique":10,"justify":"TC"
            }}}),
        )
        .unwrap();
    assert_eq!(changed["changed"], 1);
    let edited = attributes(&s, a);
    assert_eq!(edited[0].text.value, "Edited");
    assert_eq!(edited[0].text.style, "Notes");
    assert_eq!((edited[0].text.height, edited[0].text.width_factor), (5.0, 0.7));
    assert_eq!((edited[0].text.rotation, edited[0].text.oblique), (45_f64.to_radians(), 10_f64.to_radians()));
    assert_eq!((edited[0].text.halign, edited[0].text.valign), (HAlign::Center, VAlign::Top));
    assert_eq!(edited[0].text.justify_point(), anchor);
    assert_eq!(edited[1], before[1]);
    assert_eq!(attributes(&s, b), other);
    assert_eq!(s.doc().unwrap().block("Equipment").unwrap(), &definition);
    assert_eq!(edited[0].props, before[0].props);
    assert_eq!(edited[0].modes, before[0].modes);
    let bytes = cadcraft_io::write_dxf(s.doc().unwrap());
    let reopened = cadcraft_io::read_dxf(bytes.as_bytes()).unwrap();
    let EntityKind::Insert(restored) = &reopened.entity(a).unwrap().kind else { panic!("insert") };
    // The existing writer computes group 10's baseline for justified text; group 11
    // is the actual placement anchor. Permit only that derived XY normalization.
    assert_eq!(restored.attribs[0].text.insert.z, edited[0].text.insert.z);
    let mut expected_kind = s.doc().unwrap().entity(a).unwrap().kind.clone();
    if let EntityKind::Insert(i) = &mut expected_kind {
        i.attribs[0].text.insert = restored.attribs[0].text.insert;
    }
    assert_eq!(reopened.entity(a).unwrap().kind, expected_kind);
    let before_render = cadcraft_engine::render::build(s.doc().unwrap(), &cadcraft_engine::doc::Space::Model, &Default::default());
    let after_render = cadcraft_engine::render::build(&reopened, &cadcraft_engine::doc::Space::Model, &Default::default());
    assert_eq!(before_render.prims, after_render.prims);
    assert_eq!(before_render.verts, after_render.verts);
    assert_eq!(before_render.tris, after_render.tris);
    assert_eq!(before_render.bounds, after_render.bounds);
    s.execute("undo", &json!({})).unwrap();
    assert_eq!(attributes(&s, a), before);
    s.execute("redo", &json!({})).unwrap();
    assert_eq!(attributes(&s, a), edited);
}

#[test]
fn invalid_formats_cannot_partially_change_values_or_other_attributes() {
    let (mut s, a, _) = setup();
    let before = cadcraft_io::write_dxf(s.doc().unwrap());
    for formats in [
        Value::Null,
        json!([]),
        json!({"Missing":{"height":4}}),
        json!({"T":{"height":4},"t":{"height":5}}),
        json!({"T":{"textStyle":"Missing"}}),
        json!({"T":{"height":0}}),
        json!({"T":{"height":"4"}}),
        json!({"T":{"widthFactor":101}}),
        json!({"T":{"oblique":86}}),
        json!({"T":{"rotation":1e10}}),
        json!({"T":{"justify":"A"}}),
        json!({"T":{"justify":"F"}}),
        json!({"T":{"justify":"bad"}}),
        json!({"T":{"backwards":true}}),
        json!({"T":{"height":4},"U":{"height":null}}),
    ] {
        assert!(s.execute("attedit", &json!({"handle":a.hex(),"values":{"T":"Must not change"},"formats":formats})).is_err(), "{formats}");
        assert_eq!(cadcraft_io::write_dxf(s.doc().unwrap()), before, "{formats}");
    }
}

#[test]
fn justification_switches_preserve_the_existing_reference_anchor() {
    let (mut s, a, _) = setup();
    s.doc_mut()
        .unwrap()
        .modify_entity(a, |e| {
            if let EntityKind::Insert(i) = &mut e.kind {
                i.attribs[0].text.insert.z = 7.0;
            }
        })
        .unwrap();
    let original = attributes(&s, a);
    let anchor = original[0].text.justify_point();
    for code in ["C", "R", "M", "TL", "TC", "TR", "ML", "MC", "MR", "BL", "BC", "BR", "L"] {
        s.execute("attedit", &json!({"handle":a.hex(),"formats":{"T":{"justify":code}}})).unwrap();
        let current = attributes(&s, a);
        assert_eq!(current[0].text.justify_point(), anchor, "{code}");
        assert_eq!(current[1], original[1]);
        assert_eq!(current[0].text.value, original[0].text.value);
        assert_eq!(current[0].text.height, original[0].text.height);
        let anchor_z = if code == "L" { current[0].text.insert.z } else { current[0].text.align_pt.unwrap().z };
        assert_eq!(anchor_z, 7.0, "{code}");
    }
}

#[test]
fn oversized_and_ambiguous_format_requests_fail_before_values_change() {
    let (mut s, a, _) = setup();
    let formats: serde_json::Map<_, _> = (0..4097).map(|n| (format!("T{n}"), json!({"height":4}))).collect();
    let before = cadcraft_io::write_dxf(s.doc().unwrap());
    let error = s.execute("attedit", &json!({"handle":a.hex(),"values":{"T":"Must not change"},"formats":formats})).unwrap_err();
    assert!(error.to_string().contains("too many attribute formats"));
    assert_eq!(cadcraft_io::write_dxf(s.doc().unwrap()), before);
    s.doc_mut()
        .unwrap()
        .modify_entity(a, |e| {
            if let EntityKind::Insert(i) = &mut e.kind {
                let mut duplicate = i.attribs[0].clone();
                duplicate.tag = "t".into();
                i.attribs.push(duplicate);
            }
        })
        .unwrap();
    let before = cadcraft_io::write_dxf(s.doc().unwrap());
    let error = s.execute("attedit", &json!({"handle":a.hex(),"values":{"U":"Must not change"},"formats":{"T":{"height":4}}})).unwrap_err();
    assert!(error.to_string().contains("ambiguous"));
    assert_eq!(cadcraft_io::write_dxf(s.doc().unwrap()), before);
}

#[test]
fn loaded_aligned_and_fit_points_survive_other_format_edits() {
    for (mode, code) in [(HAlign::Aligned, "A"), (HAlign::Fit, "F")] {
        let (mut s, a, _) = setup();
        s.doc_mut()
            .unwrap()
            .modify_entity(a, |e| {
                if let EntityKind::Insert(i) = &mut e.kind {
                    let text = &mut i.attribs[0].text;
                    text.halign = mode;
                    text.align_pt = Some(cadcraft_engine::geom::Vec3::new(text.insert.x + 10.0, text.insert.y + 5.0, 3.0));
                }
            })
            .unwrap();
        let before = attributes(&s, a);
        s.execute("attedit", &json!({"handle":a.hex(),"formats":{"T":{"height":4,"justify":code}}})).unwrap();
        let edited = attributes(&s, a);
        assert_eq!(edited[0].text.insert, before[0].text.insert);
        assert_eq!(edited[0].text.align_pt, before[0].text.align_pt);
        assert_eq!(edited[0].text.halign, mode);
        assert_eq!(edited[0].text.height, 4.0);
    }
}

#[test]
fn edited_justified_attribute_recomputes_export_baseline() {
    let (mut initial, a, _) = setup();
    initial.execute("attedit", &json!({"handle":a.hex(),"formats":{"T":{"height":5,"justify":"TC"}}})).unwrap();
    let imported = cadcraft_io::read_dxf(cadcraft_io::write_dxf(initial.doc().unwrap()).as_bytes()).unwrap();
    let mut s = Session::empty();
    s.open_drawing(imported, "Imported", None);
    let before = attributes(&s, a);
    let anchor = before[0].text.justify_point();
    let baseline = before[0].text.insert.xy();
    assert!(baseline.dist(anchor) > 0.1, "fixture must contain the previously computed baseline");
    s.execute("attedit", &json!({"handle":a.hex(),"formats":{"T":{"height":5.5}}})).unwrap();
    let reopened = cadcraft_io::read_dxf(cadcraft_io::write_dxf(s.doc().unwrap()).as_bytes()).unwrap();
    let EntityKind::Insert(i) = &reopened.entity(a).unwrap().kind else { panic!("insert") };
    let updated = &i.attribs[0].text;
    assert_eq!(updated.justify_point(), anchor);
    // With the same font, angle, value and width factor, this baseline offset must
    // scale with height. Retaining the old baseline would misplace a DWG consumer.
    let expected = anchor + (baseline - anchor) * (5.5 / 5.0);
    assert!(updated.insert.xy().dist(expected) < 1e-8, "cached baseline {:?}; expected {:?}", updated.insert.xy(), expected);
}

#[test]
fn constant_multiline_and_locked_attributes_refuse_formatting_without_value_edits() {
    let (mut s, a, _) = setup();
    s.doc_mut()
        .unwrap()
        .modify_entity(a, |e| {
            if let EntityKind::Insert(i) = &mut e.kind {
                i.attribs[0].constant = true;
            }
        })
        .unwrap();
    let before = attributes(&s, a);
    assert!(s.execute("attedit", &json!({"handle":a.hex(),"values":{"U":"Changed"},"formats":{"T":{"height":4}}})).is_err());
    assert_eq!(attributes(&s, a), before);
    let m = s.execute("mtext", &json!({"at":[0,0],"text":"{\\C1;Keep}\\PFormatting","height":3,"width":30})).unwrap();
    let mh = Handle::parse_hex(m["handle"].as_str().unwrap()).unwrap();
    let EntityKind::MText(mtext) = s.doc().unwrap().entity(mh).unwrap().kind.clone() else { panic!("mtext") };
    s.doc_mut()
        .unwrap()
        .modify_entity(a, |e| {
            if let EntityKind::Insert(i) = &mut e.kind {
                i.attribs[0].constant = false;
                i.attribs[0].multiline = Some(cadcraft_engine::doc::MultilineAttrib { mtext, read_as: i.attribs[0].text.value.clone() });
            }
        })
        .unwrap();
    let before = attributes(&s, a);
    assert!(s.execute("attedit", &json!({"handle":a.hex(),"values":{"U":"Changed"},"formats":{"T":{"height":4}}})).is_err());
    assert_eq!(attributes(&s, a), before);
    s.doc_mut().unwrap().layer_mut("0").unwrap().locked = true;
    assert!(s.execute("attedit", &json!({"handle":a.hex(),"values":{"U":"Changed"},"formats":{"U":{"height":4}}})).is_err());
    assert_eq!(attributes(&s, a), before);
}
