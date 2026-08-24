<script lang="ts">
  import { onMount } from "svelte";
  import { invoke, isTauri } from "$lib/invoke";
  import {
    AlertCircle,
    BookOpenCheck,
    CalendarDays,
    ClipboardCheck,
    CloudOff,
    Home,
    MonitorSmartphone,
    UserRound,
  } from "lucide-svelte";
  import Logo from "$lib/assets/Logo.svelte";
  import PageShell from "$lib/components/ui/PageShell.svelte";
  import PullToRefresh from "$lib/components/ui/PullToRefresh.svelte";
  import StateCard from "$lib/components/ui/StateCard.svelte";
  import * as m from "$lib/paraglide/messages.js";
  import type { Locale } from "$lib/paraglide/runtime.js";
  import { connectivity } from "$lib/state/connectivity.svelte";
  import { updates } from "$lib/features/updates/updates.svelte";
  import UpdateNotice from "$lib/features/updates/UpdateNotice.svelte";
  import HomeView from "./HomeView.svelte";
  import AcademicViewSkeleton from "./AcademicViewSkeleton.svelte";
  import AbsencesViewSkeleton from "./AbsencesViewSkeleton.svelte";
  import AccountViewSkeleton from "./AccountViewSkeleton.svelte";
  import CalendarViewSkeleton from "./CalendarViewSkeleton.svelte";
  import GradeAlertDrawer from "./GradeAlertDrawer.svelte";
  import { isSameWeek, startOfDay, startOfWeek } from "./date-utils";
  import { openExternalUrl } from "./course-utils";
  import { getDisplayName, getPortalHost } from "./portal-utils";
  import { cn } from '$lib/utils';
  import type {
    CalendarEvent,
    Grade,
    GradeSyncResult,
    ScheduleErrorCode,
    ScheduleResult,
    ScheduleState,
    ScheduleView,
  } from "./types";

  type Props = {
    username: string;
    portalUrl: string;
    locale: Locale;
    credentialsWarning: boolean;
    sundaysVisible: boolean;
    onLocaleChange: (locale: Locale) => Promise<void>;
    onLogout: () => Promise<void>;
  };

  const DEFAULT_WEEK_DURATION = 7;
  const SCHEDULE_CACHE_TTL_MS = 5 * 60_000;

  type CalendarViewComponent = (typeof import("./CalendarView.svelte"))["default"];
  type GradesViewComponent = (typeof import("./GradesView.svelte"))["default"];
  type AbsencesViewComponent = (typeof import("./AbsencesView.svelte"))["default"];
  type MoreViewComponent = (typeof import("./MoreView.svelte"))["default"];
  type ScheduleCacheEntry = { result: ScheduleResult; expiresAt: number };
  type PlanningSettingsResult = { sundaysVisible: boolean };
  type GradeSyncState = { kind: "loading" } | { kind: "ready" } | { kind: "error" };

  let {
    username,
    portalUrl,
    locale,
    credentialsWarning,
    sundaysVisible,
    onLocaleChange,
    onLogout,
  }: Props = $props();

  let activeView = $state<ScheduleView>("today");
  let weekStart = $state(startOfWeek(new Date()));
  let selectedDate = $state(startOfDay(new Date()));
  let now = $state(new Date());
  let schedule = $state<ScheduleState>({ kind: "loading" });
  let grades = $state<Grade[]>([]);
  let gradeSyncState = $state<GradeSyncState>({ kind: "loading" });
  let unreadGradeAlerts = $state<Grade[]>([]);
  let drawerAlerts = $state<Grade[]>([]);
  let gradeAlertDrawerOpen = $state(false);
  let scheduleRefreshing = $state(false);
  let visibleSundays = $state(false);
  let CalendarView = $state<CalendarViewComponent | null>(null);
  let GradesView = $state<GradesViewComponent | null>(null);
  let AbsencesView = $state<AbsencesViewComponent | null>(null);
  let MoreView = $state<MoreViewComponent | null>(null);
  let gradesRefresh = $state<(() => Promise<void>) | undefined>();
  let absencesRefresh = $state<(() => Promise<void>) | undefined>();
  let moreRefresh = $state<(() => Promise<void>) | undefined>();
  let viewportElement = $state<HTMLDivElement | null>(null);
  let requestSequence = 0;
  const scheduleCache = new Map<string, ScheduleCacheEntry>();
  const pendingSchedules = new Map<string, Promise<ScheduleResult>>();

  /**
   * Every data surface here is served by the Rust side. Without it there is
   * nothing to show and nothing to invent, so the shell says so instead.
   */
  const backendAvailable = isTauri();

  const copy = $derived.by(() => {
    locale;
    return {
      appName: m.app_name(),
      navToday: m.nav_today(),
      navSchedule: m.nav_schedule(),
      navGrades: m.nav_grades(),
      navAbsences: m.nav_absences(),
      navAccount: m.nav_account(),
      navLabel: m.nav_label(),
      navSectionTitle: m.menu_navigation(),
      skipToContent: m.skip_to_content(),
      logout: m.logout(),
      accountLabel: m.account_label(),
      credentialsWarning: m.credentials_not_saved(),
      openTempo: m.open_tempo(),
      backToLogin: m.back_to_login(),
      retry: m.planning_retry(),
      errorHeading: m.planning_error_heading(),
      offlineHeading: m.sync_offline(),
      offlineDescription: m.sync_offline_description(),
      desktopRequired: m.error_desktop_required(),
      lazyLoading: m.resource_loading(),
    };
  });

  const displayName = $derived(getDisplayName(username, locale));
  const portalHost = $derived(getPortalHost(portalUrl));

  const navigationItems = $derived([
    { id: "today" as ScheduleView, label: copy.navToday, icon: Home },
    { id: "schedule" as ScheduleView, label: copy.navSchedule, icon: CalendarDays },
    { id: "grades" as ScheduleView, label: copy.navGrades, icon: BookOpenCheck },
    { id: "absences" as ScheduleView, label: copy.navAbsences, icon: ClipboardCheck },
  ]);

  onMount(() => {
    visibleSundays = sundaysVisible;
    let timer: number | undefined;
    const startClock = () => {
      if (timer !== undefined || document.hidden) return;
      now = new Date();
      timer = window.setInterval(() => (now = new Date()), 30_000);
    };
    const stopClock = () => {
      if (timer === undefined) return;
      window.clearInterval(timer);
      timer = undefined;
    };
    const handleVisibilityChange = () => {
      if (document.hidden) stopClock();
      else startClock();
    };
    startClock();
    document.addEventListener("visibilitychange", handleVisibilityChange);
    const preloadTimer = window.setTimeout(() => void preloadViewComponents(), 0);
    // The update check is the least urgent request of the session: it waits for
    // the schedule to be on screen, and the store throttles repeat runs.
    const updateTimer = window.setTimeout(() => void updates.checkOnStart(), 3_000);
    void loadInitialSchedule();
    void syncGrades();
    return () => {
      window.clearTimeout(preloadTimer);
      window.clearTimeout(updateTimer);
      stopClock();
      document.removeEventListener("visibilitychange", handleVisibilityChange);
    };
  });

  async function loadInitialSchedule() {
    await Promise.all([
      loadSchedule(weekStart, DEFAULT_WEEK_DURATION),
      loadPlanningSettings(),
    ]);
  }

  async function loadPlanningSettings() {
    if (!backendAvailable) return;
    try {
      const settings = await invoke<PlanningSettingsResult>("get_planning_settings");
      visibleSundays = settings.sundaysVisible;
    } catch {
      visibleSundays = sundaysVisible;
    }
  }

  function scheduleCacheKey(startDate: Date, durationDays: number) {
    return startOfDay(startDate).toISOString() + "_" + durationDays;
  }

  function setScheduleState(nextState: ScheduleState) {
    schedule = nextState;
  }

  async function syncGrades(force = false) {
    if (!backendAvailable) return;

    try {
      const result = await invoke<GradeSyncResult>("sync_grades", { force });
      grades = result.grades;
      unreadGradeAlerts = result.unreadAlerts;
      gradeSyncState = { kind: "ready" };
    } catch (error) {
      if (gradeSyncState.kind === "loading") gradeSyncState = { kind: "error" };
      console.error("Failed to sync grades", error);
    }
  }

  async function loadSchedule(startDate: Date, durationDays = DEFAULT_WEEK_DURATION, force = false) {
    if (!backendAvailable) return;

    const key = scheduleCacheKey(startDate, durationDays);
    const cached = scheduleCache.get(key);
    if (!force && cached && cached.expiresAt > Date.now()) {
      setScheduleState({
        kind: "ready",
        events: cached.result.events,
        fetchedAt: cached.result.fetchedAt,
        cacheKey: key,
      });
      return;
    }

    const currentSequence = ++requestSequence;
    if (schedule.kind !== "ready") {
      setScheduleState({ kind: "loading" });
    } else {
      scheduleRefreshing = true;
    }

    try {
      let pending = pendingSchedules.get(key);
      if (!pending || force) {
        pending = fetchScheduleFromBackend(startDate, durationDays);
        pendingSchedules.set(key, pending);
      }
      const result = await pending;
      pendingSchedules.delete(key);
      scheduleCache.set(key, {
        result,
        expiresAt: Date.now() + SCHEDULE_CACHE_TTL_MS,
      });

      if (currentSequence !== requestSequence) return;

      setScheduleState({
        kind: "ready",
        events: result.events,
        fetchedAt: result.fetchedAt,
        cacheKey: key,
      });
    } catch (error) {
      pendingSchedules.delete(key);
      if (currentSequence !== requestSequence) return;
      const code = parseScheduleError(error);
      setScheduleState({ kind: "error", code });
    } finally {
      if (currentSequence === requestSequence) {
        scheduleRefreshing = false;
      }
    }
  }

  async function fetchScheduleFromBackend(startDate: Date, durationDays: number): Promise<ScheduleResult> {
    return await invoke<ScheduleResult>("get_schedule", {
      request: {
        start: startOfDay(startDate).toISOString(),
        duration: durationDays,
      },
    });
  }

  function openGradeAlerts() {
    drawerAlerts = [...unreadGradeAlerts];
    gradeAlertDrawerOpen = true;
  }

  async function closeGradeAlerts() {
    gradeAlertDrawerOpen = false;
    if (drawerAlerts.length === 0 || !backendAvailable) return;
    try {
      await invoke("mark_grade_alerts_read");
      unreadGradeAlerts = unreadGradeAlerts.filter(
        (alert) => !drawerAlerts.some((dismissed) => dismissed.id === alert.id)
      );
      drawerAlerts = [];
    } catch (error) {
      console.error("Failed to dismiss grade alerts", error);
    }
  }

  function parseScheduleError(error: unknown): ScheduleErrorCode {
    if (typeof error === "object" && error !== null && "code" in error && typeof error.code === "string") {
      switch (error.code) {
        case "session_expired":
        case "planning_unavailable":
        case "planning_invalid_response":
        case "invalid_schedule_range":
        case "internal_error":
          return error.code;
      }
    }
    return "internal_error";
  }

  function scheduleErrorMessage(code: ScheduleErrorCode) {
    locale;
    switch (code) {
      case "session_expired": return m.planning_session_expired();
      case "planning_unavailable": return m.planning_unavailable();
      case "planning_invalid_response": return m.planning_invalid_response();
      case "invalid_schedule_range": return m.planning_invalid_range();
      case "internal_error": return m.planning_generic_error();
    }
  }

  /**
   * The launch notice was tapped. More is where an update is installed, and the
   * card scrolls itself into view once that lazily loaded view has mounted.
   */
  function openUpdateCard() {
    updates.revealFromNotice();
    setView("more");
  }

  function setView(view: ScheduleView) {
    void loadViewComponent(view);
    activeView = view;
    if (view === "today" && !isSameWeek(weekStart, now)) {
      weekStart = startOfWeek(new Date());
      selectedDate = startOfDay(new Date());
      void loadSchedule(weekStart, DEFAULT_WEEK_DURATION);
    }
  }

  async function loadViewComponent(view: ScheduleView) {
    switch (view) {
      case "schedule":
        CalendarView ??= (await import("./CalendarView.svelte")).default;
        break;
      case "grades":
        GradesView ??= (await import("./GradesView.svelte")).default;
        break;
      case "absences":
        AbsencesView ??= (await import("./AbsencesView.svelte")).default;
        break;
      case "more":
        MoreView ??= (await import("./MoreView.svelte")).default;
        break;
    }
  }

  async function preloadViewComponents() {
    await Promise.allSettled([
      loadViewComponent("schedule"),
      loadViewComponent("grades"),
      loadViewComponent("absences"),
      loadViewComponent("more"),
    ]);
  }

  async function handleCalendarPeriodChange(startDate: Date, durationDays: number) {
    weekStart = startOfWeek(startDate);
    await loadSchedule(startDate, durationDays);
  }

  async function openTempoSession(event: CalendarEvent) {
    await openExternalUrl(event.tempoUrl);
  }

  async function handleGlobalRefresh() {
    switch (activeView) {
      case "today":
        await Promise.all([
          loadSchedule(weekStart, DEFAULT_WEEK_DURATION, true),
          syncGrades(true),
        ]);
        break;
      case "schedule":
        await loadSchedule(weekStart, DEFAULT_WEEK_DURATION, true);
        break;
      case "grades":
        if (gradesRefresh) {
          await gradesRefresh();
        } else {
          await syncGrades(true);
        }
        break;
      case "absences":
        if (absencesRefresh) {
          await absencesRefresh();
        }
        break;
      case "more":
        if (moreRefresh) {
          await moreRefresh();
        }
        break;
    }
  }
  // The rail is the only desktop shape: fixed-width icon targets, each name
  // served on hover rather than by a panel that stays expanded.
  const railTarget =
    'group relative grid size-[2.85rem] place-items-center rounded-md bg-transparent' +
    ' transition-control active:scale-(--press-scale)';

  const tooltip =
    'pointer-events-none absolute top-1/2 left-[calc(100%+var(--space-2))] z-raised flex flex-col' +
    ' rail-tooltip -translate-x-1 -translate-y-1/2 rounded-sm bg-primary-deep px-3 py-2' +
    ' text-left whitespace-nowrap text-card shadow-md opacity-0' +
    ' transition-[opacity,translate] duration-fast ease-out' +
    ' group-focus-visible:translate-x-0 group-focus-visible:opacity-100' +
    ' fine-group-hover:translate-x-0 fine-group-hover:opacity-100';

  const tooltipTitle = 'text-xs leading-[1.3] font-bold';
  const tooltipSub = 'text-2xs leading-[1.3] font-semibold opacity-72';

  // A count badge hangs off the icon's own box: the offsets are optical nudges,
  // not spacing steps, and a hairline ring detaches it from the glyph beneath.
  const countBadge =
    'absolute grid place-items-center rounded-pill border-[1.5px] border-card bg-primary' +
    ' px-1 text-2xs font-extrabold tabular-nums text-primary-foreground';

  const dockPill =
    'flex min-h-11 flex-col items-center justify-center gap-[0.15rem] rounded-lg' +
    ' bg-transparent px-[0.15rem] py-1 transition-control active:scale-(--press-scale)';

  // An available update is worth a mark on the door to it, not a number.
  const updateDot =
    'absolute -top-px -right-[3px] size-[0.55rem] rounded-full border-[1.5px]' +
    ' border-card bg-primary';

  const dockLabel = 'text-2xs leading-[1.1] font-bold tracking-[-0.01em]';

  const viewEnter = 'flex w-full flex-1 flex-col animate-fade-in-fast-forwards';

  const banner =
    'mx-5 mt-4 flex items-center gap-3 rounded-md px-4 py-3 text-base';
</script>

<div
  class="app-shell relative flex size-full max-h-full flex-row overflow-hidden
         bg-background text-foreground"
>
  <!-- Reaching the content past the titlebar, brand, rail toggle, five
       destinations and avatar is ten stops otherwise. -->
  <a
    class="absolute top-2 left-2 z-overlay -translate-y-[250%] rounded-md bg-secondary px-4 py-2
           text-sm font-semibold text-secondary-foreground no-underline
           transition-[translate] duration-fast ease-out focus-visible:translate-y-0"
    href="#app-main-content">{copy.skipToContent}</a
  >

  <!-- 1. LEFT NAVIGATION RAIL (icons only; the label is a hover tooltip) -->
  <!-- Tooltips hang outside the rail's box, so nothing here may clip. -->
  <aside
    class="desktop-app-sidebar z-sidebar h-full max-h-full w-[3.85rem] min-w-[3.85rem]
           max-w-[3.85rem] flex-col items-center gap-5 overflow-visible border-r
           border-border-subtle bg-card px-1 py-4"
    aria-label={copy.navLabel}
  >
    <div class="flex shrink-0 justify-center">
      <button
        type="button"
        class={cn(railTarget, 'text-inherit fine-hover:bg-muted fine-hover:text-primary-deep')}
        onclick={() => setView("today")}
        aria-label={copy.appName}
      >
        <Logo size={22} variant="icon" />
        <span class={tooltip} aria-hidden="true">
          <span class={tooltipTitle}>{copy.appName}</span>
          <span class={tooltipSub}>{portalHost}</span>
        </span>
      </button>
    </div>

    <nav
      class="sidebar-nav-list flex min-h-0 flex-1 flex-col items-center gap-1"
      aria-label={copy.navSectionTitle}
    >
      {#each navigationItems as item (item.id)}
        {@const Icon = item.icon}
        {@const isActive = activeView === item.id}
        <button
          class={cn(
            railTarget,
            'p-2 fine-hover:bg-muted fine-hover:text-primary-deep',
            isActive ? 'bg-muted text-primary-deep' : 'text-muted-foreground'
          )}
          onclick={() => setView(item.id)}
          type="button"
          aria-current={isActive ? "page" : undefined}
          aria-label={item.label}
        >
          <div class="relative grid shrink-0 place-items-center">
            <Icon size={19} aria-hidden="true" />
            {#if item.id === "grades" && unreadGradeAlerts.length > 0}
              <span class={cn(countBadge, '-top-[3px] -right-[6px] h-[0.85rem] min-w-[0.85rem]')}
                >{unreadGradeAlerts.length}</span
              >
            {/if}
          </div>
          <span class={tooltip} aria-hidden="true">
            <span class={tooltipTitle}>{item.label}</span>
          </span>
        </button>
      {/each}
    </nav>

    <div
      class="flex w-full shrink-0 flex-col items-center gap-1 border-t border-border-subtle pt-3"
    >
      <!-- The account entry is the single door to the "more" surface: it used to
           share it with a redundant ellipsis destination. -->
      <button
        type="button"
        class={cn(
          'group relative grid size-(--tap-min) shrink-0 place-items-center rounded-full',
          'bg-primary-deep text-card transition-control active:scale-(--press-scale)',
          'fine-hover:bg-primary-deep-hover',
          activeView === "more" ? 'shadow-[0_0_0_3px_var(--primary)]' : 'shadow-[0_0_0_0_transparent]'
        )}
        aria-current={activeView === "more" ? "page" : undefined}
        aria-label={copy.accountLabel}
        onclick={() => setView("more")}
      >
        <UserRound size={16} aria-hidden="true" />
        {#if updates.available}
          <span class={updateDot} role="img" aria-label={m.update_badge_label()}></span>
        {/if}
        <span class={tooltip} aria-hidden="true">
          <span class={tooltipTitle}>{displayName}</span>
          <!-- A username that is not an address is already the display name. -->
          {#if username && username !== displayName}
            <span class={tooltipSub}>{username}</span>
          {/if}
        </span>
      </button>
    </div>
  </aside>

  <!-- 2. MAIN CONTENT VIEWPORT -->
  <div
    class="main-content-viewport flex h-full max-h-full min-w-0 flex-1 flex-col
           overflow-x-hidden overflow-y-auto overscroll-y-contain"
    bind:this={viewportElement}
  >
    <!-- The dock is the primary navigation on compact windows, so it comes
         before the content it navigates rather than last in the tab order. -->
    <nav
      class="bottom-nav pointer-events-none fixed inset-x-0 bottom-0 z-nav justify-center
             px-3 pt-1 pb-[max(var(--space-2),var(--safe-bottom))]"
      aria-label={copy.navLabel}
    >
      <!-- A dock that genuinely floats above the content, on frosted glass. -->
      <div
        class="pointer-events-auto grid w-[min(100%,32rem)] grid-cols-5 gap-1 rounded-xl
               border border-dock-edge bg-dock-veil px-1 py-[0.3rem] shadow-lg
               backdrop-blur-[20px]"
      >
        {#each navigationItems as item (item.id)}
          {@const Icon = item.icon}
          {@const isActive = activeView === item.id}
          <button
            class={cn(
              dockPill,
              isActive ? 'bg-muted text-primary-deep' : 'text-muted-foreground'
            )}
            type="button"
            aria-current={isActive ? "page" : undefined}
            onclick={() => setView(item.id)}
          >
            <div class="relative flex items-center justify-center">
              <Icon size={20} strokeWidth={isActive ? 2.4 : 1.9} aria-hidden="true" />
              {#if item.id === "grades" && unreadGradeAlerts.length > 0}
                <span class={cn(countBadge, '-top-[3px] -right-[7px] h-[0.95rem] min-w-[0.95rem]')}
                  >{unreadGradeAlerts.length}</span
                >
              {/if}
            </div>
            <span class={cn(dockLabel, isActive && 'font-extrabold')}>{item.label}</span>
          </button>
        {/each}

        <!-- Same single door as the rail's avatar: the ellipsis tab it replaces
             pointed at this very view. -->
        <button
          class={cn(
            dockPill,
            activeView === "more" ? 'bg-muted text-primary-deep' : 'text-muted-foreground'
          )}
          type="button"
          aria-current={activeView === "more" ? "page" : undefined}
          onclick={() => setView("more")}
        >
          <div class="relative flex items-center justify-center">
            <UserRound size={20} strokeWidth={activeView === "more" ? 2.4 : 1.9} aria-hidden="true" />
            {#if updates.available}
              <span class={updateDot} role="img" aria-label={m.update_badge_label()}></span>
            {/if}
          </div>
          <span class={cn(dockLabel, activeView === "more" && 'font-extrabold')}
            >{copy.navAccount}</span
          >
        </button>
      </div>
    </nav>

    <PullToRefresh onRefresh={handleGlobalRefresh} scrollElement={viewportElement}>
      <div
        class="main-viewport flex w-full flex-1 flex-col focus-visible:outline-none"
        id="app-main-content"
        tabindex="-1"
      >
        {#if credentialsWarning}
          <div
            class={cn(
              banner,
              'border border-warning bg-warning-surface font-semibold text-warning-strong'
            )}
            role="status"
          >
            <AlertCircle size={18} aria-hidden="true" />
            <span>{copy.credentialsWarning}</span>
          </div>
        {/if}

        {#if unreadGradeAlerts.length > 0 && activeView === "today"}
          <button
            class={cn(
              banner,
              'min-h-(--tap-min) border border-muted-strong bg-muted text-left font-bold',
              'text-primary-deep transition-control active:scale-(--press-scale)',
              'fine-hover:bg-muted-strong'
            )}
            type="button"
            onclick={openGradeAlerts}
          >
            <BookOpenCheck size={19} aria-hidden="true" />
            <span>{m.new_grades_banner({ count: unreadGradeAlerts.length })}</span>
          </button>
        {/if}

        {#if !backendAvailable}
          <!-- No Rust side means no portal session and no data. Naming that is the
               only honest thing this shell can render. -->
          <div class="mx-5 my-8">
            <StateCard
              kind="error"
              icon={MonitorSmartphone}
              title={copy.desktopRequired}
            />
          </div>
        {:else if activeView === "today"}
          <div class={viewEnter}>
            <HomeView
              {username}
              events={schedule.kind === "ready" ? schedule.events : []}
              {grades}
              gradesLoading={gradeSyncState.kind === "loading"}
              {now}
              {locale}
              scheduleState={schedule}
              onOpenSchedule={(date) => {
                if (date) selectedDate = startOfDay(date);
                setView("schedule");
              }}
              onOpenGrades={() => setView("grades")}
              onOpenTempo={openTempoSession}
              onRefresh={handleGlobalRefresh}
              refreshing={schedule.kind === "loading" || scheduleRefreshing}
              fetchedAt={schedule.kind === "ready" ? schedule.fetchedAt : null}
            />
          </div>
        {:else if activeView === "schedule"}
          <div class={viewEnter}>
            {#if schedule.kind === "error"}
              <!-- A dead network on this machine is not the portal being down, and
                   the recovery differs, so the two are never merged. -->
              <div class="mx-5 my-8">
                {#if !connectivity.online}
                  <StateCard
                    kind="error"
                    icon={CloudOff}
                    title={copy.offlineHeading}
                    description={copy.offlineDescription}
                    actionLabel={copy.retry}
                    onAction={() => loadSchedule(weekStart, DEFAULT_WEEK_DURATION, true)}
                  />
                {:else if schedule.code === "session_expired"}
                  <StateCard
                    kind="expired"
                    icon={AlertCircle}
                    title={copy.errorHeading}
                    description={scheduleErrorMessage(schedule.code)}
                    actionLabel={copy.backToLogin}
                    onAction={onLogout}
                  />
                {:else}
                  <StateCard
                    kind="error"
                    icon={AlertCircle}
                    title={copy.errorHeading}
                    description={scheduleErrorMessage(schedule.code)}
                    actionLabel={copy.retry}
                    onAction={() => loadSchedule(weekStart, DEFAULT_WEEK_DURATION, true)}
                  />
                {/if}
              </div>
            {:else if CalendarView}
              <CalendarView
                events={schedule.kind === "ready" ? schedule.events : []}
                {locale}
                sundaysVisible={visibleSundays}
                {now}
                initialScope="week"
                {selectedDate}
                loading={schedule.kind === "loading" || scheduleRefreshing}
                onPeriodChange={handleCalendarPeriodChange}
                onRefresh={() => loadSchedule(weekStart, DEFAULT_WEEK_DURATION, true)}
                onOpenTempo={openTempoSession}
              />
            {:else}
              <CalendarViewSkeleton ariaLabel={copy.lazyLoading} full />
            {/if}
          </div>
        {:else if activeView === "grades"}
          <div class={cn(viewEnter, 'min-w-0 flex-auto')}>
            {#if GradesView}
              <GradesView {locale} {onLogout} bind:refresh={gradesRefresh} />
            {:else}
              <PageShell>
                <AcademicViewSkeleton ariaLabel={copy.lazyLoading} heroLabel={copy.navGrades} />
              </PageShell>
            {/if}
          </div>
        {:else if activeView === "absences"}
          <div class={cn(viewEnter, 'min-w-0 flex-auto')}>
            {#if AbsencesView}
              <AbsencesView {locale} {onLogout} bind:refresh={absencesRefresh} />
            {:else}
              <PageShell>
                <AbsencesViewSkeleton ariaLabel={copy.lazyLoading} heroLabel={copy.navAbsences} />
              </PageShell>
            {/if}
          </div>
        {:else if activeView === "more"}
          <div class={cn(viewEnter, 'min-w-0 flex-auto')}>
            {#if MoreView}
              <MoreView
                {username}
                {portalUrl}
                {locale}
                {onLocaleChange}
                {onLogout}
                bind:refresh={moreRefresh}
              />
            {:else}
              <PageShell>
                <AccountViewSkeleton ariaLabel={copy.lazyLoading} includeTabs />
              </PageShell>
            {/if}
          </div>
        {/if}
      </div>
    </PullToRefresh>

    <!-- Where this sits is a layout-mode decision, so every offset lives in the
         block below rather than in utilities: a phone gets a banner across the
         top, a desktop window a card in the free bottom-right corner. -->
    <div class="update-notice-slot pointer-events-none absolute z-overlay flex">
      <UpdateNotice {locale} onOpen={openUpdateCard} />
    </div>

    {#if gradeAlertDrawerOpen}
      <GradeAlertDrawer
        alerts={drawerAlerts}
        {locale}
        onClose={closeGradeAlerts}
        onOpenGrades={() => setView("grades")}
      />
    {/if}
  </div>
</div>

<style>
  /* The rail's tooltip arrow: a rotated square rather than a border triangle, so
     it inherits the bubble's own background and cannot desync from it. */
  .rail-tooltip::before {
    content: "";
    position: absolute;
    top: 50%;
    right: 100%;
    width: 0.45rem;
    height: 0.45rem;
    background: inherit;
    transform: translate(50%, -50%) rotate(45deg);
  }

  /* =========================================================================
     LAYOUT MODE

     This block stays hand-written CSS on purpose. The shell picks between the
     dock and the rail on window width, and TitleBar's `mobile-app` /
     `desktop-app` root classes have to override that width query — `mobile:dev`
     runs the mobile shell in a wide desktop window. The root-class selectors
     carry one class more than the query's, and a media query adds no
     specificity, so they win on specificity alone. Expressed as utilities both
     sides would tie at (0,1,0) and the outcome would hang on source order.

     Consequence: never put a `display`, `padding-top` or `padding-bottom`
     utility on these four elements. It would lose here, silently.
     ========================================================================= */

  /* Android hands the app an edge-to-edge window whose top sits under the
     status bar, and a webview measures no inset for it. The shell pays that
     inset above the scroller rather than inside it, so the cleared strip stays
     put instead of sliding under the clock as the page moves. Every platform
     without system-bar insets resolves `--safe-top` to 0. */
  .app-shell {
    padding-top: var(--safe-top);
  }

  .desktop-app-sidebar {
    display: none;
  }

  .bottom-nav {
    display: flex;
  }

  .main-viewport {
    padding-bottom: calc(4.5rem + var(--safe-bottom));
  }

  /* A banner across the top, where a phone already puts its own notifications.
     `--safe-top` is paid here because an absolutely positioned child is laid
     against the shell's padding box, which on Android starts under the status
     bar. */
  .update-notice-slot {
    top: var(--safe-top);
    right: 0;
    bottom: auto;
    left: 0;
    justify-content: center;
    padding: var(--space-2) 0.75rem 0;
  }

  @media (min-width: 48rem) {
    .app-shell {
      padding-top: 0;
    }

    .desktop-app-sidebar {
      display: flex;
    }

    .bottom-nav {
      display: none;
    }

    .main-viewport {
      padding-bottom: 0;
    }

    /* A desktop window reads left to right from the top: a banner there lands on
       the content. The bottom-right corner is the only one nothing else uses —
       the rail owns the left edge, the avatar the bottom of it. */
    .update-notice-slot {
      top: auto;
      right: 0;
      bottom: 0;
      left: auto;
      justify-content: flex-end;
      width: min(26rem, calc(100% - 2rem));
      padding: 0 1rem 1rem;
    }
  }

  :global(html.mobile-app) .app-shell {
    padding-top: var(--safe-top);
  }

  :global(html.mobile-app) .desktop-app-sidebar {
    display: none;
  }

  :global(html.mobile-app) .bottom-nav {
    display: flex;
  }

  :global(html.mobile-app) .main-viewport {
    padding-bottom: calc(4.5rem + var(--safe-bottom));
  }

  :global(html.mobile-app) .update-notice-slot {
    top: var(--safe-top);
    right: 0;
    bottom: auto;
    left: 0;
    justify-content: center;
    padding: var(--space-2) 0.75rem 0;
  }

  :global(html.desktop-app) .app-shell {
    padding-top: 0;
  }

  :global(html.desktop-app) .desktop-app-sidebar {
    display: flex;
  }

  :global(html.desktop-app) .bottom-nav {
    display: none;
  }

  :global(html.desktop-app) .main-viewport {
    padding-bottom: 0;
  }

  :global(html.desktop-app) .update-notice-slot {
    top: auto;
    right: 0;
    bottom: 0;
    left: auto;
    justify-content: flex-end;
    width: min(26rem, calc(100% - 2rem));
    padding: 0 1rem 1rem;
  }
</style>
