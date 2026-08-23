package com.betteraimaira.app

import android.content.Context
import android.os.Bundle
import androidx.activity.enableEdgeToEdge

class MainActivity : TauriActivity() {
  companion object {
    init {
      System.loadLibrary("betteraimaira_lib")
    }
  }

  override fun onCreate(savedInstanceState: Bundle?) {
    enableEdgeToEdge()
    super.onCreate(savedInstanceState)
    // The Rust side reads the Android context through `ndk_context` for the
    // keyring store and the package installer. Nothing publishes it unless the
    // activity does, and reading it uninitialised aborts the process.
    initNdkContext(applicationContext)
  }

  private external fun initNdkContext(context: Context)
}
