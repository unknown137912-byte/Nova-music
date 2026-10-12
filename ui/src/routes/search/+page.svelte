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

<div class="search-container anim-fade-up">
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
    <div class="loading-state anim-fade-up">
      <div class="cosmic-spinner anim-glow"></div>
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
        <div class="results-group anim-fade-up">
          <h2 class="group-title">Songs</h2>
          <div class="song-list stagger">
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
                  <img src={song.thumbnail || '/placeholder.svg'} alt="" class="song-thumb" />
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
        <div class="results-group anim-fade-up">
          <h2 class="group-title">Albums</h2>
          <div class="cards-grid stagger">
            {#each results.albums as album}
              <div
                class="album-card"
                on:click={() => onOpenPlaylist(album.id)}
                role="button"
                tabindex="0"
                on:keydown={(e) => { if (e.key === 'Enter') onOpenPlaylist(album.id); }}
              >
                <div class="album-thumb-box">
                  <img src={album.thumbnail || '/placeholder.svg'} alt="" class="album-thumb" />
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
        <div class="results-group anim-fade-up">
          <h2 class="group-title">Artists</h2>
          <div class="cards-grid stagger">
            {#each results.artists as artist}
              <div
                class="artist-card"
                on:click={() => onOpenArtist(artist.id)}
                role="button"
                tabindex="0"
                on:keydown={(e) => { if (e.key === 'Enter') onOpenArtist(artist.id); }}
              >
                <img src={artist.thumbnail || '/placeholder.svg'} alt="" class="artist-avatar" />
                <div class="card-title text-center">{artist.title}</div>
                <div class="card-subtitle text-center">Artist</div>
              </div>
            {/each}
          </div>
        </div>
      {/if}

      <!-- Playlists Section -->
      {#if (activeFilter === 'all' || activeFilter === 'playlists') && results.playlists && results.playlists.length > 0}
        <div class="results-group anim-fade-up">
          <h2 class="group-title">Playlists</h2>
          <div class="cards-grid stagger">
            {#each results.playlists as pl}
              <div
                class="album-card"
                on:click={() => onOpenPlaylist(pl.id)}
                role="button"
                tabindex="0"
                on:keydown={(e) => { if (e.key === 'Enter') onOpenPlaylist(pl.id); }}
              >
                <div class="album-thumb-box">
                  <img src={pl.thumbnail || '/placeholder.svg'} alt="" class="album-thumb" />
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
    <div class="empty-prompt anim-fade-up">
      <div class="prompt-icon">🪐</div>
      <h3>Search YouTube Music</h3>
      <p>Type any song name, artist, album or soundtrack above to discover music.</p>
    </div>
  {/if}
</div>

<style>
.search-container {
  padding: 32px clamp(24px, 4vw, 58px) 80px;
  max-width: none;
  min-height: 100%;
  margin: 0;
  background: radial-gradient(ellipse at 48% 0%, rgba(var(--cover-rgb), 0.1), transparent 48%), linear-gradient(145deg, rgba(var(--cover-deep), 0.15), transparent 55%), var(--nova-background);
  color: var(--nova-text);
}

.search-results-heading {
  display: flex;
  align-items: end;
  justify-content: space-between;
  margin: 6px 0 24px;
}
.search-results-heading span {
  color: var(--nova-accent-soft);
  font-size: 10px;
  font-weight: 800;
  letter-spacing: 0.5px;
}
.search-results-heading h1 {
  margin: 8px 0 6px;
  color: var(--nova-text);
  font-size: clamp(26px, 3.5vw, 36px);
  font-weight: 700;
  letter-spacing: -0.5px;
}
.search-results-heading p {
  margin: 0;
  color: var(--nova-muted);
  font-size: 14px;
}

.filter-tabs {
  gap: 12px;
  margin-bottom: 24px;
}
.tab-btn {
  color: var(--nova-muted);
  background: rgba(255,255,255,0.06);
  border: 1px solid transparent;
  border-radius: 20px;
  padding: 8px 18px;
  font-size: 13px;
  font-weight: 600;
  transition: all 0.2s ease;
}
.tab-btn:hover {
  color: var(--nova-text);
  background: rgba(255,255,255,0.1);
}
.tab-btn.active {
  color: #fff;
  background: var(--nova-accent);
  border-color: var(--nova-accent);
  box-shadow: 0 0 0 2px rgba(var(--cover-rgb), 0.15);
}

.results-layout {
  display: flex;
  flex-direction: column;
  gap: 32px;
}
.results-group {
  display: flex;
  flex-direction: column;
  gap: 16px;
}
.group-title {
  font-size: 20px;
  font-weight: 700;
  letter-spacing: -0.3px;
  color: var(--nova-text);
  margin: 0;
}
.song-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.song-row {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 12px 20px;
  border-radius: 12px;
  background: rgba(255,255,255,0.04);
  border: 1px solid var(--nova-border);
  cursor: pointer;
  transition: all 0.25s ease;
  backdrop-filter: blur(20px);
  -webkit-backdrop-filter: blur(20px);
}
.song-row:hover {
  background: rgba(255,255,255,0.08);
  border-color: var(--nova-border-strong);
  transform: translateY(-2px);
}
.song-index {
  width: 24px;
  text-align: center;
  color: var(--nova-muted);
  font-size: 14px;
}
.song-thumb-box {
  position: relative;
  width: 56px;
  height: 56px;
  border-radius: 8px;
  overflow: hidden;
  background: #00000015;
}
.song-thumb {
  width: 100%;
  height: 100%;
  object-fit: cover;
  transition: transform 0.3s ease;
}
.song-row:hover .song-thumb {
  transform: scale(1.04);
}
.row-play-overlay {
  position: absolute;
  inset: 0;
  background: rgba(0,0,0,0.4);
  display: flex;
  align-items: center;
  justify-content: center;
  color: #fff;
  font-size: 0.9rem;
  opacity: 0;
  transition: opacity 0.2s ease;
}
.song-row:hover .row-play-overlay {
  opacity: 1;
}
.song-details {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 4px;
  min-width: 0;
}
.song-title {
  color: var(--nova-text);
  font-size: 16px;
  font-weight: 600;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.song-artist {
  color: var(--nova-muted);
  font-size: 14px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.song-duration {
  color: var(--nova-muted);
  font-size: 14px;
  padding: 0 12px;
}
.row-play-btn {
  background: rgba(255,255,255,0.1);
  border: 1px solid rgba(255,255,255,0.2);
  color: var(--nova-text);
  width: 38px;
  height: 38px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  font-size: 0.9rem;
  transition: all 0.2s ease;
}
.row-play-btn:hover {
  background: var(--nova-accent);
  color: #fff;
  transform: scale(1.08);
}

.cards-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(160px, 1fr));
  gap: 20px;
}
.album-card, .artist-card {
  display: flex;
  flex-direction: column;
  gap: 10px;
  padding: 16px;
  border-radius: 14px;
  background: transparent;
  border: 1px solid transparent;
  cursor: pointer;
  transition: all 0.25s ease;
  backdrop-filter: blur(20px);
  -webkit-backdrop-filter: blur(20px);
}
.album-card:hover, .artist-card:hover {
  background: rgba(255,255,255,0.06);
  border-color: var(--nova-border-strong);
  transform: translateY(-4px);
  box-shadow: 0 8px 24px rgba(0,0,0,0.2);
}
.album-thumb-box {
  position: relative;
  aspect-ratio: 1;
  border-radius: 10px;
  overflow: hidden;
  background: #00000015;
}
.album-thumb {
  width: 100%;
  height: 100%;
  object-fit: cover;
  transition: transform 0.3s ease;
}
.album-card:hover .album-thumb,
.artist-card:hover .artist-avatar {
  transform: scale(1.04);
}
.card-play-btn {
  position: absolute;
  bottom: 12px;
  right: 12px;
  width: 40px;
  height: 40px;
  border-radius: 50%;
  background: var(--nova-accent);
  color: #fff;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 1rem;
  box-shadow: 0 6px 16px rgba(0,0,0,0.3);
  opacity: 0;
  transform: translateY(6px);
  transition: all 0.2s ease;
}
.album-card:hover .card-play-btn,
.artist-card:hover .card-play-btn {
  opacity: 1;
  transform: translateY(0);
}
.artist-avatar {
  aspect-ratio: 1;
  border-radius: 50%;
  object-fit: cover;
  background: #00000015;
}
.card-title {
  font-size: 16px;
  font-weight: 600;
  color: var(--nova-text);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.card-subtitle {
  font-size: 14px;
  color: var(--nova-muted);
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
  padding: 96px 0;
  text-align: center;
}
.prompt-icon {
  font-size: 4rem;
  margin-bottom: 16px;
  opacity: 0.7;
}
.empty-prompt h3 {
  margin: 0 0 8px 0;
  color: var(--nova-text);
  font-size: 20px;
}
.empty-prompt p {
  margin: 0;
  color: var(--nova-muted);
  font-size: 16px;
  max-width: 420px;
  line-height: 1.5;
}
.cosmic-spinner {
  width: 56px;
  height: 56px;
  border-radius: 50%;
  border: 4px solid rgba(255,255,255,0.1);
  border-top-color: var(--nova-accent);
  animation: spin 0.9s linear infinite;
  margin-bottom: 16px;
}
@keyframes spin {
  to { transform: rotate(360deg); }
}
.error-msg {
  padding: 24px;
  border-radius: 14px;
  background: rgba(239, 68, 68, 0.1);
  border: 1px solid rgba(239, 68, 68, 0.2);
  color: #f87171;
  font-size: 15px;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 12px;
}
.error-msg::before {
  content: "⚠️";
  font-size: 1.5rem;
}
</style>