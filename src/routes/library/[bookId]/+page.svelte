<script lang="ts">
  import { page } from "$app/state";
  import { BookPage } from "@/pages/book";
</script>

<BookPage bookId={page.params.bookId} />
