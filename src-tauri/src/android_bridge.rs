//! The one way into the app's own Kotlin classes.
//!
//! `FindClass` — what a JNI call by class name does — resolves against the class
//! loader of the Java frame below it. A thread the runtime attached from native
//! code has no such frame, so the lookup falls back to the system class loader,
//! which carries the platform classes and nothing else. Every call made from a
//! Tauri command thread therefore failed with `ClassNotFoundException` before
//! reaching Kotlin at all.
//!
//! Asking the Android context for its own class loader is thread-independent, so
//! that is what every caller here goes through.

use jni::objects::{JClass, JObject};
use jni::JNIEnv;

use crate::error::CommandError;

/// The class every native call lands in.
const APK_INSTALLER: &str = "com.betteraimaira.app.ApkInstaller";

/// Attaches the current thread, resolves `ApkInstaller` and hands both to the
/// caller together with the Android context, which is the first argument of
/// every method on that class.
pub fn with_installer<R>(
    error_code: &'static str,
    call: impl FnOnce(&mut JNIEnv<'_>, &JClass<'_>, &JObject<'_>) -> Result<R, CommandError>,
) -> Result<R, CommandError> {
    let context = ndk_context::android_context();
    let vm = unsafe { jni::JavaVM::from_raw(context.vm().cast()) }
        .map_err(|_| CommandError::new(error_code))?;
    let mut env = vm
        .attach_current_thread()
        .map_err(|_| CommandError::new(error_code))?;

    // `ndk_context` owns a global ref to the Android context. A `JObject` does
    // not release the reference it wraps, so borrowing it here is safe and the
    // global ref stays alive for the next call.
    let activity = unsafe { JObject::from_raw(context.context().cast()) };
    let class = load_class(&mut env, &activity, APK_INSTALLER, error_code)?;

    call(&mut env, &class, &activity)
}

fn load_class<'local>(
    env: &mut JNIEnv<'local>,
    activity: &JObject<'_>,
    name: &str,
    error_code: &'static str,
) -> Result<JClass<'local>, CommandError> {
    let loader = env
        .call_method(activity, "getClassLoader", "()Ljava/lang/ClassLoader;", &[])
        .and_then(|value| value.l())
        .map_err(|_| {
            clear_pending_exception(env);
            CommandError::new(error_code)
        })?;

    let name = env
        .new_string(name)
        .map_err(|_| CommandError::new(error_code))?;

    let class = env
        .call_method(
            &loader,
            "loadClass",
            "(Ljava/lang/String;)Ljava/lang/Class;",
            &[(&name).into()],
        )
        .and_then(|value| value.l())
        .map_err(|_| {
            clear_pending_exception(env);
            CommandError::new(error_code)
        })?;

    Ok(JClass::from(class))
}

/// Drops a pending Java exception, after writing it to logcat: swallowing one
/// silently is what kept the class loader failure invisible.
pub fn clear_pending_exception(env: &mut JNIEnv<'_>) {
    if env.exception_check().unwrap_or(false) {
        let _ = env.exception_describe();
        let _ = env.exception_clear();
    }
}
