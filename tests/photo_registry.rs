use std::fs;

#[test]
fn every_card_photo_has_a_license_row() {
    let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/static/img/cards");
    let registry = fs::read_to_string(format!("{dir}/LICENSES.md")).expect("registry exists");
    for entry in fs::read_dir(dir).expect("cards directory exists") {
        let name = entry.expect("readable entry").file_name();
        let name = name.to_string_lossy();
        if name != "LICENSES.md" {
            assert!(
                registry.contains(&format!("\n| {name} |")),
                "{name} has no row in LICENSES.md"
            );
        }
    }
}
