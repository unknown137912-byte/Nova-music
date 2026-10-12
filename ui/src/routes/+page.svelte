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
  let listeningMix: any = null;
  let activeChipParams: string | null = null;
  let homeContinuation = '';
  let loadingMore = false;
  let loadMoreError = '';
  let homeRequestSerial = 0;
  let loadedHomeKey: string | null = null;
  let refreshingHome = false;
  let refreshError = '';
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

  function homeFeedKey(params?: string | null): string {
    return `nova-home-feed-v2:${params || 'all'}`;
  }

  function readCachedHome(key: string): any | null {
    try {
      const cached = JSON.parse(localStorage.getItem(key) || 'null');
      return Array.isArray(cached?.sections) ? cached : null;
    } catch (_) {
      return null;
    }
  }

  function saveCachedHome(key: string, value: any) {
    try { localStorage.setItem(key, JSON.stringify(value)); }
    catch (_) { /* Feed cache is optional; keep the live feed usable if storage is full. */ }
  }

  async function loadHome(params?: string | null) {
    const requestSerial = ++homeRequestSerial;
    const feedKey = homeFeedKey(params);
    const cachedHome = readCachedHome(feedKey);
    const sameFeed = loadedHomeKey === feedKey && Boolean(homeData);
    const initialHome = cachedHome || (sameFeed ? homeData : null);
    homeData = initialHome;
    loadedHomeKey = initialHome ? feedKey : null;
    loading = !initialHome;
    refreshingHome = true;
    error = null;
    refreshError = '';
    loadingMore = false;
    loadMoreError = '';
    homeContinuation = String(initialHome?.continuation || '');
    activeChipParams = params || null;
    if (params) {
      recentTracks = [];
      listeningMix = null;
    }

    // Let the YouTube Music feed paint as soon as it arrives. Local taste and
    // history lookups must not hold the whole Home screen behind a spinner.
    const settingsTask = invoke('get_settings').catch(() => ({}));
    const recommendationsTask = params ? Promise.resolve(null) : invoke('get_listening_recommendations').catch(() => null);
    try {
      const freshHome: any = await invoke('get_home', { params: params || null });
      if (requestSerial !== homeRequestSerial) return;
      homeData = freshHome;
      loadedHomeKey = feedKey;
      homeContinuation = String(homeData?.continuation || '');
      saveCachedHome(feedKey, homeData);
      error = null;
      refreshError = '';
    } catch (reason: any) {
      if (requestSerial !== homeRequestSerial) return;
      const message = typeof reason === 'string' ? reason : (reason?.message || 'Could not load your YouTube Music home');
      if (initialHome) refreshError = message;
      else {
        error = message;
        homeData = null;
        loadedHomeKey = null;
      }
    } finally {
      if (requestSerial === homeRequestSerial) {
        loading = false;
        refreshingHome = false;
      }
    }

    if (requestSerial !== homeRequestSerial) return;
    void settingsTask.then((settings: any) => {
      if (requestSerial !== homeRequestSerial) return;
      if (params || settings?.history_enabled === 'false') {
        recentTracks = [];
        listeningMix = null;
        return;
      }
      void invoke('get_history').then((history: any) => {
        if (requestSerial === homeRequestSerial) recentTracks = normalizeHistory(history).slice(0, 8);
      }).catch(() => { if (requestSerial === homeRequestSerial) recentTracks = []; });
    });
    void recommendationsTask.then((recommendations: any) => {
      if (requestSerial === homeRequestSerial) listeningMix = recommendations;
    });
  }

  async function loadMoreHome() {
    if (!homeContinuation || loadingMore || loading || !homeData) return;
    const continuation = homeContinuation;
    const requestSerial = homeRequestSerial;
    loadingMore = true;
    loadMoreError = '';
    try {
      const page: any = await invoke('get_home_continuation', { continuation });
      if (requestSerial !== homeRequestSerial) return;
      const combined = [...(homeData?.sections || [])];
      for (const section of page?.sections || []) {
        const title = String(section.title || 'Recommended');
        const existing = combined.find((item) => String(item.title || 'Recommended') === title);
        if (!existing) combined.push(section);
        else {
          const seen = new Set((existing.items || []).map((item: any) => String(item.id || item.videoId || item.video_id || item.title)));
          existing.items = [...(existing.items || []), ...(section.items || []).filter((item: any) => {
            const key = String(item.id || item.videoId || item.video_id || item.title);
            if (seen.has(key)) return false;
            seen.add(key);
            return true;
          })];
        }
      }
      homeData = { ...homeData, sections: combined, continuation: page?.continuation || null };
      homeContinuation = String(page?.continuation || '');
    } catch (reason: any) {
      if (requestSerial === homeRequestSerial) {
        loadMoreError = typeof reason === 'string' ? reason : (reason?.message || 'Could not load more recommendations.');
      }
    } finally {
      if (requestSerial === homeRequestSerial) loadingMore = false;
    }
  }

  function observeHomeEnd(node: HTMLElement) {
    const observer = new IntersectionObserver((entries) => {
      if (entries.some((entry) => entry.isIntersecting)) void loadMoreHome();
    }, { root: node.closest('.main-content'), rootMargin: '700px 0px' });
    observer.observe(node);
    return { destroy: () => observer.disconnect() };
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
  $: showPersonalSections = activeChipParams === null;
  $: moodSections = (homeData?.sections || []).filter((section: any) => !activeChipParams
    || !/^(jump back in|recently played|quick picks|picked for your taste|made for you)$/i.test(String(section.title || '').trim()));
  $: sections = [
    ...moodSections,
    ...(!activeChipParams && listeningMix?.items?.length ? [{
      id: 'nova-listening-mix',
      title: 'Picked for your taste',
      items: listeningMix.items,
    }] : []),
  ];
</script>

<div class="home-container">
  <section class="home-intro anim-fade-up">
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

  {#if refreshingHome && homeData}
    <div class="home-refreshing" role="status"><span></span>Refreshing this mix</div>
  {/if}

  {#if loading && !homeData}
    <div class="home-skeleton" role="status" aria-label="Loading music recommendations">
      <div class="skeleton-chip-row">{#each [0,1,2,3,4,5,6,7] as chip}<span style={`--skeleton-width:${[46,54,82,92,76,88,70,82][chip]}px`}></span>{/each}</div>
      <section><div class="skeleton-heading"></div><div class="skeleton-recent-grid">{#each [0,1,2,3,4,5] as item}<div class="skeleton-recent"><i></i><span><b></b><b></b></span></div>{/each}</div></section>
      <section><div class="skeleton-heading short"></div><div class="skeleton-card-grid">{#each [0,1,2,3,4] as item}<div class="skeleton-card"><i></i><b></b><b></b></div>{/each}</div></section>
      <p>Loading your music…</p>
    </div>
  {:else if error}
    <div class="home-error">
      <div class="error-symbol"><Sparkles size={20} /></div>
      <div><h2>Your music is taking a moment</h2><p>{error}</p></div>
      <button on:click={() => loadHome(activeChipParams)}>Try again</button>
    </div>
  {:else}
    {#if refreshError}<div class="home-refresh-error" role="status">Showing your last loaded picks. <button on:click={() => loadHome(activeChipParams)}>Try again</button></div>{/if}
    {#if showPersonalSections && recentTracks.length}
      <section class="home-section jump-section anim-fade-up">
        <div class="home-section-heading">
          <div class="section-title-wrap"><Clock size={17} /><h2>Jump back in</h2></div>
          <span class="section-caption">Recently played</span>
        </div>
        <div class="recent-grid stagger">
          {#each recentTracks.slice(0, 6) as item (item.videoId || item.video_id || item.id || item.title)}
            <div class="recent-card-wrap">
              <button class="recent-card" on:click={() => playItem(item)}>
                <img src={item.thumbnail || item.thumbnail_url || '/placeholder.svg'} alt="" loading="lazy" class="recent-thumb" />
                <span class="recent-copy"><strong>{item.title || item.name || 'Unknown track'}</strong><small>{displaySubtitle(item)}</small></span>
                <span class="recent-play"><Play size={15} fill="currentColor" /></span>
              </button>
              <TrackActions track={item} playlists={playlistOptions} onAction={onTrackAction} />
            </div>
          {/each}
        </div>
      </section>
    {:else if showPersonalSections}
      <section class="home-section shortcut-section anim-fade-up">
        <div class="home-section-heading">
          <div class="section-title-wrap"><LayoutGrid size={17} /><h2>Shortcuts</h2></div>
        </div>
        <button class="shortcut-card" on:click={() => onOpenPlaylist('NOVA_LIKED')}>
          <span class="shortcut-art"><Heart size={21} fill="currentColor" /></span>
          <span><strong>Liked Music</strong><small>Your personal collection</small></span>
          <span class="recent-play"><Play size={15} fill="currentColor" /></span>
        </button>
      </section>
    {/if}

    {#each sections as section (section.id || section.title)}
      {#if section.items?.length}
        <section class="home-section anim-fade-up">
          <div class="home-section-heading">
            <div class="section-title-wrap">
              {#if String(section.title || '').toLowerCase().includes('artist')}<UserRound size={17} />{:else if String(section.title || '').toLowerCase().includes('album')}<Disc size={17} />{:else}<Sparkles size={17} />{/if}
              <h2>{section.title}</h2>
            </div>
            {#if section.moreBrowseId}
              <button class="see-all" on:click={() => onOpenPlaylist(section.moreBrowseId)}>Show all <span>›</span></button>
            {/if}
          </div>
          <div class="shelf-scroller stagger">
            {#each section.items as item (item.id || item.videoId || item.title)}
              <div class="shelf-card-wrap">
                <button class="shelf-card" on:click={() => openItem(item)}>
                  <span class="shelf-art-wrap">
                    <img src={item.thumbnail || item.thumbnail_url || item.cover || '/placeholder.svg'} alt="" loading="lazy" class="shelf-thumb" />
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

    {#if homeContinuation}
      <div class="home-more-sentinel" use:observeHomeEnd aria-hidden="true"></div>
      {#if loadMoreError}<p class="home-more-error" role="status">{loadMoreError}</p>{/if}
      <button class="home-load-more" on:click={loadMoreHome} disabled={loadingMore}>{loadingMore ? 'Finding more music…' : 'Load more recommendations'}</button>
    {/if}

    {#if sections.length === 0 && recentTracks.length === 0}
      <div class="home-empty anim-fade-up">
        <div class="empty-icon"><Music2 size={26} /></div>
        <h2>{activeChipParams ? 'No matching picks yet' : 'Start with a search'}</h2>
        <p>{activeChipParams ? 'YouTube Music has no recommendations for this mood right now. Try another mood or refresh the page.' : 'Your YouTube Music home and recent plays will appear here when available.'}</p>
        {#if activeChipParams}<button class="refresh-mood" on:click={() => loadHome(activeChipParams)}>Refresh this mood</button>{/if}
      </div>
    {/if}
  {/if}
</div>

<style>
  .home-container { min-height: 100%; padding: 48px clamp(24px, 4vw, 64px) 80px; background: radial-gradient(ellipse at 42% 0%, rgba(var(--cover-rgb), 0.08), transparent 50%), linear-gradient(145deg, rgba(var(--cover-deep), 0.12), transparent 60%), var(--nova-background); }
  .home-intro { min-height: 80px; display: flex; align-items: center; justify-content: space-between; margin-bottom: 24px; }
  .greeting-block { display: flex; align-items: center; gap: 20px; }
  .greeting-avatar { width: 56px; height: 56px; display: grid; place-items: center; color: #fff; background: var(--nova-accent); border-radius: 50%; font-size: 22px; font-weight: 700; box-shadow: 0 4px 12px rgba(var(--cover-rgb), 0.24); }
  .greeting-eyebrow { margin-bottom: 4px; color: var(--nova-muted); font-size: 10px; font-weight: 600; letter-spacing: 0.5px; }
  .greeting-block h1 { margin: 0; color: var(--nova-text); font-size: clamp(26px, 3.5vw, 36px); font-weight: 700; letter-spacing: -0.5px; line-height: 1.2; }
  .history-button { width: 44px; height: 44px; display: grid; place-items: center; color: var(--nova-muted); background: rgba(255,255,255,0.08); border: 1px solid rgba(255,255,255,0.12); border-radius: 50%; cursor: pointer; transition: all 0.2s ease; }
  .history-button:hover { color: var(--nova-text); background: rgba(255,255,255,0.16); }
  .home-chips { display: flex; align-items: center; gap: 10px; overflow-x: auto; padding: 6px 0 20px; margin: 0 0 14px; scrollbar-width: none; }
  .home-chips::-webkit-scrollbar, .shelf-scroller::-webkit-scrollbar { display: none; }
  .mood-chip { min-height: 38px; flex: 0 0 auto; padding: 0 18px; color: var(--nova-muted); background: rgba(255,255,255,0.07); border: 1px solid rgba(255,255,255,0.06); border-radius: 22px; font-size: 12px; font-weight: 600; cursor: pointer; transition: all 0.2s ease; }
  .mood-chip:hover { background: rgba(255,255,255,0.12); }
  .mood-chip.active { color: #111; background: rgba(255,255,255,0.08); }
  .mood-chip.all-chip { width: 40px; padding: 0; display: grid; place-items: center; color: var(--nova-muted); background: transparent; border-color: rgba(255,255,255,0.2); }
  .mood-chip.all-chip.active { color: #121414; background: rgba(255,255,255,0.08); }
  .home-section { margin: 22px 0 32px; }
  .home-section-heading { min-height: 36px; display: flex; align-items: center; justify-content: space-between; gap: 20px; margin: 0 0 14px; }
  .section-title-wrap { min-width: 0; display: flex; align-items: center; gap: 12px; color: var(--nova-muted); }
  .section-title-wrap h2 { overflow: hidden; margin: 0; color: var(--nova-text); font-size: 20px; font-weight: 700; letter-spacing: -0.3px; text-overflow: ellipsis; white-space: nowrap; }
  .section-caption { color: var(--nova-muted); font-size: 12px; }
  .see-all { display: flex; align-items: center; gap: 8px; color: var(--nova-muted); background: transparent; border: 0; font-size: 11px; font-weight: 600; cursor: pointer; transition: color 0.2s ease; }
  .see-all:hover { color: var(--nova-text); }
  .see-all span { font-size: 20px; line-height: 12px; }
  .recent-grid { display: grid; grid-template-columns: repeat(3, minmax(0,1fr)); gap: 12px; }
  .recent-card-wrap { position: relative; min-width: 0; }
  .recent-card { width: 100%; min-width: 0; min-height: 76px; display: flex; align-items: center; gap: 14px; padding: 0 54px 0 0; overflow: hidden; color: var(--nova-text); background: linear-gradient(120deg, rgba(255,255,255,0.08), rgba(255,255,255,0.04)); border: 1px solid rgba(255,255,255,0.08); border-radius: 10px; box-shadow: inset 0 1px 0 rgba(255,255,255,0.1); backdrop-filter: blur(20px); -webkit-backdrop-filter: blur(20px); cursor: pointer; transition: all 0.25s ease; }
  .recent-card-wrap :global(.track-actions) { position: absolute; z-index: 2; top: 50%; right: 8px; transform: translateY(-50%); }
  .recent-card:hover { background: linear-gradient(120deg, rgba(255,255,255,0.12), rgba(255,255,255,0.08)); transform: translateY(-2px); }
  .recent-card > img { width: 76px; height: 76px; flex: 0 0 76px; object-fit: cover; transition: transform 0.3s ease; }
  .recent-card:hover > img { transform: scale(1.03); }
  .recent-copy { min-width: 0; flex: 1; display: flex; flex-direction: column; gap: 6px; }
  .recent-copy strong, .recent-copy small { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .recent-copy strong { color: var(--nova-text); font-size: 14px; font-weight: 600; }
  .recent-copy small { color: var(--nova-muted); font-size: 12px; }
  .recent-play { width: 40px; height: 40px; display: grid; place-items: center; color: #fff; background: var(--nova-accent); border-radius: 50%; opacity: 0; transform: translateY(6px); transition: opacity 0.18s ease, transform 0.18s ease; }
  .recent-card:hover .recent-play, .shortcut-card:hover .recent-play { opacity: 1; transform: translateY(0); }
  .shortcut-card { position: relative; width: min(360px,100%); min-height: 82px; display: flex; align-items: center; gap: 16px; padding: 12px 18px 12px 12px; color: var(--nova-text); background: linear-gradient(135deg, rgba(42,48,44,0.82), rgba(20,22,21,0.76)); border: 1px solid rgba(255,255,255,0.1); border-radius: 12px; box-shadow: inset 0 1px 0 rgba(255,255,255,0.12); backdrop-filter: blur(22px); -webkit-backdrop-filter: blur(22px); cursor: pointer; transition: all 0.25s ease; }
  .shortcut-card:hover { background: linear-gradient(135deg, rgba(54,62,57,0.88), rgba(26,29,27,0.82)); }
  .shortcut-art { width: 58px; height: 58px; flex: 0 0 58px; display: grid; place-items: center; color: white; background: linear-gradient(135deg, #5b7dff, #bb43cb); border-radius: 8px; }
  .shortcut-card > span:nth-child(2) { flex: 1; display: flex; flex-direction: column; gap: 6px; }
  .shortcut-card strong { font-size: 14px; }
  .shortcut-card small { color: var(--nova-muted); font-size: 12px; }
  .shelf-scroller { display: grid; grid-auto-columns: minmax(140px, 1fr); grid-auto-flow: column; gap: 18px; overflow-x: auto; padding: 4px 0 10px; scrollbar-width: none; }
  .shelf-card-wrap { position: relative; min-width: 0; max-width: 220px; }
  .shelf-card { min-width: 0; max-width: 220px; display: flex; flex-direction: column; padding: 14px; color: var(--nova-text); background: transparent; border: 1px solid transparent; border-radius: 12px; cursor: pointer; transition: all 0.25s ease; }
  .shelf-card-wrap .shelf-card { width: 100%; }
  .shelf-card-wrap :global(.track-actions) { position: absolute; z-index: 2; top: 20px; right: 16px; }
  .shelf-card:hover { background: linear-gradient(145deg, rgba(255,255,255,0.08), rgba(255,255,255,0.03)); border-color: rgba(255,255,255,0.1); box-shadow: inset 0 1px 0 rgba(255,255,255,0.08); backdrop-filter: blur(18px); -webkit-backdrop-filter: blur(18px); }
  .shelf-art-wrap { position: relative; display: block; width: 100%; aspect-ratio: 1; margin-bottom: 14px; overflow: hidden; background: #202323; border-radius: 8px; box-shadow: 0 5px 18px rgba(0,0,0,0.2); }
  .shelf-art-wrap > img { width: 100%; height: 100%; display: block; object-fit: cover; transition: transform 0.3s ease; }
  .shelf-card:hover .shelf-art-wrap > img { transform: scale(1.04); }
  .shelf-play { position: absolute; right: 14px; bottom: 14px; width: 46px; height: 46px; display: grid; place-items: center; padding-left: 4px; color: #fff; background: var(--nova-accent); border-radius: 50%; opacity: 0; box-shadow: 0 8px 16px rgba(0,0,0,0.35); transform: translateY(8px); transition: opacity 0.2s ease, transform 0.2s ease; }
  .shelf-card:hover .shelf-play { opacity: 1; transform: translateY(0); }
  .shelf-title { overflow: hidden; color: var(--nova-text); font-size: 14px; font-weight: 700; text-overflow: ellipsis; white-space: nowrap; }
  .shelf-subtitle { overflow: hidden; margin-top: 6px; color: var(--nova-muted); font-size: 12px; text-overflow: ellipsis; white-space: nowrap; }
  .home-skeleton { display: flex; flex-direction: column; gap: 28px; padding: 0 0 48px; }
  .home-skeleton > section { display: flex; flex-direction: column; gap: 16px; }
  .home-skeleton > p { margin: 0; color: var(--nova-muted); font-size: 13px; }
  .skeleton-chip-row { display: flex; gap: 10px; overflow: hidden; }
  .skeleton-chip-row span { width: var(--skeleton-width); height: 36px; flex: 0 0 var(--skeleton-width); background: rgba(255,255,255,0.08); border-radius: 22px; }
  .skeleton-heading { width: 190px; height: 22px; background: rgba(255,255,255,0.1); border-radius: 6px; }
  .skeleton-heading.short { width: 145px; }
  .skeleton-recent-grid { display: grid; grid-template-columns: repeat(3,minmax(0,1fr)); gap: 12px; }
  .skeleton-recent { min-height: 76px; display: flex; align-items: center; gap: 14px; overflow: hidden; background: rgba(255,255,255,0.05); border: 1px solid rgba(255,255,255,0.04); border-radius: 10px; }
  .skeleton-recent i { width: 76px; height: 76px; flex: 0 0 76px; background: rgba(255,255,255,0.07); }
  .skeleton-recent span { min-width: 0; flex: 1; display: flex; flex-direction: column; gap: 10px; padding-right: 16px; }
  .skeleton-recent b, .skeleton-card b { height: 10px; background: rgba(255,255,255,0.09); border-radius: 5px; }
  .skeleton-recent b:last-child { width: 62%; }
  .skeleton-card-grid { display: grid; grid-template-columns: repeat(5,minmax(0,1fr)); gap: 14px; }
  .skeleton-card { min-width: 0; display: flex; flex-direction: column; gap: 12px; }
  .skeleton-card i { width: 100%; aspect-ratio: 1; background: rgba(255,255,255,0.06); border-radius: 8px; }
  .skeleton-card b:last-child { width: 70%; }
  .skeleton-recent i, .skeleton-recent b, .skeleton-card i, .skeleton-card b, .skeleton-heading, .skeleton-chip-row span { animation: skeleton-pulse 1.8s ease-in-out infinite alternate; }
  .home-refreshing { display: flex; align-items: center; gap: 10px; height: 22px; margin: -4px 0 12px; color: var(--nova-muted); font-size: 12px; }
  .home-refreshing span { width: 8px; height: 8px; background: var(--nova-accent); border-radius: 50%; box-shadow: 0 0 10px rgba(var(--cover-rgb), 0.7); animation: refresh-pulse 1.2s ease-in-out infinite alternate; }
  .home-refresh-error { margin: 0 0 16px; color: var(--nova-muted); font-size: 12px; }
  .home-refresh-error button { margin-left: 8px; padding: 5px 10px; color: #fff; background: rgba(255,255,255,0.1); border: 1px solid rgba(255,255,255,0.12); border-radius: 16px; font-size: 11px; cursor: pointer; transition: all 0.2s ease; }
  @keyframes skeleton-pulse { to { opacity: 0.42; } }
  @keyframes refresh-pulse { to { opacity: 0.4; transform: scale(0.75); } }
  @keyframes spin { to { transform: rotate(360deg); } }
  .home-error { display: flex; align-items: center; gap: 16px; margin: 40px 0; padding: 20px; background: #1a1a1a; border: 1px solid rgba(255,255,255,0.1); border-radius: 12px; }
  .error-symbol, .empty-icon { display: grid; place-items: center; width: 48px; height: 48px; color: var(--nova-muted); background: #252828; border-radius: 50%; }
  .home-error > div:nth-child(2) { min-width: 0; flex: 1; }
  .home-error h2, .home-empty h2 { margin: 0 0 5px; color: var(--nova-text); font-size: 16px; }
  .home-error p, .home-empty p { margin: 0; color: var(--nova-muted); font-size: 13px; }
  .home-error button { padding: 9px 13px; color: #111; background: #f2f4f4; border: 0; border-radius: 18px; font-size: 11px; font-weight: 700; cursor: pointer; transition: all 0.2s ease; }
  .home-empty { min-height: 320px; display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 12px; text-align: center; }
  .home-empty h2 { margin-top: 5px; }
  .home-empty p { max-width: 300px; line-height: 1.5; }
  .home-more-sentinel { width: 100%; height: 2px; }
  .home-more-error { margin: 0 auto; color: #ffadad; text-align: center; font-size: 13px; }
  .home-load-more, .refresh-mood { min-height: 40px; display: block; margin: 14px auto 26px; padding: 0 18px; color: var(--nova-muted); background: rgba(255,255,255,0.08); border: 1px solid rgba(255,255,255,0.12); border-radius: 20px; font-size: 12px; cursor: pointer; transition: all 0.25s ease; }
  .home-load-more:hover:not(:disabled), .refresh-mood:hover { background: rgba(255,255,255,0.12); transform: translateY(-2px); }
  .home-load-more:disabled { opacity: 0.55; cursor: wait; }
  .refresh-mood { margin: 12px 0 0; }
  @media (max-width: 1200px) { .recent-grid { grid-template-columns: repeat(2,minmax(0,1fr)); } .shelf-scroller { grid-auto-columns: minmax(130px, 1fr); } }
  @media (max-width: 800px) { .home-container { padding: 30px 22px 65px; } .recent-grid { grid-template-columns: 1fr; } .greeting-block h1 { font-size: 24px; } }
</style>