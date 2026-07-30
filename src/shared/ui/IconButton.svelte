<script lang="ts">
  import { Tooltip } from "bits-ui";
  import type { Snippet } from "svelte";

  let {
    label,
    icon,
    onclick,
    disabled = false,
  }: { label: string; icon: Snippet; onclick?: () => void; disabled?: boolean } = $props();
</script>

<Tooltip.Provider delayDuration={250}>
  <Tooltip.Root>
    <Tooltip.Trigger
      aria-label={label}
      {onclick}
      {disabled}
      class="grid size-11 place-items-center rounded-lg border border-rule bg-paper-raised text-ink outline-none hover:bg-leaf-soft focus-visible:ring-3 focus-visible:ring-focus disabled:opacity-50"
    >
      {@render icon()}
    </Tooltip.Trigger>
    <Tooltip.Portal>
      <Tooltip.Content
        role="tooltip"
        sideOffset={6}
        class="z-50 rounded-md bg-spine px-2.5 py-1.5 text-xs font-medium text-white shadow-paper"
      >
        {label}
      </Tooltip.Content>
    </Tooltip.Portal>
  </Tooltip.Root>
</Tooltip.Provider>
