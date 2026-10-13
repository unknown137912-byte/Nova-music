<script lang="ts">
import { onMount } from 'svelte';
import { fade } from 'svelte/transition';
import { invoke } from '@tauri-apps/api/core';
import { artworkUrl, recoverArtwork } from '../../lib/artwork';

let loading = true;
let error = null;
let track = null;
let isPlaying = false;
let progress = 0;
let duration = 0;
let volume = 50;
let shuffle = false;
let repeat = 'off'; // off, one, all
let queue = [];
let queueIndex = -1;

onMount(async () => {
  try {
    loading = true;
    // Try to get current track from the player
    const data = await invoke('get_now_playing');
    track = data;
    isPlaying = true;
    progress = 0;
    duration = track.duration || 0;
    volume = 50;
    shuffle = false;
    repeat = 'off';
    queue = [];
    queueIndex = -1;
    loading = false;
  } catch (err: any) {
    error = err.message || 'Failed to load now playing';
    loading = false;
  }
});

function formatTime(seconds: number) {
  if (isNaN(seconds)) return '0:00';
  const m = Math.floor(seconds / 60);
  const s = seconds % 60;
  return `${m}:${s.toString().padStart(2, '0')}`;
}

function togglePlay() {
  invoke<boolean>('pause_playback').then((paused) => isPlaying = !paused)
    .catch((err) => error = String(err));
}

function seekTime(seconds: number) {
  invoke('seek', { position: seconds }).catch((err) => error = String(err));
}

function previousTrack() {
  invoke('previous_track').catch((err) => error = String(err));
}

function skipTrack() {
  invoke('skip_track').catch((err) => error = String(err));
}

function toggleShuffle() {
  shuffle = !shuffle;
}

function setRepeat(mode: string) {
  repeat = mode;
}
</script>

<div class="now-playing-page" transition:fade>
  {#if loading}
    <div class="loading-section">
      <div class="loading-spinner"></div>
      <p>Loading now playing...</p>
    </div>
  {:else if error}
    <div class="error-section">
      <div class="error-message">
        <h3>Error Loading Now Playing</h3>
        <p>{error}</p>
        <button on:click={() => window.location.reload()}>Try Again</button>
      </div>
    </div>
  {:else}
    <div class="now-playing-container">
      <!-- Album Artwork -->
      <div class="artwork-area">
        {#if track}
          <div class="artwork-wrapper">
            <img src={artworkUrl(track.thumbnail, 1400) || '/placeholder.svg'} alt={track.title} on:error={(event) => recoverArtwork(event, track)} />
            <div class="artwork-overlay">
              <h2>{track.title}</h2>
              <p>{track.artist}</p>
            </div>
          </div>
        {/if}
      </div>

      <!-- Playback Controls -->
      <div class="controls">
        <!-- Prev -->
        <button class="control-btn prev-btn" on:click={previousTrack} title="Previous">
          <svg>...</svg>
        </button>

        <!-- Play/Pause -->
        <button class="control-btn play-pause-btn" on:click={togglePlay} title={isPlaying ? 'Pause' : 'Play'}>
          <svg>...</svg>
        </button>

        <!-- Next -->
        <button class="control-btn next-btn" on:click={skipTrack} title="Next">
          <svg>...</svg>
        </button>
      </div>

      <!-- Progress Bar -->
      <div class="progress-bar">
        <div class="progress-fill" style="width: ${(progress / duration) * 100}%"></div>
      </div>
      <div class="progress-time">
        <span>{formatTime(progress)}</span>
        <span>/ {formatTime(duration)}</span>
      </div>

      <!-- Queue and Controls -->
      <div class="control-panel">
        <div class="shuffle-repeat">
          <button class="mode-btn" on:click={() => toggleShuffle()} title={shuffle ? 'Shuffle on' : 'Shuffle off'}>
            {shuffle ? '🔀' : '🔁'}
          </button>
          <button class="mode-btn" on:click={() => setRepeat('off')} title="Repeat off">↺</button>
          <button class="mode-btn" on:click={() => setRepeat('one')} title="Repeat one">↻</button>
          <button class="mode-btn" on:click={() => setRepeat('all')} title="Repeat all">🔂</button>
        </div>

        <!-- Queue button -->
        <button class="queue-btn" title="View queue">
          Queue
          {#if queue.length > 0}
            <span>{queue.length}</span>
          {/if}
        </button>
      </div>
    </div>
  {/if}
</div>

<style>
.now-playing-page {
  min-height: 100vh;
  background: var(--bg-primary);
  color: var(--text-primary);
}

.now-playing-container {
  max-width: 1200px;
  margin: 0 auto;
  padding: var(--space-8) 0;
  display: flex;
  flex-direction: column;
  align-items: center;
}

.artwork-area {
  width: 100%;
  max-width: 400px;
  margin: var(--space-8) 0;
}

.artwork-wrapper {
  position: relative;
  width: 100%;
  aspect-ratio: 1 / 1;
  border-radius: var(--radius-xl);
  overflow: hidden;
}

.artwork-wrapper img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.artwork-overlay {
  position: absolute;
  bottom: 0;
  left: 0;
  right: 0;
  background: linear-gradient(transparent, var(--bg-primary));
  padding: var(--space-4);
  color: var(--text-primary);
}

.artwork-overlay h2 {
  font-size: var(--text-2xl);
  font-weight: var(--weight-heavy);
  margin-bottom: var(--space-1);
}

.artwork-overlay p {
  font-size: var(--text-lg);
  color: var(--text-secondary);
}

.controls {
  display: flex;
  gap: var(--space-4);
  justify-content: center;
  margin: var(--space-4) 0;
}

.control-btn {
  width: 48px;
  height: 48px;
  border: none;
  border-radius: var(--radius-lg);
  background: var(--bg-tertiary);
  color: var(--text-primary);
  cursor: pointer;
  transition: all 0.2s ease;
  font-size: var(--text-lg);
}

.control-btn:hover:not(:disabled) {
  background: var(--bg-secondary);
  transform: scale(1.1);
}

.prev-btn svg,
.play-pause-btn svg,
.next-btn svg {
  width: 24px;
  height: 24px;
  fill: currentColor;
}

.play-pause-btn svg {
  width: 32px;
  height: 32px;
}

.progress-bar {
  width: 100%;
  max-width: 400px;
  height: 4px;
  background: var(--bg-tertiary);
  border-radius: 2px;
  margin: var(--space-4) 0;
  overflow: hidden;
}

.progress-fill {
  height: 100%;
  background: var(--accent-600);
  width: 0%;
  transition: width 0.1s linear;
}

.progress-time {
  display: flex;
  justify-content: space-between;
  margin-top: var(--space-2);
  color: var(--text-tertiary);
  font-size: var(--text-sm);
  font-family: 'SF Mono', monospace;
}

.control-panel {
  width: 100%;
  max-width: 400px;
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin: var(--space-4) 0;
}

.shuffle-repeat {
  display: flex;
  gap: var(--space-2);
  align-items: center;
}

.mode-btn {
  width: 32px;
  height: 32px;
  border: none;
  border-radius: var(--radius-lg);
  background: var(--bg-tertiary);
  color: var(--text-primary);
  cursor: pointer;
  font-size: var(--text-lg);
  transition: all 0.2s ease;
}

.mode-btn:hover:not(:disabled) {
  background: var(--bg-secondary);
}

.queue-btn {
  padding: var(--space-2) var(--space-4);
  background: var(--bg-tertiary);
  border: 1px solid var(--border-primary);
  border-radius: var(--radius-md);
  color: var(--text-primary);
  cursor: pointer;
  font-size: var(--text-sm);
  transition: all 0.2s ease;
}

.queue-btn:hover:not(:disabled) {
  background: var(--bg-secondary);
}
</style>
