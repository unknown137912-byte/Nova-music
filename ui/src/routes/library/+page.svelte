<script lang="ts">
import { onMount } from 'svelte';
import { invoke } from '@tauri-apps/api/core';
import TrackActions from '../../lib/TrackActions.svelte';

export let onPlayTrack: (track: any) => void = () => {};
export let onOpenPlaylist: (id: string) => void = () => {};
export let onOpenArtist: (id: string) => void = () => {};
export let onTrackAction: (track: any, action: string) => void = () => {};
export let playlistOptions: any[] = [];

let loading = true;
let libraryData: any = null;
let albumData: any[] = [];
let artistData: any[] = [];
let historyGroups: any[] = [];
let accountInfo: any = null;

async function loadData() {
  try {
    loading = true;
    const [lib, settings, acc]: [any, any, any] = await Promise.all([
      invoke('get_library').catch(() => []),
      invoke('get_settings').catch(() => ({})),
      invoke('get_account_info').catch(() => null),
    ]);
    libraryData = Array.isArray(lib) ? lib : (lib?.playlists || []);
    albumData = Array.isArray(lib?.albums) ? lib.albums : [];
    artistData = Array.isArray(lib?.artists) ? lib.artists : [];
    if (settings?.history_enabled === 'false') {
      historyGroups = [];
    } else {
      const hist: any = await invoke('get_history').catch(() => []);
      historyGroups = Array.isArray(hist) ? hist : [];
    }
    accountInfo = acc;
    loading = false;
  } catch (err) {
    console.error('Failed to load library data:', err);
    loading = false;
  }
}

onMount(() => {
  loadData();
});

function handlePlayHistoryItem(item: any) {
  onPlayTrack({
    id: item.videoId || item.video_id || item.id,
    title: item.title,
    artist: item.artist || item.subtitle || 'YouTube Music',
    thumbnail: item.thumbnail,
    duration: item.duration ? parseDuration(item.duration) : 180,
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

<div class="library-container">
  <div class="library-header">
    <div class="header-badge">✦ COLLECTION</div>
    <div class="title-row">
      <h1>Your Library</h1>
      {#if accountInfo?.is_logged_in}
        <span class="user-pill">
          👤 {accountInfo.name || 'Synced Account'}
        </span>
      {/if}
    </div>
    <p>Saved albums, personal playlists, and synced YouTube Music history</p>
  </div>

  {#if loading}
    <div class="loading-state">
      <div class="cosmic-spinner"></div>
      <p>Loading your music galaxy...</p>
    </div>
  {:else}
    <!-- Quick Tiles -->
    <div class="quick-tiles">
      <button type="button" class="tile liked-tile" on:click={() => onOpenPlaylist('VLLM')}>
        <div class="tile-icon">❤️</div>
        <div class="tile-content">
          <h3>Liked Songs</h3>
          <p>Your favorite music from YouTube</p>
        </div>
      </button>

      <button type="button" class="tile repeat-tile" on:click={() => onOpenPlaylist('ON_REPEAT')}>
        <div class="tile-icon">🔁</div>
        <div class="tile-content">
          <h3>On Repeat</h3>
          <p>Your most played tracks this month</p>
        </div>
      </button>
    </div>

    <!-- Recent History Section -->
    {#if historyGroups && historyGroups.length > 0}
      <div class="library-section">
        <h2>Recently Played (Account History)</h2>
        <div class="history-list">
          {#each historyGroups as group}
            {#if group.items && group.items.length > 0}
              <div class="group-header">{group.header || 'Recent'}</div>
              {#each group.items as item}
                <div class="history-row-wrap">
                  <button type="button" class="history-row" on:click={() => handlePlayHistoryItem(item)}>
                    <img src={item.thumbnail || '/placeholder.jpg'} alt="" class="history-thumb" />
                    <div class="history-meta">
                      <div class="history-title">{item.title}</div>
                      <div class="history-sub">{item.artist || item.subtitle || 'Track'}</div>
                    </div>
                    <div class="history-play">▶</div>
                  </button>
                  <TrackActions track={item} playlists={playlistOptions} onAction={onTrackAction} />
                </div>
              {/each}
            {/if}
          {/each}
        </div>
      </div>
    {/if}

    <!-- Playlists Grid -->
    {#if libraryData && Array.isArray(libraryData) && libraryData.length > 0}
      <div class="library-section">
        <h2>Your Playlists</h2>
        <div class="playlists-grid">
          {#each libraryData as item}
            <button type="button"
              class="lib-card"
              on:click={() => onOpenPlaylist(item.id)}
            >
              <img src={item.thumbnail || '/placeholder.jpg'} alt="" class="card-art" />
              <div class="card-title">{item.title}</div>
              <div class="card-subtitle">{item.subtitle || 'Playlist'}</div>
            </button>
          {/each}
        </div>
      </div>
    {:else if !accountInfo?.is_logged_in}
      <div class="empty-notice">
        <div class="notice-icon">♫</div>
        <h3>Connect your YouTube Account</h3>
        <p>Go to Settings to sign in with your Google account or paste your session cookie to load your playlists and watch history.</p>
      </div>
    {/if}

    {#if albumData.length}
      <div class="library-section">
        <h2>Your Albums</h2>
        <div class="playlists-grid">
          {#each albumData as item (item.id)}
            <button type="button" class="lib-card" on:click={() => onOpenPlaylist(item.id)}>
              <img src={item.thumbnail || '/placeholder.jpg'} alt="" class="card-art" />
              <div class="card-title">{item.title}</div><div class="card-subtitle">{item.subtitle || 'Album'}</div>
            </button>
          {/each}
        </div>
      </div>
    {/if}

    {#if artistData.length}
      <div class="library-section">
        <h2>Your Artists</h2>
        <div class="playlists-grid">
          {#each artistData as item (item.id)}
            <button type="button" class="lib-card" on:click={() => onOpenArtist(item.id)}>
              <img src={item.thumbnail || '/placeholder.jpg'} alt="" class="card-art" />
              <div class="card-title">{item.title}</div><div class="card-subtitle">{item.subtitle || 'Artist'}</div>
            </button>
          {/each}
        </div>
      </div>
    {/if}
  {/if}
</div>

<style>
.library-container {
  padding: 24px 32px 80px 32px;
  max-width: 1400px;
  margin: 0 auto;
}

.library-header {
  margin-bottom: 30px;
}

.header-badge {
  color: #a78bfa;
  font-size: 0.75rem;
  font-weight: 700;
  letter-spacing: 0.08em;
  margin-bottom: 6px;
}

.title-row {
  display: flex;
  align-items: center;
  gap: 16px;
  margin-bottom: 6px;
}

.title-row h1 {
  font-size: 2.2rem;
  font-weight: 800;
  color: #f8fafc;
  margin: 0;
}

.user-pill {
  background: rgba(139, 92, 246, 0.15);
  border: 1px solid rgba(139, 92, 246, 0.35);
  color: #c4b5fd;
  padding: 4px 12px;
  border-radius: 999px;
  font-size: 0.85rem;
  font-weight: 600;
}

.library-header p {
  color: #94a3b8;
  font-size: 0.95rem;
  margin: 0;
}

.quick-tiles {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
  gap: 20px;
  margin-bottom: 36px;
}

.tile {
  display: flex;
  align-items: center;
  gap: 18px;
  padding: 24px;
  border-radius: 16px;
  cursor: pointer;
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
  border: 1px solid rgba(255, 255, 255, 0.05);
}

.liked-tile {
  background: linear-gradient(135deg, rgba(236, 72, 153, 0.15) 0%, rgba(30, 27, 75, 0.4) 100%);
  border-color: rgba(236, 72, 153, 0.3);
}

.liked-tile:hover {
  transform: translateY(-3px);
  box-shadow: 0 10px 24px -6px rgba(236, 72, 153, 0.3);
}

.repeat-tile {
  background: linear-gradient(135deg, rgba(99, 102, 241, 0.15) 0%, rgba(30, 27, 75, 0.4) 100%);
  border-color: rgba(99, 102, 241, 0.3);
}

.repeat-tile:hover {
  transform: translateY(-3px);
  box-shadow: 0 10px 24px -6px rgba(99, 102, 241, 0.3);
}

.tile-icon {
  font-size: 2rem;
}

.tile-content h3 {
  margin: 0 0 4px 0;
  color: #f8fafc;
  font-size: 1.15rem;
}

.tile-content p {
  margin: 0;
  color: #94a3b8;
  font-size: 0.85rem;
}

.library-section {
  margin-bottom: 36px;
}

.library-section h2 {
  font-size: 1.3rem;
  color: #f8fafc;
  margin: 0 0 16px 0;
}

.group-header {
  color: #a78bfa;
  font-size: 0.82rem;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  margin: 12px 0 6px 0;
}

.history-list {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.history-row {
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

.history-row:hover {
  background: rgba(40, 36, 60, 0.6);
  border-color: rgba(139, 92, 246, 0.25);
}

.history-thumb {
  width: 44px;
  height: 44px;
  border-radius: 6px;
  object-fit: cover;
  background: #111116;
}

.history-meta {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 2px;
  min-width: 0;
}

.history-title {
  color: #f1f5f9;
  font-size: 0.92rem;
  font-weight: 600;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.history-sub {
  color: #94a3b8;
  font-size: 0.8rem;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.history-play {
  width: 32px;
  height: 32px;
  border-radius: 50%;
  background: rgba(139, 92, 246, 0.2);
  color: #c4b5fd;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 0.8rem;
}

.history-row:hover .history-play {
  background: #8b5cf6;
  color: #fff;
}

.playlists-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(180px, 1fr));
  gap: 20px;
}

.lib-card {
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding: 12px;
  border-radius: 14px;
  background: rgba(24, 24, 32, 0.4);
  border: 1px solid rgba(255, 255, 255, 0.04);
  cursor: pointer;
  transition: all 0.2s ease;
}

.lib-card:hover {
  background: rgba(35, 33, 52, 0.7);
  border-color: rgba(139, 92, 246, 0.3);
  transform: translateY(-4px);
}

.card-art {
  width: 100%;
  aspect-ratio: 1;
  border-radius: 10px;
  object-fit: cover;
  background: #111116;
}

.card-title {
  color: #f1f5f9;
  font-size: 0.9rem;
  font-weight: 600;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.card-subtitle {
  color: #94a3b8;
  font-size: 0.78rem;
}

.empty-notice {
  text-align: center;
  padding: 60px 20px;
  background: rgba(24, 24, 34, 0.4);
  border: 1px dashed rgba(139, 92, 246, 0.25);
  border-radius: 18px;
}

.notice-icon {
  font-size: 2.5rem;
  margin-bottom: 12px;
}

.empty-notice h3 {
  margin: 0 0 6px 0;
  color: #f1f5f9;
}

.empty-notice p {
  margin: 0;
  color: #64748b;
  max-width: 440px;
  margin: 0 auto;
}

.loading-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 80px 0;
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

.library-container { max-width: none; margin: 0; padding: 36px clamp(24px, 4vw, 58px) 86px; color: #f3f5f4; background: radial-gradient(ellipse at 42% 0%, rgba(51,101,76,.17), transparent 42%), rgba(8,9,9,.88); }
.library-header { margin-bottom: 26px; }
.header-badge { color: #aeb4b0; }
.title-row h1 { font-size: clamp(27px, 3vw, 36px); letter-spacing: -.04em; }
.user-pill { color: #e5e9e6; background: #202322; border-color: #343936; }
.library-header p { color: #a0a6a2; }
.quick-tiles { gap: 12px; margin-bottom: 30px; }
.tile { padding: 18px 20px; background: linear-gradient(135deg, rgba(43,48,45,.79), rgba(19,21,20,.76)); border: 1px solid rgba(255,255,255,.085); border-radius: 9px; box-shadow: inset 0 1px 0 rgba(255,255,255,.09); -webkit-backdrop-filter: blur(20px) saturate(1.28); backdrop-filter: blur(20px) saturate(1.28); }
.liked-tile, .repeat-tile { background: linear-gradient(135deg, rgba(43,48,45,.82), rgba(19,21,20,.76)); border-color: rgba(255,255,255,.09); }
.liked-tile:hover, .repeat-tile:hover { transform: translateY(-2px); background: linear-gradient(135deg, rgba(58,65,60,.88), rgba(25,28,26,.82)); box-shadow: 0 12px 25px rgba(0,0,0,.18); }
.tile-content h3 { font-size: 14px; }
.tile-content p { color: #a0a6a2; }
.library-section { margin-bottom: 29px; }
.library-section h2 { font-size: 19px; letter-spacing: -.025em; }
.group-header { color: #b3b9b5; }
.history-list { gap: 2px; }
.history-row { padding: 7px 10px; background: rgba(255,255,255,.025); border-color: rgba(255,255,255,.025); border-radius: 6px; -webkit-backdrop-filter: blur(14px); backdrop-filter: blur(14px); }
.history-row:hover { background: rgba(255,255,255,.085); border-color: rgba(255,255,255,.06); }
.history-title { color: #f1f3f2; }
.history-sub { color: #a0a6a2; }
.playlists-grid { gap: 16px; }
.lib-card { padding: 10px; background: rgba(255,255,255,.025); border-color: rgba(255,255,255,.035); border-radius: 9px; -webkit-backdrop-filter: blur(14px); backdrop-filter: blur(14px); }
.lib-card:hover { background: rgba(255,255,255,.09); border-color: rgba(255,255,255,.08); transform: translateY(-2px); }
.card-title { color: #f1f3f2; }
.card-subtitle { color: #9da39f; }
.empty-notice h3 { color: #f1f3f2; }
.empty-notice p { color: #9da39f; }
.cosmic-spinner { border-color: rgba(30,215,96,.14); border-top-color: #1ed760; }


.tile, .history-row, .lib-card { width: 100%; color: #f3f5f4; font: inherit; text-align: left; cursor: pointer; }
.tile { justify-content: flex-start; }
.history-row { border: 1px solid transparent; }
.history-row-wrap { position: relative; min-width: 0; }
.history-row-wrap .history-row { padding-right: 48px; }
.history-row-wrap :global(.track-actions) { position: absolute; z-index: 2; top: 50%; right: 7px; transform: translateY(-50%); }
.lib-card { text-align: left; }
</style>
