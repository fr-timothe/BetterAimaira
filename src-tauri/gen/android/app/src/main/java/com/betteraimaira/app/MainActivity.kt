package com.betteraimaira.app

import android.content.Context
import android.os.Bundle
import android.webkit.JavascriptInterface
import android.webkit.WebView
import androidx.activity.enableEdgeToEdge
import androidx.core.view.ViewCompat
import androidx.core.view.WindowCompat
import androidx.core.view.WindowInsetsCompat
import java.util.Locale

class MainActivity : TauriActivity() {
  companion object {
    init {
      System.loadLibrary("betteraimaira_lib")
    }
  }

  /** The window insets in CSS pixels, so the page can use them verbatim. */
  @Volatile
  private var insets = WindowInsetsInCssPixels(0.0, 0.0, 0.0, 0.0)

  private data class WindowInsetsInCssPixels(
    val top: Double,
    val right: Double,
    val bottom: Double,
    val left: Double
  ) {
    fun toJson(): String = String.format(
      Locale.US,
      "{\"top\":%.2f,\"right\":%.2f,\"bottom\":%.2f,\"left\":%.2f}",
      top,
      right,
      bottom,
      left
    )
  }

  override fun onCreate(savedInstanceState: Bundle?) {
    enableEdgeToEdge()
    super.onCreate(savedInstanceState)
    // The shell is a light surface in every configuration, so the system bars
    // have to draw their clock and glyphs dark or they vanish into it.
    WindowCompat.getInsetsController(window, window.decorView).apply {
      isAppearanceLightStatusBars = true
      isAppearanceLightNavigationBars = true
    }
    // The Rust side reads the Android context through `ndk_context` for the
    // keyring store and the package installer. Nothing publishes it unless the
    // activity does, and reading it uninitialised aborts the process.
    initNdkContext(applicationContext)
  }

  // An Android webview only reports `env(safe-area-inset-*)` for display
  // cutouts, never for the system bars, so an edge-to-edge window lays the page
  // out underneath the status bar and the gesture pill. The real insets are
  // handed over twice: the page pulls them through the bridge on its first
  // paint, and every later change is pushed to it.
  override fun onWebViewCreate(webView: WebView) {
    webView.addJavascriptInterface(NativeInsetsBridge(), "__nativeInsets")

    ViewCompat.setOnApplyWindowInsetsListener(webView) { view, windowInsets ->
      val systemInsets = windowInsets.getInsets(
        WindowInsetsCompat.Type.systemBars() or WindowInsetsCompat.Type.displayCutout()
      )
      val density = view.resources.displayMetrics.density.toDouble()
      insets = WindowInsetsInCssPixels(
        top = systemInsets.top / density,
        right = systemInsets.right / density,
        bottom = systemInsets.bottom / density,
        left = systemInsets.left / density
      )
      publishInsets(webView)
      // The insets stay unconsumed: the page, not this listener, pays them.
      windowInsets
    }
  }

  private fun publishInsets(webView: WebView) {
    val payload = insets.toJson()
    webView.post {
      webView.evaluateJavascript(
        "window.__applyNativeInsets && window.__applyNativeInsets($payload)",
        null
      )
    }
  }

  private inner class NativeInsetsBridge {
    @JavascriptInterface
    fun get(): String = insets.toJson()
  }

  private external fun initNdkContext(context: Context)
}
