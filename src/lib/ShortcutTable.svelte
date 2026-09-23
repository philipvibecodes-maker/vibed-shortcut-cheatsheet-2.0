<script lang="ts">
  import type { Shortcut } from "./types";

  let { displayName, shortcuts }: { displayName: string; shortcuts: Shortcut[] } =
    $props();
</script>

<div class="title">{displayName}</div>
<table>
  <tbody>
    {#each shortcuts as shortcut (shortcut.action)}
      <tr>
        <td class="action">{shortcut.action}</td>
        <td class="keys"><kbd>{shortcut.keys}</kbd></td>
      </tr>
    {/each}
  </tbody>
</table>

<style>
  .title {
    color: var(--fg-dim);
    font-size: 0.8em;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    padding-bottom: 0.5em;
  }

  table {
    border-collapse: collapse;
    white-space: nowrap;
  }

  td {
    padding: 0.15em 0;
  }

  .action {
    padding-right: 1.5em;
  }

  .keys {
    text-align: right;
  }

  kbd {
    background: var(--key-bg);
    border: 1px solid var(--border);
    border-radius: 4px;
    padding: 0.1em 0.45em;
    font-family: ui-monospace, monospace;
    font-size: 0.9em;
  }
</style>
