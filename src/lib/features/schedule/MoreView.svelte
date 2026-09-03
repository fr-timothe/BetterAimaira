<script lang="ts">
  import { onMount, untrack } from 'svelte';
  import {
    AlertCircle,
    BarChart3,
    CloudOff,
    Download,
    FileCheck,
    FileSpreadsheet,
    FileText,
    GraduationCap,
    IdCard,
    Info,
    Languages,
    LogOut,
    MapPin,
    RefreshCw,
    School,
    ShieldCheck,
    Table2,
  } from 'lucide-svelte';
  import Badge from '$lib/components/ui/Badge.svelte';
  import Button from '$lib/components/ui/Button.svelte';
  import Card from '$lib/components/ui/Card.svelte';
  import IconButton from '$lib/components/ui/IconButton.svelte';
  import PageShell from '$lib/components/ui/PageShell.svelte';
  import SectionHeader from '$lib/components/ui/SectionHeader.svelte';
  import SegmentedControl from '$lib/components/ui/SegmentedControl.svelte';
  import SessionExpiredCard from '$lib/components/ui/SessionExpiredCard.svelte';
  import Sheet from '$lib/components/ui/Sheet.svelte';
  import Skeleton from '$lib/components/ui/Skeleton.svelte';
  import StateCard from '$lib/components/ui/StateCard.svelte';
  import Switch from '$lib/components/ui/Switch.svelte';
  import type { IconComponent } from '$lib/components/ui/icon';
  import * as m from '$lib/paraglide/messages.js';
  import type { Locale } from '$lib/paraglide/runtime.js';
  import { connectivity } from '$lib/state/connectivity.svelte';
  import type { AnalyticsStatus } from '$lib/features/analytics/analytics-service';
  import { analyticsStatus, setAnalyticsConsent } from '$lib/features/analytics/analytics-service';
  import UpdateCard from '$lib/features/updates/UpdateCard.svelte';
  import AccountViewSkeleton from './AccountViewSkeleton.svelte';
  import { loadPortalResource } from './portal-cache';
  import QuestionnairesView from './QuestionnairesView.svelte';
  import { cn } from '$lib/utils';
  import {
    documentKindLabel,
    downloadPortalDocument,
    getPortalHost,
    parseResourceError,
    resourceErrorMessage,
  } from './portal-utils';
  import type {
    MoreSection,
    PortalDocument,
    PortalDocumentKind,
    PortalField,
    PortalResourceErrorCode,
    PortalResourceState,
    PortalTable,
  } from './types';

  type Props = {
    username: string;
    portalUrl: string;
    locale: Locale;
    onLocaleChange: (locale: Locale) => Promise<void>;
    onLogout: () => Promise<void>;
    refresh?: () => Promise<void>;
  };

  let { username, portalUrl, locale, onLocaleChange, onLogout, refresh = $bindable() }: Props = $props();

  $effect(() => {
    refresh = async () => {
      if (activeTab === 'profile') {
        await loadProfile(true);
      } else if (activeTab === 'documents') {
        await loadDocuments(true);
      } else if (activeTab === 'questionnaires' && questionnairesRefresh) {
        await questionnairesRefresh();
      }
    };
  });

  /** Stamped by Vite from `tauri.conf.json` at build time; see `vite.config.js`. */
  const appVersion = __APP_VERSION__;

  let activeTab = $state<MoreSection>('profile');
  let profileState = $state<PortalResourceState>({ kind: 'loading' });
  let documentsState = $state<PortalResourceState>({ kind: 'loading' });
  let downloadingPath = $state<string | null>(null);
  let downloadError = $state(false);
  /** Where the last document landed, so the row can point at it. */
  let downloadedPath = $state<string | null>(null);
  let changingLocale = $state(false);
  /**
   * Null until the Rust side answers. Consent is never mirrored optimistically:
   * the switch shows what the store confirmed, so a write that fails cannot
   * leave a reader believing reporting stopped when it did not.
   */
  let analytics = $state<AnalyticsStatus | null>(null);
  let savingConsent = $state(false);
  let consentFailed = $state(false);
  let isLoggingOut = $state(false);
  let logoutDialogOpen = $state(false);
  let questionnairesRefresh = $state<(() => Promise<void>) | undefined>();

  let profileSeq = 0;
  let documentsSeq = 0;
  let profileRefreshing = $state(false);
  let documentsRefreshing = $state(false);

  /**
   * A refresh that failed while data was already on screen used to be swallowed:
   * the icon spun, stopped, and the stale data stayed unmarked. These flags are
   * what `FreshnessLabel` reads to say so.
   */
  let profileFailed = $state(false);
  let documentsFailed = $state(false);

  const copy = $derived.by(() => {
    return {
      tabsLabel: m.more_heading(),
      profileTab: m.profile_tab(),
      documentsTab: m.documents_tab(),
      questionnairesTab: m.questionnaires_tab(),
      languageLabel: m.language_label(),
      languageFr: m.language_fr(),
      languageEn: m.language_en(),
      logout: m.logout(),
      logoutConfirmTitle: m.logout_confirm_title(),
      logoutConfirmDescription: m.logout_confirm_description(),
      cancel: m.cancel(),
      close: m.close(),
      loading: m.resource_loading(),
      refresh: m.resource_refresh(),
      errorHeading: m.resource_error_heading(),
      retry: m.resource_retry(),
      unknownHeading: m.resource_unknown_heading(),
      unknownDescription: m.resource_unknown_description(),
      emptyHeading: m.resource_empty_heading(),
      emptyDescription: m.resource_empty_description(),
      offline: m.sync_offline(),
      offlineDescription: m.sync_offline_description(),
      download: m.download_document(),
      downloading: m.downloading_document(),
      downloadError: m.document_download_error(),
      downloadSaved: (path: string) => m.document_saved_to({ path }),
      privacyTitle: m.privacy_section_title(),
      analyticsLabel: m.onboarding_analytics_accept(),
      analyticsDescription: m.onboarding_analytics_description(),
      analyticsFootnote: m.onboarding_analytics_footnote(),
      analyticsError: m.onboarding_analytics_error(),
      aboutTitle: m.about_section_title(),
      versionLabel: m.about_version_label(),
    };
  });

  const tabOptions = $derived([
    { value: 'profile', label: copy.profileTab },
    { value: 'documents', label: copy.documentsTab },
    { value: 'questionnaires', label: copy.questionnairesTab },
  ]);

  function handleTabChange(value: string) {
    if (value === 'profile' || value === 'documents' || value === 'questionnaires') {
      activeTab = value;
    }
  }

  $effect(() => {
    const currentTab = activeTab;
    untrack(() => {
      if (currentTab === 'profile' && profileState.kind === 'loading') {
        void loadProfile();
      } else if (currentTab === 'documents' && documentsState.kind === 'loading') {
        void loadDocuments();
      }
    });
  });

  async function loadProfile(force = false) {
    const seq = ++profileSeq;
    const hasData = profileState.kind === 'ready';
    if (hasData) {
      profileRefreshing = true;
    } else if (profileState.kind !== 'loading') {
      profileState = { kind: 'loading' };
    }
    try {
      const page = await loadPortalResource('profile', force);
      if (seq !== profileSeq) return;
      profileState = { kind: 'ready', page };
      profileFailed = false;
    } catch (error) {
      if (seq !== profileSeq) return;
      const code = parseResourceError(error, 'profile_unavailable');
      // An expired session cannot be refreshed away, so it replaces the stale
      // data rather than being flagged on top of it: the action is sign-in.
      if (!hasData || code === 'session_expired') {
        profileState = { kind: 'error', code };
        profileFailed = false;
      } else {
        profileFailed = true;
      }
    } finally {
      if (seq === profileSeq) profileRefreshing = false;
    }
  }

  async function loadDocuments(force = false) {
    const seq = ++documentsSeq;
    const hasData = documentsState.kind === 'ready';
    if (hasData) {
      documentsRefreshing = true;
    } else if (documentsState.kind !== 'loading') {
      documentsState = { kind: 'loading' };
    }
    downloadError = false;
    try {
      const page = await loadPortalResource('documents', force);
      if (seq !== documentsSeq) return;
      documentsState = { kind: 'ready', page };
      documentsFailed = false;
    } catch (error) {
      if (seq !== documentsSeq) return;
      const code = parseResourceError(error, 'documents_unavailable');
      if (!hasData || code === 'session_expired') {
        documentsState = { kind: 'error', code };
        documentsFailed = false;
      } else {
        documentsFailed = true;
      }
    } finally {
      if (seq === documentsSeq) documentsRefreshing = false;
    }
  }

  async function downloadDocument(document: PortalDocument) {
    downloadingPath = document.requestPath;
    downloadError = false;
    downloadedPath = null;
    try {
      const result = await downloadPortalDocument(document);
      downloadedPath = result.path;
    } catch {
      downloadError = true;
    } finally {
      downloadingPath = null;
    }
  }

  /**
   * The reporting answer lives on the Rust side, so the row has nothing honest
   * to show until it replies. A build that cannot report — the browser preview
   * among them — answers `available: false` and the card never appears.
   */
  onMount(() => {
    void analyticsStatus()
      .then((status) => {
        analytics = status;
      })
      .catch(() => {
        // Unreadable consent is not a setting the reader can act on, so the
        // row stays hidden rather than offering a switch that writes nowhere.
        analytics = null;
      });
  });

  async function handleConsentChange(enabled: boolean) {
    if (savingConsent) return;
    savingConsent = true;
    consentFailed = false;
    try {
      analytics = await setAnalyticsConsent(enabled);
    } catch {
      consentFailed = true;
    } finally {
      savingConsent = false;
    }
  }

  async function handleLocaleSwitch(newLocale: Locale) {
    if (newLocale === locale || changingLocale) return;
    changingLocale = true;
    try {
      await onLocaleChange(newLocale);
    } finally {
      changingLocale = false;
    }
  }

  function handleLocaleSelect(event: Event) {
    const select = event.currentTarget;
    if (!(select instanceof HTMLSelectElement)) return;

    if (select.value === 'fr' || select.value === 'en') {
      void handleLocaleSwitch(select.value);
    }
  }

  async function handleLogoutClick() {
    if (isLoggingOut) return;
    isLoggingOut = true;
    try {
      await onLogout();
    } finally {
      isLoggingOut = false;
    }
  }

  const studentProfile = $derived.by(() => {
    // The field category names below come from the message catalogue, and
    // Paraglide message functions are not reactive: this keeps the dependency.
    if (profileState.kind !== 'ready') {
      return {
        fullName: username,
        campus: null,
        categorizedFields: [] as Array<{
          category: string;
          icon: IconComponent;
          fields: PortalField[];
        }>,
        tables: [] as PortalTable[],
      };
    }

    const fields = profileState.page.fields ?? [];
    const findField = (regex: RegExp) => {
      const match = fields.find((f) => regex.test(f.label) && f.value.trim().length > 0);
      return match ? match.value.trim() : null;
    };

    const nameField = /^(nom\s*complet|nom\s*et\s*pr[ée]nom|pr[ée]nom\s*et\s*nom|nom|identité)$/i;
    const schoolField = /^(campus|site|[ée]tablissement|centre|ville)$/i;
    const nameFromField = findField(nameField);
    const fullName = nameFromField || (profileState.page.title.trim() ? profileState.page.title.trim() : username);
    const campus = findField(schoolField);

    const personalFields: PortalField[] = [];
    const academicFields: PortalField[] = [];
    const contactFields: PortalField[] = [];
    const generalFields: PortalField[] = [];

    for (const field of fields) {
      const label = field.label.trim();
      const val = field.value.trim();
      if (!val || val === '-') continue;
      if (nameField.test(label) || schoolField.test(label)) continue;

      if (/date\s*de\s*naissance|lieu\s*de\s*naissance|nationalit[ée]|sexe|genre|civilit[ée]/i.test(label)) {
        personalFields.push(field);
      } else if (/formation|fili[èe]re|promotion|groupe|options?|sp[ée]cialit[ée]|niveau|ann[ée]e|dipl[ôo]me|cursus|semestre/i.test(label)) {
        academicFields.push(field);
      } else if (/adresse|code\s*postal|ville|pays|t[ée]l[ée]phone|portable|courriel|mail|contact/i.test(label)) {
        contactFields.push(field);
      } else {
        generalFields.push(field);
      }
    }

    const categorizedFields: Array<{ category: string; icon: IconComponent; fields: PortalField[] }> = [];

    if (academicFields.length > 0) {
      categorizedFields.push({
        category: m.profile_category_academic(),
        icon: GraduationCap,
        fields: academicFields,
      });
    }
    if (personalFields.length > 0) {
      categorizedFields.push({
        category: m.profile_category_identity(),
        icon: IdCard,
        fields: personalFields,
      });
    }
    if (contactFields.length > 0) {
      categorizedFields.push({
        category: m.profile_category_contact(),
        icon: MapPin,
        fields: contactFields,
      });
    }
    if (generalFields.length > 0) {
      categorizedFields.push({
        category: m.profile_category_other(),
        icon: Info,
        fields: generalFields,
      });
    }

    return {
      fullName,
      campus,
      categorizedFields,
      tables: profileState.page.tables ?? [],
    };
  });

  function documentIcon(kind: PortalDocumentKind): IconComponent {
    if (kind === 'schoolCertificate' || kind === 'enrollmentCertificate') return FileCheck;
    if (kind === 'gradeBulletin' || kind === 'gradeTranscript' || kind === 'gradeReport') {
      return FileSpreadsheet;
    }
    return FileText;
  }

  function documentTone(kind: PortalDocumentKind): 'accent' | 'success' | 'neutral' {
    if (kind === 'schoolCertificate' || kind === 'enrollmentCertificate') return 'success';
    if (kind === 'gradeBulletin' || kind === 'gradeTranscript' || kind === 'gradeReport') {
      return 'accent';
    }
    return 'neutral';
  }

  const documentCountLabel = $derived.by(() => {
    return documentsState.kind === 'ready'
      ? m.documents_pdf_count({ count: documentsState.page.documents.length })
      : '';
  });

  const schoolName = $derived(studentProfile.campus || getPortalHost(portalUrl));
  const tabPanel = 'flex min-w-0 flex-col gap-4 animate-fade-in';
  // Panels are `Card`; this only supplies the inner rhythm.
  const panelBody = 'flex min-w-0 flex-col gap-4';
  const documentsList =
    'grid grid-cols-1 gap-3 md:grid-cols-[repeat(auto-fill,minmax(min(100%,21rem),1fr))]';
  const documentCard =
    'flex min-w-0 items-center gap-3 rounded-lg bg-surface-sunken p-3' +
    ' transition-colors duration-fast ease-out';

  const documentTones = {
    accent: 'bg-muted text-primary-deep',
    success: 'bg-success-surface text-success-strong',
    neutral: 'bg-card text-muted-foreground'
  } as const;
</script>

{#snippet resourceError(code: PortalResourceErrorCode, retry: () => void)}
  {#if code === 'session_expired'}
    <SessionExpiredCard onRetry={retry} {onLogout} {locale} />
  {:else}
    <StateCard
      kind="error"
      title={copy.errorHeading}
      description={resourceErrorMessage(code)}
      icon={AlertCircle}
      actionLabel={copy.retry}
      onAction={retry}
    />
  {/if}
{/snippet}

{#snippet offlineState(retry: () => void)}
  <!-- A device without a network path is not a portal outage, and must not be
       reported as one. -->
  <StateCard
    kind="error"
    title={copy.offline}
    description={copy.offlineDescription}
    icon={CloudOff}
    actionLabel={copy.retry}
    onAction={retry}
  />
{/snippet}

{#snippet syncToolbar(fetchedAt: number, refreshing: boolean, failed: boolean, refresh: () => void)}
  <!-- `desktop-only` owns the display here; adding a display utility would
       lose to its !important rules without a word. -->
  <header class="desktop-only flex-wrap items-center justify-end gap-3">
    <IconButton label={copy.refresh} loading={refreshing} onclick={refresh}>
      <RefreshCw size={18} aria-hidden="true" />
    </IconButton>
  </header>
{/snippet}

<PageShell>
  <SegmentedControl
    options={tabOptions}
    value={activeTab}
    label={copy.tabsLabel}
    onChange={handleTabChange}
  />

  {#if activeTab === 'profile'}
    <div class={tabPanel} role="tabpanel" aria-label={copy.profileTab}>
      {#if profileState.kind === 'loading'}
        <AccountViewSkeleton ariaLabel={copy.loading} />
      {:else if profileState.kind === 'error' && !connectivity.online}
        {@render offlineState(() => void loadProfile(true))}
      {:else if profileState.kind === 'error'}
        {@render resourceError(profileState.code, () => void loadProfile(true))}
      {:else}
        {@render syncToolbar(
          profileState.page.fetchedAt,
          profileRefreshing,
          profileFailed,
          () => void loadProfile(true)
        )}

        <article
          class="flex flex-col gap-2 rounded-xl bg-muted px-4 py-5 md:px-5 md:py-6"
        >
          <h2
            class="text-xl leading-[1.2] font-extrabold tracking-[-0.02em] wrap-anywhere
                   text-foreground">{studentProfile.fullName}</h2
          >
          <p
            class="flex min-w-0 items-center gap-2 text-base font-semibold text-primary-deep
                   [&>svg]:flex-none"
          >
            <School size={17} aria-hidden="true" />
            <span class="wrap-anywhere">{schoolName}</span>
          </p>
        </article>

        {#if studentProfile.categorizedFields.length === 0 && studentProfile.tables.length === 0}
          <StateCard
            kind="empty"
            icon={Info}
            title={profileState.page.markupRecognized ? copy.emptyHeading : copy.unknownHeading}
            description={profileState.page.markupRecognized
              ? copy.emptyDescription
              : copy.unknownDescription}
          />
        {:else}
          {#if studentProfile.categorizedFields.length > 0}
            <div
              class="grid grid-cols-1 gap-4
                     md:grid-cols-[repeat(auto-fill,minmax(min(100%,20rem),1fr))]"
            >
              {#each studentProfile.categorizedFields as section (section.category)}
                <Card>
                  <div class={panelBody}>
                    <SectionHeader icon={section.icon} title={section.category} level={3} />

                    <dl
                      class="flex flex-col gap-1
                             md:grid md:grid-cols-[repeat(auto-fill,minmax(min(100%,13rem),1fr))]
                             md:gap-2"
                    >
                      {#each section.fields as field (`${field.label}:${field.value}`)}
                        <div
                          class="flex flex-col gap-[0.15rem] rounded-md bg-surface-sunken px-3 py-2"
                        >
                          <dt class="text-xs font-bold text-muted-foreground">{field.label}</dt>
                          <dd
                            class="text-base font-medium tabular-nums wrap-anywhere text-foreground"
                          >{field.value}</dd>
                        </div>
                      {/each}
                    </dl>
                  </div>
                </Card>
              {/each}
            </div>
          {/if}

          {#each studentProfile.tables as table, tableIndex (`${table.context.join(':')}:${table.caption ?? tableIndex}`)}
            <Card padding="none">
              <div class={cn(panelBody, 'p-4 md:px-0 md:pt-5 md:pb-0')}>
                <SectionHeader
                  class="md:px-5"
                  icon={Table2}
                  title={table.caption || table.context.at(-1) || m.profile_table_fallback()}
                  subtitle={m.profile_table_row_count({ count: table.rows.length })}
                  level={3}
                />

                <div class="overflow-x-auto [-webkit-overflow-scrolling:touch]">
                  <table class="portal-table">
                    {#if table.headers.length > 0}
                      <thead>
                        <tr>
                          {#each table.headers as header, headerIndex (headerIndex)}
                            <th scope="col">{header}</th>
                          {/each}
                        </tr>
                      </thead>
                    {/if}
                    <tbody>
                      {#each table.rows as row, rowIndex (`${tableIndex}:${rowIndex}`)}
                        <tr>
                          {#each row as cell, cellIndex (cellIndex)}
                            <td>
                              {#if table.headers[cellIndex]}
                                <span class="cell-label">{table.headers[cellIndex]}</span>
                              {/if}
                              <span class="font-medium tabular-nums wrap-anywhere text-foreground">{cell}</span>
                            </td>
                          {/each}
                        </tr>
                      {/each}
                    </tbody>
                  </table>
                </div>
              </div>
            </Card>
          {/each}
        {/if}
      {/if}

      <Card>
        <div
          class="flex min-w-0 flex-col items-center justify-between gap-4 min-[30rem]:flex-row"
        >
          <div class="flex w-full min-w-0 items-center gap-2 min-[30rem]:w-auto">
            <span
              class="grid size-8 flex-none place-items-center rounded-sm bg-muted text-primary-deep"
              aria-hidden="true"><Languages size={18} /></span
            >
            <label class="text-base font-bold" for="profile-language">{copy.languageLabel}</label>
          </div>

          <select
            class="min-h-(--tap-min) w-full rounded-md border border-border-subtle
                   bg-surface-sunken pr-8 pl-3 font-semibold text-foreground
                   disabled:opacity-62 min-[30rem]:w-auto min-[30rem]:min-w-[9.5rem]"
            id="profile-language"
            value={locale}
            disabled={changingLocale}
            aria-busy={changingLocale ? 'true' : undefined}
            onchange={handleLocaleSelect}
          >
            <option value="fr">{copy.languageFr}</option>
            <option value="en">{copy.languageEn}</option>
          </select>
        </div>
      </Card>

      {#if analytics?.available}
        <Card>
          <div class="flex min-w-0 flex-col gap-3">
            <SectionHeader icon={ShieldCheck} title={copy.privacyTitle} level={3} />

            <div
              class="flex min-w-0 flex-col items-center justify-between gap-4 min-[30rem]:flex-row"
            >
              <div class="flex w-full min-w-0 items-center gap-2 min-[30rem]:w-auto">
                <span
                  class="grid size-8 flex-none place-items-center rounded-sm bg-muted text-primary-deep"
                  aria-hidden="true"><BarChart3 size={18} /></span
                >
                <label class="text-base font-bold" for="profile-analytics"
                  >{copy.analyticsLabel}</label
                >
              </div>

              <Switch
                id="profile-analytics"
                checked={analytics.enabled}
                busy={savingConsent}
                onChange={(enabled) => void handleConsentChange(enabled)}
              />
            </div>

            <p class="text-sm leading-[1.5] text-pretty text-muted-foreground">
              {copy.analyticsDescription}
            </p>
            <p class="text-xs leading-[1.5] text-pretty text-muted-foreground">
              {copy.analyticsFootnote}
            </p>

            {#if consentFailed}
              <p
                class="flex items-center gap-2 rounded-md bg-danger-surface px-3 py-2 text-sm
                       font-semibold text-danger-strong"
                role="alert"
              >
                <AlertCircle size={16} aria-hidden="true" />
                <span>{copy.analyticsError}</span>
              </p>
            {/if}
          </div>
        </Card>
      {/if}

      <UpdateCard {locale} />

      <Card>
        <div class="flex min-w-0 flex-col gap-3">
          <SectionHeader icon={Info} title={copy.aboutTitle} level={3} />

          <div class="flex min-w-0 flex-wrap items-center justify-between gap-2">
            <span class="text-base font-semibold text-foreground">{copy.versionLabel}</span>
            <span class="font-semibold tabular-nums wrap-anywhere text-muted-foreground select-text"
              >{appVersion}</span
            >
          </div>
        </div>
      </Card>

      <div class="mt-4 border-t border-border-subtle pt-4">
        <Button variant="danger" size="lg" block onclick={() => (logoutDialogOpen = true)}>
          <LogOut size={18} aria-hidden="true" />
          <span>{copy.logout}</span>
        </Button>
      </div>
    </div>
  {:else if activeTab === 'documents'}
    <div class={tabPanel} role="tabpanel" aria-label={copy.documentsTab}>
      {#if documentsState.kind === 'loading'}
        <div role="status" aria-live="polite" aria-busy="true" aria-label={copy.loading}>
          <Card>
            <div class={panelBody}>
              <SectionHeader icon={FileText} title={copy.documentsTab} level={3} />

              <div class={documentsList}>
                {#each Array(3) as _, index (index)}
                  <div class={documentCard}>
                    <Skeleton shape="circle" width="2.75rem" height="2.75rem" />
                    <div class="flex min-w-0 flex-1 flex-col gap-2">
                      <div class="flex items-center gap-2">
                        <Skeleton shape="block" width="4.5rem" height="1.45rem" />
                        <Skeleton shape="block" width="2.5rem" height="1.45rem" />
                      </div>
                      <Skeleton shape="title" width={index === 1 ? '64%' : '78%'} />
                      <Skeleton shape="text" width="48%" />
                    </div>
                    <Skeleton shape="circle" width="2.75rem" height="2.75rem" />
                  </div>
                {/each}
              </div>
            </div>
          </Card>
        </div>
      {:else if documentsState.kind === 'error' && !connectivity.online}
        {@render offlineState(() => void loadDocuments(true))}
      {:else if documentsState.kind === 'error'}
        {@render resourceError(documentsState.code, () => void loadDocuments(true))}
      {:else}
        {@render syncToolbar(
          documentsState.page.fetchedAt,
          documentsRefreshing,
          documentsFailed,
          () => void loadDocuments(true)
        )}

        {#if downloadError}
          <p
            class="flex items-center gap-2 rounded-md bg-danger-surface px-3 py-2 text-sm
                   font-semibold text-danger-strong"
            role="alert"
          >
            <AlertCircle size={16} aria-hidden="true" />
            <span>{copy.downloadError}</span>
          </p>
        {:else if downloadedPath}
          <p
            class="flex items-center gap-2 rounded-md bg-surface-sunken px-3 py-2 text-sm
                   font-semibold text-muted-foreground"
            role="status"
          >
            <FileText size={16} aria-hidden="true" />
            <span class="break-all">{copy.downloadSaved(downloadedPath)}</span>
          </p>
        {/if}

        {#if documentsState.page.documents.length === 0}
          <StateCard
            kind="empty"
            icon={FileText}
            title={copy.emptyHeading}
            description={copy.emptyDescription}
          />
        {:else}
          <Card>
            <div class={panelBody}>
              <SectionHeader
                icon={FileText}
                title={copy.documentsTab}
                subtitle={documentCountLabel}
                level={3}
              />

              <div class={documentsList}>
                {#each documentsState.page.documents as doc (doc.requestPath)}
                  {@const DocIcon = documentIcon(doc.kind)}
                  {@const tone = documentTone(doc.kind)}
                  {@const busy = downloadingPath === doc.requestPath}
                  <article class={cn(documentCard, 'hover:bg-muted')}>
                    <span
                      class={cn(
                        'grid size-11 flex-none place-items-center rounded-md',
                        documentTones[tone]
                      )}
                      aria-hidden="true"
                    >
                      <DocIcon size={21} />
                    </span>

                    <div class="flex min-w-0 flex-1 flex-col gap-1">
                      <div class="flex flex-wrap gap-1">
                        <Badge {tone}>{documentKindLabel(doc.kind)}</Badge>
                        <Badge>PDF</Badge>
                      </div>
                      <h4 class="text-base leading-[1.3] font-bold wrap-anywhere text-foreground"
                        >{doc.label}</h4
                      >
                      {#if doc.suggestedFilename}
                        <p class="truncate text-xs text-muted-foreground"
                          >{doc.suggestedFilename}</p
                        >
                      {/if}
                    </div>

                    <IconButton
                      label={`${busy ? copy.downloading : copy.download} : ${doc.label}`}
                      loading={busy}
                      onclick={() => void downloadDocument(doc)}
                    >
                      <Download size={18} aria-hidden="true" />
                    </IconButton>
                  </article>
                {/each}
              </div>
            </div>
          </Card>
        {/if}
      {/if}
    </div>
  {:else if activeTab === 'questionnaires'}
    <div class={tabPanel} role="tabpanel" aria-label={copy.questionnairesTab}>
      <QuestionnairesView {locale} {onLogout} bind:refresh={questionnairesRefresh} />
    </div>
  {/if}
</PageShell>

{#if logoutDialogOpen}
  <Sheet
    title={copy.logoutConfirmTitle}
    closeLabel={copy.close}
    onClose={() => {
      if (!isLoggingOut) logoutDialogOpen = false;
    }}
  >
    <div class="grid grid-cols-[auto_1fr] gap-4 p-5">
      <span
        class="grid size-11 place-items-center rounded-md bg-danger-surface text-danger-strong"
        aria-hidden="true"><LogOut size={22} /></span
      >
      <div>
        <h2 class="text-xl leading-[1.25] font-extrabold">{copy.logoutConfirmTitle}</h2>
        <p class="mt-2 text-base leading-[1.5] text-muted-foreground"
          >{copy.logoutConfirmDescription}</p
        >
      </div>
      <div class="col-span-full grid grid-cols-1 gap-2 min-[30rem]:grid-cols-2">
        <Button variant="outline" block disabled={isLoggingOut} onclick={() => (logoutDialogOpen = false)}>
          {copy.cancel}
        </Button>
        <Button variant="danger" block loading={isLoggingOut} onclick={() => void handleLogoutClick()}>
          {#if !isLoggingOut}<LogOut size={17} aria-hidden="true" />{/if}
          {copy.logout}
        </Button>
      </div>
    </div>
  </Sheet>
{/if}

<style>
  /* Portal tables. The base layout is the small-screen card fallback: every cell
     carries its own header, and the real table only assembles once there is
     room. Switching a table's own display model means rewriting table, thead,
     tbody, tr, td and th together, so it stays one block of CSS rather than a
     class on every cell — and `.cell-label` is part of the same mechanism. */
  .portal-table {
    display: block;
    width: 100%;
    border-collapse: collapse;
    font-size: var(--text-base);
    text-align: left;
  }

  .portal-table thead {
    display: none;
  }

  .portal-table tbody,
  .portal-table tr,
  .portal-table td {
    display: block;
  }

  .portal-table tr {
    padding: var(--space-3);
    background: var(--surface-sunken);
    border-radius: var(--radius-md);
  }

  .portal-table tr + tr {
    margin-top: var(--space-2);
  }

  .portal-table td {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    gap: var(--space-3);
    padding: var(--space-1) 0;
  }

  .cell-label {
    flex: 0 0 auto;
    color: var(--muted-foreground);
    font-size: var(--text-xs);
    font-weight: var(--weight-bold);
  }

  @media (min-width: 48rem) {
    .portal-table {
      display: table;
    }

    .portal-table thead {
      display: table-header-group;
    }

    .portal-table tbody {
      display: table-row-group;
    }

    .portal-table tr,
    .portal-table tr + tr {
      display: table-row;
      margin: 0;
      padding: 0;
      background: transparent;
      border-radius: 0;
    }

    .portal-table th {
      padding: var(--space-3) var(--space-4);
      color: var(--muted-foreground);
      background: var(--surface-sunken);
      border-bottom: 1px solid var(--border-subtle);
      font-size: var(--text-xs);
      font-weight: var(--weight-bold);
      white-space: nowrap;
    }

    .portal-table td {
      display: table-cell;
      padding: var(--space-3) var(--space-4);
      border-bottom: 1px solid var(--border-subtle);
      vertical-align: middle;
    }

    .portal-table tbody tr:last-child td {
      border-bottom: 0;
    }

    .cell-label {
      display: none;
    }
  }

  @media (hover: hover) {
    .portal-table tbody tr:hover td {
      background: var(--surface-sunken);
    }
  }
</style>
