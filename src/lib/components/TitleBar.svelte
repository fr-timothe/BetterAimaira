<script lang="ts">
  import { onMount } from 'svelte';
  import { isTauri } from '@tauri-apps/api/core';
  import Logo from '$lib/assets/Logo.svelte';
  import * as m from '$lib/paraglide/messages.js';

  let isDesktop = $state(false);
  let isMac = $state(false);
  let isWindows = $state(false);
  let isLinux = $state(false);
  let isMaximized = $state(false);
  let isTrafficHovered = $state(false);

  let appWindow: any = null;

  onMount(() => {
    let disposed = false;
    let resizeFrame: number | undefined;
    let unlistenResize: (() => void) | undefined;
    const ua = navigator.userAgent.toLowerCase();
    const plat = (navigator.platform || '').toLowerCase();
    const isMobileUa = /android|iphone|ipad|ipod|mobile/i.test(ua);
    const isMobilePlat = /android|iphone|ipad|ipod/i.test(plat);

    isMac = !isMobileUa && !isMobilePlat && (plat.includes('mac') || ua.includes('macintosh') || (ua.includes('mac os x') && !ua.includes('like mac os x')));
    isWindows = !isMobileUa && !isMobilePlat && (plat.includes('win') || ua.includes('windows'));
    isLinux = !isMobileUa && !isMobilePlat && (plat.includes('linux') || ua.includes('linux')) && !ua.includes('android');

    const inTauri = isTauri();
    const isBrowserDesktopPreview = typeof window !== 'undefined'
      && window.location.search.includes('desktop');
    const isForcedMobile = typeof window !== 'undefined' && (
      window.location.search.includes('mobile') ||
      window.location.search.includes('platform=mobile')
    );
    isDesktop = !isForcedMobile && !isMobileUa && !isMobilePlat && (inTauri || isBrowserDesktopPreview) && (isMac || isWindows || isLinux);

    if (typeof document !== 'undefined') {
      if (isDesktop) {
        document.documentElement.classList.remove('mobile-app');
        document.documentElement.classList.add('desktop-app');
        if (isMac) document.documentElement.classList.add('platform-mac');
        if (isWindows) document.documentElement.classList.add('platform-windows');
        if (isLinux) document.documentElement.classList.add('platform-linux');
      } else {
        document.documentElement.classList.remove('desktop-app', 'platform-mac', 'platform-windows', 'platform-linux');
        document.documentElement.classList.add('mobile-app');
      }
    }

    if (inTauri && isDesktop) {
      import('@tauri-apps/api/window').then(async ({ getCurrentWindow }) => {
        if (disposed) return;
        appWindow = getCurrentWindow();
        try {
          isMaximized = await appWindow.isMaximized();
          updateMaximizedClass(isMaximized);

          unlistenResize = await appWindow.onResized(() => {
            if (resizeFrame !== undefined) return;
            resizeFrame = window.requestAnimationFrame(async () => {
              resizeFrame = undefined;
              if (!appWindow || disposed) return;
              const maximized = await appWindow.isMaximized();
              if (disposed) return;
              isMaximized = maximized;
              updateMaximizedClass(maximized);
            });
          });
          if (disposed) unlistenResize?.();
        } catch {
          // Graceful fallback
        }
      });
    }

    return () => {
      disposed = true;
      if (resizeFrame !== undefined) window.cancelAnimationFrame(resizeFrame);
      unlistenResize?.();
    };
  });

  function updateMaximizedClass(max: boolean) {
    if (typeof document !== 'undefined') {
      if (max) {
        document.documentElement.classList.add('window-maximized');
      } else {
        document.documentElement.classList.remove('window-maximized');
      }
    }
  }

  $effect(() => {
    if (typeof document !== 'undefined') {
      if (isDesktop) {
        document.documentElement.style.setProperty('--titlebar-height', '36px');
      } else {
        document.documentElement.style.setProperty('--titlebar-height', '0px');
      }
    }
  });

  async function handleMinimize(event?: MouseEvent) {
    event?.stopPropagation();
    try {
      if (appWindow) {
        await appWindow.minimize();
      }
    } catch (err) {
      console.error('Failed to minimize window:', err);
    }
  }

  async function handleToggleMaximize(event?: MouseEvent) {
    event?.stopPropagation();
    try {
      if (appWindow) {
        await appWindow.toggleMaximize();
        isMaximized = await appWindow.isMaximized();
        updateMaximizedClass(isMaximized);
      } else {
        isMaximized = !isMaximized;
        updateMaximizedClass(isMaximized);
      }
    } catch (err) {
      console.error('Failed to toggle maximize window:', err);
    }
  }

  async function handleClose(event?: MouseEvent) {
    event?.stopPropagation();
    try {
      if (appWindow) {
        await appWindow.close();
      }
    } catch (err) {
      console.error('Failed to close window:', err);
    }
  }
</script>

{#if isDesktop}
  {#if isMac}
    <!-- ==================== macOS Titlebar ==================== -->
    <header class="titlebar titlebar-macos">
      <!-- macOS Traffic Lights. The glyph strokes below carry Apple's own
           darkened traffic-light tints: they are OS convention, not product
           palette, so they stay literal. -->
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <div
        class="macos-traffic-lights"
        onmouseenter={() => (isTrafficHovered = true)}
        onmouseleave={() => (isTrafficHovered = false)}
        onmousedown={(e) => e.stopPropagation()}
      >
        <button
          type="button"
          class="traffic-btn traffic-close"
          title={m.close()}
          aria-label={m.close()}
          onmousedown={(e) => e.stopPropagation()}
          onclick={handleClose}
        >
          {#if isTrafficHovered}
            <svg aria-hidden="true" focusable="false" width="6" height="6" viewBox="0 0 6 6">
              <path d="M1 1L5 5M5 1L1 5" stroke="#4C0000" stroke-width="1.2" stroke-linecap="round" />
            </svg>
          {/if}
        </button>

        <button
          type="button"
          class="traffic-btn traffic-minimize"
          title={m.window_minimize()}
          aria-label={m.window_minimize()}
          onmousedown={(e) => e.stopPropagation()}
          onclick={handleMinimize}
        >
          {#if isTrafficHovered}
            <svg aria-hidden="true" focusable="false" width="6" height="2" viewBox="0 0 6 2">
              <rect width="6" height="1.2" fill="#5E3F00" rx="0.5" />
            </svg>
          {/if}
        </button>

        <button
          type="button"
          class="traffic-btn traffic-maximize"
          title={m.window_fullscreen()}
          aria-label={m.window_fullscreen()}
          onmousedown={(e) => e.stopPropagation()}
          onclick={handleToggleMaximize}
        >
          {#if isTrafficHovered}
            <svg aria-hidden="true" focusable="false" width="6" height="6" viewBox="0 0 6 6">
              <path d="M1 4L4 1M1 1H4V4" stroke="#004D00" stroke-width="1" stroke-linecap="round" />
            </svg>
          {/if}
        </button>
      </div>

      <!-- Center Brand Pill -->
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <div
        class="macos-center-drag"
        data-tauri-drag-region
        role="presentation"
        ondblclick={handleToggleMaximize}
      >
        <div class="macos-brand-pill" data-tauri-drag-region>
          <Logo size={14} variant="mark" />
          <span class="macos-brand-title" data-tauri-drag-region>Better<span class="brand-accent">Aimaira</span></span>
        </div>
      </div>

      <div class="macos-right-spacer" data-tauri-drag-region></div>
    </header>
  {:else}
    <!-- ==================== Windows & Linux Titlebar ==================== -->
    <header class="titlebar titlebar-windows" class:titlebar-linux={isLinux}>
      <!-- Left: Logo & Brand Lockup -->
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <div
        class="win-brand-lockup"
        data-tauri-drag-region
        role="presentation"
        ondblclick={handleToggleMaximize}
      >
        <Logo size={16} variant="mark" />
        <span class="win-brand-title" data-tauri-drag-region>
          Better<span class="brand-accent">Aimaira</span>
        </span>
      </div>

      <!-- Center: Drag Space -->
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <div
        class="win-drag-space"
        data-tauri-drag-region
        role="presentation"
        ondblclick={handleToggleMaximize}
      ></div>

      <!-- Right: Modern Window Controls (Flush, precise hit targets) -->
      <!-- The buttons fill this cluster edge to edge and each stops mousedown
           itself, and `-webkit-app-region: no-drag` already covers the box, so
           the group needs no listener of its own. -->
      <div class="win-controls-cluster" role="group" aria-label={m.window_controls()}>
        <button
          type="button"
          class="win-action-btn win-min-btn"
          title={m.window_minimize()}
          aria-label={m.window_minimize()}
          onmousedown={(e) => e.stopPropagation()}
          onclick={handleMinimize}
        >
          <svg aria-hidden="true" focusable="false" width="10" height="1" viewBox="0 0 10 1">
            <rect width="10" height="1" fill="currentColor" />
          </svg>
        </button>

        <button
          type="button"
          class="win-action-btn win-max-btn"
          title={isMaximized ? m.window_restore() : m.window_maximize()}
          aria-label={isMaximized ? m.window_restore() : m.window_maximize()}
          onmousedown={(e) => e.stopPropagation()}
          onclick={handleToggleMaximize}
        >
          {#if isMaximized}
            <svg aria-hidden="true" focusable="false" width="10" height="10" viewBox="0 0 10 10" fill="none">
              <rect x="2" y="0.5" width="7.5" height="7.5" stroke="currentColor" stroke-width="1" />
              <polyline points="0.5,2.5 0.5,9.5 7.5,9.5" stroke="currentColor" stroke-width="1" />
            </svg>
          {:else}
            <svg aria-hidden="true" focusable="false" width="10" height="10" viewBox="0 0 10 10" fill="none">
              <rect x="0.5" y="0.5" width="9" height="9" stroke="currentColor" stroke-width="1" rx="0.5" />
            </svg>
          {/if}
        </button>

        <button
          type="button"
          class="win-action-btn win-close-btn"
          title={m.close()}
          aria-label={m.close()}
          onmousedown={(e) => e.stopPropagation()}
          onclick={handleClose}
        >
          <svg aria-hidden="true" focusable="false" width="10" height="10" viewBox="0 0 10 10">
            <path d="M1 1L9 9M9 1L1 9" stroke="currentColor" stroke-width="1.1" stroke-linecap="round" />
          </svg>
        </button>
      </div>
    </header>
  {/if}
{/if}

<style>
  /* 36px is the window chrome band the OS reserves for us, mirrored into
     `--titlebar-height` from script. It is device geometry, not a spacing step,
     so it stays in px and does not scale with the root font size. */
  .titlebar {
    position: relative;
    top: 0;
    left: 0;
    right: 0;
    z-index: var(--z-titlebar);
    height: 36px;
    display: flex;
    align-items: center;
    user-select: none;
    -webkit-user-select: none;
    cursor: default;
    box-sizing: border-box;
    flex-shrink: 0;
  }

  /* `--primary` is a fill token that only clears 2.28:1 on white; the brand
     accent here is text, so it takes the deep step. */
  .brand-accent {
    color: var(--primary-deep);
  }

  /* ---------------- MacOS Modern Titlebar ---------------- */
  .titlebar-macos {
    padding: 0 var(--space-3);
    background: color-mix(in oklch, var(--background) 85%, transparent);
    backdrop-filter: blur(20px);
    -webkit-backdrop-filter: blur(20px);
    border-bottom: 1px solid var(--border-subtle);
    justify-content: space-between;
  }

  /* 64px mirrors `.macos-right-spacer` so the brand pill lands optically
     centred. Both track the traffic lights' fixed 12px geometry, so a rem
     value here would drift off-centre under root font scaling. */
  .macos-traffic-lights {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    width: 64px;
    -webkit-app-region: no-drag;
  }

  /* 12px circles below the 44px `--tap-min` floor on purpose: macOS traffic
     lights are a recognised OS control and enlarging them would break the
     alignment users read the window frame by. The 0.5px rim is the hairline
     Apple draws — a full pixel reads as a hard outline on a HiDPI panel. */
  .traffic-btn {
    width: 12px;
    height: 12px;
    border-radius: 50%;
    border: 0.5px solid transparent;
    display: grid;
    place-items: center;
    padding: 0;
    cursor: pointer;
    -webkit-app-region: no-drag;
    transition: transform var(--duration-instant) var(--ease-out), filter var(--duration-fast) var(--ease-out);
  }

  .traffic-btn:active {
    transform: scale(var(--press-scale));
  }

  /* Apple's traffic light hues, glyph tints included. Users identify the window
     controls by these exact colours, so they are OS convention rather than
     product palette and must not be tokenised. */
  .traffic-close {
    background: #FF5F56;
    border-color: #E0443E;
  }

  .traffic-minimize {
    background: #FFBD2E;
    border-color: #DEA123;
  }

  .traffic-maximize {
    background: #27C93F;
    border-color: #1AAB29;
  }

  .macos-center-drag {
    flex-grow: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100%;
    -webkit-app-region: drag;
  }

  /* Border only, no shadow: elevation is declared once, and a translucent bar
     over the desktop cannot carry a credible cast shadow anyway. */
  .macos-brand-pill {
    display: inline-flex;
    align-items: center;
    gap: var(--space-2);
    padding: var(--space-1) var(--space-3);
    border-radius: var(--radius-pill);
    background: color-mix(in oklch, var(--card) 70%, transparent);
    border: 1px solid var(--border-subtle);
    -webkit-app-region: drag;
  }

  /* `line-height: 1` keeps the label inside the fixed 36px band even when the
     user has raised the system font size. */
  .macos-brand-title {
    font-size: var(--text-2xs);
    font-weight: var(--weight-bold);
    line-height: 1;
    letter-spacing: -0.01em;
    color: var(--foreground);
    -webkit-app-region: drag;
  }

  .macos-right-spacer {
    width: 64px;
    -webkit-app-region: drag;
  }

  /* ---------------- Windows Modern Titlebar (Seamless & Flush) ---------------- */
  .titlebar-windows {
    padding-left: var(--space-3);
    background: var(--background);
    border-bottom: 1px solid var(--border-subtle);
    justify-content: space-between;
  }

  .win-brand-lockup {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    flex-shrink: 0;
    height: 100%;
    -webkit-app-region: drag;
  }

  .win-brand-title {
    font-size: var(--text-2xs);
    font-weight: var(--weight-bold);
    line-height: 1;
    letter-spacing: -0.01em;
    color: var(--foreground);
    -webkit-app-region: drag;
  }

  .win-drag-space {
    flex-grow: 1;
    height: 100%;
    -webkit-app-region: drag;
  }

  .win-controls-cluster {
    display: flex;
    align-items: stretch;
    height: 100%;
    margin: 0;
    padding: 0;
    gap: 0;
    flex-shrink: 0;
    -webkit-app-region: no-drag;
  }

  /* 46 x 36px is the Fluent window control geometry. It is under the 44px
     `--tap-min` floor on one axis, and that is deliberate: window chrome is
     pointer-only, and widening these buttons would misalign the app from every
     other window on the desktop. */
  .win-action-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 46px;
    height: 100%;
    margin: 0;
    padding: 0;
    background: transparent;
    border: none;
    border-radius: 0;
    color: var(--foreground);
    cursor: pointer;
    -webkit-app-region: no-drag;
    transition: background-color var(--duration-fast) var(--ease-out), color var(--duration-fast) var(--ease-out);
  }

  @media (hover: hover) {
    .win-action-btn:hover {
      background: color-mix(in oklch, var(--foreground) 8%, transparent);
    }
  }

  .win-action-btn:active {
    background: color-mix(in oklch, var(--foreground) 14%, transparent);
  }

  /* Microsoft's Fluent close-button reds, with the white glyph that pairs with
     them. This is the one control every Windows user expects to turn red, so
     the pair stays literal instead of taking the product palette. Declared
     after the shared `:hover` / `:active` rules above so it wins on order
     without `!important`. */
  @media (hover: hover) {
    .win-close-btn:hover {
      background: #e81123;
      color: #ffffff;
    }
  }

  .win-close-btn:active {
    background: #c70f1e;
    color: #ffffff;
  }

  /* ---------------- Linux Window Controls ---------------- */
  /* Narrower than Fluent: GNOME/Adwaita header bar buttons are ~38px, and the
     same OS-alignment argument as `.win-action-btn` applies. */
  .titlebar-linux .win-action-btn {
    width: 38px;
  }
</style>
