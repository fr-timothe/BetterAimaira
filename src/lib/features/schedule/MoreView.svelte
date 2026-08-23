<script lang="ts">
  import { untrack } from 'svelte';
  import {
    AlertCircle,
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
    Table2,
  } from 'lucide-svelte';
  import Badge from '$lib/components/ui/Badge.svelte';
  import Button from '$lib/components/ui/Button.svelte';
  import Card from '$lib/components/ui/Card.svelte';
  import IconButton from '$lib/components/ui/IconButton.svelte';
  import PageShell from '$lib/components/ui/PageShell.svelte';
  import SectionHeader from '$lib/components/ui/SectionHeader.svelte';
  import SegmentedControl from '$lib/components/ui/SegmentedControl.svelte';
  import Sheet from '$lib/components/ui/Sheet.svelte';
  import Skeleton from '$lib/components/ui/Skeleton.svelte';
  import StateCard from '$lib/components/ui/StateCard.svelte';
  import type { IconComponent } from '$lib/components/ui/icon';
  import * as m from '$lib/paraglide/messages.js';
  import type { Locale } from '$lib/paraglide/runtime.js';
  import { connectivity } from '$lib/state/connectivity.svelte';
  import AccountViewSkeleton from './AccountViewSkeleton.svelte';
  import { loadPortalResource } from './portal-cache';
  import QuestionnairesView from './QuestionnairesView.svelte';
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

  let activeTab = $state<MoreSection>('profile');
  let profileState = $state<PortalResourceState>({ kind: 'loading' });
  let documentsState = $state<PortalResourceState>({ kind: 'loading' });
  let downloadingPath = $state<string | null>(null);
  let downloadError = $state(false);
  let changingLocale = $state(false);
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
    locale;
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
      backToLogin: m.back_to_login(),
      unknownHeading: m.resource_unknown_heading(),
      unknownDescription: m.resource_unknown_description(),
      emptyHeading: m.resource_empty_heading(),
      emptyDescription: m.resource_empty_description(),
      sessionExpired: m.account_disconnected(),
      offline: m.sync_offline(),
      offlineDescription: m.sync_offline_description(),
      download: m.download_document(),
      downloading: m.downloading_document(),
      downloadError: m.document_download_error(),
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
    try {
      await downloadPortalDocument(document);
    } catch {
      downloadError = true;
    } finally {
      downloadingPath = null;
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
    locale;
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
    locale;
    return documentsState.kind === 'ready'
      ? m.documents_pdf_count({ count: documentsState.page.documents.length })
      : '';
  });

  const schoolName = $derived(studentProfile.campus || getPortalHost(portalUrl));
</script>

{#snippet resourceError(code: PortalResourceErrorCode, retry: () => void)}
  <StateCard
    kind={code === 'session_expired' ? 'expired' : 'error'}
    title={code === 'session_expired' ? copy.sessionExpired : copy.errorHeading}
    description={resourceErrorMessage(code)}
    icon={AlertCircle}
    actionLabel={code === 'session_expired' ? copy.backToLogin : copy.retry}
    onAction={code === 'session_expired' ? () => void onLogout() : retry}
  />
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
  <header class="sync-toolbar desktop-only">
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
    <div class="tab-panel" role="tabpanel" aria-label={copy.profileTab}>
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

        <article class="identity-card">
          <h2 class="identity-name">{studentProfile.fullName}</h2>
          <p class="identity-school">
            <School size={17} aria-hidden="true" />
            <span>{schoolName}</span>
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
            <div class="panel-grid">
              {#each studentProfile.categorizedFields as section (section.category)}
                <Card>
                  <div class="panel-body">
                    <SectionHeader icon={section.icon} title={section.category} level={3} />

                    <dl class="fields-list">
                      {#each section.fields as field (`${field.label}:${field.value}`)}
                        <div class="field-row">
                          <dt>{field.label}</dt>
                          <dd>{field.value}</dd>
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
              <div class="panel-body table-panel">
                <SectionHeader
                  icon={Table2}
                  title={table.caption || table.context.at(-1) || m.profile_table_fallback()}
                  subtitle={m.profile_table_row_count({ count: table.rows.length })}
                  level={3}
                />

                <div class="portal-table-scroll">
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
                              <span class="cell-value">{cell}</span>
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
        <div class="language-setting">
          <div class="language-copy">
            <span class="language-icon" aria-hidden="true"><Languages size={18} /></span>
            <label for="profile-language">{copy.languageLabel}</label>
          </div>

          <select
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

      <div class="logout-section">
        <Button variant="danger" size="lg" block onclick={() => (logoutDialogOpen = true)}>
          <LogOut size={18} aria-hidden="true" />
          <span>{copy.logout}</span>
        </Button>
      </div>
    </div>
  {:else if activeTab === 'documents'}
    <div class="tab-panel" role="tabpanel" aria-label={copy.documentsTab}>
      {#if documentsState.kind === 'loading'}
        <div class="documents-skeleton" role="status" aria-live="polite" aria-busy="true" aria-label={copy.loading}>
          <Card>
            <div class="panel-body">
              <SectionHeader icon={FileText} title={copy.documentsTab} level={3} />

              <div class="documents-list">
                {#each Array(3) as _, index (index)}
                  <div class="document-card">
                    <Skeleton shape="circle" width="2.75rem" height="2.75rem" />
                    <div class="document-skeleton-copy">
                      <div class="document-tags-skeleton">
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
          <p class="inline-alert" role="alert">
            <AlertCircle size={16} aria-hidden="true" />
            <span>{copy.downloadError}</span>
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
            <div class="panel-body">
              <SectionHeader
                icon={FileText}
                title={copy.documentsTab}
                subtitle={documentCountLabel}
                level={3}
              />

              <div class="documents-list">
                {#each documentsState.page.documents as doc (doc.requestPath)}
                  {@const DocIcon = documentIcon(doc.kind)}
                  {@const tone = documentTone(doc.kind)}
                  {@const busy = downloadingPath === doc.requestPath}
                  <article class="document-card">
                    <span
                      class="document-icon"
                      class:tone-accent={tone === 'accent'}
                      class:tone-success={tone === 'success'}
                      class:tone-neutral={tone === 'neutral'}
                      aria-hidden="true"
                    >
                      <DocIcon size={21} />
                    </span>

                    <div class="document-meta">
                      <div class="document-tags">
                        <Badge {tone}>{documentKindLabel(doc.kind)}</Badge>
                        <Badge>PDF</Badge>
                      </div>
                      <h4 class="document-label">{doc.label}</h4>
                      {#if doc.suggestedFilename}
                        <p class="document-filename">{doc.suggestedFilename}</p>
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
    <div class="tab-panel" role="tabpanel" aria-label={copy.questionnairesTab}>
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
    <div class="logout-dialog">
      <span class="logout-dialog-icon" aria-hidden="true"><LogOut size={22} /></span>
      <div class="logout-dialog-copy">
        <h2>{copy.logoutConfirmTitle}</h2>
        <p>{copy.logoutConfirmDescription}</p>
      </div>
      <div class="logout-dialog-actions">
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
  .tab-panel {
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
    min-width: 0;
    animation: fade-in var(--duration-normal) var(--ease-out);
  }

  .sync-toolbar {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    justify-content: flex-end;
    gap: var(--space-3);
  }

  /* Panels are `Card`; this only supplies the inner rhythm. */
  .panel-body {
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
    min-width: 0;
  }

  .panel-grid {
    display: grid;
    grid-template-columns: 1fr;
    gap: var(--space-4);
  }

  .identity-card {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
    padding: var(--space-5) var(--space-4);
    background: var(--muted);
    border-radius: var(--radius-xl);
  }

  .identity-name {
    margin: 0;
    color: var(--foreground);
    font-size: var(--text-xl);
    font-weight: var(--weight-heavy);
    line-height: 1.2;
    letter-spacing: -0.02em;
    overflow-wrap: anywhere;
  }

  .identity-school {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    margin: 0;
    color: var(--primary-deep);
    font-size: var(--text-base);
    font-weight: var(--weight-semibold);
    min-width: 0;
  }

  .identity-school :global(svg) {
    flex: 0 0 auto;
  }

  .identity-school span {
    overflow-wrap: anywhere;
  }

  /* Field lists */
  .fields-list {
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
    margin: 0;
  }

  .field-row {
    display: flex;
    flex-direction: column;
    gap: 0.15rem;
    padding: var(--space-2) var(--space-3);
    background: var(--surface-sunken);
    border-radius: var(--radius-md);
  }

  .field-row dt {
    color: var(--muted-foreground);
    font-size: var(--text-xs);
    font-weight: var(--weight-bold);
  }

  .field-row dd {
    margin: 0;
    color: var(--foreground);
    font-size: var(--text-base);
    font-weight: var(--weight-medium);
    font-variant-numeric: tabular-nums;
    overflow-wrap: anywhere;
  }

  /* Portal tables. The base layout is the small-screen card fallback: every cell
     carries its own header, and the real table only assembles once there is room. */
  .table-panel {
    padding: var(--space-4);
  }

  .portal-table-scroll {
    overflow-x: auto;
    -webkit-overflow-scrolling: touch;
  }

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

  .cell-value {
    color: var(--foreground);
    font-weight: var(--weight-medium);
    font-variant-numeric: tabular-nums;
    overflow-wrap: anywhere;
  }

  /* Documents */
  .documents-list {
    display: grid;
    grid-template-columns: 1fr;
    gap: var(--space-3);
  }

  .document-card {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    min-width: 0;
    padding: var(--space-3);
    background: var(--surface-sunken);
    border-radius: var(--radius-lg);
    transition: background-color var(--duration-fast) var(--ease-out);
  }

  .document-icon {
    display: grid;
    width: 2.75rem;
    height: 2.75rem;
    flex: 0 0 2.75rem;
    place-items: center;
    border-radius: var(--radius-md);
  }

  .document-icon.tone-accent {
    color: var(--primary-deep);
    background: var(--muted);
  }

  .document-icon.tone-success {
    color: var(--success-strong);
    background: var(--success-surface);
  }

  .document-icon.tone-neutral {
    color: var(--muted-foreground);
    background: var(--card);
  }

  .document-meta {
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
    min-width: 0;
    flex: 1;
  }

  .document-tags {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-1);
  }

  .document-label {
    margin: 0;
    color: var(--foreground);
    font-size: var(--text-base);
    font-weight: var(--weight-bold);
    line-height: 1.3;
    overflow-wrap: anywhere;
  }

  .document-filename {
    margin: 0;
    color: var(--muted-foreground);
    font-size: var(--text-xs);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .language-setting {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-4);
    min-width: 0;
  }

  .language-copy {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    min-width: 0;
    width: 100%;
  }

  .language-icon {
    display: grid;
    width: 2rem;
    height: 2rem;
    flex: 0 0 2rem;
    place-items: center;
    color: var(--primary-deep);
    background: var(--muted);
    border-radius: var(--radius-sm);
  }

  .language-copy label {
    font-size: var(--text-base);
    font-weight: var(--weight-bold);
  }

  .language-setting select {
    width: 100%;
    min-height: var(--tap-min);
    padding: 0 var(--space-8) 0 var(--space-3);
    color: var(--foreground);
    background: var(--surface-sunken);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-md);
    font: inherit;
    font-weight: var(--weight-semibold);
  }

  .language-setting select:disabled {
    opacity: 0.62;
  }

  .logout-section {
    margin-top: var(--space-4);
    padding-top: var(--space-4);
    border-top: 1px solid var(--border-subtle);
  }

  .logout-dialog {
    display: grid;
    grid-template-columns: auto 1fr;
    gap: var(--space-4);
    padding: var(--space-5);
  }

  .logout-dialog-icon {
    display: grid;
    width: 2.75rem;
    height: 2.75rem;
    place-items: center;
    color: var(--danger-strong);
    background: var(--danger-surface);
    border-radius: var(--radius-md);
  }

  .logout-dialog-copy h2 {
    margin: 0;
    font-size: var(--text-xl);
    font-weight: var(--weight-heavy);
    line-height: 1.25;
  }

  .logout-dialog-copy p {
    margin: var(--space-2) 0 0;
    color: var(--muted-foreground);
    font-size: var(--text-base);
    line-height: 1.5;
  }

  .logout-dialog-actions {
    display: grid;
    grid-column: 1 / -1;
    grid-template-columns: 1fr;
    gap: var(--space-2);
  }

  @media (min-width: 30rem) {
    .language-setting {
      flex-direction: row;
    }

    .language-copy {
      width: auto;
    }

    .language-setting select {
      width: auto;
      min-width: 9.5rem;
    }

    .logout-dialog-actions {
      grid-template-columns: 1fr 1fr;
    }
  }

  /* Inline messages */
  .inline-alert {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    margin: 0;
    padding: var(--space-2) var(--space-3);
    border-radius: var(--radius-md);
    font-size: var(--text-sm);
    font-weight: var(--weight-semibold);
  }

  .inline-alert {
    color: var(--danger-strong);
    background: var(--danger-surface);
  }

  /* Loading */
  .document-skeleton-copy {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
    flex: 1;
    min-width: 0;
  }

  .document-tags-skeleton {
    display: flex;
    align-items: center;
    gap: var(--space-2);
  }

  @media (min-width: 48rem) {
    .identity-card {
      padding: var(--space-6) var(--space-5);
    }

    .panel-grid {
      grid-template-columns: repeat(auto-fill, minmax(min(100%, 20rem), 1fr));
    }

    .fields-list {
      display: grid;
      grid-template-columns: repeat(auto-fill, minmax(min(100%, 13rem), 1fr));
      gap: var(--space-2);
    }

    .documents-list {
      grid-template-columns: repeat(auto-fill, minmax(min(100%, 21rem), 1fr));
    }

    .table-panel {
      padding: var(--space-5) 0 0;
    }

    .table-panel :global(.ui-section-header) {
      padding: 0 var(--space-5);
    }

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
    .document-card:hover {
      background: var(--muted);
    }

    .portal-table tbody tr:hover td {
      background: var(--surface-sunken);
    }
  }
</style>
