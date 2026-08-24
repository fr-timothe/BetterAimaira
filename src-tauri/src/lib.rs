mod aimaira;
#[cfg(target_os = "android")]
mod android_bridge;
mod commands;
mod credentials;
mod error;
mod grade_sync;
mod state;
mod updater;

use state::SessionState;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    #[allow(unused_mut)]
    let mut builder = tauri::Builder::default().plugin(tauri_plugin_opener::init());

    #[cfg(desktop)]
    {
        builder = builder.plugin(tauri_plugin_updater::Builder::new().build());
    }

    // Development-only automation bridge: exposes the webview to the local MCP driver so the
    // running app can be compared against the live Aimaira portal. Gated on both the opt-in
    // feature and a debug build, so no release bundle can ever link or start it.
    #[cfg(all(feature = "dev-automation", debug_assertions))]
    {
        builder = builder.plugin(
            tauri_plugin_mcp_bridge::Builder::new()
                .bind_address("127.0.0.1")
                .build(),
        );
    }

    builder
        .setup(|app| {
            let data_directory = app.path().app_data_dir()?;
            app.manage(grade_sync::GradeSyncStore::new(
                data_directory.join("grades.sqlite"),
            ));
            // The Android package installer answers on a broadcast that outlives
            // the command which started the install, so its verdict is forwarded
            // through a handle published here.
            #[cfg(target_os = "android")]
            updater::remember_app_handle(app.handle());
            Ok(())
        })
        .manage(SessionState::default())
        .invoke_handler(tauri::generate_handler![
            commands::login,
            commands::restore_session,
            commands::saved_identity,
            commands::logout,
            commands::get_schedule,
            commands::get_planning_settings,
            commands::get_portal_resource,
            commands::sync_grades,
            commands::mark_grade_alerts_read,
            commands::download_portal_document,
            commands::get_questionnaire_detail,
            commands::normalize_portal_url,
            updater::check_for_update,
            updater::install_update,
            updater::default_update_channel,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(target_os = "android")]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_betteraimaira_app_MainActivity_initNdkContext(
    env: jni::JNIEnv,
    _class: jni::objects::JObject,
    context: jni::objects::JObject,
) {
    use jni::objects::GlobalRef;
    use std::ffi::c_void;
    use std::sync::OnceLock;

    static CONTEXT: OnceLock<Option<GlobalRef>> = OnceLock::new();
    CONTEXT.get_or_init(|| match env.new_global_ref(&context) {
        Ok(reference) => {
            let vm = env
                .get_java_vm()
                .expect("Android Java VM should be available")
                .get_java_vm_pointer() as *mut c_void;
            unsafe {
                ndk_context::initialize_android_context(vm, reference.as_obj().as_raw() as _);
            }
            Some(reference)
        }
        Err(_) => None,
    });
}
