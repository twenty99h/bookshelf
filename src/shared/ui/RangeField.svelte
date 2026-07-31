<script lang="ts">
  let {
    id,
    label,
    value = $bindable(),
    min,
    max,
    step = 1,
    onCommit,
  }: {
    id: string;
    label: string;
    value: number;
    min: number;
    max: number;
    step?: number;
    onCommit?: () => void;
  } = $props();
</script>

<label class="block text-[10px] text-mist-faint" for={id}>{label}</label>
<input {id} aria-label={label} type="range" {min} {max} {step} bind:value onchange={onCommit} />
