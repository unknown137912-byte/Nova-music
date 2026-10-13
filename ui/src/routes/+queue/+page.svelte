<script lang="ts">
import { onMount } from 'svelte';
import { invoke } from '@tauri-apps/api/core';

let loading = true;
let error = null;
let queue = [];
let isPlaying = false;
let currentTrackIndex = -1;

onMount(async () => {
  try {
    loading = true;
    const data: any[] = await invoke<any[]>('get_queue');
    queue = data;
    isPlaying = true;
    currentTrackIndex = 0;
    loading = false;
  } catch (err: any) {
    error = err.message || 'Failed to load queue';
    loading = false;
  }
});

function formatTime(seconds: number) {
  if (isNaN(seconds)) return '0:00';
  const m = Math.floor(seconds / 60);
  const s = seconds % 60;
  return `${m}:${s.toString().padStart(2, '0')}`;
}
</script>

<div class="queue-page">
  {#if loading}
    <div class="loading-section">
      <div class="loading-spinner"></div>
      <p>Loading queue...</p>
    </div>
  {:else if error}
    <div class="error-section">
      <div class="error-message">
        <h3>Error Loading Queue</h3>
        <p>{error}</p>
        <button on:click={() => window.location.reload()}>Try Again</button>
      </div>
    </div>
  {:else}
    <div class="queue-container">
      {#if queue.length === 0}
        <p class="empty-state">Queue is empty.</p>
      {:else}
        <div class="queue-header">
          <h2>Up Next</h2>
          <span>{queue.length} tracks</span>
        </div>

        {#each queue as track, index}
          <div class="queue-item {index === currentTrackIndex ? 'playing' : ''}" role="button" tabindex="0"
               on:click={() => invoke('play_track', { trackId: track.videoId || track.video_id || track.id })}
               on:keydown={(event) => { if (event.key === 'Enter' || event.key === ' ') { event.preventDefault(); void invoke('play_track', { trackId: track.videoId || track.video_id || track.id }); } }}>
            <div class="queue-number">{index + 1}</div>
            <div class="queue-info">
              <h4>{track.title}</h4>
              <p>{track.artist}</p>
            </div>
            <div class="queue-duration">{formatTime(track.duration)}</div>
          </div>
        {/each}
      {/if}
    </div>
  {/if}
</div>

<style>
.queue-page {
  max-width: 1200px;
  margin: 0 auto;
  padding: var(--space-8) 0;
}

.queue-container {
  background: var(--bg-secondary);
  border-radius: var(--radius-lg);
  padding: var(--space-6);
  border: 1px solid var(--border-primary);
}

.queue-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: var(--space-6);
  padding-bottom: var(--space-4);
  border-bottom: 1px solid var(--border-primary);
}

.queue-header h2 {
  font-size: var(--text-2xl);
  color: var(--text-primary);
}

.queue-header span {
  color: var(--text-tertiary);
  font-size: var(--text-sm);
}

.queue-item {
  display: grid;
  grid-template-columns: auto 1fr auto;
  gap: var(--space-4);
  padding: var(--space-3) 0;
  border-top: 1px solid var(--border-primary);
  align-items: center;
}

.queue-item.playing {
  background: var(--bg-tertiary);
}

.queue-number {
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--bg-primary);
  border-radius: 50%;
  font-weight: var(--weight-semibold);
  color: var(--text-secondary);
  margin-right: var(--space-3);
}

.queue-info h4 {
  font-size: var(--text-base);
  font-weight: var(--weight-medium);
  color: var(--text-primary);
  margin-bottom: var(--space-1);
}

.queue-info p {
  color: var(--text-tertiary);
  font-size: var(--text-sm);
}

.queue-duration {
  color: var(--text-tertiary);
  font-size: var(--text-sm);
  font-family: 'SF Mono', monospace;
}

.empty-state {
  color: var(--text-tertiary);
  text-align: center;
  padding: var(--space-8) 0;
}
</style>
