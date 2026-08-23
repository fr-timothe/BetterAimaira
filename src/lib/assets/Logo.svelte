<script lang="ts">
  type LogoVariant = 'mark' | 'icon' | 'lockup';
  type LogoTheme = 'light' | 'dark' | 'auto';

  interface Props {
    size?: number | string;
    variant?: LogoVariant;
    theme?: LogoTheme;
    class?: string;
    showText?: boolean;
    ariaLabel?: string;
  }

  let {
    size = 24,
    variant = 'mark',
    theme = 'auto',
    class: className = '',
    showText = false,
    ariaLabel = 'BetterAimaira',
  }: Props = $props();

  const dimension = $derived(typeof size === 'number' ? `${size}px` : size);
</script>

{#if variant === 'lockup' || showText}
  <div
    class="betteraimaira-logo-lockup {className}"
    style:--logo-size={dimension}
    role="img"
    aria-label={ariaLabel}
  >
    <svg
      class="logo-mark"
      viewBox="0 0 100 100"
      fill="none"
      xmlns="http://www.w3.org/2000/svg"
      style:width={dimension}
      style:height={dimension}
      aria-hidden="true"
    >
      <rect x="22" y="24" width="16" height="52" rx="8" class="logo-stem-left" />
      <rect x="62" y="38" width="16" height="38" rx="8" class="logo-stem-right" />
      <rect x="42" y="24" width="16" height="32" rx="8" fill="var(--primary, #00B9E8)" />
      <circle cx="50" cy="68" r="5" fill="var(--primary, #00B9E8)" />
    </svg>
    <span class="logo-text">Better<span class="logo-text-accent">Aimaira</span></span>
  </div>
{:else if variant === 'icon'}
  <svg
    class="betteraimaira-logo-icon {className}"
    viewBox="0 0 100 100"
    fill="none"
    xmlns="http://www.w3.org/2000/svg"
    style:width={dimension}
    style:height={dimension}
    role="img"
    aria-label={ariaLabel}
  >
    <!-- Background Squircle with subtle border for dark/light context -->
    <rect width="100" height="100" rx="24" class="logo-icon-bg" fill="#13253F" stroke="rgba(255, 255, 255, 0.12)" stroke-width="2" />
    <!-- Left Pillar (Solid White) -->
    <rect x="22" y="24" width="16" height="52" rx="8" fill="#FFFFFF" fill-opacity="0.95" />
    <!-- Right Pillar (Muted Light-Slate Blue) -->
    <rect x="62" y="38" width="16" height="38" rx="8" fill="#4B6A94" />
    <!-- Center Active Slot (Cyan) -->
    <rect x="42" y="24" width="16" height="32" rx="8" fill="#00B9E8" />
    <!-- Pulse Point -->
    <circle cx="50" cy="68" r="5" fill="#00B9E8" />
  </svg>
{:else}
  <!-- Bare Mark Variant (Transparent) -->
  <svg
    class="betteraimaira-logo-mark {className}"
    class:dark-theme={theme === 'dark'}
    class:light-theme={theme === 'light'}
    viewBox="0 0 100 100"
    fill="none"
    xmlns="http://www.w3.org/2000/svg"
    style:width={dimension}
    style:height={dimension}
    role="img"
    aria-label={ariaLabel}
  >
    <rect x="22" y="24" width="16" height="52" rx="8" class="logo-stem-left" />
    <rect x="62" y="38" width="16" height="38" rx="8" class="logo-stem-right" />
    <rect x="42" y="24" width="16" height="32" rx="8" class="logo-slot-active" />
    <circle cx="50" cy="68" r="5" class="logo-beacon" />
  </svg>
{/if}

<style>
  .betteraimaira-logo-lockup {
    display: inline-flex;
    align-items: center;
    gap: 0.55em;
    user-select: none;
    line-height: 1;
  }

  .logo-text {
    font-size: calc(var(--logo-size, 24px) * 0.85);
    font-weight: 800;
    letter-spacing: -0.03em;
    color: var(--foreground, #13253F);
  }

  .logo-text-accent {
    color: var(--primary, #00B9E8);
  }

  .betteraimaira-logo-mark,
  .logo-mark {
    display: inline-block;
    vertical-align: middle;
    flex-shrink: 0;
  }

  .betteraimaira-logo-icon {
    display: inline-block;
    vertical-align: middle;
    flex-shrink: 0;
    border-radius: calc(var(--logo-size, 24px) * 0.24);
  }

  /* Default theme behavior */
  .logo-stem-left {
    fill: var(--foreground, #13253F);
  }

  .logo-stem-right {
    fill: color-mix(in oklch, var(--foreground, #13253F) 55%, transparent);
  }

  .logo-slot-active,
  .logo-beacon {
    fill: var(--primary, #00B9E8);
  }

  /* Explicit dark theme overrides */
  .dark-theme .logo-stem-left {
    fill: #FFFFFF;
  }

  .dark-theme .logo-stem-right {
    fill: #627D98;
  }

  /* Explicit light theme overrides */
  .light-theme .logo-stem-left {
    fill: #13253F;
  }

  .light-theme .logo-stem-right {
    fill: #223D63;
    fill-opacity: 0.55;
  }
</style>
