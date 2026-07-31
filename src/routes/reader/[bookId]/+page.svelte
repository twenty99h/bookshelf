<script lang="ts">
  import { page } from "$app/state";
  import { ReaderPage } from "@/pages/reader";
</script>

<ReaderPage bookId={page.params.bookId} />
