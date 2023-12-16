<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import { fly } from "svelte/transition";
  import Icon from "@iconify/svelte";
  import { Moon } from "svelte-loading-spinners";

  import { type DownloadOutput, DownloadStatus } from "./Downloads";

  export let download: DownloadOutput;

  const dispatch = createEventDispatcher();

  let editing = false;
  let fields: DownloadOutput = { ...download };

  function switchToEditing() {
    editing = true;
  }

  function cancelEditing() {
    editing = false;
  }

  function emitEdit() {
    dispatch("save", {
      download: { ...fields, status: DownloadStatus.Initial, failure: null },
      callback: () => {
        editing = false;
      },
    });
  }

  function emitRemove() {
    dispatch("remove", {
      id: download?.id,
    });
  }

  function emitDownload() {
    dispatch("download", {
      download,
    });
  }
</script>

<div
  class="download-item {editing ? 'editing' : ''} {download.status ===
  DownloadStatus.Failed
    ? 'failed'
    : ''}"
>
  {#if download.status === DownloadStatus.Completed}
    <Icon icon="material-symbols:download-done-rounded" />
  {:else if download.status === DownloadStatus.Downloading}
    <Moon color="var(--color-on-tertiary-container" size="1.2" unit="rem" />
  {:else if download.status === DownloadStatus.Failed}
    <Icon icon="material-symbols:error-outline-rounded" />
  {/if}
  <div class="download-item__lhs">
    {#if editing && download.status !== DownloadStatus.Downloading}
      <form
        transition:fly={{ y: -70 }}
        class="download-item__form"
        id={`download-item-${download.id}`}
        on:submit|preventDefault={emitEdit}
      >
        <label class="download-url">
          <span class="label-text"> URL </span>
          <input
            type="text"
            placeholder="URL"
            bind:value={fields.input.url}
            required
            autocomplete="off"
          />
        </label>
        <label class="download-op">
          <span class="label-text"> Original Poster </span>
          <input
            type="text"
            placeholder="Original Poster"
            bind:value={fields.input.op}
            required
          />
        </label>
        <label class="download-sub">
          <span class="label-text"> Subreddit </span>
          <input
            list="subreddits"
            type="text"
            placeholder="Subreddit"
            bind:value={fields.input.sub}
            required
          />
        </label>
        <label class="download-title">
          <span class="label-text"> Title </span>
          <input
            type="text"
            placeholder="Title"
            bind:value={fields.title}
            required
            autocomplete="off"
          />
        </label>
      </form>
    {:else}
      <span class="download-item__text" transition:fly={{ y: 70 }}>
        <a href={download.input.url} class="download-item__link" target="_blank"
          >[{download.input.sub}] [{download.input.op}] {download.title}</a
        >
        {#if download.status === DownloadStatus.Failed}
          {download.failure}
        {/if}
      </span>
    {/if}
  </div>
  {#if download.status !== DownloadStatus.Downloading}
    <div class="item-actions">
      {#if download.status === DownloadStatus.Initial && !editing}
        <button
          class="download-item__button download-item__button--download"
          type="button"
          title="Download"
          on:click={emitDownload}
          ><Icon icon="material-symbols:download-rounded" /></button
        >
      {/if}
      {#if editing}
        <button
          class="download-item__button download-item__button--save"
          type="submit"
          form={`download-item-${download.id}`}
          title="Save"
          ><Icon icon="material-symbols:save-outline-rounded" /></button
        >
        <button
          class="download-item__button download-item__button--cancel"
          type="button"
          title="Cancel"
          on:click={cancelEditing}
          ><Icon icon="material-symbols:cancel-outline-rounded" /></button
        >
      {:else}
        <button
          class="download-item__button download-item__button--edit"
          type="button"
          title="Edit"
          on:click={switchToEditing}
          ><Icon icon="material-symbols:edit" /></button
        >
      {/if}
      <button
        class="download-item__button download-item__button--remove"
        type="button"
        title="Remove"
        on:click={emitRemove}
        ><Icon icon="material-symbols:delete-outline-rounded" /></button
      >
    </div>
  {/if}
</div>

<style lang="scss">
  .download-item {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    gap: 8px;
    flex-wrap: wrap;
    color: var(--color-on-tertiary-container);
    background-color: var(--color-tertiary-container);
    padding: 0.5rem 1rem;
    border-radius: 1rem;
    transition: all 0.2s;
    height: 4rem;
    overflow: hidden;

    &.editing {
      height: 5rem;
    }

    &.failed {
      color: var(--color-on-error-container);
      background-color: var(--color-error-container);
    }

    &__lhs {
      display: flex;
      gap: 8px;
      flex: 1;
      position: relative;
    }

    &__text {
      position: absolute;
    }

    &__link {
      font-weight: 500;
      color: inherit;
      text-decoration: inherit;
      transition: color 0.2s;

      &:hover {
        color: var(--color-outline);
      }
    }

    &__form {
      display: flex;
      align-items: flex-end;
      gap: 8px;
      flex-wrap: wrap;
      width: 100%;

      & input,
      & label {
        font-size: 0.8rem;
        font-weight: 500;
      }

      & input {
        width: 100%;
      }

      & .label-text {
        margin-left: 0.8rem;
        margin-bottom: 0.2rem;
        display: block;
        text-align: start;
      }
    }

    &__button {
      line-height: 0;
      padding: 0.5rem;
      background-color: inherit;
      border-color: transparent;

      &:hover {
        border: 1px solid var(--color-on-tertiary-container);
      }

      &--remove {
        color: var(--color-error);
      }
    }
  }

  .item-actions {
    display: flex;
    gap: 4px;
  }

  label input {
    display: block;
  }

  .download-url,
  .download-op,
  .download-title {
    flex: 2;
  }

  .download-sub {
    flex: 1;
  }
</style>
