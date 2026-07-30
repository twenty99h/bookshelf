<script lang="ts">
  import { X } from "@lucide/svelte";
  import { Dialog } from "bits-ui";
  import type { Snippet } from "svelte";

  let {
    open = $bindable(false),
    title,
    description,
    trigger,
    children,
  }: {
    open?: boolean;
    title: string;
    description: string;
    trigger: Snippet;
    children: Snippet;
  } = $props();
</script>

<Dialog.Root bind:open>
  <Dialog.Trigger
    class="inline-flex min-h-11 items-center justify-center rounded-lg px-4 py-2.5 font-semibold outline-none focus-visible:ring-3 focus-visible:ring-focus"
  >
    {@render trigger()}
  </Dialog.Trigger>
  <Dialog.Portal>
    <Dialog.Overlay class="fixed inset-0 z-40 bg-black/55" />
    <Dialog.Content
      class="fixed left-1/2 top-1/2 z-50 grid max-h-[calc(100vh-2rem)] w-[min(34rem,calc(100vw-2rem))] -translate-x-1/2 -translate-y-1/2 gap-4 overflow-auto rounded-xl border border-rule bg-paper-raised p-6 shadow-paper outline-none"
    >
      <Dialog.Title class="pr-10 font-display text-2xl font-medium">{title}</Dialog.Title>
      <Dialog.Description class="text-sm text-ink-muted">{description}</Dialog.Description>
      {@render children()}
      <Dialog.Close
        aria-label="Закрыть"
        class="absolute right-4 top-4 grid size-10 place-items-center rounded-lg outline-none hover:bg-slate-100 focus-visible:ring-3 focus-visible:ring-focus"
      >
        <X class="size-5" aria-hidden="true" />
      </Dialog.Close>
    </Dialog.Content>
  </Dialog.Portal>
</Dialog.Root>
