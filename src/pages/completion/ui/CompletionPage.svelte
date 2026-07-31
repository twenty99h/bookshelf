<script lang="ts">
  import { AppWorkspace } from "@/features/workspace";
  let { bookId }: { bookId?: string } = $props();
</script>

<AppWorkspace context="completion" resourceId={bookId} />
