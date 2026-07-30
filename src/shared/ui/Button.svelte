<script lang="ts">
  import { Button } from "bits-ui";
  import type { Snippet } from "svelte";

  type Variant = "primary" | "secondary";

  let {
    children,
    variant = "secondary",
    type = "button",
    disabled = false,
    class: className = "",
    ...restProps
  }: {
    children: Snippet;
    variant?: Variant;
    type?: "button" | "submit" | "reset";
    disabled?: boolean;
    class?: string;
    [key: string]: unknown;
  } = $props();

  const variants: Record<Variant, string> = {
    primary: "bg-iris-strong text-white hover:bg-[#4d48aa]",
    secondary: "border border-white/10 bg-spine-raised text-mist hover:bg-[#303d50]",
  };
</script>

<Button.Root
  {type}
  {disabled}
  class={`inline-flex min-h-11 items-center justify-center rounded-lg px-4 py-2.5 font-semibold transition-colors focus-visible:outline-none focus-visible:ring-3 focus-visible:ring-focus disabled:cursor-not-allowed disabled:opacity-50 ${variants[variant]} ${className}`}
  {...restProps}
>
  {@render children()}
</Button.Root>
