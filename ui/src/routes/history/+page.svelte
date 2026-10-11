<script lang="ts">
import { onMount } from 'svelte';
import { invoke } from '@tauri-apps/api/core';

let loading = true;
let error = null;
let history = [];

onMount(async () => {
  try {
    loading = true;
    // For now, use dummy data since history fetch is not yet implemented
    history = [
      { id: '1', title: 'Song 1', artist: 'Artist 1', duration: 180, timestamp: Date.now() - 86400000 },
      { id: '2', title: 'Song 2', artist: 'Artist 2', duration: 200, timestamp: Date.now() - 172800000 },
      { id: '3', title: 'Song 3', artist: 'Artist 3', duration: 190, timestamp: Date.now() - 259200000 },
    ];
    loading = false;
  } catch (err: any) {
    error = err.message || 'Failed to load history';
    loading = false;
  }
});

function formatTime(seconds: number) {
  if (isNaN(seconds)) return '—';
  const m = Math.floor(seconds / 60);
  const s = seconds % 60;
  return `${m}:${s.toString().padStart(2, '0')}`;
}
</script>

<div class="history-page">
  {#if loading}
    <div class="loading-section">
      <div class="loading-spinner"></div>
      <p>Loading history...</p>
    </div>
  {:else if error}
    <div class="error-section">
      <div class="error-message">
        <h3>Error Loading History</h3>
        <p>{error}</p>
        <button on:click={() => window.location.reload()}>Try Again</button>
      </div>
    </div>
  {:else}
    <div class="history-container">
      {#if history.length > 0}
        <div class="history-list">
          {#each history as track (track.id)}
            <div class="history-item" on:click={() => invoke('play_track', { trackId: track.videoId || track.video_id || track.id })}>
              <div class="history-number">{history.indexOf(track) + 1}</div>
              <div class="history-info">
                <h4>{track.title}</h4>
                <p>{track.artist}</p>
                <p class="history-duration">{formatTime(track.duration)}</p>
                <p class="history-timestamp">{new Date(track.timestamp).toLocaleDateString()}</p>
              </div>
            </div>
          {/each}
        </div>
      {:else}
        <p class="empty-state">No history.</p>
      {/if}
    </div>
  {/if}
</div>

<style>
.history-page {
  max-width: 1200px;
  margin: 0 auto;
  padding: var(--space-8) 0;
  background: var(--bg-primary);
  color: var(--text-primary);
}

.history-container {
  max-width: 800px;
  margin: 0 auto;
}

.history-list {
  display: grid;
  gap: var(--space-4);
}

.history-item {
  display: grid;
  grid-template-columns: auto 1fr auto auto;
  gap: var(--space-4);
  padding: var(--space-3) 0;
  border-top: 1px solid var(--border-primary);
  align-items: center;
}

.history-item:hover {
  background: var(--bg-tertiary);
}

.history-number {
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

.history-info h4 {
  font-size: var(--text-base);
  font-weight: var(--weight-medium);
  color: var(--text-primary);
  margin-bottom: var(--space-1);
}

.history-info p {
  color: var(--text-tertiary);
  font-size: var(--text-sm);
  margin: 0;
}

.history-duration {
  color: var(--text-tertiary);
  font-size: var(--text-sm);
  font-family: 'SF Mono', monospace;
}

.history-timestamp {
  color: var(--text-tertiary);
  font-size: var(--text-sm);
  font-style: italic;
}

.empty-state {
  color: var(--text-tertiary);
  text-align: center;
  padding: var(--space-8) 0;
}
</style>
