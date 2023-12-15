<script lang="ts">
  import Downloads from "./lib/Downloads.svelte";
</script>

<main class="container">
  <h1><span class="reddit">Reddit</span> Audio Downloader</h1>
  <Downloads />
</main>

<style>
  .reddit {
    color: var(--color-primary);
  }

  h1 {
    margin-top: 2rem;
    font-size: 2rem;
    font-weight: 400;
    letter-spacing: -1px;
    line-height: 1;
  }
</style>
