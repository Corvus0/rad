<svelte:options immutable />

<script lang="ts">
  import { onDestroy, onMount, tick } from "svelte";
  import { invoke } from "@tauri-apps/api/tauri";
  import { listen, type UnlistenFn, type Event } from "@tauri-apps/api/event";
  import { open } from "@tauri-apps/api/dialog";
  import { open as openShell } from "@tauri-apps/api/shell";
  import { BarLoader } from "svelte-loading-spinners";
  import { fly } from "svelte/transition";
  import { flip } from "svelte/animate";
  import Icon from "@iconify/svelte";

  import { DownloadStatus, type DownloadOutput } from "./Downloads";
  import DownloadForm from "./DownloadForm.svelte";
  import DownloadItem from "./DownloadItem.svelte";

  let downloads: DownloadOutput[] = [];
  let adding = 0;
  let downloading = 0;
  let queued = 0;
  let downloadsList: HTMLElement;
  $: loading = adding + downloading > 0;
  let unlisten: UnlistenFn;
  let errorMessage: string | null = null;
  let directory = "";

  onMount(async () => {
    try {
      downloads = await invoke("get_downloads");
      directory = await invoke("get_directory");
    } catch (e) {
      errorMessage = e as string;
    }
    unlisten = await listen<DownloadOutput>(
      "update_downloads",
      updateDownloads
    );
  });

  onDestroy(() => {
    unlisten();
  });

  const scrollToBottom = async (node: HTMLElement) => {
    node.scroll({ top: node.scrollHeight, behavior: "smooth" });
  };

  function clearError() {
    errorMessage = null;
  }

  async function chooseDirectory() {
    try {
      const newDirectory = await open({
        directory: true,
        defaultPath: directory,
      });
      if (typeof newDirectory === "string") {
        await invoke("set_directory", { directory: newDirectory });
        directory = newDirectory;
      }
    } catch (error) {
      errorMessage = error as string;
    }
  }

  async function openDirectory() {
    try {
      await openShell(directory);
    } catch (error) {
      errorMessage = error as string;
    }
  }

  function updateDownloads(e: Event<DownloadOutput>) {
    const download = e.payload;
    if (download.status != DownloadStatus.Downloading) {
      downloading -= 1;
    }
    if (downloading === 0) queued = 0;
    downloads = downloads.map((d) => (d.id === download.id ? download : d));
  }

  async function addDownload(
    event: CustomEvent<{ download: DownloadOutput; callback: () => void }>
  ) {
    adding += 1;
    try {
      const download: DownloadOutput = await invoke("add_download", {
        downloadInput: event.detail.download,
      });
      downloads = downloads.concat(download);
      event.detail.callback();
      await tick();
      scrollToBottom(downloadsList);
    } catch (error) {
      errorMessage = error as string;
    }
    adding -= 1;
  }

  async function saveDownloadEdit(
    event: CustomEvent<{ download: DownloadOutput; callback: () => void }>
  ) {
    try {
      const download: DownloadOutput = await invoke("update_download", {
        download: event.detail.download,
      });
      event.detail.callback();
      downloads = downloads.map((d) => (d.id === download.id ? download : d));
    } catch (error) {
      errorMessage = error as string;
    }
  }

  async function removeDownload(event: CustomEvent<{ id: number }>) {
    try {
      await invoke("remove_download", {
        id: event.detail.id,
      });
      downloads = downloads.filter((d) => d.id !== event.detail.id);
    } catch (error) {
      errorMessage = error as string;
    }
  }

  async function clearDownloads() {
    try {
      await invoke("clear_downloads");
      downloads = [];
    } catch (e) {
      errorMessage = e as string;
    }
  }

  async function removeDownloaded() {
    try {
      await invoke("remove_completed");
      downloads = downloads.filter(
        (d) => d.status !== DownloadStatus.Completed
      );
    } catch (e) {
      errorMessage = e as string;
    }
  }

  async function downloadSingle(event: CustomEvent<{ id: number }>) {
    downloading += 1;
    queued += 1;
    try {
      await invoke("queue_download", {
        id: event.detail.id,
      });
    } catch (error) {
      errorMessage = error as string;
    }
  }

  async function downloadAll() {
    try {
      await invoke("queue_downloads");
      downloading = downloads.length;
      queued = downloads.length;
    } catch (e) {
      errorMessage = e as string;
    }
  }
</script>

<div class="downloads">
  <h1><span class="reddit">Reddit</span> Audio Downloader</h1>
  <DownloadForm on:add={addDownload} />
  <div class="progress-wrapper{loading ? ' loading' : ''}">
    {#if loading}
      <div class="progress-bar" transition:fly={{ y: -20 }}>
        <span>
          {adding > 0 ? adding : downloading > 0 ? queued - downloading : ""}
          {downloading > 0 ? `of ${queued}` : ""}
          {adding + queued === 1 ? "Link" : "Links"}
          {adding > 0 ? "Added" : "Downloaded"}
        </span>
        <BarLoader color="var(--color-primary)" size="5" unit="rem" />
      </div>
    {/if}
  </div>
  <div class="error-wrapper{errorMessage ? ' error' : ''}">
    {#if errorMessage}
      <button
        transition:fly={{ y: -50 }}
        type="button"
        on:click={clearError}
        class="failure-message"
        title="Dismiss error"
      >
        {errorMessage}
      </button>
    {/if}
  </div>
  <ul bind:this={downloadsList} class="downloads-list">
    {#each downloads as download (download.id)}
      <li
        in:fly={{ x: -100 }}
        out:fly={{ x: 100 }}
        animate:flip={{ duration: 400 }}
      >
        <DownloadItem
          {download}
          on:save={saveDownloadEdit}
          on:remove={removeDownload}
          on:download={downloadSingle}
        />
      </li>
    {:else}
      <li class="list-placeholder" in:fly={{ x: -100 }} out:fly={{ x: 100 }}>
        <h2>No downloads added yet...</h2>
      </li>
    {/each}
  </ul>
  <div class="actions">
    <div class="actions__group">
      <button class="download-all" on:click={downloadAll} disabled={loading}
        >Download All</button
      >
      <button on:click={removeDownloaded} disabled={loading}
        >Remove Downloaded</button
      >
      <button
        class="clear-downloads"
        on:click={clearDownloads}
        disabled={loading}>Clear Downloads</button
      >
    </div>
    <div class="actions__group actions__group--end">
      <button
        class="directory"
        title="Choose download directory"
        on:click={chooseDirectory}><span>{directory}</span></button
      >
      <button
        title="Open download directory"
        on:click={openDirectory}
        disabled={loading}
      >
        <Icon icon="material-symbols:folder-open-outline-rounded" /></button
      >
    </div>
  </div>
</div>

<style lang="scss">
  .downloads {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    flex: 1;
    overflow: hidden;
    padding: 2rem 4rem;
    min-width: 400px;
  }

  .downloads-list {
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: 1rem;
    margin-bottom: 1rem;
    flex: 1;
    gap: 8px;
    width: 100%;
    color: var(--color-on-surface);
    background-color: var(--color-surface-alt);
    border-radius: 1rem;
    overflow: scroll;
    position: relative;
  }

  .list-placeholder {
    position: absolute;
  }

  .actions {
    display: flex;
    justify-content: space-between;
    flex-wrap: wrap;
    width: 100%;
    gap: 0.5rem;

    &__group {
      display: flex;
      gap: 0.5rem;

      &--end {
        justify-content: flex-end;
      }
    }
  }

  .directory {
    max-width: 25rem;
    overflow: hidden;
    white-space: nowrap;
    text-wrap: nowrap;
    text-overflow: ellipsis;
    direction: rtl;
  }

  .progress-wrapper {
    max-height: 0;
    transition: max-height 0.4s ease-in-out;

    &.loading {
      max-height: 100%;
    }
  }

  .progress-bar {
    color: var(--color-on-tertiary-container);
    background-color: var(--color-tertiary-container);
    padding: 0.5rem 1rem;
    margin-bottom: 1rem;
    border-radius: 1rem;

    & span {
      display: inline-block;
      margin-bottom: 0.1rem;
    }
  }

  .error-wrapper {
    max-height: 0;
    transition: max-height 0.4s ease-in-out;

    &.error {
      max-height: 100%;
    }
  }

  .failure-message {
    color: var(--color-on-error-container);
    background-color: var(--color-error-container);
    padding: 0.5rem 1rem;
    margin-bottom: 1rem;
    border: none;
    text-align: start;
  }

  li {
    list-style: none;
    width: 100%;
  }

  button:disabled {
    &:hover {
      cursor: not-allowed;
      border-color: transparent;
    }
  }

  .download-all {
    color: var(--color-on-tertiary-container);
    background-color: var(--color-tertiary-container);
    border: none;
  }

  .clear-downloads {
    color: var(--color-error);
  }

  .reddit {
    color: var(--color-primary);
  }

  h1 {
    margin: 0;
    font-size: 2.8rem;
    font-weight: 400;
    letter-spacing: -1.6px;
    line-height: 1;
  }

  h2 {
    font-size: 2rem;
    font-weight: 400;
    line-height: 1.2;
    letter-spacing: -0.8px;
    margin-left: 1rem;
  }
</style>
