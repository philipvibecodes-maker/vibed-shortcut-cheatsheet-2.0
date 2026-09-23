<script lang="ts">
  import { onMount } from "svelte";
  import ShortcutTable from "./lib/ShortcutTable.svelte";
  import { getState, onUpdate, reportContentSize } from "./lib/ipc";
  import { handleKeydown } from "./lib/keys";
  import type { CheatsheetState } from "./lib/types";

  let state = $state<CheatsheetState>({
    displayName: "",
    shortcuts: [],
    fontSize: 14,
  });
  let frame: HTMLDivElement;

  onMount(() => {
    void getState().then((s) => (state = s));
    const unlisten = onUpdate((s) => (state = s));

    const observer = new ResizeObserver(([entry]) => {
      const { width, height } = entry.contentRect;
      void reportContentSize(Math.ceil(width), Math.ceil(height));
    });
    observer.observe(frame);

    return () => {
      observer.disconnect();
      void unlisten.then((fn) => fn());
    };
  });
</script>

<svelte:window on:keydown={handleKeydown} on:contextmenu|preventDefault />

<div
  bind:this={frame}
  class="frame"
  style:font-size="{state.fontSize}px"
>
  <ShortcutTable displayName={state.displayName} shortcuts={state.shortcuts} />
</div>

<style>
  .frame {
    display: inline-block;
    background: var(--bg);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    padding: 0.75em 1em;
    box-sizing: border-box;
  }
</style>
