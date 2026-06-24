<script lang="ts">
  interface Props {
    src: string;
    title: string;
    artist: string;
    sub: string;
    error: string | null;
  }

  let { src, title, artist, sub, error }: Props = $props();

  let time = $state(0);
  let duration = $state(0);
  let paused = $state(true);

  function format(time: any) {
    if (isNaN(time)) return "...";

    const minutes = Math.floor(time / 60);
    const seconds = Math.floor(time % 60);

    return `${minutes}:${seconds < 10 ? `0${seconds}` : seconds}`;
  }
</script>

<div class="player" class:paused>
  <audio
    {src}
    bind:currentTime={time}
    bind:duration
    bind:paused
    preload="metadata"
    onended={() => {
      time = 0;
    }}
  >
  </audio>

  <button
    class="play"
    aria-label={paused ? "play" : "pause"}
    onclick={() => (paused = !paused)}
  ></button>

  <div class="info">
    <div class="description">
      {#if error}
        <span class="error">{error}</span>
      {/if}
      <strong>{title}</strong> /
      <span>{artist} ({sub})</span>
    </div>

    <div class="time">
      <span>{format(time)}</span>
      <div
        class="slider"
        role="progressbar"
        onpointerdown={(e) => {
          const div = e.currentTarget;

          function seek(e) {
            const { left, width } = div.getBoundingClientRect();

            let p = (e.clientX - left) / width;
            if (p < 0) p = 0;
            if (p > 1) p = 1;

            time = p * duration;
          }

          seek(e);

          window.addEventListener("pointermove", seek);

          window.addEventListener(
            "pointerup",
            () => {
              window.removeEventListener("pointermove", seek);
            },
            {
              once: true,
            },
          );
        }}
      >
        <div class="progress" style="--progress: {time / duration}%"></div>
      </div>
      <span>{duration ? format(duration) : "--:--"}</span>
    </div>
  </div>
</div>

<style>
  .player {
    display: grid;
    grid-template-columns: 2.5em 1fr;
    align-items: center;
    gap: 1em;
    background: inherit;
    transition: filter 0.2s;
    color: var(--color-neutral);
    user-select: none;
    transition: all 0.2s;
  }

  .player:not(.paused) {
    color: inherit;
    filter: drop-shadow(0.5em 0.5em 1em rgba(0, 0, 0, 0.1));
  }

  button {
    width: 100%;
    aspect-ratio: 1;
    background-repeat: no-repeat;
    background-position: 50% 50%;
    border-radius: 50%;
    background-color: var(--color-on-tertiary-container);
  }

  [aria-label="pause"] {
    background-image: url(./pause.svg);
  }

  [aria-label="play"] {
    background-image: url(./play.svg);
  }

  .info {
    overflow: hidden;
  }

  .error {
    color: var(--color-on-error-container);
  }

  .description {
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    line-height: 1.2;
  }

  .time {
    display: flex;
    align-items: center;
    gap: 0.5em;
  }

  .time span {
    font-size: 0.7em;
  }

  .slider {
    flex: 1;
    height: 0.5em;
    background: var(--color-on-tertiary-container);
    border-radius: 0.5em;
    overflow: hidden;
  }

  .progress {
    width: calc(100 * var(--progress));
    height: 100%;
    background: var(--color-tertiary);
  }
</style>
