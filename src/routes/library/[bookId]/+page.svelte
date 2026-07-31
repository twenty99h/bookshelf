<script lang="ts">
  import { page } from "$app/state";
  import { WorkspacePage } from "@/pages/workspace";
</script>

<WorkspacePage context="book" resourceId={page.params.bookId} />
