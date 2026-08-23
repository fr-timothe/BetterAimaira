fn main() {
    // The base capability always ships. The development automation capability is only picked up
    // when the `dev-automation` feature is on, so release bundles never carry it.
    let development_automation = std::env::var_os("CARGO_FEATURE_DEV_AUTOMATION").is_some()
        && std::env::var("PROFILE").is_ok_and(|profile| profile == "debug");
    let pattern = if development_automation {
        "./capabilities/*/*.json"
    } else {
        "./capabilities/app/*.json"
    };
    let attributes = tauri_build::Attributes::new().capabilities_path_pattern(pattern);
    tauri_build::try_build(attributes).expect("failed to run tauri-build");
}
