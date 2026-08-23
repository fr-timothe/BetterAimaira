<script lang="ts">
  import { onMount } from "svelte";
  import { invoke, isTauri } from "@tauri-apps/api/core";
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
  import HomeView from "./HomeView.svelte";
  import AcademicViewSkeleton from "./AcademicViewSkeleton.svelte";
  import AbsencesViewSkeleton from "./AbsencesViewSkeleton.svelte";
  import AccountViewSkeleton from "./AccountViewSkeleton.svelte";
  import CalendarViewSkeleton from "./CalendarViewSkeleton.svelte";
  import GradeAlertDrawer from "./GradeAlertDrawer.svelte";
  import { isSameWeek, startOfDay, startOfWeek } from "./date-utils";
  import { openExternalUrl } from "./course-utils";
  import { getDisplayName, getPortalHost } from "./portal-utils";
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
    void loadInitialSchedule();
    void syncGrades();
    return () => {
      window.clearTimeout(preloadTimer);
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
</script>

<div class="app-shell">
  <!-- Reaching the content past the titlebar, brand, rail toggle, five
       destinations and avatar is ten stops otherwise. -->
  <a class="skip-to-content" href="#app-main-content">{copy.skipToContent}</a>

  <!-- 1. LEFT NAVIGATION RAIL (icons only; the label is a hover tooltip) -->
  <aside class="desktop-app-sidebar" aria-label={copy.navLabel}>
    <div class="sidebar-brand-box">
      <button
        type="button"
        class="brand-click-wrap rail-tip-anchor"
        onclick={() => setView("today")}
        aria-label={copy.appName}
      >
        <Logo size={22} variant="icon" />
        <span class="rail-tooltip" aria-hidden="true">
          <span class="rail-tooltip-title">{copy.appName}</span>
          <span class="rail-tooltip-sub">{portalHost}</span>
        </span>
      </button>
    </div>

    <nav class="sidebar-nav-list" aria-label={copy.navSectionTitle}>
      {#each navigationItems as item (item.id)}
        {@const Icon = item.icon}
        {@const isActive = activeView === item.id}
        <button
          class="sidebar-nav-btn rail-tip-anchor"
          class:active={isActive}
          onclick={() => setView(item.id)}
          type="button"
          aria-current={isActive ? "page" : undefined}
          aria-label={item.label}
        >
          <div class="sidebar-icon-wrap">
            <Icon size={19} aria-hidden="true" />
            {#if item.id === "grades" && unreadGradeAlerts.length > 0}
              <span class="sidebar-badge">{unreadGradeAlerts.length}</span>
            {/if}
          </div>
          <span class="rail-tooltip" aria-hidden="true">
            <span class="rail-tooltip-title">{item.label}</span>
          </span>
        </button>
      {/each}
    </nav>

    <div class="sidebar-user-footer">
      <!-- The account entry is the single door to the "more" surface: it used to
           share it with a redundant ellipsis destination. -->
      <button
        type="button"
        class="user-avatar-circle rail-tip-anchor"
        class:active={activeView === "more"}
        aria-current={activeView === "more" ? "page" : undefined}
        aria-label={copy.accountLabel}
        onclick={() => setView("more")}
      >
        <UserRound size={16} aria-hidden="true" />
        <span class="rail-tooltip" aria-hidden="true">
          <span class="rail-tooltip-title">{displayName}</span>
          <!-- A username that is not an address is already the display name. -->
          {#if username && username !== displayName}
            <span class="rail-tooltip-sub">{username}</span>
          {/if}
        </span>
      </button>
    </div>
  </aside>

  <!-- 2. MAIN CONTENT VIEWPORT -->
  <div class="main-content-viewport" bind:this={viewportElement}>
    <!-- The dock is the primary navigation on compact windows, so it comes
         before the content it navigates rather than last in the tab order. -->
    <nav class="bottom-nav" aria-label={copy.navLabel}>
      <div class="bottom-nav-container">
        {#each navigationItems as item (item.id)}
          {@const Icon = item.icon}
          {@const isActive = activeView === item.id}
          <button
            class="bottom-nav-pill"
            class:active={isActive}
            type="button"
            aria-current={isActive ? "page" : undefined}
            onclick={() => setView(item.id)}
          >
            <div class="tab-icon-wrap">
              <Icon size={20} strokeWidth={isActive ? 2.4 : 1.9} aria-hidden="true" />
              {#if item.id === "grades" && unreadGradeAlerts.length > 0}
                <span class="dock-badge">{unreadGradeAlerts.length}</span>
              {/if}
            </div>
            <span>{item.label}</span>
          </button>
        {/each}

        <!-- Same single door as the rail's avatar: the ellipsis tab it replaces
             pointed at this very view. -->
        <button
          class="bottom-nav-pill"
          class:active={activeView === "more"}
          type="button"
          aria-current={activeView === "more" ? "page" : undefined}
          onclick={() => setView("more")}
        >
          <div class="tab-icon-wrap">
            <UserRound size={20} strokeWidth={activeView === "more" ? 2.4 : 1.9} aria-hidden="true" />
          </div>
          <span>{copy.navAccount}</span>
        </button>
      </div>
    </nav>

    <PullToRefresh onRefresh={handleGlobalRefresh} scrollElement={viewportElement}>
      <div class="main-viewport" id="app-main-content" tabindex="-1">
        {#if credentialsWarning}
          <div class="credentials-warning-banner" role="status">
            <AlertCircle size={18} aria-hidden="true" />
            <span>{copy.credentialsWarning}</span>
          </div>
        {/if}

        {#if unreadGradeAlerts.length > 0 && activeView === "today"}
          <button class="new-grades-banner" type="button" onclick={openGradeAlerts}>
            <BookOpenCheck size={19} aria-hidden="true" />
            <span>{m.new_grades_banner({ count: unreadGradeAlerts.length })}</span>
          </button>
        {/if}

        {#if !backendAvailable}
          <!-- No Rust side means no portal session and no data. Naming that is the
               only honest thing this shell can render. -->
          <div class="shell-state-slot">
            <StateCard
              kind="error"
              icon={MonitorSmartphone}
              title={copy.desktopRequired}
            />
          </div>
        {:else if activeView === "today"}
          <div class="view-fade-enter">
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
          <div class="view-fade-enter">
            {#if schedule.kind === "error"}
              <!-- A dead network on this machine is not the portal being down, and
                   the recovery differs, so the two are never merged. -->
              <div class="shell-state-slot">
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
          <div class="portal-view-wrapper view-fade-enter">
            {#if GradesView}
              <GradesView {locale} {onLogout} bind:refresh={gradesRefresh} />
            {:else}
              <PageShell>
                <AcademicViewSkeleton ariaLabel={copy.lazyLoading} heroLabel={copy.navGrades} />
              </PageShell>
            {/if}
          </div>
        {:else if activeView === "absences"}
          <div class="portal-view-wrapper view-fade-enter">
            {#if AbsencesView}
              <AbsencesView {locale} {onLogout} bind:refresh={absencesRefresh} />
            {:else}
              <PageShell>
                <AbsencesViewSkeleton ariaLabel={copy.lazyLoading} heroLabel={copy.navAbsences} />
              </PageShell>
            {/if}
          </div>
        {:else if activeView === "more"}
          <div class="portal-view-wrapper view-fade-enter">
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
  .app-shell {
    position: relative;
    display: flex;
    flex-direction: row;
    width: 100%;
    height: 100%;
    max-height: 100%;
    overflow: hidden;
    box-sizing: border-box;
    color: var(--foreground);
    background: var(--background);
  }

  .skip-to-content {
    position: absolute;
    top: var(--space-2);
    left: var(--space-2);
    z-index: var(--z-overlay);
    padding: var(--space-2) var(--space-4);
    color: var(--secondary-foreground);
    background: var(--secondary);
    border-radius: var(--radius-md);
    font-size: var(--text-sm);
    font-weight: var(--weight-semibold);
    text-decoration: none;
    transform: translateY(-250%);
    transition: transform var(--duration-fast) var(--ease-out);
  }

  .skip-to-content:focus-visible {
    transform: translateY(0);
  }

  .main-viewport:focus-visible {
    outline: none;
  }

  /* ---------------- Desktop Navigation Rail ---------------- */
  /* The rail is the only desktop shape: fixed-width icons, each name served on
     hover rather than by a panel that stays expanded. */
  .desktop-app-sidebar {
    display: none;
    flex-direction: column;
    align-items: center;
    width: 3.85rem;
    min-width: 3.85rem;
    max-width: 3.85rem;
    height: 100%;
    max-height: 100%;
    padding: var(--space-4) var(--space-1);
    gap: var(--space-5);
    box-sizing: border-box;
    z-index: var(--z-sidebar);
    background: var(--card);
    border-right: 1px solid var(--border-subtle);
    /* Tooltips hang outside the rail's box, so nothing here may clip. */
    overflow: visible;
  }

  .sidebar-brand-box {
    display: flex;
    justify-content: center;
    flex-shrink: 0;
  }

  .brand-click-wrap {
    display: grid;
    width: 2.85rem;
    height: 2.85rem;
    place-items: center;
    padding: 0;
    background: transparent;
    border: 0;
    border-radius: var(--radius-md);
    color: inherit;
    transition:
      background-color var(--duration-fast) var(--ease-out),
      transform var(--duration-instant) var(--ease-out);
  }

  .brand-click-wrap:active {
    transform: scale(var(--press-scale));
  }

  .sidebar-nav-list {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--space-1);
    flex: 1;
    min-height: 0;
  }

  .sidebar-nav-btn {
    position: relative;
    display: grid;
    width: 2.85rem;
    height: 2.85rem;
    place-items: center;
    padding: var(--space-2);
    background: transparent;
    border: 0;
    border-radius: var(--radius-md);
    color: var(--muted-foreground);
    transition:
      background-color var(--duration-fast) var(--ease-out),
      color var(--duration-fast) var(--ease-out),
      transform var(--duration-instant) var(--ease-out);
  }

  .sidebar-nav-btn:active {
    transform: scale(var(--press-scale));
  }

  .sidebar-nav-btn.active {
    background: var(--muted);
    color: var(--primary-deep);
  }

  .sidebar-icon-wrap {
    position: relative;
    display: grid;
    place-items: center;
    flex-shrink: 0;
  }

  .sidebar-badge {
    /* Optical nudge, not a spacing step: the badge has to hang off the icon's
       own box, and a 0.25rem step would clear the 19px glyph entirely. */
    position: absolute;
    top: -3px;
    right: -6px;
    display: grid;
    min-width: 0.85rem;
    height: 0.85rem;
    place-items: center;
    padding: 0 var(--space-1);
    color: var(--primary-foreground);
    background: var(--primary);
    border-radius: var(--radius-pill);
    font-size: var(--text-2xs);
    font-weight: var(--weight-heavy);
    font-variant-numeric: tabular-nums;
    /* Hairline ring so the badge reads as detached from the icon underneath. */
    border: 1.5px solid var(--card);
  }

  .sidebar-user-footer {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--space-1);
    width: 100%;
    padding-top: var(--space-3);
    border-top: 1px solid var(--border-subtle);
    flex-shrink: 0;
  }

  .user-avatar-circle {
    position: relative;
    display: grid;
    width: var(--tap-min);
    height: var(--tap-min);
    flex-shrink: 0;
    place-items: center;
    padding: 0;
    color: var(--card);
    background: var(--primary-deep);
    border: 0;
    border-radius: 50%;
    /* The ring is the rail's only room for an active marker on a round target. */
    box-shadow: 0 0 0 0 transparent;
    transition:
      background-color var(--duration-fast) var(--ease-out),
      box-shadow var(--duration-fast) var(--ease-out),
      transform var(--duration-instant) var(--ease-out);
  }

  .user-avatar-circle:active {
    transform: scale(var(--press-scale));
  }

  .user-avatar-circle.active {
    box-shadow: 0 0 0 3px var(--primary);
  }

  /* ---------------- Rail Hover Labels ---------------- */
  .rail-tip-anchor {
    position: relative;
  }

  .rail-tooltip {
    position: absolute;
    top: 50%;
    left: calc(100% + var(--space-2));
    z-index: var(--z-raised);
    display: flex;
    flex-direction: column;
    padding: var(--space-2) var(--space-3);
    color: var(--card);
    background: var(--primary-deep);
    border-radius: var(--radius-sm);
    box-shadow: var(--shadow-md);
    text-align: left;
    white-space: nowrap;
    opacity: 0;
    transform: translate(-0.25rem, -50%);
    pointer-events: none;
    transition:
      opacity var(--duration-fast) var(--ease-out),
      transform var(--duration-fast) var(--ease-out);
  }

  /* Rotated square rather than a border triangle: it inherits the bubble's own
     background, so a theme change cannot desync the two. */
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

  .rail-tooltip-title {
    font-size: var(--text-xs);
    font-weight: var(--weight-bold);
    line-height: 1.3;
  }

  .rail-tooltip-sub {
    font-size: var(--text-2xs);
    font-weight: var(--weight-semibold);
    line-height: 1.3;
    opacity: 0.72;
  }

  .rail-tip-anchor:focus-visible .rail-tooltip {
    opacity: 1;
    transform: translate(0, -50%);
  }

  /* ---------------- Main Content Viewport ---------------- */
  .main-content-viewport {
    flex: 1 1 0%;
    display: flex;
    flex-direction: column;
    min-width: 0;
    height: 100%;
    max-height: 100%;
    overflow-y: auto;
    overflow-x: hidden;
    overscroll-behavior-y: contain;
    box-sizing: border-box;
  }

  .main-viewport {
    flex: 1;
    display: flex;
    flex-direction: column;
    width: 100%;
    box-sizing: border-box;
  }

  .view-fade-enter {
    animation: fade-in var(--duration-fast) var(--ease-out) forwards;
    width: 100%;
    display: flex;
    flex-direction: column;
    flex: 1;
  }

  .credentials-warning-banner {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    margin: var(--space-4) var(--space-5) 0;
    padding: var(--space-3) var(--space-4);
    color: var(--warning-strong);
    background: var(--warning-surface);
    border: 1px solid var(--warning);
    border-radius: var(--radius-md);
    font-size: var(--text-base);
    font-weight: var(--weight-semibold);
  }

  .new-grades-banner {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    min-height: var(--tap-min);
    margin: var(--space-4) var(--space-5) 0;
    padding: var(--space-3) var(--space-4);
    color: var(--primary-deep);
    background: var(--muted);
    border: 1px solid var(--muted-strong);
    border-radius: var(--radius-md);
    font-size: var(--text-base);
    font-weight: var(--weight-bold);
    text-align: left;
    transition:
      background-color var(--duration-fast) var(--ease-out),
      transform var(--duration-instant) var(--ease-out);
  }

  .new-grades-banner:active {
    transform: scale(var(--press-scale));
  }

  .shell-state-slot {
    margin: var(--space-8) var(--space-5);
  }

  .portal-view-wrapper {
    flex: 1 1 auto;
    display: flex;
    flex-direction: column;
    width: 100%;
    min-width: 0;
  }

  /* ---------------- Mobile Bottom Nav (Floating Dock) ---------------- */
  .bottom-nav {
    position: fixed;
    right: 0;
    bottom: 0;
    left: 0;
    z-index: var(--z-nav);
    display: flex;
    justify-content: center;
    padding: var(--space-1) var(--space-3) max(var(--space-2), env(safe-area-inset-bottom));
    pointer-events: none;
  }

  /* A dock that genuinely floats above the content with frosted glass effect */
  .bottom-nav-container {
    display: grid;
    grid-template-columns: repeat(5, minmax(0, 1fr));
    width: min(100%, 32rem);
    gap: var(--space-1);
    padding: 0.3rem var(--space-1);
    background: color-mix(in oklch, var(--card) 94%, transparent);
    backdrop-filter: blur(20px);
    -webkit-backdrop-filter: blur(20px);
    border: 1px solid color-mix(in oklch, var(--border-subtle) 80%, transparent);
    border-radius: var(--radius-xl);
    box-shadow: var(--shadow-lg);
    pointer-events: auto;
  }

  .bottom-nav-pill {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 0.15rem;
    min-height: 2.75rem;
    padding: 0.25rem 0.15rem;
    background: transparent;
    border: 0;
    border-radius: var(--radius-lg);
    color: var(--muted-foreground);
    transition:
      background-color var(--duration-fast) var(--ease-out),
      color var(--duration-fast) var(--ease-out),
      transform var(--duration-instant) var(--ease-out);
  }

  .bottom-nav-pill:active {
    transform: scale(var(--press-scale));
  }

  .bottom-nav-pill.active {
    background: var(--muted);
    color: var(--primary-deep);
  }

  .bottom-nav-pill.active span {
    font-weight: var(--weight-heavy);
  }

  .bottom-nav-pill span {
    font-size: var(--text-2xs);
    font-weight: var(--weight-bold);
    letter-spacing: -0.01em;
    line-height: 1.1;
  }

  .tab-icon-wrap {
    position: relative;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .dock-badge {
    /* Optical nudge so badge sits cleanly at top right of icon glyph */
    position: absolute;
    top: -3px;
    right: -7px;
    min-width: 0.95rem;
    height: 0.95rem;
    padding: 0 var(--space-1);
    display: grid;
    place-items: center;
    color: var(--primary-foreground);
    background: var(--primary);
    border-radius: var(--radius-pill);
    font-size: var(--text-2xs);
    font-weight: var(--weight-heavy);
    font-variant-numeric: tabular-nums;
    border: 1.5px solid var(--card);
  }

  @media (hover: hover) and (pointer: fine) {
    /* The rail has no room for a written label, so hover is where the name
       lives — the only place it is ever shown. */
    .rail-tip-anchor:hover .rail-tooltip {
      opacity: 1;
      transform: translate(0, -50%);
    }

    .brand-click-wrap:hover {
      background: var(--muted);
      color: var(--primary-deep);
    }

    .sidebar-nav-btn:hover {
      background: var(--muted);
      color: var(--primary-deep);
    }

    .user-avatar-circle:hover {
      background: var(--primary-deep-hover);
    }

    .new-grades-banner:hover {
      background: var(--muted-strong);
    }
  }

  /* =========================================================================
     RESPONSIVE LAYOUT MODES
     ========================================================================= */

  /* Compact windows: dock navigation, no sidebar. */
  .desktop-app-sidebar {
    display: none;
  }

  .bottom-nav {
    display: flex;
  }

  .main-viewport {
    padding-bottom: calc(4.5rem + env(safe-area-inset-bottom));
  }

  @media (min-width: 48rem) {
    .desktop-app-sidebar {
      display: flex;
    }

    .bottom-nav {
      display: none;
    }

    .main-viewport {
      padding-bottom: 0;
    }
  }

  /* TitleBar stamps `mobile-app` / `desktop-app` on the root from the platform,
     not the window width, and that decision has to win over the width query
     above — `mobile:dev` runs the mobile shell in a wide desktop window. These
     selectors already carry one class more than the query's, and a media query
     adds no specificity, so they win without `!important`. */
  :global(html.mobile-app) .desktop-app-sidebar {
    display: none;
  }

  :global(html.mobile-app) .bottom-nav {
    display: flex;
  }

  :global(html.mobile-app) .main-viewport {
    padding-bottom: calc(4.5rem + env(safe-area-inset-bottom));
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
</style>
