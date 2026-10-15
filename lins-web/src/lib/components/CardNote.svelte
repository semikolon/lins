<!--
  What a card's view says about it, on a plate beside the card, a moment after the pointer rests on
  it. The lines are the properties the label's `tooltip` setting names (lib/view, `tooltipLines`);
  each line of a property is a part of its own, cut at a few lines here, and the whole of every
  property is in the side panel when the card is opened.

  The plate is placed against the window, from the card's own box, so it reads the same in lanes
  that scroll and in a scene that is moved and zoomed. Whoever shows it hides it when the card moves.
-->
<script lang="ts">
  let { lines, rect }: { lines: string[]; rect: DOMRect | null } = $props();

  /** How long the pointer rests on a card before its note shows. */
  const DELAY_MS = 350;
  const WIDTH = 340;
  const GAP = 8;

  let shown = $state(false);
  let plate = $state<HTMLDivElement | null>(null);
  /** The note is longer than the room it was given, so its foot fades; the panel holds the rest. */
  let cut = $state(false);

  // The wait lives as long as one resting of the pointer: a new card, or none, ends it.
  $effect(() => {
    void lines;
    shown = false;
    if (!rect || lines.length === 0) return;
    const timer = setTimeout(() => (shown = true), DELAY_MS);
    return () => clearTimeout(timer);
  });

  /** Below the card or above it, on the side with more room, and never past the window's edges. */
  const place = $derived.by(() => {
    if (!rect) return '';
    const width = Math.min(WIDTH, window.innerWidth - 2 * GAP);
    const left = Math.max(GAP, Math.min(rect.left, window.innerWidth - width - GAP));
    const roomBelow = window.innerHeight - rect.bottom - 2 * GAP;
    const roomAbove = rect.top - 2 * GAP;
    const below = roomBelow >= roomAbove || roomBelow >= 320;
    const y = below ? `top:${rect.bottom + GAP}px` : `bottom:${window.innerHeight - rect.top + GAP}px`;
    return `left:${left}px;${y};width:${width}px;max-height:${Math.max(80, below ? roomBelow : roomAbove)}px`;
  });

  $effect(() => {
    void place;
    cut = !!plate && plate.scrollHeight > plate.clientHeight + 1;
  });
</script>

{#if shown && rect}
  <div class="note" class:cut role="tooltip" style={place} bind:this={plate}>
    {#each lines as line, i (i)}
      <p>{line}</p>
    {/each}
  </div>
{/if}

<style>
  .note {
    position: fixed;
    z-index: 50;
    overflow: hidden;
    padding: 10px 12px;
    border-radius: 8px;
    border: 1px solid rgba(169, 179, 198, 0.28);
    background: #20283a;
    box-shadow: 0 6px 24px rgba(0, 0, 0, 0.45);
    color: #dfe4ee;
    font: 400 12.5px/1.5 var(--font-ui, system-ui, sans-serif);
    pointer-events: none;
  }
  .note.cut {
    mask-image: linear-gradient(to bottom, #000 calc(100% - 32px), transparent);
  }
  /* Each part is cut at a few lines; the side panel holds the rest. */
  p {
    margin: 0;
    display: -webkit-box;
    -webkit-box-orient: vertical;
    -webkit-line-clamp: 5;
    line-clamp: 5;
    overflow: hidden;
    overflow-wrap: anywhere;
  }
  p + p {
    margin-top: 6px;
  }
</style>
