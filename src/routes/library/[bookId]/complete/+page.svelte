<script lang="ts">
  import { page } from "$app/state";
  import { WorkspacePage } from "@/pages/workspace";
</script>

<WorkspacePage context="completion" resourceId={page.params.bookId} />
