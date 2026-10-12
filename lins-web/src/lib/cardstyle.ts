/**
 * The sizes and colours of a card, as CSS variables, for the views that draw cards themselves.
 * One list for every such view, so a card looks the same in a lane and around a node. The values
 * are the scene's (lib/trial/scene.ts), which the trial page reads too.
 */
import { STYLE } from '$lib/trial/scene';

export const CARD_VARS = [
  `--canvas:${STYLE.canvas}`,
  `--card:${STYLE.card}`,
  `--card-hover:${STYLE.card_hover}`,
  `--card-focus:${STYLE.card_focus}`,
  `--ink:${STYLE.text}`,
  `--ink-muted:${STYLE.muted}`,
  `--card-w:${STYLE.card_w}px`,
  `--focus-w:${STYLE.focus_w}px`,
  `--pad-x:${STYLE.pad_x}px`,
  `--pad-y:${STYLE.pad_y}px`,
  `--radius:${STYLE.radius}px`,
  `--stripe:${STYLE.stripe}px`,
  `--name:500 ${STYLE.font}px/${STYLE.line}px var(--font-ui)`,
  `--focus-name:600 ${STYLE.focus_font}px/${STYLE.focus_line}px var(--font-ui)`,
  `--small:${STYLE.small_font}px`,
  `--max-lines:${STYLE.max_lines}`,
  `--focus-max-lines:${STYLE.max_lines + 2}`,
  `--header-h:${STYLE.header_h}px`,
  `--stub-h:${STYLE.stub_h}px`,
].join(';');

/**
 * A card's background when part of it is done: the done share in a faint wash of the card's own
 * colour, the rest plain. `share` is a whole percent; with none, the card is plain.
 */
export function fillBackground(share: number | null): string {
  if (share === null) return '';
  return `--share:${share}%`;
}
