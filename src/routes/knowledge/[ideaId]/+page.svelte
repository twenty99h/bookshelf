<script lang="ts">
  import { page } from "$app/state";
  import { IdeaPage } from "@/pages/idea";
</script>

<IdeaPage ideaId={page.params.ideaId} />
