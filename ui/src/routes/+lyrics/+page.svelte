<script lang="ts">
import { onMount } from 'svelte';
import { invoke } from '@tauri-apps/api/core';

let loading = true;
let error = null;
let lyrics = [];
let currentLine = 0;
let isPlaying = false;

onMount(async () => {
  try {
    loading = true;
    // Try to get lyrics from the player
    const data = await invoke('get_now_playing');
    // For now, use dummy data since lyrics fetching is not yet implemented
    lyrics = [
      { line: 'Yesterday, all my troubles seemed so far away', timestamp: 0 },
      { line: 'Now it looks as though they\'re here to stay', timestamp: 30 },
      { line: 'Oh, I believe in yesterday', timestamp: 60 },
    ];
    currentLine = 0;
    isPlaying = true;
    loading = false;
  } catch (err: any) {
    error = err.message || 'Failed to load lyrics';
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

<div class="lyrics-page">
  {#if loading}
    <div class="loading-section">
      <div class="loading-spinner"></div>
      <p>Loading lyrics...</p>
    </div>
  {:else if error}
    <div class="error-section">
      <div class="error-message">
        <h3>Error Loading Lyrics</h3>
        <p>{error}</p>
        <button on:click={() => window.location.reload()}>Try Again</button>
      </div>
    </div>
  {:else}
    <div class="lyrics-container">
      {#each lyrics as line, index}
        <div class="lyric-line {currentLine === index ? 'active' : ''}" role="button" tabindex="0"
             on:click={() => currentLine = index}
             on:keydown={(event) => { if (event.key === 'Enter' || event.key === ' ') { event.preventDefault(); currentLine = index; } }}>
          <div class="timestamp">{formatTime(line.timestamp)}</div>
          <div class="lyric-text">{line.line}</div>
        </div>
      {/each}
      {#if lyrics.length === 0}
        <p class="empty-state">No lyrics available.</p>
      {/if}
    </div>
  {/if}
</div>

<style>
.lyrics-page {
  max-width: 1200px;
  margin: 0 auto;
  padding: var(--space-8) 0;
  background: var(--bg-primary);
  color: var(--text-primary);
}

.lyrics-container {
  max-width: 800px;
  margin: 0 auto;
}

.lyric-line {
  display: flex;
  align-items: center;
  padding: var(--space-3) 0;
  border-bottom: 1px solid var(--border-primary);
  cursor: pointer;
  transition: background 0.2s ease;
}

.lyric-line.active {
  background: var(--bg-tertiary);
}

.lyric-line:last-child {
  border-bottom: none;
}

.timestamp {
  width: 40px;
  flex-shrink: 0;
  color: var(--text-tertiary);
  font-size: var(--text-sm);
  font-family: 'SF Mono', monospace;
  text-align: right;
  margin-right: var(--space-3);
}

.lyric-text {
  flex: 1;
  color: var(--text-primary);
  font-size: var(--text-lg);
  line-height: 1.5;
}

.lyric-text:hover {
  color: var(--accent-600);
}

.empty-state {
  color: var(--text-tertiary);
  text-align: center;
  padding: var(--space-8) 0;
  font-style: italic;
}
</style>
