<script lang="ts">
  import { onMount } from 'svelte';
  import { isTauri } from '@tauri-apps/api/core';
  import Logo from '$lib/assets/Logo.svelte';
  import * as m from '$lib/paraglide/messages.js';
  import { cn } from '$lib/utils';

  let isDesktop = $state(false);
  let isMac = $state(false);
  let isWindows = $state(false);
  let isLinux = $state(false);
  let isMaximized = $state(false);
  let isTrafficHovered = $state(false);

  const bar = 'relative inset-x-0 top-0 z-titlebar flex h-[36px] shrink-0 items-center select-none cursor-default';

  const light =
    'grid size-[12px] place-items-center rounded-full border-[0.5px] p-0 cursor-pointer no-drag-region' +
    ' transition-transform duration-instant ease-out active:scale-(--press-scale)';

  // `leading-none` keeps the label inside the fixed 36px band even when the
  // user has raised the system font size. `--primary` is a fill token that
  // only clears 2.28:1 on white, so the brand accent takes the deep step.
  const brandTitle = 'text-2xs leading-none font-bold tracking-[-0.01em] text-foreground';

  // 46 x 36px is the Fluent window control geometry, 38px the narrower
  // GNOME/Adwaita one. Both sit under the 44px `--tap-min` floor on one axis,
  // and that is deliberate: window chrome is pointer-only, and widening these
  // would misalign the app from every other window on the desktop.
  const controlBtn = $derived(
    cn(
      'flex h-full items-center justify-center bg-transparent text-foreground',
      'rounded-none no-drag-region cursor-pointer',
      'transition-[background-color,color] duration-fast ease-out',
      'hover:bg-ink-wash active:bg-ink-wash-strong',
      isLinux ? 'w-[38px]' : 'w-[46px]'
    )
  );

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
    <!-- 36px is the window chrome band the OS reserves for us, mirrored into
         '--titlebar-height' from script. It is device geometry, not a spacing
         step, so it stays in px and must not scale with the root font size. -->
    <header
      class="{bar} justify-between border-b border-border-subtle bg-chrome-veil px-3
             backdrop-blur-[20px]"
    >
      <!-- macOS Traffic Lights. 64px mirrors the right-hand spacer so the brand
           pill lands optically centred; both track the lights' fixed 12px
           geometry, so a rem value here would drift under font scaling. -->
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <div
        class="no-drag-region flex w-[64px] items-center gap-2"
        onmouseenter={() => (isTrafficHovered = true)}
        onmouseleave={() => (isTrafficHovered = false)}
        onmousedown={(e) => e.stopPropagation()}
      >
        <!-- 12px circles sit below the 44px '--tap-min' floor on purpose: these
             are a recognised OS control, and enlarging them would break the
             alignment users read the window frame by. The 0.5px rim is Apple's
             hairline \u2014 a full pixel reads as a hard outline on a HiDPI panel.
             The hues and the glyph strokes are Apple's own, so they stay
             literal rather than taking the product palette. -->
        <button
          type="button"
          class="{light} border-[#E0443E] bg-[#FF5F56]"
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
          class="{light} border-[#DEA123] bg-[#FFBD2E]"
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
          class="{light} border-[#1AAB29] bg-[#27C93F]"
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

      <!-- Center Brand Pill. Border only, no shadow: elevation is declared once,
           and a translucent bar over the desktop cannot carry a credible cast
           shadow anyway. -->
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <div
        class="drag-region flex h-full grow items-center justify-center"
        data-tauri-drag-region
        role="presentation"
        ondblclick={handleToggleMaximize}
      >
        <div
          class="drag-region inline-flex items-center gap-2 rounded-pill border
                 border-border-subtle bg-chrome-pill px-3 py-1"
          data-tauri-drag-region
        >
          <Logo size={14} variant="mark" />
          <!-- 'leading-none' keeps the label inside the fixed 36px band even
               when the user has raised the system font size. -->
          <span class="{brandTitle} drag-region" data-tauri-drag-region
            >Better<span class="text-primary-deep">Aimaira</span></span
          >
        </div>
      </div>

      <div class="drag-region w-[64px]" data-tauri-drag-region></div>
    </header>
  {:else}
    <!-- ==================== Windows & Linux Titlebar ==================== -->
    <header class="{bar} justify-between border-b border-border-subtle bg-background pl-3">
      <!-- Left: Logo & Brand Lockup -->
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <div
        class="drag-region flex h-full shrink-0 items-center gap-2"
        data-tauri-drag-region
        role="presentation"
        ondblclick={handleToggleMaximize}
      >
        <Logo size={16} variant="mark" />
        <span class="{brandTitle} drag-region" data-tauri-drag-region>
          Better<span class="text-primary-deep">Aimaira</span>
        </span>
      </div>

      <!-- Center: Drag Space -->
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <div
        class="drag-region h-full grow"
        data-tauri-drag-region
        role="presentation"
        ondblclick={handleToggleMaximize}
      ></div>

      <!-- Right: Modern Window Controls (Flush, precise hit targets) -->
      <!-- The buttons fill this cluster edge to edge and each stops mousedown
           itself, and 'no-drag-region' already covers the box, so the group
           needs no listener of its own. -->
      <div
        class="no-drag-region flex h-full shrink-0 items-stretch"
        role="group"
        aria-label={m.window_controls()}
      >
        <button
          type="button"
          class={controlBtn}
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
          class={controlBtn}
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

        <!-- Microsoft's Fluent close-button reds, with the white glyph that
             pairs with them. This is the one control every Windows user expects
             to turn red, so the pair stays literal instead of taking the
             product palette. -->
        <button
          type="button"
          class={cn(
            controlBtn,
            'hover:bg-[#e81123] hover:text-white active:bg-[#c70f1e] active:text-white'
          )}
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
