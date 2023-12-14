<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import Icon from "@iconify/svelte";

  import type { DownloadInput } from "./Downloads";

  const dispatch = createEventDispatcher();

  let url = "";
  let op = "";
  let sub = "GWA";

  function emitDownload() {
    const download: DownloadInput = { url, op, sub };
    dispatch("add", {
      download,
      callback: () => {
        url = "";
      },
    });
  }
</script>

<div>
  <form class="download-input" on:submit|preventDefault={emitDownload}>
    <div class="form-group">
      <label for="input-url">URL</label>
      <input
        id="input-url"
        type="text"
        placeholder="URL"
        bind:value={url}
        required
        autocomplete="off"
      />
    </div>
    <div class="form-group">
      <label for="input-op">Original Poster</label>
      <input
        id="input-op"
        type="text"
        placeholder="Original Poster"
        bind:value={op}
        required
        autocomplete="on"
      />
    </div>
    <div class="form-group">
      <label for="input-sub">Subreddit</label>
      <input
        list="subreddits"
        id="input-sub"
        type="text"
        placeholder="Subreddit"
        bind:value={sub}
        required
      />
      <datalist id="subreddits">
        <option value="GWA"></option>
        <option value="PTA"></option>
      </datalist>
    </div>
    <button title="Add download" type="submit" class="form-button"
      ><Icon icon="material-symbols:add-circle-outline-rounded" /></button
    >
  </form>
</div>

<style lang="scss">
  .download-input {
    margin: 1rem 1.5rem;
    display: flex;
    gap: 0.5rem;
    justify-content: center;
    align-items: flex-end;
    flex-wrap: wrap;

    & input,
    & label {
      font-size: 0.9rem;
      font-weight: 700;
    }

    & label {
      margin-left: 1rem;
      margin-bottom: 0.2rem;
      display: block;
      text-align: start;
      transition: all 0.3s;
    }
  }

  .form-button {
    line-height: 0rem;
    padding: 0.5rem;
  }
</style>
