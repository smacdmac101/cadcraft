use cadcraft_engine::{
    Session,
    doc::{Attrib, Common, Drawing, EntityKind, Handle, Lineweight, color::Color},
};
use serde_json::{Value, json};

fn setup() -> (Session, Handle, Handle) {
    let mut s = Session::new();
    s.execute("layer.new", &json!({"name":"AttributeTest","color":"cyan","current":false})).unwrap();
    s.execute("linetype", &json!({"load":"*"})).unwrap();
    for (tag, y) in [("T", 0), ("U", 5)] {
        s.execute("attdef", &json!({"tag":tag,"default":tag,"at":[0,y],"height":3})).unwrap();
    }
    let handles: Vec<_> = s.doc().unwrap().model.iter().map(|e| e.handle.hex()).collect();
    s.execute("block", &json!({"name":"Equipment","handles":handles,"base":[0,0],"keep":"delete"})).unwrap();
    let mut insert = |x| {
        let r = s.execute("insert", &json!({"name":"Equipment","at":[x,20],"scale":2,"rotation":30})).unwrap();
        Handle::parse_hex(r["handle"].as_str().unwrap()).unwrap()
    };
    let a = insert(10);
    let b = insert(100);
    (s, a, b)
}

fn attributes(d: &Drawing, h: Handle) -> Vec<Attrib> {
    let EntityKind::Insert(i) = &d.entity(h).unwrap().kind else { panic!("insert") };
    i.attribs.clone()
}

fn effective(d: &Drawing, a: &Attrib) -> Common {
    let definition = d.block("Equipment").unwrap().entities.iter().find(|e| matches!(&e.kind,EntityKind::AttDef(ad) if ad.tag==a.tag)).unwrap();
    a.props.over(&definition.common)
}

#[test]
fn property_value_and_format_edits_are_atomic_undoable_and_roundtrip_both_formats() {
    let (mut s, a, b) = setup();
    let before = attributes(s.doc().unwrap(), a);
    let other = attributes(s.doc().unwrap(), b);
    let definition = s.doc().unwrap().block("Equipment").unwrap().clone();
    let r=s.execute("attedit", &json!({"handle":a.hex(),"values":{"t":"Edited"},"formats":{"t":{"height":4}},"properties":{"t":{"layer":"attributetest","color":"#123456","linetype":"dashed","lineweight":0.35}}})).unwrap();
    assert_eq!(r["changed"], 1);
    let edited = attributes(s.doc().unwrap(), a);
    assert_eq!(edited[0].text.value, "Edited");
    assert_eq!(edited[0].text.height, 4.0);
    assert_eq!(edited[0].props.layer.as_deref(), Some("AttributeTest"));
    assert_eq!(edited[0].props.color, Color::parse("#123456"));
    assert_eq!(edited[0].props.linetype.as_deref(), Some("DASHED"));
    assert_eq!(edited[0].props.lineweight, Some(Lineweight::Mm100(35)));
    assert_eq!(edited[0].text.insert, before[0].text.insert);
    assert_eq!(edited[1], before[1]);
    assert_eq!(attributes(s.doc().unwrap(), b), other);
    assert_eq!(s.doc().unwrap().block("Equipment").unwrap(), &definition);
    for name in ["properties.dxf", "properties.dwg"] {
        let reopened = cadcraft_io::read(&cadcraft_io::write(s.doc().unwrap(), name).unwrap(), name).unwrap();
        let restored = attributes(&reopened, a);
        for (expected, actual) in edited.iter().zip(&restored) {
            let e = effective(s.doc().unwrap(), expected);
            let r = effective(&reopened, actual);
            assert_eq!((&r.layer, r.color, &r.linetype, r.lineweight), (&e.layer, e.color, &e.linetype, e.lineweight), "{name}");
            assert_eq!(actual.text, expected.text);
        }
    }
    s.undo().unwrap();
    assert_eq!(attributes(s.doc().unwrap(), a), before);
    s.redo().unwrap();
    assert_eq!(attributes(s.doc().unwrap(), a), edited);
}

#[test]
fn invalid_properties_never_partially_apply_values_formats_or_other_tags() {
    let (mut s, a, _) = setup();
    let before = cadcraft_io::write_dxf(s.doc().unwrap());
    for properties in [
        json!(null),
        json!([]),
        json!({"Missing":{"color":"red"}}),
        json!({"T":null}),
        json!({"T":{"unknown":1}}),
        json!({"T":{"x".repeat(1000):1}}),
        json!({"T":{"layer":"Typo"}}),
        json!({"T":{"linetype":"Typo"}}),
        json!({"T":{"color":257}}),
        json!({"T":{"color":-1}}),
        json!({"T":{"color":1.5}}),
        json!({"T":{"color":"not a color"}}),
        json!({"T":{"color":"1,".repeat(1000)}}),
        json!({"T":{"lineweight":-0.1}}),
        json!({"T":{"lineweight":3}}),
        json!({"T":{"lineweight":0.351}}),
        json!({"T":{"lineweight":"NaN"}}),
        json!({"T":{"lineweight":"inf"}}),
        json!({"T":{"lineweight":null}}),
        json!({"T":{"layer":null}}),
        json!({"T":{},"t":{}}),
        json!({"T":{"color":"red"},"U":{"lineweight":"bad"}}),
    ] {
        let result =
            s.execute("attedit", &json!({"handle":a.hex(),"values":{"T":"Must not change"},"formats":{"U":{"height":9}},"properties":properties}));
        assert!(result.is_err(), "{properties}");
        assert_eq!(cadcraft_io::write_dxf(s.doc().unwrap()), before, "{properties}");
    }
}

#[test]
fn property_noops_and_partial_changes_preserve_existing_overrides_and_text() {
    let (mut s, a, _) = setup();
    let before = attributes(s.doc().unwrap(), a);
    s.execute("attedit", &json!({"handle":a.hex(),"properties":{"T":{}}})).unwrap();
    assert_eq!(attributes(s.doc().unwrap(), a), before);
    s.execute("attedit", &json!({"handle":a.hex(),"properties":{"T":{"color":"ByBlock","linetype":"byblock","lineweight":"ByBlock"}}})).unwrap();
    let first = attributes(s.doc().unwrap(), a);
    assert!(first[0].props.layer.is_none());
    let r = s.execute("attedit", &json!({"handle":a.hex(),"properties":{"T":{"color":0,"linetype":"ByBlock","lineweight":"byblock"}}})).unwrap();
    assert_eq!(r["changed"], 0);
    assert_eq!(attributes(s.doc().unwrap(), a), first);
    s.execute("attedit", &json!({"handle":a.hex(),"properties":{"T":{"layer":"AttributeTest"}}})).unwrap();
    let edited = attributes(s.doc().unwrap(), a);
    assert_eq!(edited[0].props.color, first[0].props.color);
    assert_eq!(edited[0].props.linetype, first[0].props.linetype);
    assert_eq!(edited[0].props.lineweight, first[0].props.lineweight);
    assert_eq!(edited[0].text, before[0].text);
    assert_eq!(edited[1], before[1]);
}

#[test]
fn constant_locked_and_ambiguous_attribute_property_edits_are_refused() {
    for kind in ["constant", "locked", "ambiguous"] {
        let (mut s, a, _) = setup();
        if kind == "locked" {
            s.doc_mut().unwrap().layer_mut("0").unwrap().locked = true;
        } else {
            s.doc_mut()
                .unwrap()
                .modify_entity(a, |e| {
                    let EntityKind::Insert(i) = &mut e.kind else { panic!("insert") };
                    if kind == "constant" {
                        i.attribs[0].constant = true;
                    } else {
                        let mut duplicate = i.attribs[0].clone();
                        duplicate.tag = "t".into();
                        i.attribs.push(duplicate);
                    }
                })
                .unwrap();
        }
        let before = attributes(s.doc().unwrap(), a);
        assert!(
            s.execute("attedit", &json!({"handle":a.hex(),"values":{"U":"Must not change"},"properties":{"T":{"color":"red"}}})).is_err(),
            "{kind}"
        );
        assert_eq!(attributes(s.doc().unwrap(), a), before);
    }
}

#[test]
fn standard_lineweights_and_inherited_colors_are_validated_without_clamping() {
    let (mut s, a, _) = setup();
    for lw in Lineweight::STANDARD {
        s.execute("attedit", &json!({"handle":a.hex(),"properties":{"T":{"lineweight":f64::from(lw)/100.0}}})).unwrap();
        assert_eq!(attributes(s.doc().unwrap(), a)[0].props.lineweight, Some(Lineweight::Mm100(lw)));
    }
    for (value, expected) in [
        (json!("ByLayer"), Lineweight::ByLayer),
        (json!("ByBlock"), Lineweight::ByBlock),
        (json!("Default"), Lineweight::Default),
        (json!("0.35 mm"), Lineweight::Mm100(35)),
    ] {
        s.execute("attedit", &json!({"handle":a.hex(),"properties":{"T":{"lineweight":value}}})).unwrap();
        assert_eq!(attributes(s.doc().unwrap(), a)[0].props.lineweight, Some(expected));
    }
    for value in [json!(256), json!("ByLayer"), json!(0), json!(255), json!("123,45,67")] {
        s.execute("attedit", &json!({"handle":a.hex(),"properties":{"T":{"color":value}}})).unwrap();
        let expected = if let Value::String(s) = &value { Color::parse(s) } else { Color::parse(&value.to_string()) };
        assert_eq!(attributes(s.doc().unwrap(), a)[0].props.color, expected);
    }
}

#[test]
fn multiline_property_overrides_preserve_embedded_text_and_roundtrip() {
    let (mut s, a, _) = setup();
    let r = s.execute("mtext", &json!({"at":[0,0],"text":"First\\PSecond","height":3,"width":30})).unwrap();
    let mh = Handle::parse_hex(r["handle"].as_str().unwrap()).unwrap();
    let EntityKind::MText(mtext) = s.doc().unwrap().entity(mh).unwrap().kind.clone() else { panic!("mtext") };
    s.doc_mut()
        .unwrap()
        .modify_entity(a, |e| {
            let EntityKind::Insert(i) = &mut e.kind else { panic!("insert") };
            i.attribs[0].multiline = Some(cadcraft_engine::doc::MultilineAttrib { mtext, read_as: i.attribs[0].text.value.clone() });
        })
        .unwrap();
    let before = attributes(s.doc().unwrap(), a);
    s.execute("attedit", &json!({"handle":a.hex(),"properties":{"T":{"layer":"AttributeTest","color":"Red","linetype":"DASHED","lineweight":0.35}}}))
        .unwrap();
    let edited = attributes(s.doc().unwrap(), a);
    assert_eq!(edited[0].multiline, before[0].multiline);
    assert_eq!(edited[0].text, before[0].text);
    for name in ["multiline-properties.dxf", "multiline-properties.dwg"] {
        let reopened = cadcraft_io::read(&cadcraft_io::write(s.doc().unwrap(), name).unwrap(), name).unwrap();
        let restored = attributes(&reopened, a);
        assert_eq!(restored[0].multiline.as_ref().unwrap().mtext.contents, edited[0].multiline.as_ref().unwrap().mtext.contents, "{name}");
        let expected = effective(s.doc().unwrap(), &edited[0]);
        let actual = effective(&reopened, &restored[0]);
        assert_eq!(
            (&actual.layer, actual.color, &actual.linetype, actual.lineweight),
            (&expected.layer, expected.color, &expected.linetype, expected.lineweight),
            "{name}"
        );
    }
}
