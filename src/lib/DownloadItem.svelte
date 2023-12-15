<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import Icon from "@iconify/svelte";
  import { Moon } from "svelte-loading-spinners";

  import { type DownloadOutput, DownloadStatus } from "./Downloads";

  export let download: DownloadOutput;

  const dispatch = createEventDispatcher();

  let editing = false;
  let fields = {
    url: download.input.url,
    op: download.input.op,
    sub: download.input.sub,
    title: download.title,
  };

  function switchToEditing() {
    fields.url = download.input.url;
    fields.op = download.input.op;
    fields.sub = download.input.sub;
    fields.title = download.title;
    editing = true;
  }

  function cancelEditing() {
    editing = false;
  }

  function emitEdit() {
    editing = false;
    download.input.url = fields.url;
    download.input.op = fields.op;
    download.input.sub = fields.sub;
    download.title = fields.title;
    download.status = DownloadStatus.Initial;
    dispatch("save", {
      download,
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
  class="download-item {download.status === DownloadStatus.Failed
    ? 'failed'
    : ''}"
>
  <div class="download-item__lhs {editing ? 'editing' : ''}">
    {#if download.status === DownloadStatus.Completed}
      <Icon icon="material-symbols:download-done-rounded" />
    {:else if download.status === DownloadStatus.Downloading}
      <Moon color="var(--color-on-tertiary-container" size="1.2" unit="rem" />
    {:else if download.status === DownloadStatus.Failed}
      <Icon icon="material-symbols:error-outline-rounded" />
    {/if}
    {#if editing && download.status !== DownloadStatus.Downloading}
      <form
        class="download-item__form"
        id={`download-item-${download.id}`}
        on:submit|preventDefault={emitEdit}
      >
        <label>
          <span class="label-text"> URL </span>
          <input
            type="text"
            placeholder="URL"
            bind:value={fields.url}
            required
            autocomplete="off"
          />
        </label>
        <label>
          <span class="label-text"> Subreddit </span>
          <input
            list="subreddits"
            type="text"
            placeholder="Subreddit"
            bind:value={fields.sub}
            required
          />
        </label>
        <label>
          <span class="label-text"> Original Poster </span>
          <input
            type="text"
            placeholder="Original Poster"
            bind:value={fields.op}
            required
          />
        </label>
        <label>
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
      <a href={download.input.url} target="_blank"
        >[{download.input.sub}] [{download.input.op}] {download.title}</a
      >
      {#if download.status === DownloadStatus.Failed}
        {download.failure}
      {/if}
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

    &.failed {
      color: var(--color-on-error-container);
      background-color: var(--color-error-container);
    }

    &__lhs {
      display: flex;
      gap: 8px;
      flex: 1;
      height: 3rem;
      transition: height 0.2s;

      &.editing {
        height: 4rem;
      }
    }

    &__form {
      display: flex;
      align-items: flex-end;
      gap: 8px;
      flex-wrap: wrap;
      flex: 1;

      & input,
      & label {
        font-size: 0.8rem;
        font-weight: 500;
      }

      & label {
        flex: 1;
      }

      & input {
        width: 100%;
      }

      & .label-text {
        margin-left: 1rem;
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

  a {
    font-weight: 500;
    color: inherit;
    text-decoration: inherit;
    transition: color 0.2s;
  }

  a:hover {
    color: var(--color-outline);
  }
</style>
