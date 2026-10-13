<script lang="ts">
import { onMount } from 'svelte';
import { invoke } from '@tauri-apps/api/core';
import TrackActions from '../../lib/TrackActions.svelte';
import { RefreshCw, Clock3 } from 'lucide-svelte';
import { artworkFor, recoverArtwork } from '../../lib/artwork';

export let onPlayTrack: (track: any) => void = () => {};
export let onOpenPlaylist: (id: string) => void = () => {};
export let onOpenArtist: (id: string) => void = () => {};
export let onTrackAction: (track: any, action: string) => void = () => {};
export let playlistOptions: any[] = [];
export let libraryCollections: Record<string, any[]> = { playlists: [], albums: [], artists: [] };
export let libraryLoading = false;
export let accountInfo: any = null;
export let fallbackArtwork = '';

let loading = true;
let libraryData: any[] = [];
let albumData: any[] = [];
let artistData: any[] = [];
let historyGroups: any[] = [];
let historyLoading = false;
let historyRequestSerial = 0;
let visibleHistoryCount = 50;
let visiblePlaylistCount = 40;
let visibleAlbumCount = 40;
let visibleArtistCount = 40;

async function loadHistory() {
  const request = ++historyRequestSerial;
  historyLoading = true;
  try {
    const settings: any = await invoke('get_settings').catch(() => ({}));
    if (settings?.history_enabled === 'false') {
      historyGroups = [];
    } else {
      const hist: any = await invoke('get_history').catch(() => []);
      if (request !== historyRequestSerial) return;
      const entries = Array.isArray(hist) ? hist : [];
      historyGroups = entries.some((entry) => Array.isArray(entry?.items))
        ? entries
        : (entries.length ? [{ header: 'Recently played in NOVA', items: entries }] : []);
    }
  } catch (err) {
    console.error('Failed to load listening history:', err);
  } finally {
    if (request === historyRequestSerial) historyLoading = false;
  }
}

onMount(() => {
  void loadHistory();
  const refresh = () => void loadHistory();
  window.addEventListener('nova-account-updated', refresh);
  window.addEventListener('nova-refresh-current-view', refresh);
  window.addEventListener('nova-playlist-updated', refresh);
  window.addEventListener('nova-local-playlists-changed', refresh);
  return () => {
    window.removeEventListener('nova-account-updated', refresh);
    window.removeEventListener('nova-refresh-current-view', refresh);
    window.removeEventListener('nova-playlist-updated', refresh);
    window.removeEventListener('nova-local-playlists-changed', refresh);
  };
});

$: libraryData = Array.isArray(libraryCollections?.playlists) ? libraryCollections.playlists : [];
$: albumData = Array.isArray(libraryCollections?.albums) ? libraryCollections.albums : [];
$: artistData = Array.isArray(libraryCollections?.artists) ? libraryCollections.artists : [];
$: loading = libraryLoading;

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

function playFirstRecentTrack() {
  const first = historyGroups.flatMap((group) => Array.isArray(group?.items) ? group.items : [])[0];
  if (first) handlePlayHistoryItem(first);
}

$: recentTracks = historyGroups.flatMap((group) => Array.isArray(group?.items) ? group.items : []);
$: visibleHistoryGroups = (() => {
  let remaining = visibleHistoryCount;
  return historyGroups.map((group) => {
    const items = (Array.isArray(group?.items) ? group.items : []).slice(0, remaining);
    remaining = Math.max(0, remaining - items.length);
    return { ...group, items };
  }).filter((group) => group.items.length);
})();
$: visiblePlaylists = libraryData.slice(0, visiblePlaylistCount);
$: visibleAlbums = albumData.slice(0, visibleAlbumCount);
$: visibleArtists = artistData.slice(0, visibleArtistCount);
$: savedCollectionCount = libraryData.length + albumData.length + artistData.length;
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
    <p>Your playlists, albums, artists, and listening history in one place.</p>
    <div class="library-toolbar">
      <div class="library-filter-pills"><span class="library-pill active">All collections</span><span class="library-pill">Playlists {libraryData.length}</span><span class="library-pill">Albums {albumData.length}</span><span class="library-pill">Artists {artistData.length}</span><small>{savedCollectionCount} saved</small></div>
      <button class="library-refresh" on:click={() => window.dispatchEvent(new CustomEvent('nova-local-playlists-changed'))} disabled={loading || historyLoading}><RefreshCw size={14} class={loading || historyLoading ? 'spinning' : ''} /> Refresh</button>
    </div>
  </div>

  {#if loading}
    <div class="loading-state">
      <div class="cosmic-spinner"></div>
      <p>Loading your music galaxy...</p>
    </div>
  {:else}
    <!-- Quick Tiles -->
    <div class="quick-tiles">
      <button type="button" class="tile liked-tile" on:click={() => onOpenPlaylist('NOVA_LIKED')}>
        <div class="tile-icon">❤️</div>
        <div class="tile-content">
          <h3>Liked Songs</h3>
          <p>Your favorite music from YouTube</p>
        </div>
      </button>

      <button type="button" class="tile repeat-tile" on:click={playFirstRecentTrack} disabled={!recentTracks.length}>
        <div class="tile-icon"><Clock3 size={25} /></div>
        <div class="tile-content">
          <h3>Continue listening</h3>
          <p>{recentTracks[0]?.title || 'Your recent tracks appear here'}</p>
        </div>
        <span class="tile-play">▶</span>
      </button>
    </div>

    <!-- Recent History Section -->
    {#if historyGroups && historyGroups.length > 0}
      <div class="library-section">
        <h2>Recently played</h2>
        <div class="history-list">
          {#each visibleHistoryGroups as group}
            {#if group.items && group.items.length > 0}
              <div class="group-header">{group.header || 'Recent'}</div>
              {#each group.items as item}
                <div class="history-row-wrap">
                  <button type="button" class="history-row" on:click={() => handlePlayHistoryItem(item)}>
                    <img src={artworkFor(item, 1000, fallbackArtwork)} alt="" class="history-thumb" loading="lazy" decoding="async" on:error={(event) => recoverArtwork(event, { ...item, fallbackArtwork })} />
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
        {#if recentTracks.length > visibleHistoryCount}<button class="library-load-more" on:click={() => visibleHistoryCount += 50}>Show more listening history</button>{/if}
      </div>
    {:else if historyLoading}
      <div class="library-section history-loading"><h2>Recently played</h2><p>Loading your latest listening…</p></div>
    {/if}

    <!-- Playlists Grid -->
    {#if libraryData && Array.isArray(libraryData) && libraryData.length > 0}
      <div class="library-section">
        <h2>Your Playlists</h2>
        <div class="playlists-grid">
          {#each visiblePlaylists as item (item.id || item.playlistId || item.browseId || item.title)}
            <button type="button"
              class="lib-card"
              on:click={() => onOpenPlaylist(item.id || item.playlistId || item.browseId || item.browse_id || '')}
            >
              <img src={artworkFor(item, 1400, fallbackArtwork)} alt="" class="card-art" loading="lazy" decoding="async" on:error={(event) => recoverArtwork(event, { ...item, fallbackArtwork })} />
              <div class="card-title">{item.title}</div>
              <div class="card-subtitle">{item.subtitle || 'Playlist'}</div>
            </button>
          {/each}
        </div>
        {#if libraryData.length > visiblePlaylistCount}<button class="library-load-more" on:click={() => visiblePlaylistCount += 40}>Show more playlists</button>{/if}
      </div>
    {:else if !accountInfo?.is_logged_in && !historyGroups.length}
      <div class="empty-notice">
        <div class="notice-icon">♫</div>
        <h3>Connect your YouTube Account</h3>
        <p>Connect YouTube Music in Settings to sync playlists, albums, and recommendations.</p>
        <button class="connect-account-button" on:click={() => window.dispatchEvent(new CustomEvent('nova-open-settings'))}>Open account settings</button>
      </div>
    {/if}

    {#if albumData.length}
      <div class="library-section">
        <h2>Your Albums</h2>
        <div class="playlists-grid">
          {#each visibleAlbums as item (item.id)}
            <button type="button" class="lib-card" on:click={() => onOpenPlaylist(item.id)}>
              <img src={artworkFor(item, 1400, fallbackArtwork)} alt="" class="card-art" loading="lazy" decoding="async" on:error={(event) => recoverArtwork(event, { ...item, fallbackArtwork })} />
              <div class="card-title">{item.title}</div><div class="card-subtitle">{item.subtitle || 'Album'}</div>
            </button>
          {/each}
        </div>
        {#if albumData.length > visibleAlbumCount}<button class="library-load-more" on:click={() => visibleAlbumCount += 40}>Show more albums</button>{/if}
      </div>
    {/if}

    {#if artistData.length}
      <div class="library-section">
        <h2>Your Artists</h2>
        <div class="playlists-grid">
          {#each visibleArtists as item (item.id)}
            <button type="button" class="lib-card" on:click={() => onOpenArtist(item.id)}>
              <img src={artworkFor(item, 1000, fallbackArtwork)} alt="" class="card-art" loading="lazy" decoding="async" on:error={(event) => recoverArtwork(event, { ...item, fallbackArtwork })} />
              <div class="card-title">{item.title}</div><div class="card-subtitle">{item.subtitle || 'Artist'}</div>
            </button>
          {/each}
        </div>
        {#if artistData.length > visibleArtistCount}<button class="library-load-more" on:click={() => visibleArtistCount += 40}>Show more artists</button>{/if}
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

  .library-container { max-width: none; min-height: 100%; padding: 30px clamp(24px, 4vw, 58px) 86px; color: #f3f5f4; background: #080909; }
  .library-header { margin-bottom: 20px; }
  .header-badge { color: #969c99; font-size: 9px; font-weight: 800; letter-spacing: .15em; }
  .title-row h1 { font-size: clamp(26px, 3vw, 36px); letter-spacing: -.045em; }
  .library-header p { font-size: 12px; }
  .library-toolbar { min-height: 44px; display: flex; align-items: center; justify-content: space-between; gap: 12px; margin-top: 19px; border-bottom: 1px solid rgba(255,255,255,.09); }
  .library-filter-pills { display: flex; align-items: center; flex-wrap: wrap; gap: 7px; }
  .library-pill { padding: 7px 11px; color: #aeb4b1; background: #161918; border: 1px solid rgba(255,255,255,.045); border-radius: 18px; font-size: 10px; }
  .library-pill.active { color: #111; background: #f2f4f3; }
  .library-filter-pills small { margin-left: 5px; color: #777f7b; font-size: 10px; }
  .library-refresh { display: flex; align-items: center; gap: 6px; padding: 7px 10px; color: #c3c8c5; background: transparent; border: 1px solid rgba(255,255,255,.08); border-radius: 18px; font-size: 10px; cursor: pointer; }
  .library-refresh:hover { color: #fff; background: #1b1e1d; }
  .library-refresh:disabled { opacity: .55; cursor: default; }
  .library-load-more { min-height: 32px; margin-top: 12px; padding: 0 13px; color: rgba(255,255,255,.82); background: rgba(255,255,255,.07); border: 1px solid rgba(255,255,255,.11); border-radius: 18px; font-size: 10px; cursor: pointer; }
  .library-load-more:hover { color: #fff; background: rgba(255,255,255,.12); }
  .history-loading p { color: rgba(255,255,255,.56); font-size: 11px; }
  .quick-tiles { grid-template-columns: repeat(2, minmax(0,1fr)); gap: 10px; margin: 19px 0 28px; }
  .tile { min-height: 82px; padding: 14px 16px; border-radius: 10px; }
  .liked-tile { background: linear-gradient(110deg, #33243a, #1a1a22 72%); border-color: rgba(211,102,178,.23); }
  .repeat-tile { justify-content: flex-start; color: #f5f6f6; background: linear-gradient(110deg, #283638, #171a1a 72%); border-color: rgba(106,184,195,.19); }
  .repeat-tile:disabled { opacity: .68; cursor: default; }
  .tile-icon { min-width: 40px; }
  .tile-content h3 { margin-bottom: 4px; font-size: 13px; }
  .tile-content p { max-width: 380px; overflow: hidden; margin: 0; color: #a8aeab; font-size: 10px; text-overflow: ellipsis; white-space: nowrap; }
  .tile-play { margin-left: auto; color: #d8e5e5; opacity: 0; transition: opacity .18s ease; }
  .tile:hover .tile-play { opacity: 1; }
  .library-section { margin: 25px 0 30px; }
  .library-section h2 { margin-bottom: 10px; color: #f2f4f3; font-size: 17px; letter-spacing: -.025em; }
  .playlists-grid { grid-template-columns: repeat(auto-fill, minmax(145px, 1fr)); gap: 8px; }
  .lib-card { padding: 10px; text-align: left; background: transparent; border: 1px solid transparent; border-radius: 8px; transition: background .18s ease, border-color .18s ease, transform .18s ease; }
  .lib-card:hover { background: #171a19; border-color: rgba(255,255,255,.08); transform: translateY(-2px); }
  .card-art { border-radius: 6px; }
  .card-title { margin-top: 7px; }
  .history-row { min-height: 58px; background: transparent; border-color: transparent; border-radius: 6px; }
  .history-row:hover { background: #181b1a; border-color: transparent; }
  .empty-notice { background: #121414; border-color: rgba(255,255,255,.08); }
  .connect-account-button { margin-top: 14px; padding: 9px 13px; color: #101211; background: #f1f3f2; border: 0; border-radius: 18px; font-size: 10px; font-weight: 700; cursor: pointer; }
  .connect-account-button:hover { background: #fff; }

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
