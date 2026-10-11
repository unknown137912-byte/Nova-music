<script lang="ts">
import { onMount } from 'svelte';
import { invoke } from '@tauri-apps/api/core';
import TrackActions from '../../lib/TrackActions.svelte';

export let onPlayTrack: (track: any) => void = () => {};
export let onOpenPlaylist: (id: string) => void = () => {};
export let onOpenArtist: (id: string) => void = () => {};
export let onTrackAction: (track: any, action: string) => void = () => {};
export let playlistOptions: any[] = [];
export let initialQuery = '';

let query = '';
let loading = false;
let error: string | null = null;
let results: any = null;
let activeFilter = 'all';

onMount(() => {
  query = initialQuery.trim();
  if (query) void performSearch();
});

async function performSearch() {
  if (!query.trim()) return;
  try {
    loading = true;
    error = null;
    const res: any = await invoke('search', { query: query.trim() });
    results = res;
    loading = false;
  } catch (err: any) {
    console.error('Search failed:', err);
    error = typeof err === 'string' ? err : (err?.message || 'Search failed');
    loading = false;
  }
}

function handlePlaySong(song: any) {
  onPlayTrack({
    ...song,
    id: song.videoId || song.video_id || song.trackId || song.track_id || song.id,
    title: song.title,
    artist: song.artist || 'YouTube Music',
    thumbnail: song.thumbnail,
    duration: song.duration ? parseDuration(song.duration) : 180,
  });
}

function parseDuration(dStr: string): number {
  if (!dStr) return 180;
  const parts = dStr.split(':').map(p => parseInt(p, 10));
  if (parts.length === 2) return (parts[0] * 60) + parts[1];
  if (parts.length === 3) return (parts[0] * 3600) + (parts[1] * 60) + parts[2];
  return 180;
}
</script>

<div class="search-container">
  <header class="search-results-heading">
    <div><span>DISCOVER</span><h1>Search results</h1><p>{query ? `Showing matches for “${query}”` : 'Use the search field above to find songs, artists, albums, and playlists.'}</p></div>
  </header>

  <!-- Filters stay on the results view; the only search field lives in the app title bar. -->
  {#if results}
      <div class="filter-tabs">
        <button
          class="tab-btn {activeFilter === 'all' ? 'active' : ''}"
          on:click={() => activeFilter = 'all'}
        >
          All Results
        </button>
        <button
          class="tab-btn {activeFilter === 'songs' ? 'active' : ''}"
          on:click={() => activeFilter = 'songs'}
        >
          Songs ({results.songs?.length || 0})
        </button>
        <button
          class="tab-btn {activeFilter === 'albums' ? 'active' : ''}"
          on:click={() => activeFilter = 'albums'}
        >
          Albums ({results.albums?.length || 0})
        </button>
        <button
          class="tab-btn {activeFilter === 'artists' ? 'active' : ''}"
          on:click={() => activeFilter = 'artists'}
        >
          Artists ({results.artists?.length || 0})
        </button>
        <button
          class="tab-btn {activeFilter === 'playlists' ? 'active' : ''}"
          on:click={() => activeFilter = 'playlists'}
        >
          Playlists ({results.playlists?.length || 0})
        </button>
      </div>
  {/if}

  <!-- Content States -->
  {#if loading}
    <div class="loading-state">
      <div class="cosmic-spinner"></div>
      <p>Searching YouTube Music universe...</p>
    </div>
  {:else if error}
    <div class="error-msg">
      ⚠️ {error}
    </div>
  {:else if results}
    <div class="results-layout">
      <!-- Songs Section -->
      {#if (activeFilter === 'all' || activeFilter === 'songs') && results.songs && results.songs.length > 0}
        <div class="results-group">
          <h2 class="group-title">Songs</h2>
          <div class="song-list">
            {#each results.songs as song, idx}
              <div
                class="song-row"
                on:click={() => handlePlaySong(song)}
                role="button"
                tabindex="0"
                on:keydown={(e) => { if (e.key === 'Enter') handlePlaySong(song); }}
              >
                <div class="song-index">{idx + 1}</div>
                <div class="song-thumb-box">
                  <img src={song.thumbnail || '/placeholder.jpg'} alt="" class="song-thumb" />
                  <div class="row-play-overlay">▶</div>
                </div>
                <div class="song-details">
                  <div class="song-title">{song.title}</div>
                  <div class="song-artist">{song.artist}</div>
                </div>
                <div class="song-duration">{song.duration || ''}</div>
                <button
                  class="row-play-btn"
                  on:click|stopPropagation={() => handlePlaySong(song)}
                  title="Play"
                >
                  ▶
                </button>
                <TrackActions track={song} playlists={playlistOptions} onAction={onTrackAction} />
              </div>
            {/each}
          </div>
        </div>
      {/if}

      <!-- Albums Section -->
      {#if (activeFilter === 'all' || activeFilter === 'albums') && results.albums && results.albums.length > 0}
        <div class="results-group">
          <h2 class="group-title">Albums</h2>
          <div class="cards-grid">
            {#each results.albums as album}
              <div
                class="album-card"
                on:click={() => onOpenPlaylist(album.id)}
                role="button"
                tabindex="0"
                on:keydown={(e) => { if (e.key === 'Enter') onOpenPlaylist(album.id); }}
              >
                <div class="album-thumb-box">
                  <img src={album.thumbnail || '/placeholder.jpg'} alt="" class="album-thumb" />
                  <div class="card-play-btn">♫</div>
                </div>
                <div class="card-title">{album.title}</div>
                <div class="card-subtitle">{album.artist}</div>
              </div>
            {/each}
          </div>
        </div>
      {/if}

      <!-- Artists Section -->
      {#if (activeFilter === 'all' || activeFilter === 'artists') && results.artists && results.artists.length > 0}
        <div class="results-group">
          <h2 class="group-title">Artists</h2>
          <div class="cards-grid">
            {#each results.artists as artist}
              <div
                class="artist-card"
                on:click={() => onOpenArtist(artist.id)}
                role="button"
                tabindex="0"
                on:keydown={(e) => { if (e.key === 'Enter') onOpenArtist(artist.id); }}
              >
                <img src={artist.thumbnail || '/placeholder.jpg'} alt="" class="artist-avatar" />
                <div class="card-title text-center">{artist.title}</div>
                <div class="card-subtitle text-center">Artist</div>
              </div>
            {/each}
          </div>
        </div>
      {/if}

      <!-- Playlists Section -->
      {#if (activeFilter === 'all' || activeFilter === 'playlists') && results.playlists && results.playlists.length > 0}
        <div class="results-group">
          <h2 class="group-title">Playlists</h2>
          <div class="cards-grid">
            {#each results.playlists as pl}
              <div
                class="album-card"
                on:click={() => onOpenPlaylist(pl.id)}
                role="button"
                tabindex="0"
                on:keydown={(e) => { if (e.key === 'Enter') onOpenPlaylist(pl.id); }}
              >
                <div class="album-thumb-box">
                  <img src={pl.thumbnail || '/placeholder.jpg'} alt="" class="album-thumb" />
                  <div class="card-play-btn">▶</div>
                </div>
                <div class="card-title">{pl.title}</div>
                <div class="card-subtitle">{pl.artist || 'Playlist'}</div>
              </div>
            {/each}
          </div>
        </div>
      {/if}
    </div>
  {:else}
    <!-- Empty Initial Prompt -->
    <div class="empty-prompt">
      <div class="prompt-icon">🪐</div>
      <h3>Search YouTube Music</h3>
      <p>Type any song name, artist, album or soundtrack above to discover music.</p>
    </div>
  {/if}
</div>

<style>
.search-container {
  padding: 24px 32px 80px 32px;
  max-width: 1400px;
  margin: 0 auto;
}

.filter-tabs {
  display: flex;
  gap: 10px;
  margin-top: 18px;
}

.tab-btn {
  padding: 7px 16px;
  border-radius: 999px;
  background: rgba(30, 30, 42, 0.6);
  border: 1px solid rgba(255, 255, 255, 0.08);
  color: #94a3b8;
  font-size: 0.85rem;
  cursor: pointer;
  transition: all 0.2s ease;
}

.tab-btn:hover {
  color: #f1f5f9;
  background: rgba(45, 40, 65, 0.6);
}

.tab-btn.active {
  background: rgba(139, 92, 246, 0.25);
  border-color: #8b5cf6;
  color: #c4b5fd;
  font-weight: 600;
}

.results-layout {
  display: flex;
  flex-direction: column;
  gap: 36px;
}

.results-group {
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.group-title {
  font-size: 1.3rem;
  font-weight: 700;
  color: #f1f5f9;
  margin: 0;
}

.song-list {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.song-row {
  display: flex;
  align-items: center;
  gap: 14px;
  padding: 8px 14px;
  border-radius: 10px;
  background: rgba(24, 24, 32, 0.35);
  border: 1px solid rgba(255, 255, 255, 0.03);
  cursor: pointer;
  transition: all 0.15s ease;
}

.song-row:hover {
  background: rgba(40, 36, 60, 0.6);
  border-color: rgba(139, 92, 246, 0.25);
}

.song-index {
  width: 22px;
  text-align: center;
  color: #64748b;
  font-size: 0.85rem;
}

.song-thumb-box {
  position: relative;
  width: 44px;
  height: 44px;
  border-radius: 6px;
  overflow: hidden;
  background: #111116;
}

.song-thumb {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.row-play-overlay {
  position: absolute;
  inset: 0;
  background: rgba(0, 0, 0, 0.4);
  display: flex;
  align-items: center;
  justify-content: center;
  color: #fff;
  font-size: 0.8rem;
  opacity: 0;
  transition: opacity 0.15s ease;
}

.song-row:hover .row-play-overlay {
  opacity: 1;
}

.song-details {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 2px;
  min-width: 0;
}

.song-title {
  color: #f1f5f9;
  font-size: 0.92rem;
  font-weight: 600;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.song-artist {
  color: #94a3b8;
  font-size: 0.8rem;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.song-duration {
  color: #64748b;
  font-size: 0.82rem;
  padding: 0 10px;
}

.row-play-btn {
  background: rgba(139, 92, 246, 0.2);
  border: 1px solid rgba(139, 92, 246, 0.4);
  color: #c4b5fd;
  width: 32px;
  height: 32px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  font-size: 0.75rem;
  transition: all 0.15s ease;
}

.row-play-btn:hover {
  background: #8b5cf6;
  color: #fff;
  transform: scale(1.08);
}

.cards-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(160px, 1fr));
  gap: 18px;
}

.album-card, .artist-card {
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding: 12px;
  border-radius: 12px;
  background: rgba(24, 24, 32, 0.4);
  border: 1px solid rgba(255, 255, 255, 0.04);
  cursor: pointer;
  transition: all 0.2s ease;
}

.album-card:hover, .artist-card:hover {
  background: rgba(35, 33, 52, 0.7);
  border-color: rgba(139, 92, 246, 0.3);
  transform: translateY(-4px);
}

.album-thumb-box {
  position: relative;
  aspect-ratio: 1;
  border-radius: 8px;
  overflow: hidden;
  background: #111116;
}

.album-thumb {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.card-play-btn {
  position: absolute;
  bottom: 8px;
  right: 8px;
  width: 34px;
  height: 34px;
  border-radius: 50%;
  background: #8b5cf6;
  color: #fff;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 0.85rem;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.4);
  opacity: 0;
  transform: translateY(4px);
  transition: all 0.2s ease;
}

.album-card:hover .card-play-btn {
  opacity: 1;
  transform: translateY(0);
}

.artist-avatar {
  aspect-ratio: 1;
  border-radius: 50%;
  object-fit: cover;
  background: #111116;
}

.card-title {
  font-size: 0.88rem;
  font-weight: 600;
  color: #f1f5f9;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.card-subtitle {
  font-size: 0.76rem;
  color: #94a3b8;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.text-center {
  text-align: center;
}

.loading-state, .empty-prompt {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 80px 0;
  text-align: center;
}

.prompt-icon {
  font-size: 3rem;
  margin-bottom: 12px;
}

.empty-prompt h3 {
  margin: 0 0 6px 0;
  color: #f1f5f9;
}

.empty-prompt p {
  margin: 0;
  color: #64748b;
  max-width: 400px;
}

.cosmic-spinner {
  width: 44px;
  height: 44px;
  border-radius: 50%;
  border: 3px solid rgba(139, 92, 246, 0.15);
  border-top-color: #8b5cf6;
  animation: spin 0.8s linear infinite;
  margin-bottom: 12px;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

.error-msg {
  padding: 20px;
  border-radius: 12px;
  background: rgba(239, 68, 68, 0.15);
  border: 1px solid rgba(239, 68, 68, 0.3);
  color: #f87171;
}

.search-container { max-width: none; min-height: 100%; margin: 0; padding: 30px clamp(24px, 4vw, 58px) 86px; color: #f3f5f4; background: #080909; }
.search-results-heading { display: flex; align-items: end; justify-content: space-between; margin: 4px 0 22px; }
.search-results-heading span { color: var(--accent); font-size: 9px; font-weight: 800; letter-spacing: .16em; }
.search-results-heading h1 { margin: 7px 0 5px; color: #f4f6f5; font-size: clamp(25px, 3vw, 36px); font-weight: 750; letter-spacing: -.045em; }
.search-results-heading p { margin: 0; color: #999f9c; font-size: 12px; }
.filter-tabs { gap: 8px; margin-bottom: 20px; }
.tab-btn { color: #d6dbd8; background: #202322; border: 1px solid transparent; }
.tab-btn:hover { color: #fff; background: #2a2e2b; }
.tab-btn.active { color: #fff; background: rgba(var(--cover-rgb, 244 107 130),.25); border-color: rgba(var(--cover-rgb, 244 107 130),.56); }
.results-layout { gap: 30px; }
.group-title { font-size: 20px; letter-spacing: -.03em; }
.song-list { gap: 2px; }
.song-row { padding: 7px 10px; background: transparent; border-color: transparent; border-radius: 5px; }
.song-row :global(.track-actions) { margin-left: 2px; }
.song-row:hover { background: #1a1d1c; border-color: transparent; }
.song-index, .song-duration { color: #a1a7a3; }
.song-title { font-weight: 550; }
.song-artist { color: #a0a6a2; }
.row-play-btn { color: #f5f6f5; background: #202322; border-color: #383d39; }
.row-play-btn:hover { color: #fff; background: var(--accent); border-color: var(--accent); }
.cards-grid { gap: 16px; }
.album-card, .artist-card { background: transparent; border: 1px solid transparent; }
.album-card:hover, .artist-card:hover { background: #171a19; border-color: #282c29; }
.card-title { color: #f2f4f3; }
.card-subtitle { color: #9ea49f; }
.empty-prompt h3 { color: #f1f4f2; }
.empty-prompt p { color: #9ba19d; }
.cosmic-spinner { border-color: rgba(var(--cover-rgb, 244 107 130),.16); border-top-color: var(--accent); }
.error-msg { color: #ffc7c7; background: rgba(179,43,43,.12); border-color: rgba(245,90,90,.22); }
</style>
