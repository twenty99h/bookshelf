<script lang="ts">
  import { page } from "$app/state";
  import { CompletionPage } from "@/pages/completion";
</script>

<CompletionPage bookId={page.params.bookId} />
