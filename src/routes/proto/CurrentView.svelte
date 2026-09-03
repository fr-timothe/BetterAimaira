<script lang="ts">
  /**
   * DEV-ONLY PROTOTYPE — the incumbent, for comparison.
   *
   * The shipped `CalendarView`, unchanged, fed the same demo week and framed
   * in the same phone chrome as the three candidates. Without it the
   * candidates are only compared to each other.
   */
  import { onMount } from 'svelte';
  import CalendarView from '$lib/features/schedule/CalendarView.svelte';
  import ProtoShell from './ProtoShell.svelte';
  import { demoEvents, demoFetchedAt } from './demo';

  const reference = new Date();
  const events = demoEvents(reference);
  const fetchedAt = demoFetchedAt(reference);

  let now = $state(new Date());

  onMount(() => {
    const timer = setInterval(() => (now = new Date()), 30_000);
    return () => clearInterval(timer);
  });
</script>

<ProtoShell>
  <CalendarView {events} locale="fr" initialScope="day" {now} {fetchedAt} />
</ProtoShell>
