<script lang="ts">
  import { invoke } from '$lib/invoke';
  import { onMount } from 'svelte';
  import {
    ArrowLeft,
    CalendarClock,
    Check,
    CheckCircle2,
    ChevronRight,
    ClipboardList,
    Clock,
    GraduationCap,
    RefreshCw,
    UserRound,
  } from 'lucide-svelte';
  import Badge from '$lib/components/ui/Badge.svelte';
  import Button from '$lib/components/ui/Button.svelte';
  import Card from '$lib/components/ui/Card.svelte';
  import FreshnessLabel from '$lib/components/ui/FreshnessLabel.svelte';
  import IconButton from '$lib/components/ui/IconButton.svelte';
  import SectionHeader from '$lib/components/ui/SectionHeader.svelte';
  import Skeleton from '$lib/components/ui/Skeleton.svelte';
  import StateCard from '$lib/components/ui/StateCard.svelte';
  import * as m from '$lib/paraglide/messages.js';
  import type { Locale } from '$lib/paraglide/runtime.js';
  import PortalResourceError from './PortalResourceError.svelte';
  import { createPortalResource } from './portal-resource.svelte';
  import { parseResourceError } from './portal-utils';
  import { cn } from '$lib/utils';
  import type {
    PortalResourceErrorCode,
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
  let detailState = $state<DetailState>({ kind: 'idle' });
  let selected = $state<QuestionnaireSummary | null>(null);
  let detailSequence = 0;

  const copy = $derived.by(() => {
    return {
      heading: m.questionnaires_heading(),
      loading: m.resource_loading(),
      refresh: m.resource_refresh(),
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

  // An expired session gives way to the sign-in card instead of being flagged
  // on top of a stale list: this surface offers no other way back in.
  const questionnaires = createPortalResource({
    resource: 'questionnaires',
    fallbackErrorCode: 'questionnaires_unavailable',
    heading: () => copy.heading,
    locale: () => locale,
    expiredReplacesData: true,
  });

  const summaries = $derived(questionnaires.page?.questionnaires ?? []);

  $effect(() => {
    refresh = async () => {
      if (selected) {
        await loadDetail(selected);
      } else {
        await questionnaires.load(true);
      }
    };
  });

  onMount(() => {
    void questionnaires.load();
  });

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
  // The dock overlaps the last row on compact windows, so every scrolling column
  // ends with room for it.
  const column = 'flex min-w-0 flex-col gap-4 pb-[calc(var(--space-6)+4.5rem)] md:pb-6';
  const heading =
    'flex flex-col gap-3 border-b border-border-subtle p-4' +
    ' md:flex-row md:items-center md:justify-between md:p-5';
  const metaItem = 'inline-flex items-center gap-1 font-medium';
  const deadline =
    'inline-flex items-center gap-1 text-xs font-semibold tabular-nums text-muted-foreground';
  const listBody = 'flex flex-col pb-[calc(var(--space-6)+4.5rem)] md:pb-6';
  const row =
    'flex w-full items-center justify-between gap-3 border-b border-border-subtle' +
    ' bg-transparent p-4 text-left select-none last:border-b-0 md:px-5 md:py-4';
  const cardContent = 'flex min-w-0 flex-1 flex-col gap-2';
  const topline = 'flex flex-wrap items-center gap-2';
  const pageClass = 'border-t border-border-subtle p-4 first-of-type:border-t-0 md:p-5';
  const copyStack = 'flex min-w-0 flex-col gap-2';
  // The description and the answer both clear the number's own column.
  const indent = 'ml-[calc(1.5rem+var(--space-2))]';
</script>

<!-- The offline / expired / failed trio is told apart in one place, so the two
     surfaces below cannot disagree about which card a code deserves. -->
{#snippet errorState(code: PortalResourceErrorCode, retry: () => void)}
  <PortalResourceError {code} onRetry={retry} {onLogout} {locale} />
{/snippet}

{#if selected}
  {@const questionnaire = selected}
  {@const info = parseQuestionnaireInfo(questionnaire.title, questionnaire.context)}
  <div class={column}>
    <div class="flex items-center">
      <Button variant="ghost" onclick={closeDetail}>
        <ArrowLeft size={17} aria-hidden="true" />
        <span>{copy.back}</span>
      </Button>
    </div>

    {#if detailState.kind === 'loading'}
      <div class={column} role="status" aria-live="polite" aria-label={copy.loading}>
        <Card padding="none">
          <header class={cn(heading, 'min-h-32')}>
            <div class="flex min-w-0 flex-col items-start gap-2">
              <div class={topline}>
                <Skeleton shape="block" width="5rem" height="1.5rem" />
                <Skeleton shape="text" width="7rem" />
              </div>
              <Skeleton shape="title" width="55%" />
              <Skeleton shape="text" width="42%" />
            </div>
          </header>

          {#each Array(3) as _, index (index)}
            <section class={cn(pageClass, 'flex min-h-40 flex-col gap-4')}>
              <div class="grid grid-cols-[2rem_minmax(0,1fr)_auto] items-center gap-3">
                <Skeleton shape="circle" width="2rem" height="2rem" />
                <div class={copyStack}>
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
    {:else if detailState.kind === 'error'}
      {@render errorState(detailState.code, () => void loadDetail(questionnaire))}
    {:else if detailState.kind === 'ready'}
      <Card padding="none">
        <header class={heading}>
          <div class="flex min-w-0 flex-col items-start gap-2">
            <div class={topline}>
              <Badge tone={detailState.detail.completed ? 'success' : 'warning'}>
                {#if detailState.detail.completed}
                  <CheckCircle2 size={13} aria-hidden="true" />
                {:else}
                  <Clock size={13} aria-hidden="true" />
                {/if}
                {detailState.detail.completed ? copy.completed : copy.pending}
              </Badge>
              {#if questionnaire.deadline}
                <span class={deadline}>
                  <CalendarClock size={14} aria-hidden="true" />
                  {m.questionnaire_deadline({ date: questionnaire.deadline })}
                </span>
              {/if}
            </div>

            <h2 class="text-xl leading-[1.25] font-extrabold wrap-anywhere text-foreground"
              >{info.title}</h2
            >

            {#if info.teacher || info.campaign}
              <div class="mt-1 flex flex-wrap items-center gap-2 text-sm text-muted-foreground">
                {#if info.teacher}
                  <span class={metaItem}>
                    <GraduationCap size={15} aria-hidden="true" />
                    <span>{info.teacher}</span>
                  </span>
                {/if}
                {#if info.campaign}
                  <span
                    class={cn(
                      metaItem,
                      'rounded-sm bg-surface-sunken px-2 py-[0.15rem] text-xs text-muted-foreground'
                    )}
                  >
                    <span>{info.campaign}</span>
                  </span>
                {/if}
              </div>
            {/if}
          </div>
        </header>

        {#each detailState.detail.pages as page, pageIndex (page.id)}
          <section class={pageClass} aria-labelledby={`questionnaire-page-${page.id}`}>
            {#if page.title && page.title !== info.title && page.title !== info.campaign}
              <h3
                class="mb-3 text-lg leading-[1.3] font-bold text-foreground"
                id={`questionnaire-page-${page.id}`}
              >{page.title}</h3>
            {/if}

            <ol class="flex list-none flex-col gap-0" start={pageIndex === 0 ? 1 : undefined}>
              {#each page.questions as question, qIndex (question.id)}
                <li
                  class="grid grid-cols-[minmax(0,1fr)] gap-3 border-t border-border-subtle py-4
                         first:border-t-0 md:grid-cols-[minmax(0,1.35fr)_minmax(15rem,0.65fr)]
                         md:gap-5"
                >
                  <div class={copyStack}>
                    <div class="flex items-start gap-2">
                      <span
                        class="inline-grid size-6 shrink-0 place-items-center rounded-full
                               bg-surface-sunken text-xs font-bold text-muted-foreground"
                        aria-hidden="true">{qIndex + 1}</span
                      >
                      <h4 class="flex-1 text-base leading-[1.4] font-bold wrap-anywhere text-foreground"
                        >{question.title}</h4
                      >
                      <Badge tone={question.required ? 'accent' : 'neutral'}>
                        {question.required ? copy.required : copy.optional}
                      </Badge>
                    </div>
                    {#if question.description}
                      <div
                        class={cn(
                          indent,
                          'rounded-md border-l-3 border-border bg-surface-sunken px-3 py-2'
                        )}
                      >
                        <p class="text-sm leading-[1.5] whitespace-pre-line text-muted-foreground"
                          >{question.description}</p
                        >
                      </div>
                    {/if}
                  </div>

                  <div class={cn(indent, 'min-w-0 md:ml-0')} aria-label={copy.response}>
                    {#if question.kind === 'rating' && question.options.length > 0}
                      <!-- The answer the student submitted, read back. As a plain
                           group a reader heard "1 2 3 4 5" and never which one
                           was chosen: the selection was carried by the fill
                           colour and a hidden icon, which is the colour-only
                           encoding the product forbids, and it lost the answer
                           itself. `radiogroup` + `aria-checked` puts "3, checked,
                           3 of 5" in the accessibility tree instead.

                           No option takes a `tabindex`, not even `-1`. This is a
                           display, not a control: nothing here can change the
                           answer, so nothing should be reachable by Tab, and
                           `-1` would still make each option a scripted focus
                           target and suggest a composite widget with roving
                           focus. Role and state alone are enough — browse mode
                           and touch exploration read them without focus — and
                           `aria-readonly` on the group says the answer is
                           closed. -->
                      <div
                        class="grid max-w-96 grid-cols-5 gap-2"
                        role="radiogroup"
                        aria-readonly="true"
                        aria-label={question.title}
                      >
                        {#each question.options as option (`${question.id}:${option.value}`)}
                          {@const isSelected = isSelectedOption(question, option.value, option.label)}
                          <span
                            role="radio"
                            aria-checked={isSelected}
                            class={cn(
                              'flex min-h-(--tap-min) items-center justify-center gap-1 rounded-md',
                              'border text-sm font-bold tabular-nums',
                              isSelected
                                ? 'border-primary-deep bg-primary-deep text-secondary-foreground shadow-sm'
                                : 'border-border-subtle bg-surface-sunken text-muted-foreground'
                            )}
                          >
                            {#if isSelected}
                              <Check size={14} aria-hidden="true" />
                            {/if}
                            {option.label}
                          </span>
                        {/each}
                      </div>
                    {:else if question.answers.length > 0}
                      <div class="flex flex-col gap-1">
                        <span class="text-xs font-bold text-muted-foreground">{copy.response}</span>
                        <ul
                          class="list-none rounded-md bg-surface-sunken p-3 text-sm leading-[1.5]
                                 text-foreground"
                        >
                          {#each question.answers as answer, answerIndex (`${question.id}:${answerIndex}`)}
                            <li>{answer}</li>
                          {/each}
                        </ul>
                      </div>
                    {:else}
                      <span
                        class="inline-flex min-h-(--tap-min) items-center text-sm italic
                               text-muted-foreground">{copy.noAnswer}</span
                      >
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
{:else if questionnaires.state.kind === 'loading'}
  <div class={column} role="status" aria-live="polite" aria-label={copy.loading}>
    <Card padding="none">
      <header class={heading}>
        <SectionHeader class="w-full" icon={ClipboardList} title={copy.heading} level={3} />
      </header>
      <div class={listBody}>
        {#each Array(3) as _, index (index)}
          <div class={cn(row, 'cursor-default')}>
            <div class={cardContent}>
              <div class={topline}>
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
{:else if questionnaires.state.kind === 'error'}
  {@render errorState(questionnaires.state.code, () => void questionnaires.load(true))}
{:else}
  <Card padding="none">
    <header class={heading}>
      <SectionHeader
        class="w-full"
        icon={ClipboardList}
        title={copy.heading}
        subtitle={m.questionnaires_count({ count: summaries.length })}
        level={3}
      >
        {#snippet actions()}
          <!-- The failed refresh is stated here rather than swallowed: the list
               stays on screen, and the label says when it was actually read. -->
          <FreshnessLabel
            fetchedAt={questionnaires.fetchedAt}
            {locale}
            refreshing={questionnaires.refreshing}
            failed={questionnaires.refreshFailed}
          />

          <div class="desktop-only">
            <IconButton
              label={copy.refresh}
              loading={questionnaires.refreshing}
              onclick={() => void questionnaires.load(true)}
            >
              <RefreshCw size={18} aria-hidden="true" />
            </IconButton>
          </div>
        {/snippet}
      </SectionHeader>
    </header>

    {#if summaries.length === 0}
      <div class="p-4">
        <StateCard
          kind="empty"
          icon={ClipboardList}
          title={copy.emptyHeading}
          description={copy.emptyDescription}
        />
      </div>
    {:else}
      <div class={listBody}>
        {#each summaries as questionnaire (questionnaire.id)}
          {@const info = parseQuestionnaireInfo(questionnaire.title, questionnaire.context)}
          <button
            type="button"
            class={cn(
              row,
              'group cursor-pointer transition-[background] duration-fast ease-out',
              'hover:bg-surface-sunken'
            )}
            onclick={() => void loadDetail(questionnaire)}
          >
            <div class={cardContent}>
              <div class={topline}>
                <Badge tone={questionnaire.completed ? 'success' : 'warning'}>
                  {#if questionnaire.completed}
                    <CheckCircle2 size={12} aria-hidden="true" />
                  {:else}
                    <Clock size={12} aria-hidden="true" />
                  {/if}
                  {statusLabel(questionnaire)}
                </Badge>
                {#if questionnaire.deadline}
                  <span class={deadline}>
                    <CalendarClock size={13} aria-hidden="true" />
                    {m.questionnaire_deadline({ date: questionnaire.deadline })}
                  </span>
                {/if}
              </div>

              <h3 class="text-base leading-[1.35] font-bold wrap-anywhere text-foreground"
                >{info.title}</h3
              >

              <div class="flex flex-wrap items-center gap-2 text-xs text-muted-foreground">
                {#if info.teacher}
                  <span class={metaItem}>
                    <UserRound size={13} aria-hidden="true" />
                    <span>{info.teacher}</span>
                  </span>
                {/if}
                {#if info.campaign}
                  <span class="text-xs text-muted-foreground">{info.campaign}</span>
                {/if}
              </div>
            </div>

            <span class="flex shrink-0 items-center justify-center text-muted-foreground transition-[translate,color] duration-fast ease-out group-hover:translate-x-[3px] group-hover:text-primary-deep" aria-hidden="true">
              <ChevronRight size={18} />
            </span>
          </button>
        {/each}
      </div>
    {/if}
  </Card>
{/if}

