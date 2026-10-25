use cadcraft_engine::{
    Session,
    doc::{EntityKind, Handle},
};
use serde_json::json;
fn handle(r: serde_json::Value) -> Handle {
    Handle::parse_hex(r["handle"].as_str().unwrap()).unwrap()
}
#[test]
fn text_style_assignment_is_canonical_undoable_and_survives_dxf_without_changing_geometry() {
    let mut s = Session::new();
    s.execute("style", &json!({"name":"Notes","current":false,"annotative":true})).unwrap();
    let a = handle(s.execute("text", &json!({"at":[7,9],"text":"Single","height":3,"rotation":25})).unwrap());
    let b = handle(s.execute("mtext", &json!({"at":[12,15],"text":"{\\C1;Inline}\\PNext","height":4,"width":30})).unwrap());
    let before_a = s.doc().unwrap().entity(a).unwrap().kind.clone();
    let before_b = s.doc().unwrap().entity(b).unwrap().kind.clone();
    s.execute("properties.set", &json!({"handles":[a.hex(),b.hex()],"textStyle":"notes"})).unwrap();
    for h in [a, b] {
        assert_eq!(s.doc().unwrap().entity(h).unwrap().common.anno_scales.len(), 1);
    }
    let mut expected_a = before_a.clone();
    if let EntityKind::Text(t) = &mut expected_a {
        t.style = "Notes".into();
    }
    let mut expected_b = before_b.clone();
    if let EntityKind::MText(t) = &mut expected_b {
        t.style = "Notes".into();
    }
    assert_eq!(s.doc().unwrap().entity(a).unwrap().kind, expected_a);
    assert_eq!(s.doc().unwrap().entity(b).unwrap().kind, expected_b);
    assert_eq!(s.doc().unwrap().header.str("TEXTSTYLE", ""), "Standard");
    let saved = cadcraft_io::write_dxf(s.doc().unwrap());
    let reopened = cadcraft_io::read_dxf(saved.as_bytes()).unwrap();
    assert_eq!(reopened.entity(a).unwrap().kind, expected_a);
    assert_eq!(reopened.entity(b).unwrap().kind, expected_b);
    s.execute("undo", &json!({})).unwrap();
    assert_eq!(s.doc().unwrap().entity(a).unwrap().kind, before_a);
    assert_eq!(s.doc().unwrap().entity(b).unwrap().kind, before_b);
    assert!(s.doc().unwrap().entity(a).unwrap().common.anno_scales.is_empty());
    s.execute("redo", &json!({})).unwrap();
    s.execute("properties.set", &json!({"handles":[a.hex(),b.hex()],"textStyle":"Standard"})).unwrap();
    for h in [a, b] {
        assert!(s.doc().unwrap().entity(h).unwrap().common.anno_scales.is_empty());
    }
}
#[test]
fn invalid_styles_fail_atomically_and_unrelated_geometry_is_untouched() {
    let mut s = Session::new();
    let a = handle(s.execute("text", &json!({"at":[0,0],"text":"A","height":2})).unwrap());
    let r = s.execute("line", &json!({"points":[[0,0],[10,0]]})).unwrap();
    let b = Handle::parse_hex(r["handles"][0].as_str().unwrap()).unwrap();
    let before = cadcraft_io::write_dxf(s.doc().unwrap());
    for name in [json!("Missing"), json!(42), json!(null)] {
        assert!(s.execute("properties.set", &json!({"handles":[a.hex(),b.hex()],"textStyle":name,"layer":"MustNotExist"})).is_err());
        assert_eq!(cadcraft_io::write_dxf(s.doc().unwrap()), before);
    }
    s.execute("style", &json!({"name":"Notes","current":false,"annotative":true})).unwrap();
    let line = (**s.doc().unwrap().entity(b).unwrap()).clone();
    s.execute("properties.set", &json!({"handles":[a.hex(),b.hex()],"textStyle":"Notes"})).unwrap();
    assert_eq!(**s.doc().unwrap().entity(b).unwrap(), line);
    s.execute("properties.set", &json!({"handles":[a.hex()],"textStyle":"Notes","annotative":false})).unwrap();
    assert!(s.doc().unwrap().entity(a).unwrap().common.anno_scales.is_empty());
}
