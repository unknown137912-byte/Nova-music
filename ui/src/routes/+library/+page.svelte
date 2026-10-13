<script lang="ts">
import { onMount } from 'svelte';
import { invoke } from '@tauri-apps/api/core';
import { artworkUrl, recoverArtwork } from '../../lib/artwork';

let loading = true;
let error = null;
let library = {
    likedSongs: [],
    playlists: [],
    albums: [],
    artists: [],
    history: []
};

onMount(async () => {
  try {
    loading = true;
    const data: any = await invoke('get_library');
    library = {
      likedSongs: data?.likedSongs || data?.songs || [],
      playlists: Array.isArray(data) ? data : (data?.playlists || []),
      albums: data?.albums || [],
      artists: data?.artists || [],
      history: data?.history || [],
    };
    loading = false;
  } catch (err: any) {
    error = err.message || 'Failed to load library';
    loading = false;
  }
});

function formatDuration(seconds: number) {
  if (isNaN(seconds)) return '—';
  const minutes = Math.floor(seconds / 60);
  const secs = seconds % 60;
  return `${minutes}:${secs.toString().padStart(2, '0')}`;
}

function activateWithKeyboard(event: KeyboardEvent, action: () => unknown) {
  if (event.key !== 'Enter' && event.key !== ' ') return;
  event.preventDefault();
  void action();
}
</script>

<div class="library-page">
  {#if loading}
    <div class="loading-section">
      <div class="loading-spinner"></div>
      <p>Loading your library...</p>
    </div>
  {:else if error}
    <div class="error-section">
      <div class="error-message">
        <h3>Error Loading Library</h3>
        <p>{error}</p>
        <button on:click={() => window.location.reload()}>Try Again</button>
      </div>
    </div>
  {:else}
    <div class="library-container">
      <!-- Liked Songs -->
      <section class="library-section">
        <h2>Liked Songs</h2>
        {#if library.likedSongs.length > 0}
          <div class="library-grid">
            {#each library.likedSongs as song (song.id)}
              <div class="library-card" role="button" tabindex="0" on:click={() => invoke('play_track', { trackId: song.videoId || song.video_id || song.id })} on:keydown={(event) => activateWithKeyboard(event, () => invoke('play_track', { trackId: song.videoId || song.video_id || song.id }))}>
                <div class="library-thumbnail">
                  <img src={artworkUrl(song.thumbnail, 1000) || '/placeholder.svg'} alt={song.title} on:error={(event) => recoverArtwork(event, song)} />
                </div>
                <div class="library-info">
                  <h3>{song.title}</h3>
                  <p>{song.artist}</p>
                  <p class="library-duration">{formatDuration(song.duration)}</p>
                </div>
              </div>
            {/each}
          </div>
        {:else}
          <p class="empty-state">No liked songs.</p>
        {/if}
      </section>

      <!-- Playlists -->
      <section class="library-section">
        <h2>Playlists</h2>
        {#if library.playlists.length > 0}
          <div class="library-grid">
            {#each library.playlists as playlist (playlist.id)}
              <div class="library-card" role="button" tabindex="0" on:click={() => invoke('get_playlist', { playlistId: playlist.id })} on:keydown={(event) => activateWithKeyboard(event, () => invoke('get_playlist', { playlistId: playlist.id }))}>
                <div class="library-thumbnail">
                  <img src={artworkUrl(playlist.thumbnail, 1400) || '/placeholder.svg'} alt={playlist.title} on:error={(event) => recoverArtwork(event, playlist)} />
                </div>
                <div class="library-info">
                  <h3>{playlist.title}</h3>
                  <p>{playlist.trackCount} tracks</p>
                </div>
              </div>
            {/each}
          </div>
        {:else}
          <p class="empty-state">No playlists.</p>
        {/if}
      </section>

      <!-- Albums -->
      <section class="library-section">
        <h2>Albums</h2>
        {#if library.albums.length > 0}
          <div class="library-grid">
            {#each library.albums as album (album.id)}
              <div class="library-card" role="button" tabindex="0" on:click={() => invoke('get_album', { albumId: album.id })} on:keydown={(event) => activateWithKeyboard(event, () => invoke('get_album', { albumId: album.id }))}>
                <div class="library-thumbnail">
                  <img src={artworkUrl(album.thumbnail, 1400) || '/placeholder.svg'} alt={album.title} on:error={(event) => recoverArtwork(event, album)} />
                </div>
                <div class="library-info">
                  <h3>{album.title}</h3>
                  <p>{album.artist}</p>
                  <p class="library-duration">{formatDuration(album.duration)}</p>
                </div>
              </div>
            {/each}
          </div>
        {:else}
          <p class="empty-state">No albums.</p>
        {/if}
      </section>

      <!-- Artists -->
      <section class="library-section">
        <h2>Artists</h2>
        {#if library.artists.length > 0}
          <div class="library-grid">
            {#each library.artists as artist (artist.id)}
              <div class="library-card" role="button" tabindex="0" on:click={() => invoke('get_artist', { artistId: artist.id })} on:keydown={(event) => activateWithKeyboard(event, () => invoke('get_artist', { artistId: artist.id }))}>
                <div class="library-thumbnail">
                  <img src={artworkUrl(artist.thumbnail, 1000) || '/placeholder.svg'} alt={artist.name} on:error={(event) => recoverArtwork(event, artist)} />
                </div>
                <div class="library-info">
                  <h3>{artist.name}</h3>
                  <p>{artist.subtitle || '—'}</p>
                </div>
              </div>
            {/each}
          </div>
        {:else}
          <p class="empty-state">No artists.</p>
        {/if}
      </section>

      <!-- History -->
      <section class="library-section">
        <h2>Recent History</h2>
        {#if library.history.length > 0}
          <div class="library-grid">
            {#each library.history as track (track.id)}
              <div class="library-card" role="button" tabindex="0" on:click={() => invoke('play_track', { trackId: track.videoId || track.video_id || track.id })} on:keydown={(event) => activateWithKeyboard(event, () => invoke('play_track', { trackId: track.videoId || track.video_id || track.id }))}>
                <div class="library-thumbnail">
                  <img src={artworkUrl(track.thumbnail, 1000) || '/placeholder.svg'} alt={track.title} on:error={(event) => recoverArtwork(event, track)} />
                </div>
                <div class="library-info">
                  <h4>{track.title}</h4>
                  <p>{track.artist}</p>
                  <p class="library-duration">{formatDuration(track.duration)}</p>
                </div>
              </div>
            {/each}
          </div>
        {:else}
          <p class="empty-state">No history.</p>
        {/if}
      </section>
    </div>
  {/if}
</div>

<style>
.library-page {
  max-width: 1200px;
  margin: 0 auto;
  padding: var(--space-8) 0;
}

.library-container {
  display: grid;
  gap: var(--space-8);
}

.library-section {
  background: var(--bg-secondary);
  border-radius: var(--radius-lg);
  padding: var(--space-6);
  border: 1px solid var(--border-primary);
}

.library-section h2 {
  font-size: var(--text-2xl);
  margin-bottom: var(--space-6);
  color: var(--text-primary);
}

.library-grid {
  display: grid;
  gap: var(--space-4);
}

/* Mobile */
@media (max-width: 768px) {
  .library-page {
    padding: var(--space-4) 0;
  }
  
  .library-section {
    padding: var(--space-4);
  }
  
  .library-section h2 {
    font-size: var(--text-xl);
  }
}
</style>
