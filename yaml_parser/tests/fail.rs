use insta::{Settings, assert_snapshot, glob};
use std::{fs, path::Path};

#[test]
fn fail_snapshot() {
    glob!("fail/*.yaml", |path| {
        let input = fs::read_to_string(path).unwrap();
        let err_msg = if let Err(err) = yaml_parser::parse(&input) {
            err.to_string()
        } else {
            panic!("expected '{}' to fail", path.display());
        };

        build_settings(path).bind(|| {
            let name = path.file_stem().unwrap().to_str().unwrap();
            assert_snapshot!(name, err_msg);
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
