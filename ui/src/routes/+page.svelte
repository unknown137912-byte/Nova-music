<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { Clock, Disc, Heart, LayoutGrid, Music2, Play, Sparkles, UserRound } from 'lucide-svelte';
  import TrackActions from '../lib/TrackActions.svelte';

  export let accountName = '';
  export let onPlayTrack: (track: any) => void = () => {};
  export let onOpenPlaylist: (id: string) => void = () => {};
  export let onOpenArtist: (id: string) => void = () => {};
  export let onTrackAction: (track: any, action: string) => void = () => {};
  export let playlistOptions: any[] = [];

  let loading = true;
  let error: string | null = null;
  let homeData: any = null;
  let recentTracks: any[] = [];
  let activeChipParams: string | null = null;
  let refreshListener: (() => void) | undefined;
  let currentGreeting = 'Good evening';

  function setGreeting() {
    const hour = new Date().getHours();
    currentGreeting = hour < 12 ? 'Good morning' : hour < 18 ? 'Good afternoon' : 'Good evening';
  }

  function normalizeList(value: any): any[] {
    if (Array.isArray(value)) return value;
    if (Array.isArray(value?.items)) return value.items;
    if (Array.isArray(value?.songs)) return value.songs;
    if (Array.isArray(value?.tracks)) return value.tracks;
    if (Array.isArray(value?.results)) return value.results;
    return [];
  }

  function normalizeHistory(value: any): any[] {
    if (Array.isArray(value)) {
      return value.flatMap((entry) => Array.isArray(entry?.items) ? entry.items : [entry]);
    }
    return normalizeList(value);
  }

  async function loadHome(params?: string | null) {
    loading = true;
    error = null;
    activeChipParams = params || null;
    const requests = await Promise.allSettled([
      invoke('get_home', { params: params || null }),
      invoke('get_settings'),
    ]);

    if (requests[0].status === 'fulfilled') {
      homeData = requests[0].value;
    } else {
      const err: any = requests[0].reason;
      error = typeof err === 'string' ? err : (err?.message || 'Could not load your YouTube Music home');
      homeData = null;
    }
    const settings: any = requests[1].status === 'fulfilled' ? requests[1].value : {};
    if (settings?.history_enabled === 'false') {
      recentTracks = [];
    } else {
      try { recentTracks = normalizeHistory(await invoke('get_history')).slice(0, 8); }
      catch (_) { recentTracks = []; }
    }
    loading = false;
  }

  onMount(() => {
    setGreeting();
    loadHome();
    const refresh = () => loadHome(activeChipParams);
    window.addEventListener('nova-refresh-current-view', refresh);
    refreshListener = () => window.removeEventListener('nova-refresh-current-view', refresh);
    return () => refreshListener?.();
  });

  function parseDuration(value: any): number | undefined {
    if (typeof value === 'number' && Number.isFinite(value)) return value;
    if (typeof value !== 'string' || !value.trim()) return undefined;
    const parts = value.split(':').map((part) => Number(part));
    if (parts.some((part) => !Number.isFinite(part))) return undefined;
    if (parts.length === 2) return parts[0] * 60 + parts[1];
    if (parts.length === 3) return parts[0] * 3600 + parts[1] * 60 + parts[2];
    return undefined;
  }

  function playItem(item: any) {
    const id = item?.videoId || item?.video_id || item?.trackId || item?.track_id || item?.id;
    if (!id) return;
    onPlayTrack({
      ...item,
      id,
      title: item.title || item.name || 'Unknown track',
      artist: item.artist || item.artists || item.subtitle || 'YouTube Music',
      thumbnail: item.thumbnail || item.thumbnail_url || item.cover || '',
      duration: parseDuration(item.duration_seconds ?? item.duration),
    });
  }

  function openItem(item: any) {
    if (!item) return;
    const id = item.browseId || item.browse_id || item.playlistId || item.id;
    const kind = String(item.kind || item.type || '').toLowerCase();
    if (kind.includes('artist') && id) {
      onOpenArtist(id);
      return;
    }
    const collection = ['playlist', 'album', 'artist', 'station', 'radio'].some((part) => kind.includes(part))
      || String(id || '').startsWith('VL')
      || String(id || '').startsWith('MPRE')
      || String(id || '').startsWith('OLAK')
      || String(id || '').startsWith('RD');
    if (collection && id) onOpenPlaylist(id);
    else playItem(item);
  }

  function displaySubtitle(item: any) {
    return item?.subtitle || item?.artist || item?.artists || item?.description || (item?.kind === 'song' ? 'Track' : 'YouTube Music');
  }

  function isTrackItem(item: any) {
    const kind = String(item?.kind || item?.type || '').toLowerCase();
    return Boolean(item?.videoId || item?.video_id || item?.trackId || item?.track_id || kind.includes('song') || kind.includes('track'));
  }

  $: greetingName = accountName ? accountName.split('@')[0] : 'there';
  $: sections = homeData?.sections || [];
</script>

<div class="home-container">
  <section class="home-intro">
    <div class="greeting-block">
      <div class="greeting-avatar">{accountName ? accountName.charAt(0).toUpperCase() : 'N'}</div>
      <div>
        <div class="greeting-eyebrow">YOUR MUSIC, YOUR MOMENT</div>
        <h1>{currentGreeting}, {greetingName}</h1>
      </div>
    </div>
    <button class="history-button" on:click={() => window.dispatchEvent(new CustomEvent('nova-open-history'))} title="Recently played">
      <Clock size={17} />
    </button>
  </section>

  {#if homeData?.chips?.length}
    <nav class="home-chips" aria-label="Music moods">
      <button class:active={activeChipParams === null} class="mood-chip all-chip" on:click={() => loadHome(null)} aria-label="All moods"><LayoutGrid size={16} /></button>
      <button class="mood-chip" class:active={activeChipParams === null} on:click={() => loadHome(null)}>All</button>
      {#each homeData.chips as chip (chip.params || chip.title)}
        <button class="mood-chip" class:active={activeChipParams === chip.params} on:click={() => loadHome(chip.params)}>{chip.title}</button>
      {/each}
    </nav>
  {/if}

  {#if loading}
    <div class="home-loading"><span class="loading-spinner"></span><p>Finding your next favorite…</p></div>
  {:else if error}
    <div class="home-error">
      <div class="error-symbol"><Sparkles size={20} /></div>
      <div><h2>Your music is taking a moment</h2><p>{error}</p></div>
      <button on:click={() => loadHome(activeChipParams)}>Try again</button>
    </div>
  {:else}
    {#if recentTracks.length}
      <section class="home-section jump-section">
        <div class="home-section-heading">
          <div class="section-title-wrap"><Clock size={17} /><h2>Jump back in</h2></div>
          <span class="section-caption">Recently played</span>
        </div>
        <div class="recent-grid">
          {#each recentTracks.slice(0, 6) as item (item.videoId || item.video_id || item.id || item.title)}
            <div class="recent-card-wrap">
              <button class="recent-card" on:click={() => playItem(item)}>
                <img src={item.thumbnail || item.thumbnail_url || '/placeholder.jpg'} alt="" loading="lazy" />
                <span class="recent-copy"><strong>{item.title || item.name || 'Unknown track'}</strong><small>{displaySubtitle(item)}</small></span>
                <span class="recent-play"><Play size={15} fill="currentColor" /></span>
              </button>
              <TrackActions track={item} playlists={playlistOptions} onAction={onTrackAction} />
            </div>
          {/each}
        </div>
      </section>
    {:else}
      <section class="home-section shortcut-section">
        <div class="home-section-heading">
          <div class="section-title-wrap"><LayoutGrid size={17} /><h2>Shortcuts</h2></div>
        </div>
        <button class="shortcut-card" on:click={() => onOpenPlaylist('LM')}>
          <span class="shortcut-art"><Heart size={21} fill="currentColor" /></span>
          <span><strong>Liked Music</strong><small>Your personal collection</small></span>
          <span class="recent-play"><Play size={15} fill="currentColor" /></span>
        </button>
      </section>
    {/if}

    {#each sections as section (section.id || section.title)}
      {#if section.items?.length}
        <section class="home-section">
          <div class="home-section-heading">
            <div class="section-title-wrap">
              {#if String(section.title || '').toLowerCase().includes('artist')}<UserRound size={17} />{:else if String(section.title || '').toLowerCase().includes('album')}<Disc size={17} />{:else}<Sparkles size={17} />{/if}
              <h2>{section.title}</h2>
            </div>
            {#if section.moreBrowseId}
              <button class="see-all" on:click={() => onOpenPlaylist(section.moreBrowseId)}>Show all <span>›</span></button>
            {/if}
          </div>
          <div class="shelf-scroller">
            {#each section.items as item (item.id || item.videoId || item.title)}
              <div class="shelf-card-wrap">
                <button class="shelf-card" on:click={() => openItem(item)}>
                  <span class="shelf-art-wrap">
                    <img src={item.thumbnail || item.thumbnail_url || item.cover || '/placeholder.jpg'} alt="" loading="lazy" />
                    <span class="shelf-play"><Play size={17} fill="currentColor" /></span>
                  </span>
                  <span class="shelf-title">{item.title || item.name || 'Untitled'}</span>
                  <span class="shelf-subtitle">{displaySubtitle(item)}</span>
                </button>
                {#if isTrackItem(item)}<TrackActions track={item} playlists={playlistOptions} onAction={onTrackAction} />{/if}
              </div>
            {/each}
          </div>
        </section>
      {/if}
    {/each}

    {#if sections.length === 0 && recentTracks.length === 0}
      <div class="home-empty">
        <div class="empty-icon"><Music2 size={26} /></div>
        <h2>Start with a search</h2>
        <p>Your YouTube Music home and recent plays will appear here when available.</p>
      </div>
    {/if}
  {/if}
</div>

<style>
  .home-container { min-height: 100%; padding: 36px clamp(24px, 4vw, 58px) 72px; background: radial-gradient(ellipse at 42% 0%, rgba(var(--cover-rgb, 244 107 130),.12), transparent 48%), linear-gradient(145deg, rgba(var(--cover-deep-rgb, 30 17 30),.23), transparent 55%), rgba(8,9,9,.76); }
  .home-intro { min-height: 74px; display: flex; align-items: center; justify-content: space-between; margin-bottom: 22px; }
  .greeting-block { display: flex; align-items: center; gap: 16px; }
  .greeting-avatar { width: 50px; height: 50px; display: grid; place-items: center; color: #fff; background: var(--accent); border-radius: 50%; font-size: 20px; font-weight: 750; box-shadow: 0 8px 22px rgba(var(--cover-rgb),.24); }
  .greeting-eyebrow { margin-bottom: 3px; color: #a8adad; font-size: 9px; font-weight: 750; letter-spacing: .14em; }
  .greeting-block h1 { margin: 0; color: #fff; font-size: clamp(24px, 3vw, 34px); font-weight: 750; letter-spacing: -.045em; line-height: 1.2; }
  .history-button { width: 40px; height: 40px; display: grid; place-items: center; color: #c5c9c9; background: rgba(255,255,255,.07); border: 1px solid rgba(255,255,255,.09); border-radius: 50%; cursor: pointer; }
  .history-button:hover { color: white; background: rgba(255,255,255,.12); }
  .home-chips { display: flex; align-items: center; gap: 8px; overflow-x: auto; padding: 5px 0 18px; margin: 0 0 12px; scrollbar-width: none; }
  .home-chips::-webkit-scrollbar, .shelf-scroller::-webkit-scrollbar { display: none; }
  .mood-chip { min-height: 34px; flex: 0 0 auto; padding: 0 16px; color: #e5e7e7; background: rgba(255,255,255,.075); border: 1px solid rgba(255,255,255,.035); border-radius: 20px; font-size: 11px; font-weight: 600; cursor: pointer; transition: background .15s ease, color .15s ease; }
  .mood-chip:hover { background: rgba(255,255,255,.13); }
  .mood-chip.active { color: #111; background: #f7f8f8; }
  .mood-chip.all-chip { width: 36px; padding: 0; display: grid; place-items: center; color: #d8dddd; background: transparent; border-color: rgba(255,255,255,.18); }
  .mood-chip.all-chip.active { color: #121414; background: #f7f8f8; }
  .home-section { margin: 18px 0 28px; }
  .home-section-heading { min-height: 31px; display: flex; align-items: center; justify-content: space-between; gap: 16px; margin: 0 0 12px; }
  .section-title-wrap { min-width: 0; display: flex; align-items: center; gap: 9px; color: #b4b9b9; }
  .section-title-wrap h2 { overflow: hidden; margin: 0; color: #f7f8f8; font-size: 17px; font-weight: 750; letter-spacing: -.03em; text-overflow: ellipsis; white-space: nowrap; }
  .section-caption { color: #858a8a; font-size: 10px; }
  .see-all { display: flex; align-items: center; gap: 6px; color: #b6baba; background: transparent; border: 0; font-size: 10px; font-weight: 650; cursor: pointer; }
  .see-all:hover { color: #fff; }
  .see-all span { font-size: 19px; line-height: 12px; }
  .recent-grid { display: grid; grid-template-columns: repeat(3, minmax(0,1fr)); gap: 8px; }
  .recent-card-wrap { position: relative; min-width: 0; }
  .recent-card { width: 100%; min-width: 0; min-height: 66px; display: flex; align-items: center; gap: 11px; padding: 0 47px 0 0; overflow: hidden; color: #eee; text-align: left; background: linear-gradient(120deg, rgba(255,255,255,.105), rgba(255,255,255,.045)); border: 1px solid rgba(255,255,255,.065); border-radius: 7px; box-shadow: inset 0 1px 0 rgba(255,255,255,.09); -webkit-backdrop-filter: blur(18px) saturate(1.25); backdrop-filter: blur(18px) saturate(1.25); cursor: pointer; transition: background .16s ease, transform .16s ease; }
  .recent-card-wrap :global(.track-actions) { position: absolute; z-index: 2; top: 50%; right: 5px; transform: translateY(-50%); }
  .recent-card:hover { background: linear-gradient(120deg, rgba(255,255,255,.16), rgba(255,255,255,.075)); transform: translateY(-1px); }
  .recent-card > img { width: 66px; height: 66px; flex: 0 0 66px; object-fit: cover; }
  .recent-copy { min-width: 0; flex: 1; display: flex; flex-direction: column; gap: 4px; }
  .recent-copy strong, .recent-copy small { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .recent-copy strong { color: #f7f8f8; font-size: 11px; font-weight: 700; }
  .recent-copy small { color: #a9adad; font-size: 10px; }
  .recent-play { width: 34px; height: 34px; flex: 0 0 34px; display: grid; place-items: center; color: #fff; background: var(--accent); border-radius: 50%; opacity: 0; transform: translateY(4px); transition: opacity .15s ease, transform .15s ease; }
  .recent-card:hover .recent-play, .shortcut-card:hover .recent-play { opacity: 1; transform: translateY(0); }
  .shortcut-card { position: relative; width: min(340px,100%); min-height: 72px; display: flex; align-items: center; gap: 12px; padding: 8px 14px 8px 8px; color: #f5f6f6; text-align: left; background: linear-gradient(135deg, rgba(42,48,44,.82), rgba(20,22,21,.76)); border: 1px solid rgba(255,255,255,.09); border-radius: 8px; box-shadow: inset 0 1px 0 rgba(255,255,255,.11); -webkit-backdrop-filter: blur(20px) saturate(1.3); backdrop-filter: blur(20px) saturate(1.3); cursor: pointer; }
  .shortcut-card:hover { background: linear-gradient(135deg, rgba(54,62,57,.88), rgba(26,29,27,.82)); }
  .shortcut-art { width: 54px; height: 54px; flex: 0 0 54px; display: grid; place-items: center; color: white; background: linear-gradient(135deg, #5b7dff, #bb43cb); border-radius: 5px; }
  .shortcut-card > span:nth-child(2) { flex: 1; display: flex; flex-direction: column; gap: 4px; }
  .shortcut-card strong { font-size: 12px; }
  .shortcut-card small { color: #a0a5a5; font-size: 10px; }
  .shelf-scroller { display: grid; grid-auto-columns: minmax(130px, 1fr); grid-auto-flow: column; gap: 14px; overflow-x: auto; padding: 2px 0 8px; scrollbar-width: none; }
  .shelf-card-wrap { position: relative; min-width: 0; max-width: 205px; }
  .shelf-card { min-width: 0; max-width: 205px; display: flex; flex-direction: column; padding: 8px; color: #f3f4f4; text-align: left; background: transparent; border: 1px solid transparent; border-radius: 8px; cursor: pointer; transition: background .18s ease, border-color .18s ease; }
  .shelf-card-wrap .shelf-card { width: 100%; }
  .shelf-card-wrap :global(.track-actions) { position: absolute; z-index: 2; top: 14px; right: 12px; }
  .shelf-card:hover { background: linear-gradient(145deg, rgba(255,255,255,.09), rgba(255,255,255,.025)); border-color: rgba(255,255,255,.065); box-shadow: inset 0 1px 0 rgba(255,255,255,.06); -webkit-backdrop-filter: blur(18px) saturate(1.2); backdrop-filter: blur(18px) saturate(1.2); }
  .shelf-art-wrap { position: relative; display: block; width: 100%; aspect-ratio: 1; margin-bottom: 10px; overflow: hidden; background: #202323; border-radius: 6px; box-shadow: 0 5px 16px rgba(0,0,0,.17); }
  .shelf-art-wrap > img { width: 100%; height: 100%; display: block; object-fit: cover; transition: transform .3s ease; }
  .shelf-card:hover .shelf-art-wrap > img { transform: scale(1.04); }
  .shelf-play { position: absolute; right: 10px; bottom: 10px; width: 40px; height: 40px; display: grid; place-items: center; padding-left: 2px; color: #fff; background: var(--accent); border-radius: 50%; opacity: 0; box-shadow: 0 6px 14px rgba(0,0,0,.35); transform: translateY(6px); transition: opacity .16s ease, transform .16s ease; }
  .shelf-card:hover .shelf-play { opacity: 1; transform: translateY(0); }
  .shelf-title { overflow: hidden; color: #f0f2f2; font-size: 11px; font-weight: 700; text-overflow: ellipsis; white-space: nowrap; }
  .shelf-subtitle { overflow: hidden; margin-top: 4px; color: #969b9b; font-size: 10px; text-overflow: ellipsis; white-space: nowrap; }
  .home-loading { min-height: 290px; display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 13px; color: #969b9b; font-size: 12px; }
  .loading-spinner { width: 25px; height: 25px; border: 2px solid #444; border-top-color: #f6f7f7; border-radius: 50%; animation: spin .8s linear infinite; }
  @keyframes spin { to { transform: rotate(360deg); } }
  .home-error { display: flex; align-items: center; gap: 14px; margin: 36px 0; padding: 17px; background: #171919; border: 1px solid rgba(255,255,255,.07); border-radius: 9px; }
  .error-symbol, .empty-icon { display: grid; place-items: center; width: 44px; height: 44px; color: #e5e7e7; background: #252828; border-radius: 50%; }
  .home-error > div:nth-child(2) { min-width: 0; flex: 1; }
  .home-error h2, .home-empty h2 { margin: 0 0 4px; color: #f5f6f6; font-size: 14px; }
  .home-error p, .home-empty p { margin: 0; color: #9da2a2; font-size: 11px; }
  .home-error button { padding: 8px 12px; color: #111; background: #f2f4f4; border: 0; border-radius: 16px; font-size: 10px; font-weight: 700; cursor: pointer; }
  .home-empty { min-height: 300px; display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 9px; text-align: center; }
  .home-empty h2 { margin-top: 4px; }
  .home-empty p { max-width: 280px; line-height: 1.5; }
  @media (max-width: 1100px) { .recent-grid { grid-template-columns: repeat(2,minmax(0,1fr)); } .shelf-scroller { grid-auto-columns: minmax(125px, 1fr); } }
  @media (max-width: 760px) { .home-container { padding: 25px 18px 60px; } .recent-grid { grid-template-columns: 1fr; } .greeting-block h1 { font-size: 23px; } }
</style>
