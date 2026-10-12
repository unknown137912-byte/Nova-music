<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import TrackActions from '../../lib/TrackActions.svelte';
  import { RefreshCw, Clock3 } from 'lucide-svelte';

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
        const entries = Array.isArray(hist) ? hist : [];
        historyGroups = entries.some((entry) => Array.isArray(entry?.items))
          ? entries
          : (entries.length ? [{ header: 'Recently played in NOVA', items: entries }] : []);
      }
      accountInfo = acc;
      loading = false;
    } catch (err) {
      console.error('Failed to load library data:', err);
      loading = false;
    }
  }

  onMount(() => {
    void loadData();
    const refresh = () => void loadData();
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
  $: savedCollectionCount = libraryData.length + albumData.length + artistData.length;
</script>

<div class="library-container anim-fade-up">
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
      <button class="library-refresh" on:click={loadData} disabled={loading}><RefreshCw size={14} class={loading ? 'spinning' : ''} /> Refresh</button>
    </div>
  </div>

  {#if loading}
    <div class="loading-state">
      <div class="cosmic-spinner anim-glow"></div>
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
          {#each historyGroups as group}
            {#if group.items && group.items.length > 0}
              <div class="group-header">{group.header || 'Recent'}</div>
              {#each group.items as item}
                <div class="history-row-wrap">
                  <button type="button" class="history-row" on:click={() => handlePlayHistoryItem(item)}>
                    <img src={item.thumbnail || '/placeholder.svg'} alt="" class="history-thumb" />
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
        <div class="playlists-grid stagger">
          {#each libraryData as item (item.id || item.playlistId || item.browseId || item.title)}
            <button type="button"
              class="lib-card"
              on:click={() => onOpenPlaylist(item.id || item.playlistId || item.browseId || item.browse_id || '')}
            >
              <img src={item.thumbnail || '/placeholder.svg'} alt="" class="card-art" />
              <div class="card-title">{item.title}</div>
              <div class="card-subtitle">{item.subtitle || 'Playlist'}</div>
            </button>
          {/each}
        </div>
      </div>
    {:else if !accountInfo?.is_logged_in && !historyGroups.length}
      <div class="empty-notice anim-fade-up">
        <div class="notice-icon">♫</div>
        <h3>Connect your YouTube Account</h3>
        <p>Connect YouTube Music in Settings to sync playlists, albums, and recommendations.</p>
        <button class="connect-account-button" on:click={() => window.dispatchEvent(new CustomEvent('nova-open-settings'))}>Open account settings</button>
      </div>
    {/if}

    {#if albumData.length}
      <div class="library-section anim-fade-up">
        <h2>Your Albums</h2>
        <div class="playlists-grid stagger">
          {#each albumData as item (item.id)}
            <button type="button" class="lib-card" on:click={() => onOpenPlaylist(item.id)}>
              <img src={item.thumbnail || '/placeholder.svg'} alt="" class="card-art" />
              <div class="card-title">{item.title}</div><div class="card-subtitle">{item.subtitle || 'Album'}</div>
            </button>
          {/each}
        </div>
      </div>
    {/if}

    {#if artistData.length}
      <div class="library-section anim-fade-up">
        <h2>Your Artists</h2>
        <div class="playlists-grid stagger">
          {#each artistData as item (item.id)}
            <button type="button" class="lib-card" on:click={() => onOpenArtist(item.id)}>
              <img src={item.thumbnail || '/placeholder.svg'} alt="" class="card-art" />
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
    padding: 48px clamp(24px, 4vw, 64px) 80px;
    max-width: none;
    min-height: 100%;
    margin: 0;
    background: radial-gradient(ellipse at 42% 0%, rgba(var(--cover-rgb), 0.12), transparent 48%), linear-gradient(145deg, rgba(var(--cover-deep), 0.18), transparent 55%), var(--nova-background);
    color: var(--nova-text);
  }
  .library-header { margin-bottom: 36px; }
  .header-badge {
    color: var(--nova-accent-soft);
    font-size: 10px;
    font-weight: 800;
    letter-spacing: 0.5px;
  }
  .title-row {
    display: flex;
    align-items: center;
    gap: 20px;
    margin-bottom: 12px;
  }
  .title-row h1 {
    font-size: clamp(28px, 3.5vw, 38px);
    font-weight: 800;
    color: var(--nova-text);
    margin: 0;
    letter-spacing: -0.5px;
  }
  .user-pill {
    background: rgba(var(--cover-rgb), 0.15);
    border: 1px solid rgba(var(--cover-rgb), 0.35);
    color: var(--nova-accent);
    padding: 6px 14px;
    border-radius: 999px;
    font-size: 14px;
    font-weight: 600;
  }
  .library-header p {
    color: var(--nova-muted);
    font-size: 16px;
    margin: 0;
  }
  .library-toolbar {
    min-height: 50px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
    margin-top: 24px;
    border-bottom: 1px solid var(--nova-border);
  }
  .library-filter-pills {
    display: flex;
    align-items: center;
    flex-wrap: wrap;
    gap: 10px;
  }
  .library-pill {
    padding: 10px 16px;
    color: var(--nova-muted);
    background: rgba(255,255,255,0.06);
    border: 1px solid transparent;
    border-radius: 22px;
    font-size: 14px;
    font-weight: 600;
    transition: all 0.2s ease;
  }
  .library-pill.active {
    color: #fff;
    background: var(--nova-accent);
    border-color: var(--nova-accent);
    box-shadow: 0 0 0 2px rgba(var(--cover-rgb), 0.15);
  }
  .library-filter-pills small {
    margin-left: 8px;
    color: var(--nova-muted);
    font-size: 14px;
  }
  .library-refresh {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 10px 16px;
    color: var(--nova-muted);
    background: transparent;
    border: 1px solid var(--nova-border);
    border-radius: 20px;
    font-size: 14px;
    cursor: pointer;
    transition: all 0.2s ease;
  }
  .library-refresh:hover {
    color: var(--nova-text);
    background: rgba(255,255,255,0.1);
  }
  .library-refresh:disabled {
    opacity: 0.55;
    cursor: wait;
  }
  .quick-tiles {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
    gap: 24px;
    margin-bottom: 32px;
  }
  .tile {
    display: flex;
    align-items: center;
    gap: 24px;
    padding: 28px;
    border-radius: 18px;
    cursor: pointer;
    transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
    border: 1px solid rgba(255,255,255,0.06);
    backdrop-filter: blur(20px);
    -webkit-backdrop-filter: blur(20px);
  }
  .liked-tile {
    background: linear-gradient(135deg, rgba(236, 72, 153, 0.18) 0%, rgba(30, 27, 75, 0.4) 100%);
    border-color: rgba(236, 72, 153, 0.35);
  }
  .liked-tile:hover {
    transform: translateY(-4px);
    box-shadow: 0 12px 28px -6px rgba(236, 72, 153, 0.35);
  }
  .repeat-tile {
    background: linear-gradient(135deg, rgba(99, 102, 241, 0.18) 0%, rgba(30, 27, 75, 0.4) 100%);
    border-color: rgba(99, 102, 241, 0.35);
  }
  .repeat-tile:hover {
    transform: translateY(-4px);
    box-shadow: 0 12px 28px -6px rgba(99, 102, 241, 0.35);
  }
  .tile-icon {
    font-size: 2.2rem;
  }
  .tile-content h3 {
    margin: 0 0 6px 0;
    color: var(--nova-text);
    font-size: 20px;
  }
  .tile-content p {
    margin: 0;
    color: var(--nova-muted);
    font-size: 16px;
    max-width: 400px;
    line-height: 1.5;
  }
  .library-section {
    margin-bottom: 40px;
  }
  .library-section h2 {
    font-size: 22px;
    color: var(--nova-text);
    margin: 0 0 20px 0;
  }
  .group-header {
    color: var(--nova-accent-soft);
    font-size: 14px;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    margin: 16px 0 12px 0;
  }
  .history-list {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  .history-row {
    display: flex;
    align-items: center;
    gap: 18px;
    padding: 14px 24px;
    border-radius: 12px;
    background: rgba(255,255,255,0.04);
    border: 1px solid var(--nova-border);
    cursor: pointer;
    transition: all 0.25s ease;
    backdrop-filter: blur(20px);
    -webkit-backdrop-filter: blur(20px);
  }
  .history-row:hover {
    background: rgba(255,255,255,0.08);
    border-color: var(--nova-border-strong);
    transform: translateY(-2px);
  }
  .history-thumb {
    width: 52px;
    height: 52px;
    border-radius: 8px;
    object-fit: cover;
    background: #00000015;
  }
  .history-meta {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 6px;
    min-width: 0;
  }
  .history-title {
    color: var(--nova-text);
    font-size: 18px;
    font-weight: 600;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .history-sub {
    color: var(--nova-muted);
    font-size: 16px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .history-play {
    width: 38px;
    height: 38px;
    border-radius: 50%;
    background: rgba(255,255,255,0.1);
    color: var(--nova-text);
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 1rem;
    transition: all 0.2s ease;
  }
  .history-row:hover .history-play {
    background: var(--nova-accent);
    color: #fff;
    transform: scale(1.08);
  }
  .playlists-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
    gap: 24px;
  }
  .lib-card {
    display: flex;
    flex-direction: column;
    gap: 12px;
    padding: 16px;
    border-radius: 16px;
    background: rgba(255,255,255,0.04);
    border: 1px solid var(--nova-border);
    cursor: pointer;
    transition: all 0.3s ease;
    backdrop-filter: blur(20px);
    -webkit-backdrop-filter: blur(20px);
  }
  .lib-card:hover {
    background: rgba(255,255,255,0.08);
    border-color: var(--nova-border-strong);
    transform: translateY(-4px);
    box-shadow: 0 8px 24px rgba(0,0,0,0.2);
  }
  .card-art {
    width: 100%;
    aspect-ratio: 1;
    border-radius: 12px;
    object-fit: cover;
    background: #00000015;
    transition: transform 0.3s ease;
  }
  .lib-card:hover .card-art {
    transform: scale(1.04);
  }
  .card-title {
    color: var(--nova-text);
    font-size: 18px;
    font-weight: 600;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .card-subtitle {
    color: var(--nova-muted);
    font-size: 16px;
  }
  .empty-notice {
    text-align: center;
    padding: 64px 24px;
    background: rgba(24,24,34,0.4);
    border: 1px dashed rgba(139,92,246,0.25);
    border-radius: 22px;
  }
  .notice-icon {
    font-size: 3rem;
    margin-bottom: 16px;
  }
  .empty-notice h3 {
    margin: 0 0 8px 0;
    color: var(--nova-text);
    font-size: 20px;
  }
  .empty-notice p {
    margin: 0;
    color: var(--nova-muted);
    font-size: 18px;
    max-width: 460px;
    line-height: 1.5;
  }
  .loading-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: 96px 0;
  }
  .cosmic-spinner {
    width: 64px;
    height: 64px;
    border-radius: 50%;
    border: 4px solid rgba(255,255,255,0.1);
    border-top-color: var(--nova-accent);
    animation: spin 1s linear infinite;
    margin-bottom: 16px;
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