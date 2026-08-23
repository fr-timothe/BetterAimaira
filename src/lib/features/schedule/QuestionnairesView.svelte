<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { onMount } from 'svelte';
  import {
    AlertCircle,
    ArrowLeft,
    CalendarClock,
    Check,
    CheckCircle2,
    ChevronRight,
    ClipboardList,
    Clock,
    CloudOff,
    GraduationCap,
    RefreshCw,
    UserRound,
  } from 'lucide-svelte';
  import Badge from '$lib/components/ui/Badge.svelte';
  import Button from '$lib/components/ui/Button.svelte';
  import Card from '$lib/components/ui/Card.svelte';
  import IconButton from '$lib/components/ui/IconButton.svelte';
  import SectionHeader from '$lib/components/ui/SectionHeader.svelte';
  import Skeleton from '$lib/components/ui/Skeleton.svelte';
  import StateCard from '$lib/components/ui/StateCard.svelte';
  import * as m from '$lib/paraglide/messages.js';
  import type { Locale } from '$lib/paraglide/runtime.js';
  import { connectivity } from '$lib/state/connectivity.svelte';
  import { loadPortalResource } from './portal-cache';
  import { parseResourceError, resourceErrorMessage } from './portal-utils';
  import type {
    PortalResourceErrorCode,
    PortalResourceState,
    QuestionnaireDetail,
    QuestionnaireQuestion,
    QuestionnaireSummary,
  } from './types';

  type DetailState =
    | { kind: 'idle' }
    | { kind: 'loading' }
    | { kind: 'ready'; detail: QuestionnaireDetail }
    | { kind: 'error'; code: PortalResourceErrorCode };

  type Props = {
    locale: Locale;
    onLogout: () => Promise<void>;
    refresh?: () => Promise<void>;
  };

  let { locale, onLogout, refresh = $bindable() }: Props = $props();
  let listState = $state<PortalResourceState>({ kind: 'loading' });
  let detailState = $state<DetailState>({ kind: 'idle' });
  let selected = $state<QuestionnaireSummary | null>(null);
  let refreshing = $state(false);
  let listSequence = 0;
  let detailSequence = 0;

  const copy = $derived.by(() => {
    locale;
    return {
      heading: m.questionnaires_heading(),
      loading: m.resource_loading(),
      refresh: m.resource_refresh(),
      retry: m.resource_retry(),
      backToLogin: m.back_to_login(),
      errorHeading: m.resource_error_heading(),
      offline: m.sync_offline(),
      offlineDescription: m.sync_offline_description(),
      emptyHeading: m.questionnaires_empty_heading(),
      emptyDescription: m.questionnaires_empty_description(),
      completed: m.questionnaire_status_completed(),
      pending: m.questionnaire_status_pending(),
      open: m.questionnaire_open(),
      back: m.questionnaire_back(),
      required: m.questionnaire_required(),
      optional: m.questionnaire_optional(),
      noAnswer: m.questionnaire_no_answer(),
      response: m.questionnaire_response(),
      pageFallback: m.questionnaire_page_fallback(),
    };
  });

  $effect(() => {
    refresh = async () => {
      if (selected) {
        await loadDetail(selected);
      } else {
        await loadList(true);
      }
    };
  });

  onMount(() => {
    void loadList();
  });

  async function loadList(force = false) {
    const sequence = ++listSequence;
    const hasData = listState.kind === 'ready';
    if (hasData) refreshing = true;

    try {
      const page = await loadPortalResource('questionnaires', force);
      if (sequence !== listSequence) return;
      listState = { kind: 'ready', page };
    } catch (error) {
      if (sequence !== listSequence) return;
      const code = parseResourceError(error, 'questionnaires_unavailable');
      if (!hasData || code === 'session_expired') {
        listState = { kind: 'error', code };
      }
    } finally {
      if (sequence === listSequence) refreshing = false;
    }
  }

  async function loadDetail(questionnaire: QuestionnaireSummary) {
    const sequence = ++detailSequence;
    selected = questionnaire;
    detailState = { kind: 'loading' };

    try {
      const detail = await invoke<QuestionnaireDetail>('get_questionnaire_detail', {
        request: { responsePath: questionnaire.responsePath },
      });
      if (sequence !== detailSequence) return;
      detailState = { kind: 'ready', detail };
    } catch (error) {
      if (sequence !== detailSequence) return;
      detailState = {
        kind: 'error',
        code: parseResourceError(error, 'questionnaires_unavailable'),
      };
    }
  }

  function closeDetail() {
    detailSequence += 1;
    selected = null;
    detailState = { kind: 'idle' };
  }

  function parseQuestionnaireInfo(title: string, context: string | null) {
    if (!context || !context.trim()) {
      return {
        title: title.trim(),
        teacher: null,
        campaign: null,
      };
    }

    const cleaned = context.replace(/\s*-\s*-\s*/g, ' — ').replace(/\s+--\s+/g, ' — ').trim();
    const parts = cleaned.split(' — ');

    if (parts.length >= 2) {
      const subject = parts[0].trim();
      const teacher = parts.slice(1).join(' — ').trim();
      return {
        title: subject || title.trim(),
        teacher: teacher || null,
        campaign: title.trim() !== subject ? title.trim() : null,
      };
    }

    return {
      title: cleaned || title.trim(),
      teacher: null,
      campaign: title.trim() !== cleaned ? title.trim() : null,
    };
  }

  function statusLabel(questionnaire: QuestionnaireSummary): string {
    return questionnaire.status.trim() || (questionnaire.completed ? copy.completed : copy.pending);
  }

  function isSelectedOption(question: QuestionnaireQuestion, value: string, label: string): boolean {
    return question.answers.includes(value) || question.answers.includes(label);
  }
</script>

{#snippet errorState(code: PortalResourceErrorCode, retry: () => void)}
  <StateCard
    kind={code === 'session_expired' ? 'expired' : 'error'}
    icon={AlertCircle}
    title={code === 'session_expired' ? m.account_disconnected() : copy.errorHeading}
    description={resourceErrorMessage(code)}
    actionLabel={code === 'session_expired' ? copy.backToLogin : copy.retry}
    onAction={code === 'session_expired' ? () => void onLogout() : retry}
  />
{/snippet}

{#snippet offlineState(retry: () => void)}
  <StateCard
    kind="error"
    icon={CloudOff}
    title={copy.offline}
    description={copy.offlineDescription}
    actionLabel={copy.retry}
    onAction={retry}
  />
{/snippet}

{#if selected}
  {@const questionnaire = selected}
  {@const info = parseQuestionnaireInfo(questionnaire.title, questionnaire.context)}
  <div class="questionnaire-detail">
    <div class="detail-toolbar">
      <Button variant="ghost" onclick={closeDetail}>
        <ArrowLeft size={17} aria-hidden="true" />
        <span>{copy.back}</span>
      </Button>
    </div>

    {#if detailState.kind === 'loading'}
      <div class="detail-skeleton" role="status" aria-live="polite" aria-label={copy.loading}>
        <Card padding="none">
          <header class="detail-heading detail-heading-skeleton">
            <div class="detail-title-group">
              <div class="detail-tags">
                <Skeleton shape="block" width="5rem" height="1.5rem" />
                <Skeleton shape="text" width="7rem" />
              </div>
              <Skeleton shape="title" width="55%" />
              <Skeleton shape="text" width="42%" />
            </div>
          </header>

          {#each Array(3) as _, index (index)}
            <section class="question-page question-skeleton-row">
              <div class="question-title-skeleton">
                <Skeleton shape="circle" width="2rem" height="2rem" />
                <div class="question-copy-skeleton">
                  <Skeleton shape="title" width={index === 1 ? '62%' : '76%'} />
                  <Skeleton shape="text" width="45%" />
                </div>
                <Skeleton shape="block" width="4.5rem" height="1.5rem" />
              </div>
              <Skeleton shape="block" height={index === 0 ? '4.5rem' : '3.5rem'} />
            </section>
          {/each}
        </Card>
      </div>
    {:else if detailState.kind === 'error' && !connectivity.online}
      {@render offlineState(() => void loadDetail(questionnaire))}
    {:else if detailState.kind === 'error'}
      {@render errorState(detailState.code, () => void loadDetail(questionnaire))}
    {:else if detailState.kind === 'ready'}
      <Card padding="none">
        <header class="detail-heading">
          <div class="detail-title-group">
            <div class="detail-tags">
              <Badge tone={detailState.detail.completed ? 'success' : 'warning'}>
                {#if detailState.detail.completed}
                  <CheckCircle2 size={13} aria-hidden="true" />
                {:else}
                  <Clock size={13} aria-hidden="true" />
                {/if}
                {detailState.detail.completed ? copy.completed : copy.pending}
              </Badge>
              {#if questionnaire.deadline}
                <span class="deadline">
                  <CalendarClock size={14} aria-hidden="true" />
                  {m.questionnaire_deadline({ date: questionnaire.deadline })}
                </span>
              {/if}
            </div>

            <h2>{info.title}</h2>

            {#if info.teacher || info.campaign}
              <div class="detail-meta">
                {#if info.teacher}
                  <span class="meta-item">
                    <GraduationCap size={15} aria-hidden="true" />
                    <span>{info.teacher}</span>
                  </span>
                {/if}
                {#if info.campaign}
                  <span class="meta-item campaign-tag">
                    <span>{info.campaign}</span>
                  </span>
                {/if}
              </div>
            {/if}
          </div>
        </header>

        {#each detailState.detail.pages as page, pageIndex (page.id)}
          <section class="question-page" aria-labelledby={`questionnaire-page-${page.id}`}>
            {#if page.title && page.title !== info.title && page.title !== info.campaign}
              <h3 id={`questionnaire-page-${page.id}`}>{page.title}</h3>
            {/if}

            <ol class="questions" start={pageIndex === 0 ? 1 : undefined}>
              {#each page.questions as question, qIndex (question.id)}
                <li class="question-row">
                  <div class="question-copy">
                    <div class="question-title-line">
                      <span class="question-num" aria-hidden="true">{qIndex + 1}</span>
                      <h4>{question.title}</h4>
                      <Badge tone={question.required ? 'accent' : 'neutral'}>
                        {question.required ? copy.required : copy.optional}
                      </Badge>
                    </div>
                    {#if question.description}
                      <div class="question-description-box">
                        <p>{question.description}</p>
                      </div>
                    {/if}
                  </div>

                  <div class="answer" aria-label={copy.response}>
                    {#if question.kind === 'rating' && question.options.length > 0}
                      <div class="rating-answer" role="group" aria-label={question.title}>
                        {#each question.options as option (`${question.id}:${option.value}`)}
                          {@const isSelected = isSelectedOption(question, option.value, option.label)}
                          <span class="rating-pill" class:selected={isSelected}>
                            {#if isSelected}
                              <Check size={14} aria-hidden="true" />
                            {/if}
                            {option.label}
                          </span>
                        {/each}
                      </div>
                    {:else if question.answers.length > 0}
                      <div class="text-answer">
                        <span class="answer-label">{copy.response}</span>
                        <ul>
                          {#each question.answers as answer, answerIndex (`${question.id}:${answerIndex}`)}
                            <li>{answer}</li>
                          {/each}
                        </ul>
                      </div>
                    {:else}
                      <span class="no-answer">{copy.noAnswer}</span>
                    {/if}
                  </div>
                </li>
              {/each}
            </ol>
          </section>
        {/each}
      </Card>
    {/if}
  </div>
{:else if listState.kind === 'loading'}
  <div class="list-skeleton" role="status" aria-live="polite" aria-label={copy.loading}>
    <Card padding="none">
      <header class="list-heading">
        <SectionHeader icon={ClipboardList} title={copy.heading} level={3} />
      </header>
      <div class="questionnaire-list">
        {#each Array(3) as _, index (index)}
          <div class="questionnaire-card questionnaire-card-skeleton">
            <div class="card-content">
              <div class="card-topline">
                <Skeleton shape="block" width="5rem" height="1.5rem" />
                <Skeleton shape="text" width="7rem" />
              </div>
              <Skeleton shape="title" width={index === 1 ? '58%' : '72%'} />
              <Skeleton shape="text" width="46%" />
            </div>
            <Skeleton shape="circle" width="2.25rem" height="2.25rem" />
          </div>
        {/each}
      </div>
    </Card>
  </div>
{:else if listState.kind === 'error' && !connectivity.online}
  {@render offlineState(() => void loadList(true))}
{:else if listState.kind === 'error'}
  {@render errorState(listState.code, () => void loadList(true))}
{:else}
  <Card padding="none">
    <header class="list-heading">
      <SectionHeader
        icon={ClipboardList}
        title={copy.heading}
        subtitle={m.questionnaires_count({ count: listState.page.questionnaires.length })}
        level={3}
      >
        {#snippet actions()}
          <div class="desktop-only">
            <IconButton label={copy.refresh} loading={refreshing} onclick={() => void loadList(true)}>
              <RefreshCw size={18} aria-hidden="true" />
            </IconButton>
          </div>
        {/snippet}
      </SectionHeader>
    </header>

    {#if listState.page.questionnaires.length === 0}
      <div class="empty-wrap">
        <StateCard
          kind="empty"
          icon={ClipboardList}
          title={copy.emptyHeading}
          description={copy.emptyDescription}
        />
      </div>
    {:else}
      <div class="questionnaire-list">
        {#each listState.page.questionnaires as questionnaire (questionnaire.id)}
          {@const info = parseQuestionnaireInfo(questionnaire.title, questionnaire.context)}
          <button
            type="button"
            class="questionnaire-card"
            onclick={() => void loadDetail(questionnaire)}
          >
            <div class="card-content">
              <div class="card-topline">
                <Badge tone={questionnaire.completed ? 'success' : 'warning'}>
                  {#if questionnaire.completed}
                    <CheckCircle2 size={12} aria-hidden="true" />
                  {:else}
                    <Clock size={12} aria-hidden="true" />
                  {/if}
                  {statusLabel(questionnaire)}
                </Badge>
                {#if questionnaire.deadline}
                  <span class="deadline">
                    <CalendarClock size={13} aria-hidden="true" />
                    {m.questionnaire_deadline({ date: questionnaire.deadline })}
                  </span>
                {/if}
              </div>

              <h3 class="course-title">{info.title}</h3>

              <div class="course-meta">
                {#if info.teacher}
                  <span class="meta-item">
                    <UserRound size={13} aria-hidden="true" />
                    <span>{info.teacher}</span>
                  </span>
                {/if}
                {#if info.campaign}
                  <span class="meta-campaign">{info.campaign}</span>
                {/if}
              </div>
            </div>

            <span class="card-arrow" aria-hidden="true">
              <ChevronRight size={18} />
            </span>
          </button>
        {/each}
      </div>
    {/if}
  </Card>
{/if}

<style>
  .questionnaire-detail,
  .list-skeleton,
  .detail-skeleton {
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
    min-width: 0;
    padding-bottom: calc(var(--space-6) + 4.5rem);
  }

  .detail-toolbar {
    display: flex;
    align-items: center;
  }

  .list-heading,
  .detail-heading {
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
    padding: var(--space-4);
    border-bottom: 1px solid var(--border-subtle);
  }

  .list-heading :global(.ui-section-header) {
    width: 100%;
  }

  .detail-title-group {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: var(--space-2);
    min-width: 0;
  }

  .detail-tags {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: var(--space-2);
  }

  .detail-heading h2 {
    margin: 0;
    color: var(--foreground);
    font-size: var(--text-xl);
    font-weight: var(--weight-heavy);
    line-height: 1.25;
    overflow-wrap: anywhere;
  }

  .detail-meta {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: var(--space-2);
    margin-top: var(--space-1);
    color: var(--muted-foreground);
    font-size: var(--text-sm);
  }

  .meta-item {
    display: inline-flex;
    align-items: center;
    gap: var(--space-1);
    font-weight: var(--weight-medium);
  }

  .campaign-tag {
    color: var(--muted-foreground);
    background: var(--surface-sunken);
    padding: 0.15rem 0.5rem;
    border-radius: var(--radius-sm);
    font-size: var(--text-xs);
  }

  .deadline {
    display: inline-flex;
    align-items: center;
    gap: var(--space-1);
    color: var(--muted-foreground);
    font-size: var(--text-xs);
    font-weight: var(--weight-semibold);
    font-variant-numeric: tabular-nums;
  }

  /* Questionnaire List */
  .questionnaire-list {
    display: flex;
    flex-direction: column;
    padding-bottom: calc(var(--space-6) + 4.5rem);
  }

  .questionnaire-card {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-3);
    width: 100%;
    padding: var(--space-4);
    background: transparent;
    border: 0;
    border-bottom: 1px solid var(--border-subtle);
    cursor: pointer;
    text-align: left;
    transition: background var(--duration-fast) var(--ease-out);
    user-select: none;
    -webkit-user-select: none;
  }

  .questionnaire-card:last-child {
    border-bottom: 0;
  }

  .card-content {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
    min-width: 0;
    flex: 1;
  }

  .card-topline {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: var(--space-2);
  }

  .course-title {
    margin: 0;
    color: var(--foreground);
    font-size: var(--text-base);
    font-weight: var(--weight-bold);
    line-height: 1.35;
    overflow-wrap: anywhere;
  }

  .course-meta {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: var(--space-2);
    font-size: var(--text-xs);
    color: var(--muted-foreground);
  }

  .meta-campaign {
    color: var(--muted-foreground);
    font-size: var(--text-xs);
  }

  .card-arrow {
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--muted-foreground);
    transition: transform var(--duration-fast) var(--ease-out), color var(--duration-fast) var(--ease-out);
    flex-shrink: 0;
  }

  /* Questions Page */
  .question-page {
    padding: var(--space-4);
  }

  .question-page + .question-page {
    border-top: 1px solid var(--border-subtle);
  }

  .question-page > h3 {
    margin: 0 0 var(--space-3);
    color: var(--foreground);
    font-size: var(--text-lg);
    font-weight: var(--weight-bold);
    line-height: 1.3;
  }

  .questions {
    display: flex;
    flex-direction: column;
    gap: 0;
    margin: 0;
    padding: 0;
    list-style: none;
  }

  .question-row {
    display: grid;
    grid-template-columns: minmax(0, 1fr);
    gap: var(--space-3);
    padding: var(--space-4) 0;
  }

  .question-row + .question-row {
    border-top: 1px solid var(--border-subtle);
  }

  .question-copy {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
    min-width: 0;
  }

  .question-title-line {
    display: flex;
    align-items: flex-start;
    gap: var(--space-2);
  }

  .question-num {
    display: inline-grid;
    place-items: center;
    width: 1.5rem;
    height: 1.5rem;
    flex-shrink: 0;
    border-radius: var(--radius-full);
    background: var(--surface-sunken);
    color: var(--muted-foreground);
    font-size: var(--text-xs);
    font-weight: var(--weight-bold);
  }

  .question-title-line h4 {
    flex: 1;
    margin: 0;
    color: var(--foreground);
    font-size: var(--text-base);
    font-weight: var(--weight-bold);
    line-height: 1.4;
    overflow-wrap: anywhere;
  }

  .question-description-box {
    margin-left: calc(1.5rem + var(--space-2));
    padding: var(--space-2) var(--space-3);
    background: var(--surface-sunken);
    border-radius: var(--radius-md);
    border-left: 3px solid var(--border);
  }

  .question-description-box p {
    margin: 0;
    color: var(--muted-foreground);
    font-size: var(--text-sm);
    line-height: 1.5;
    white-space: pre-line;
  }

  .answer {
    min-width: 0;
    margin-left: calc(1.5rem + var(--space-2));
  }

  .rating-answer {
    display: grid;
    grid-template-columns: repeat(5, 1fr);
    gap: var(--space-2);
    max-width: 24rem;
  }

  .rating-pill {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: var(--space-1);
    min-height: var(--tap-min);
    color: var(--muted-foreground);
    background: var(--surface-sunken);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-md);
    font-size: var(--text-sm);
    font-weight: var(--weight-bold);
    font-variant-numeric: tabular-nums;
  }

  .rating-pill.selected {
    color: var(--secondary-foreground);
    background: var(--primary-deep);
    border-color: var(--primary-deep);
    box-shadow: var(--shadow-sm);
  }

  .text-answer {
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
  }

  .answer-label {
    color: var(--muted-foreground);
    font-size: var(--text-xs);
    font-weight: var(--weight-bold);
  }

  .text-answer ul {
    margin: 0;
    padding: var(--space-3);
    color: var(--foreground);
    background: var(--surface-sunken);
    border-radius: var(--radius-md);
    font-size: var(--text-sm);
    line-height: 1.5;
    list-style: none;
  }

  .no-answer {
    display: inline-flex;
    align-items: center;
    min-height: var(--tap-min);
    color: var(--muted-foreground);
    font-size: var(--text-sm);
    font-style: italic;
  }

  .questionnaire-card-skeleton {
    cursor: default;
  }

  .detail-heading-skeleton {
    min-height: 8rem;
  }

  .question-skeleton-row {
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
    min-height: 10rem;
  }

  .question-title-skeleton {
    display: grid;
    grid-template-columns: 2rem minmax(0, 1fr) auto;
    align-items: center;
    gap: var(--space-3);
  }

  .question-copy-skeleton {
    display: flex;
    min-width: 0;
    flex-direction: column;
    gap: var(--space-2);
  }

  .empty-wrap {
    padding: var(--space-4);
  }

  @media (min-width: 48rem) {
    .questionnaire-detail,
    .questionnaire-list {
      padding-bottom: var(--space-6);
    }

    .list-heading,
    .detail-heading,
    .question-page {
      padding: var(--space-5);
    }

    .list-heading,
    .detail-heading {
      flex-direction: row;
      align-items: center;
      justify-content: space-between;
    }

    .questionnaire-card {
      padding: var(--space-4) var(--space-5);
    }

    .question-row {
      grid-template-columns: minmax(0, 1.35fr) minmax(15rem, 0.65fr);
      gap: var(--space-5);
    }

    .answer {
      margin-left: 0;
    }
  }

  @media (hover: hover) {
    .questionnaire-card:hover {
      background: var(--surface-sunken);
    }

    .questionnaire-card:hover .card-arrow {
      transform: translateX(3px);
      color: var(--primary-deep);
    }
  }
</style>
