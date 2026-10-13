<script lang="ts">
import { onMount } from 'svelte';
import { invoke } from '@tauri-apps/api/core';
import { artworkUrl, recoverArtwork } from '../../lib/artwork';

let loading = true;
let playlist = null;
let error = null;

onMount(async () => {
  try {
    loading = true;
    
    // TODO: Get playlist from URL params
    // For now, just simulate
    await new Promise(resolve => setTimeout(resolve, 1000));
    
    playlist = {
      id: '1',
      title: 'Sample Playlist',
      description: 'A sample playlist for testing',
      thumbnail: '/placeholder.svg',
      trackCount: 10,
      tracks: [
        { id: '1', title: 'Track 1', artist: 'Artist 1', duration: 180 },
        { id: '2', title: 'Track 2', artist: 'Artist 2', duration: 200 },
        { id: '3', title: 'Track 3', artist: 'Artist 3', duration: 190 },
      ]
    };
    
    loading = false;
  } catch (err) {
    error = err;
    loading = false;
  }
});

function formatDuration(seconds) {
  const minutes = Math.floor(seconds / 60);
  const secs = seconds % 60;
  return `${minutes}:${secs.toString().padStart(2, '0')}`;
}
</script>

<div class="playlist-page">
  {#if loading}
    <div class="loading-playlist">
      <div class="loading-spinner"></div>
      <p>Loading playlist...</p>
    </div>
  {:else if error}
    <div class="error-playlist">
      <p>Error: {error}</p>
      <button on:click={() => window.location.reload()}>Try Again</button>
    </div>
  {:else if playlist}
    <header class="playlist-header">
      <div class="playlist-thumbnail">
        <img src={artworkUrl(playlist.thumbnail, 1400) || '/placeholder.svg'} alt={playlist.title} on:error={(event) => recoverArtwork(event, playlist)} />
      </div>
      <div class="playlist-info">
        <h1>{playlist.title}</h1>
        <p class="playlist-description">{playlist.description}</p>
        <div class="playlist-meta">
          <span>{playlist.trackCount} tracks</span>
          <span>•</span>
          <span>~{playlist.tracks.reduce((sum, t) => sum + t.duration, 0)} min</span>
        </div>
        <div class="playlist-actions">
          <button class="play-btn" on:click={() => invoke('play_track', { trackId: playlist.tracks[0]?.videoId || playlist.tracks[0]?.video_id || playlist.tracks[0]?.id })}>
            Play
          </button>
          <button class="shuffle-btn" disabled>Shuffle</button>
        </div>
      </div>
    </header>

    <main class="playlist-tracks">
      <h2>Tracks</h2>
      <div class="tracks-list">
        {#each playlist.tracks as track, index (track.videoId || track.video_id || track.id)}
          <div class="track-item" role="button" tabindex="0" on:click={() => invoke('play_track', { trackId: track.videoId || track.video_id || track.id })} on:keydown={(event) => { if (event.key === 'Enter' || event.key === ' ') { event.preventDefault(); void invoke('play_track', { trackId: track.videoId || track.video_id || track.id }); } }}>
            <div class="track-number">{index + 1}</div>
            <div class="track-info">
              <h4>{track.title}</h4>
              <p>{track.artist}</p>
            </div>
            <div class="track-duration">{formatDuration(track.duration)}</div>
          </div>
        {/each}
      </div>
    </main>
  {/if}
</div>

<style>
.playlist-page {
  max-width: 1200px;
  margin: 0 auto;
  padding: var(--space-8) 0;
}

.loading-playlist, .error-playlist {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  min-height: 300px;
  text-align: center;
  color: var(--text-secondary);
}

.loading-spinner {
  width: 40px;
  height: 40px;
  border: 3px solid var(--border-primary);
  border-top-color: var(--accent-600);
  border-radius: 50%;
  animation: spin 1s linear infinite;
  margin-bottom: var(--space-4);
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

.playlist-header {
  display: flex;
  gap: var(--space-8);
  margin-bottom: var(--space-8);
  padding: var(--space-6);
  background: var(--bg-secondary);
  border-radius: var(--radius-xl);
  border: 1px solid var(--border-primary);
}

.playlist-thumbnail {
  width: 256px;
  height: 256px;
  border-radius: var(--radius-lg);
  overflow: hidden;
  flex-shrink: 0;
}

.playlist-thumbnail img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.playlist-info {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  justify-content: center;
}

.playlist-info h1 {
  font-size: var(--text-4xl);
  font-weight: var(--weight-bold);
  margin-bottom: var(--space-2);
}

.playlist-description {
  color: var(--text-secondary);
  margin-bottom: var(--space-4);
  font-size: var(--text-lg);
}

.playlist-meta {
  color: var(--text-tertiary);
  font-size: var(--text-base);
  margin-bottom: var(--space-6);
}

.playlist-actions {
  display: flex;
  gap: var(--space-4);
}

.play-btn, .shuffle-btn {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  padding: var(--space-3) var(--space-6);
  border: none;
  border-radius: var(--radius-full);
  font-size: var(--text-base);
  font-weight: var(--weight-medium);
  cursor: pointer;
  transition: all 0.2s ease;
}

.play-btn {
  background: var(--accent-600);
  color: white;
}

.play-btn:hover:not(:disabled) {
  background: var(--accent-700);
  transform: scale(1.02);
}

.shuffle-btn {
  background: var(--bg-tertiary);
  color: var(--text-primary);
  border: 1px solid var(--border-primary);
}

.shuffle-btn:hover:not(:disabled) {
  background: var(--bg-secondary);
}

.playlist-tracks h2 {
  font-size: var(--text-xl);
  font-weight: var(--weight-semibold);
  margin-bottom: var(--space-4);
}

.tracks-list {
  display: flex;
  flex-direction: column;
  gap: var(--space-2);
}

.track-item {
  display: flex;
  align-items: center;
  padding: var(--space-3);
  background: var(--bg-secondary);
  border-radius: var(--radius-md);
  cursor: pointer;
  transition: all 0.2s ease;
  border: 1px solid var(--border-primary);
}

.track-item:hover:not(:disabled) {
  background: var(--bg-tertiary);
  border-color: var(--accent-600);
}

.track-number {
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--bg-primary);
  border-radius: 50%;
  font-weight: var(--weight-semibold);
  color: var(--text-secondary);
  margin-right: var(--space-4);
}

.track-info {
  flex: 1;
}

.track-info h4 {
  font-size: var(--text-base);
  font-weight: var(--weight-medium);
  margin-bottom: var(--space-1);
  color: var(--text-primary);
}

.track-info p {
  font-size: var(--text-sm);
  color: var(--text-tertiary);
}

.track-duration {
  color: var(--text-tertiary);
  font-size: var(--text-sm);
  font-family: var(--font-mono);
}

@media (max-width: 768px) {
  .playlist-header {
    flex-direction: column;
    align-items: center;
    text-align: center;
    gap: var(--space-6);
  }
  
  .playlist-thumbnail {
    width: 200px;
    height: 200px;
  }
  
  .playlist-actions {
    justify-content: center;
  }
}
</style>
