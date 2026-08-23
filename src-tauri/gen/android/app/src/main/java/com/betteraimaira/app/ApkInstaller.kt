package com.betteraimaira.app

import android.app.PendingIntent
import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import android.content.IntentFilter
import android.content.pm.PackageInstaller
import android.net.Uri
import android.os.Build
import android.provider.Settings
import androidx.core.content.ContextCompat
import androidx.core.content.IntentCompat
import java.io.File

/**
 * Installs an update APK that the Rust side already downloaded and verified as
 * complete.
 *
 * A Tauri app cannot replace its own APK: only the system package installer
 * can. This drives a `PackageInstaller` session and forwards the confirmation
 * dialog the system asks for, so the user stays inside one flow instead of
 * hunting through the download tray.
 *
 * Called from Rust through JNI, so the name, the signature and the class must
 * stay in sync with `src-tauri/src/updater.rs`, and ProGuard must keep them.
 */
object ApkInstaller {
    private const val STATUS_ACTION = "com.betteraimaira.app.INSTALL_STATUS"
    private const val SESSION_NAME = "betteraimaira-update"

    /** The session install is asynchronous; only one receiver is ever needed. */
    private var receiver: InstallStatusReceiver? = null

    /**
     * Returns what happened, because the caller has to say something honest to
     * the user: `installing`, `permission_required` (the user was sent to the
     * "install unknown apps" screen) or `failed`.
     */
    @JvmStatic
    fun install(context: Context, apkPath: String): String {
        val applicationContext = context.applicationContext
        val apk = File(apkPath)
        if (!apk.isFile || apk.length() == 0L) return "failed"

        if (!canInstallPackages(applicationContext)) {
            requestInstallPermission(applicationContext)
            return "permission_required"
        }

        return try {
            registerStatusReceiver(applicationContext)
            val installer = applicationContext.packageManager.packageInstaller
            val params =
                PackageInstaller.SessionParams(PackageInstaller.SessionParams.MODE_FULL_INSTALL)
            params.setAppPackageName(applicationContext.packageName)

            val sessionId = installer.createSession(params)
            installer.openSession(sessionId).use { session ->
                session.openWrite(SESSION_NAME, 0, apk.length()).use { output ->
                    apk.inputStream().use { input -> input.copyTo(output) }
                    session.fsync(output)
                }
                session.commit(statusSender(applicationContext, sessionId))
            }
            "installing"
        } catch (error: Exception) {
            "failed"
        }
    }

    private fun canInstallPackages(context: Context): Boolean {
        if (Build.VERSION.SDK_INT < Build.VERSION_CODES.O) return true
        return context.packageManager.canRequestPackageInstalls()
    }

    /**
     * Android refuses to show the installer until this app holds the "install
     * unknown apps" right, and the grant lives in Settings, not in a dialog.
     */
    private fun requestInstallPermission(context: Context) {
        if (Build.VERSION.SDK_INT < Build.VERSION_CODES.O) return
        val intent =
            Intent(
                    Settings.ACTION_MANAGE_UNKNOWN_APP_SOURCES,
                    Uri.parse("package:${context.packageName}"),
                )
                .addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
        try {
            context.startActivity(intent)
        } catch (error: Exception) {
            // Some vendor builds hide the screen; the user can still install the
            // APK by hand from the cache directory.
        }
    }

    /**
     * `FLAG_MUTABLE` matters: the system fills the confirmation intent into this
     * pending intent, and an immutable one would arrive empty on API 31+.
     */
    private fun statusSender(context: Context, sessionId: Int) =
        PendingIntent.getBroadcast(
                context,
                sessionId,
                Intent(STATUS_ACTION).setPackage(context.packageName),
                if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.S) {
                    PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_MUTABLE
                } else {
                    PendingIntent.FLAG_UPDATE_CURRENT
                },
            )
            .intentSender

    private fun registerStatusReceiver(context: Context) {
        if (receiver != null) return
        val created = InstallStatusReceiver()
        ContextCompat.registerReceiver(
            context,
            created,
            IntentFilter(STATUS_ACTION),
            ContextCompat.RECEIVER_NOT_EXPORTED,
        )
        receiver = created
    }

    private fun unregisterStatusReceiver(context: Context) {
        val current = receiver ?: return
        receiver = null
        try {
            context.applicationContext.unregisterReceiver(current)
        } catch (error: IllegalArgumentException) {
            // Already gone with the process; nothing left to release.
        }
    }

    /**
     * Forwards the installer's verdict to Rust, which turns it into an event the
     * page listens to. The session install answers on a broadcast long after
     * `install` returned, so without this a refused or failed install left the
     * interface claiming the system installer had taken over.
     *
     * Implemented in `src-tauri/src/updater.rs`; the name and the signature
     * must stay in sync with it.
     */
    private external fun reportInstallStatus(succeeded: Boolean, message: String)

    private class InstallStatusReceiver : BroadcastReceiver() {
        override fun onReceive(context: Context, intent: Intent) {
            val status =
                intent.getIntExtra(PackageInstaller.EXTRA_STATUS, PackageInstaller.STATUS_FAILURE)
            if (status == PackageInstaller.STATUS_PENDING_USER_ACTION) {
                // The system wants the user to confirm the install: it hands back
                // the activity to launch, and nothing happens until we do.
                val confirmation =
                    IntentCompat.getParcelableExtra(intent, Intent.EXTRA_INTENT, Intent::class.java)
                confirmation?.addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
                confirmation?.let { context.startActivity(it) }
                return
            }

            unregisterStatusReceiver(context)

            // A successful install replaces this process, so the event only ever
            // reaches the page on failure — which is exactly the case that used
            // to be invisible.
            val message = intent.getStringExtra(PackageInstaller.EXTRA_STATUS_MESSAGE) ?: ""
            try {
                reportInstallStatus(status == PackageInstaller.STATUS_SUCCESS, message)
            } catch (error: UnsatisfiedLinkError) {
                // The broadcast can outlive the native library on a cold restart;
                // the page has nothing to show at that point anyway.
            }
        }
    }
}
