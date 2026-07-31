<script lang="ts">
  import { AppWorkspace } from "@/features/workspace";
  let { ideaId }: { ideaId?: string } = $props();
</script>

<AppWorkspace context="idea" resourceId={ideaId} />
