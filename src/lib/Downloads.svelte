<svelte:options immutable />

<script lang="ts">
  import { onDestroy, onMount, tick } from "svelte";
  import { invoke } from "@tauri-apps/api/tauri";
  import { listen, type UnlistenFn, type Event } from "@tauri-apps/api/event";
  import { BarLoader } from "svelte-loading-spinners";
  import { fly } from "svelte/transition";
  import { flip } from "svelte/animate";

  import { DownloadStatus, type DownloadOutput } from "./Downloads";
  import DownloadForm from "./DownloadForm.svelte";
  import DownloadItem from "./DownloadItem.svelte";

  let downloads: DownloadOutput[] = [];
  let adding = 0;
  let downloading = 0;
  let downloadsList: HTMLElement;
  $: loading = adding + downloading > 0;
  let unlisten: UnlistenFn;
  let errorMessage: string | null = null;

  onMount(async () => {
    try {
      downloads = await invoke("get_downloads");
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

  function updateDownloads(e: Event<DownloadOutput>) {
    const download = e.payload;
    if (download.status != DownloadStatus.Downloading) {
      downloading -= 1;
    }
    downloads = downloads.map((d) => {
      if (d.id === download.id) {
        return download;
      } else {
        return d;
      }
    });
  }

  async function addDownload(
    event: CustomEvent<{ download: DownloadOutput; callback: () => void }>
  ) {
    adding += 1;
    try {
      downloads = await invoke("add_download", {
        downloadInput: event.detail.download,
      });
      event.detail.callback();
      await tick();
      scrollToBottom(downloadsList);
    } catch (error) {
      errorMessage = error as string;
    }
    adding -= 1;
  }

  async function saveDownloadEdit(
    event: CustomEvent<{ download: DownloadOutput }>
  ) {
    try {
      downloads = await invoke("update_download", {
        download: event.detail.download,
      });
    } catch (error) {
      errorMessage = error as string;
    }
  }

  async function removeDownload(event: CustomEvent<{ id: number }>) {
    try {
      downloads = await invoke("remove_download", {
        id: event.detail.id,
      });
    } catch (error) {
      errorMessage = error as string;
    }
  }

  async function clearDownloads() {
    try {
      downloads = await invoke("clear_downloads");
    } catch (e) {
      errorMessage = e as string;
    }
  }

  async function removeCompleted() {
    try {
      downloads = await invoke("remove_completed");
    } catch (e) {
      errorMessage = e as string;
    }
  }

  async function downloadSingle(
    event: CustomEvent<{ download: DownloadOutput }>
  ) {
    downloading += 1;
    try {
      await invoke("queue_download", {
        download: event.detail.download,
      });
    } catch (error) {
      errorMessage = error as string;
    }
  }

  async function downloadAll() {
    try {
      await invoke("queue_downloads");
    } catch (e) {
      errorMessage = e as string;
    }
  }
</script>

<div class="downloads">
  <DownloadForm on:add={addDownload} />
  <div class="failure-wrapper {errorMessage ? 'error' : ''}">
    {#if errorMessage}
      <button
        in:fly={{ y: 100 }}
        out:fly={{ y: 100 }}
        type="button"
        on:click={clearError}
        class="failure-message"
        title="Dismiss error"
      >
        {errorMessage}
      </button>
    {/if}
  </div>
  <div class="status-wrapper {loading ? 'loading' : ''}">
    {#if loading}
      <div in:fly={{ y: 20, duration: 200 }} out:fly={{ y: 20, duration: 200 }}>
        <BarLoader size="4" unit="rem" />
      </div>
    {/if}
  </div>
  <ul bind:this={downloadsList} class="downloads-list">
    {#each downloads as download (download.id)}
      <li
        in:fly={{ x: -100 }}
        out:fly={{ x: 100 }}
        animate:flip={{ duration: 200 }}
      >
        <DownloadItem
          {download}
          on:save={saveDownloadEdit}
          on:remove={removeDownload}
          on:download={downloadSingle}
        />
      </li>
    {:else}
      <h2>No downloads added yet...</h2>
    {/each}
  </ul>
  <div class="actions">
    <button on:click={clearDownloads} disabled={loading}
      ><span class="clear-downloads">Clear Downloads</span></button
    >
    <button on:click={removeCompleted} disabled={loading}
      >Remove Completed</button
    >
    <button on:click={downloadAll} disabled={loading}>Download All</button>
  </div>
</div>

<style>
  .downloads {
    display: flex;
    flex-direction: column;
    align-items: center;
    flex: 1;
    overflow: hidden;
  }

  .downloads-list {
    display: flex;
    flex-direction: column;
    align-items: center;
    overflow-x: hidden;
    padding-left: 0;
    margin-bottom: 1rem;
    flex: 1;
    gap: 8px;
    width: 100%;
  }

  .actions {
    display: flex;
    gap: 0.5rem;
    justify-content: center;
    align-items: center;
    flex-wrap: wrap;
  }

  .failure-message {
    margin: 0.5rem 0;
    color: #ba3329;
  }

  .failure-wrapper,
  .status-wrapper {
    height: 0;
    transition: height 0.2s;
  }

  .failure-wrapper.error {
    height: 4rem;
  }

  .status-wrapper.loading {
    height: 0.25rem;
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
    &:hover {
      background-color: #0f0f0f98;
    }
  }

  .clear-downloads {
    color: #ba3329;
  }

  h2 {
    font-size: 1.6rem;
    font-weight: 400;
    line-height: 1.4;
    letter-spacing: -0.5px;
  }
</style>
