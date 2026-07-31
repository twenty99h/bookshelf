<script lang="ts">
  import { page } from "$app/state";
  import { WorkspacePage } from "@/pages/workspace";
</script>

<WorkspacePage context="reader" resourceId={page.params.bookId} />
