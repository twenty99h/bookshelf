<script lang="ts">
  import { page } from "$app/state";
  import { AppWorkspace } from "@/pages/app-workspace";
</script>

<AppWorkspace context="completion" resourceId={page.params.bookId} />
