use insta::{assert_snapshot, glob, Settings};
use std::{fs, path::Path};

#[test]
fn pass_snapshot() {
    glob!("pass/*.yaml", |path| {
        let input = fs::read_to_string(path).unwrap();
        let tree = match yaml_parser::parse(&input) {
            Ok(tree) => tree,
            Err(err) => panic!("failed to parse '{}': {err}", path.display()),
        };
        assert_eq!(
            tree.to_string(),
            input,
            "syntax tree of '{}' does not match source",
            path.display()
        );

        build_settings(path).bind(|| {
            let name = path.file_stem().unwrap().to_str().unwrap();
            assert_snapshot!(name, format!("{tree:#?}"));
        });
    });
}

fn build_settings(path: &Path) -> Settings {
    let mut settings = Settings::clone_current();
    settings.set_snapshot_path(path.parent().unwrap());
    settings.remove_snapshot_suffix();
    settings.set_prepend_module_to_snapshot(false);
    settings.remove_input_file();
    settings.set_omit_expression(true);
    settings.remove_input_file();
    settings.remove_info();
    settings
}
