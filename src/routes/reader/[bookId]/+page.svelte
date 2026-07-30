<script lang="ts">
  import { page } from "$app/state";
  import { AppWorkspace } from "@/pages/app-workspace";
</script>

<AppWorkspace context="reader" resourceId={page.params.bookId} />
