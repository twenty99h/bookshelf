<script lang="ts">
  import { page } from "$app/state";
  import { WorkspacePage } from "@/pages/workspace";
</script>

<WorkspacePage context="idea" resourceId={page.params.ideaId} />
