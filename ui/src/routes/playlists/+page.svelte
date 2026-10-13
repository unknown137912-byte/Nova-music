<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { Clock3, ListMusic, Music2, Play, Search, Sparkles, Plus, Trash2, Shuffle } from 'lucide-svelte';
  import TrackActions from '../../lib/TrackActions.svelte';
  import { artworkUrl, recoverArtwork } from '../../lib/artwork';

  export let activePlaylistId: string | null = null;
  export let onPlayTrack: (track: any) => void = () => {};
  export let onOpenArtist: (id: string) => void = () => {};
  export let onTrackAction: (track: any, action: string) => void = () => {};
  export let playlistOptions: any[] = [];

  let loading = false;
  let error = '';
  let currentCollection: any = null;
  let libraryPlaylists: any[] = [];
  let libraryAlbums: any[] = [];
  let libraryArtists: any[] = [];
  export let category = 'Playlists';
  let filterText = '';
  let requestedId: string | null | undefined = undefined;
  let requestVersion = 0;
  let newPlaylistTitle = '';
  let savingPlaylist = false;
  let confirmingDelete = false;
  let playlistMessage = '';

  function playlistTint(source: string): string {
    let hash = 2166136261;
    for (let index = 0; index < source.length; index += 1) hash = Math.imul(hash ^ source.charCodeAt(index), 16777619);
    return `hsl(${(hash >>> 0) % 360} 62% 32%)`;
  }

  onMount(() => {
    const refresh = (event: Event) => {
      if (activePlaylistId && (event as CustomEvent<string>).detail === activePlaylistId) void loadCollection(activePlaylistId);
    };
    window.addEventListener('nova-playlist-updated', refresh);
    const refreshLiked = () => {
      if (activePlaylistId === 'NOVA_LIKED') void loadCollection('NOVA_LIKED');
    };
    window.addEventListener('nova-liked-changed', refreshLiked);
    const refreshLibrary = () => {
      if (!activePlaylistId) void loadOverview();
    };
    window.addEventListener('nova-local-playlists-changed', refreshLibrary);
    return () => {
      window.removeEventListener('nova-playlist-updated', refresh);
      window.removeEventListener('nova-liked-changed', refreshLiked);
      window.removeEventListener('nova-local-playlists-changed', refreshLibrary);
    };
  });

  function displayList(value: any): any[] {
    if (Array.isArray(value)) return value;
    if (Array.isArray(value?.items)) return value.items;
    return [];
  }

  async function loadOverview() {
    const request = ++requestVersion;
    loading = true;
    error = '';
    currentCollection = null;
    try {
      const result: any = await invoke('get_library');
      if (request !== requestVersion) return;
      libraryPlaylists = (Array.isArray(result) ? result : (result?.playlists || []))
        .filter((item: any) => !['LM', 'VLLM'].includes(String(item?.id || item?.browseId || item?.browse_id || ''))
          && !/^(liked music|liked songs)$/i.test(String(item?.title || item?.name || '')));
      libraryAlbums = result?.albums || [];
      libraryArtists = result?.artists || [];
    } catch (reason: any) {
      if (request !== requestVersion) return;
      error = typeof reason === 'string' ? reason : (reason?.message || 'Could not load your library.');
    } finally {
      if (request === requestVersion) loading = false;
    }
  }

  async function loadCollection(id: string) {
    const request = ++requestVersion;
    loading = true;
    error = '';
    playlistMessage = '';
    confirmingDelete = false;
    currentCollection = null;
    if (id === 'NOVA_LIKED') {
      try {
        const tracks: any[] = await invoke('get_liked_tracks');
        if (request !== requestVersion) return;
        const likedTracks = Array.isArray(tracks) ? tracks : [];
        currentCollection = {
          id,
          title: 'Liked Music',
          subtitle: 'Saved on this device',
          thumbnail: likedTracks[0]?.thumbnail || '',
          items: likedTracks,
        };
      } catch (reason: any) {
        if (request === requestVersion) error = typeof reason === 'string' ? reason : (reason?.message || 'Could not load your liked music.');
      } finally {
        if (request === requestVersion) loading = false;
      }
      return;
    }
    const isAlbum = /^(MPRE|OLAK)/i.test(id);
    const firstCommand = isAlbum ? 'get_album' : 'get_playlist';
    const firstKey = isAlbum ? { albumId: id } : { playlistId: id };
    const fallbackCommand = isAlbum ? 'get_playlist' : 'get_album';
    const fallbackKey = isAlbum ? { playlistId: id } : { albumId: id };
    try {
      let result: any;
      try {
        result = await invoke(firstCommand, firstKey);
      } catch (_) {
        result = await invoke(fallbackCommand, fallbackKey);
      }
      if (request !== requestVersion) return;
      currentCollection = result;
    } catch (reason: any) {
      if (request !== requestVersion) return;
      error = typeof reason === 'string' ? reason : (reason?.message || 'Could not open this collection.');
    } finally {
      if (request === requestVersion) loading = false;
    }
  }

  $: if (activePlaylistId !== requestedId) {
    requestedId = activePlaylistId;
    if (activePlaylistId) void loadCollection(activePlaylistId);
    else void loadOverview();
  }

  $: collectionItems = displayList(currentCollection?.items);
  $: isAlbum = Boolean(currentCollection && (currentCollection.artist || String(currentCollection.subtitle || '').toLowerCase().includes('album')));
  $: isLocalPlaylist = Boolean(activePlaylistId?.startsWith('NOVA_LOCAL_'));
  $: isLikedCollection = activePlaylistId === 'NOVA_LIKED';
  $: currentCover = artworkUrl(String(currentCollection?.thumbnail || currentCollection?.cover || ''));
  $: collectionTint = playlistTint(currentCover || String(currentCollection?.id || currentCollection?.title || 'nova'));
  $: visibleItems = (category === 'Albums' ? libraryAlbums : category === 'Artists' ? libraryArtists : libraryPlaylists)
    .filter((item) => !filterText || String(item.title || item.name || '').toLowerCase().includes(filterText.toLowerCase()));
  $: totalSeconds = collectionItems.reduce((total: number, item: any) => total + durationInSeconds(item.duration), 0);
  let publishedContextKey = '';
  $: {
    if (currentCollection && activePlaylistId) {
      const artwork = currentCollection.thumbnail || currentCollection.cover || '';
      const key = `${activePlaylistId}:${artwork}:${collectionItems.length}`;
      if (key !== publishedContextKey) {
        publishedContextKey = key;
        window.dispatchEvent(new CustomEvent('nova-playlist-context', {
          detail: { title: currentCollection.title || '', thumbnail: artwork, items: collectionItems },
        }));
      }
    } else if (!activePlaylistId && publishedContextKey) {
      publishedContextKey = '';
      window.dispatchEvent(new CustomEvent('nova-playlist-context', { detail: { title: '', thumbnail: '', items: [] } }));
    }
  }

  function durationInSeconds(value: any): number {
    if (typeof value === 'number' && Number.isFinite(value)) return value;
    if (typeof value !== 'string' || !value.trim()) return 0;
    const parts = value.split(':').map((part) => Number(part));
    if (parts.some((part) => !Number.isFinite(part))) return 0;
    if (parts.length === 2) return parts[0] * 60 + parts[1];
    if (parts.length === 3) return parts[0] * 3600 + parts[1] * 60 + parts[2];
    return 0;
  }

  function formatDuration(value: any): string {
    const seconds = durationInSeconds(value);
    return `${Math.floor(seconds / 60)}:${String(Math.floor(seconds % 60)).padStart(2, '0')}`;
  }

  function formatTotal(seconds: number): string {
    const hours = Math.floor(seconds / 3600);
    const minutes = Math.floor((seconds % 3600) / 60);
    return hours ? `${hours} hr ${minutes} min` : `${minutes} min`;
  }

  function playSong(track: any) {
    const id = track?.videoId || track?.video_id || track?.trackId || track?.track_id || track?.id;
    if (!id) return;
    onPlayTrack({
      ...track,
      id,
      videoId: id,
      title: track.title || track.name || 'Unknown track',
      artist: track.artists || track.artist || track.subtitle || currentCollection?.artist || 'YouTube Music',
      artistId: track.artistId || track.artist_id || currentCollection?.artistId,
      album: track.album || currentCollection?.title,
      albumId: track.albumId || track.album_id || (isAlbum ? activePlaylistId : undefined),
      thumbnail: track.thumbnail || currentCollection?.thumbnail || currentCollection?.cover || '',
      duration: durationInSeconds(track.duration),
    });
  }

  function openItem(item: any) {
    const id = item?.browseId || item?.browse_id || item?.playlistId || item?.playlist_id || item?.id;
    if (!id) return;
    if (category === 'Artists' || String(item?.kind || item?.type || '').toLowerCase().includes('artist')) {
      onOpenArtist(id);
      return;
    }
    activePlaylistId = id;
  }

  function playFirst() {
    if (collectionItems.length) playSong(collectionItems[0]);
  }

  function shuffleCollection() {
    if (!collectionItems.length) return;
    window.dispatchEvent(new CustomEvent('nova-shuffle-request'));
    playSong(collectionItems[Math.floor(Math.random() * collectionItems.length)]);
  }

  async function createPlaylist(event: SubmitEvent) {
    event.preventDefault();
    const title = newPlaylistTitle.trim();
    if (!title || savingPlaylist) return;
    savingPlaylist = true;
    playlistMessage = '';
    try {
      const playlist: any = await invoke('create_local_playlist', { title });
      newPlaylistTitle = '';
      window.dispatchEvent(new CustomEvent('nova-local-playlists-changed'));
      activePlaylistId = playlist.id || playlist.playlistId;
    } catch (reason: any) {
      playlistMessage = typeof reason === 'string' ? reason : (reason?.message || 'Could not create playlist.');
    } finally {
      savingPlaylist = false;
    }
  }

  async function deletePlaylist() {
    if (!activePlaylistId || !isLocalPlaylist) return;
    if (!confirmingDelete) {
      confirmingDelete = true;
      return;
    }
    try {
      await invoke('delete_local_playlist', { playlistId: activePlaylistId });
      confirmingDelete = false;
      activePlaylistId = null;
      window.dispatchEvent(new CustomEvent('nova-local-playlists-changed'));
    } catch (reason: any) {
      playlistMessage = typeof reason === 'string' ? reason : (reason?.message || 'Could not delete playlist.');
    }
  }
</script>

<div class="collections-page" class:has-collection={Boolean(currentCollection)} style={`--playlist-tint:${collectionTint};--collection-artwork:url(\"${currentCover}\")`}>
  {#if loading}
    <div class="page-state"><span class="spinner"></span><p>{activePlaylistId ? 'Opening your collection…' : 'Loading your library…'}</p></div>
  {:else if error}
    <div class="page-state error-state"><Sparkles size={25} /><h2>Could not load this view</h2><p>{error}</p><button on:click={() => activePlaylistId ? loadCollection(activePlaylistId) : loadOverview()}>Try again</button></div>
  {:else if currentCollection}
    <section class="collection-hero" style={`--cover:url("${currentCollection.thumbnail || currentCollection.cover || ''}")`}>
      <div class="collection-hero-content">
        <div class="collection-cover"><img src={currentCover || '/placeholder.svg'} alt="" on:error={(event) => recoverArtwork(event, currentCollection)} /></div>
        <div class="collection-info">
          <span class="collection-kind">{isLikedCollection ? 'Your music' : isAlbum ? 'Album' : 'Playlist'}</span>
          <h1>{currentCollection.title || 'Untitled collection'}</h1>
          {#if currentCollection.description}<p class="collection-description">{currentCollection.description}</p>{/if}
          <div class="collection-meta">
            {#if currentCollection.artist}<strong>{currentCollection.artist}</strong>{:else if currentCollection.subtitle}<strong>{currentCollection.subtitle}</strong>{/if}
            {#if collectionItems.length}<span>{collectionItems.length} songs</span>{/if}
            {#if totalSeconds}<span>{formatTotal(totalSeconds)}</span>{/if}
          </div>
          <div class="collection-actions">
            <button class="play-collection" on:click={playFirst} disabled={!collectionItems.length}><Play size={18} fill="currentColor" /> Play</button>
            <button class="shuffle-collection" on:click={shuffleCollection} disabled={!collectionItems.length}><Shuffle size={15} /> Shuffle</button>
            {#if isLocalPlaylist}
              <button class="delete-playlist" on:click={deletePlaylist}><Trash2 size={15} /> {confirmingDelete ? 'Confirm delete' : 'Delete playlist'}</button>
              {#if confirmingDelete}<button class="cancel-delete" on:click={() => confirmingDelete = false}>Cancel</button>{/if}
            {/if}
          </div>
          {#if playlistMessage}<p class="playlist-message" role="alert">{playlistMessage}</p>{/if}
        </div>
      </div>
    </section>

    <section class="track-table" aria-label="Collection tracks">
      <div class="track-table-head">
        <span class="index-col">#</span><span class="title-col">Title</span><span class="plays-col">Plays</span><span class="duration-col"><Clock3 size={16} /></span>
      </div>
      {#each collectionItems as track, index (track.videoId || track.video_id || track.id || index)}
        <div class="track-row">
          <button class="track-row-main" on:click={() => playSong(track)} aria-label={`Play ${track.title || 'track'}`}>
            <span class="index-col track-index"><span class="row-number">{index + 1}</span><span class="row-play"><Play size={15} fill="currentColor" /></span></span>
            <span class="title-col track-title-cell">
              <img src={artworkUrl(track.thumbnail || currentCollection.thumbnail || currentCollection.cover || '') || '/placeholder.svg'} alt="" on:error={(event) => recoverArtwork(event, track)} />
              <span class="track-copy"><strong>{track.title || 'Unknown track'}</strong><small>{track.artists || track.artist || track.subtitle || currentCollection.artist || 'YouTube Music'}</small></span>
            </span>
            <span class="plays-col track-plays">{track.playCount || track.play_count || '—'}</span>
            <span class="duration-col track-duration">{formatDuration(track.duration)}</span>
          </button>
          <TrackActions track={track} playlists={playlistOptions} onAction={onTrackAction} />
        </div>
      {/each}
      {#if !collectionItems.length}<div class="empty-tracks"><Music2 size={22} /><span>This collection doesn’t have any tracks to show.</span></div>{/if}
    </section>
  {:else}
    <div class="library-overview">
      <header class="library-overview-header">
        <div><span class="eyebrow">NOVA MUSIC</span><h1>Your Library</h1></div>
        <div class="library-tools">
          <label class="library-search"><Search size={16} /><input bind:value={filterText} placeholder="Filter your library" aria-label="Filter your library" /></label>
          {#if category === 'Playlists'}
            <form class="playlist-create" on:submit={createPlaylist}>
              <input bind:value={newPlaylistTitle} maxlength="80" placeholder="New playlist name" aria-label="New playlist name" />
              <button type="submit" disabled={savingPlaylist || !newPlaylistTitle.trim()}><Plus size={15} /> Create</button>
            </form>
          {/if}
        </div>
      </header>
      {#if playlistMessage}<p class="playlist-message" role="alert">{playlistMessage}</p>{/if}
      <nav class="library-categories" aria-label="Library categories">
        {#each ['Playlists', 'Albums', 'Artists'] as item}
          <button class:active={category === item} on:click={() => category = item}>{item}</button>
        {/each}
      </nav>
      {#if visibleItems.length}
        <div class="collection-grid">
          {#each visibleItems as item (item.id || item.playlistId || item.browseId || item.title)}
            <button class="collection-card" on:click={() => openItem(item)}>
              <span class="collection-card-art"><img src={artworkUrl(item.thumbnail || item.cover || item.thumbnail_url || '') || '/placeholder.svg'} alt="" on:error={(event) => recoverArtwork(event, item)} /><span class="card-play"><Play size={17} fill="currentColor" /></span></span>
              <strong>{item.title || item.name || 'Untitled'}</strong>
              <small>{item.subtitle || item.artist || (category === 'Artists' ? 'Artist' : category.slice(0, -1))}</small>
            </button>
          {/each}
        </div>
      {:else}
        <div class="library-empty"><span class="empty-art"><ListMusic size={24} /></span><h2>{filterText ? 'No matches' : `No saved ${category.toLowerCase()} yet`}</h2><p>{filterText ? 'Try another name.' : 'Save music from YouTube Music and it will appear in your library.'}</p></div>
      {/if}
    </div>
  {/if}
</div>

<style>
  .collections-page { position: relative; min-height: 100%; padding-bottom: 88px; color: #f5f7f5; background: radial-gradient(ellipse at 48% 0%, color-mix(in srgb, var(--playlist-tint) 37%, transparent), transparent 52%), #080909; transition: background .35s ease; }
  .collections-page.has-collection::before { position: fixed; z-index: 0; inset: 56px 0 86px; background-image: linear-gradient(180deg, rgba(8,9,9,.48), rgba(8,9,9,.73)), var(--collection-artwork); background-size: cover; background-position: center 36%; filter: blur(48px) saturate(1.38); opacity: .36; transform: scale(1.08); pointer-events: none; content: ''; transition: opacity .4s ease, filter .5s ease; }
  .collections-page.has-collection > :not(.collection-backdrop) { position: relative; z-index: 1; }
  .collection-hero { position: relative; isolation: isolate; min-height: 312px; display: flex; align-items: end; overflow: hidden; padding: 28px clamp(24px,4vw,58px) 26px; background: linear-gradient(180deg, rgba(8,9,9,.1), rgba(8,9,9,.5) 45%, #080909 100%); }
  .collection-hero::before { position: absolute; z-index: -2; inset: -30px; background-image: radial-gradient(ellipse at center, color-mix(in srgb, var(--playlist-tint) 68%, transparent), transparent 72%), var(--cover); background-color: var(--playlist-tint); background-position: center, center 38%; background-size: cover; filter: blur(42px) saturate(1.42); opacity: .82; transform: scale(1.15); content: ''; transition: background-color .35s ease; }
  .collection-hero::after { position: absolute; z-index: -1; inset: 0; background: linear-gradient(90deg, rgba(7,8,8,.55), rgba(7,8,8,.14) 65%, rgba(7,8,8,.35)), linear-gradient(0deg, #080909 0%, rgba(8,9,9,.2) 80%); content: ''; }
  .collection-hero-content { width: 100%; display: flex; align-items: end; gap: clamp(20px,3vw,32px); }
  .collection-cover { width: clamp(150px,19vw,228px); flex: 0 0 clamp(150px,19vw,228px); aspect-ratio: 1; overflow: hidden; border: 1px solid rgba(255,255,255,.16); border-radius: 7px; box-shadow: 0 20px 42px rgba(0,0,0,.4), inset 0 1px 0 rgba(255,255,255,.2); }
  .collection-cover img { width: 100%; height: 100%; display: block; object-fit: cover; }
  .collection-info { min-width: 0; padding-bottom: 4px; }
  .collection-kind, .eyebrow { color: #e6ebe7; font-size: 10px; font-weight: 750; letter-spacing: .12em; text-transform: uppercase; }
  .collection-info h1 { overflow: hidden; margin: 8px 0 10px; color: white; font-size: clamp(32px,5vw,64px); font-weight: 800; letter-spacing: -.055em; line-height: 1; text-overflow: ellipsis; }
  .collection-description { max-width: 700px; margin: 0 0 10px; color: #d0d6d2; font-size: 12px; }
  .collection-meta { display: flex; flex-wrap: wrap; align-items: center; gap: 7px; color: #d6dcd8; font-size: 11px; }
  .collection-meta > * + *::before { margin-right: 7px; color: #9da49f; content: '·'; }
  .collection-meta strong { color: #fff; font-weight: 700; }
  .collection-actions { display: flex; align-items: center; gap: 18px; margin-top: 20px; }
  .play-collection { min-width: 100px; height: 44px; display: inline-flex; align-items: center; justify-content: center; gap: 9px; padding: 0 18px; color: #fff; background: var(--accent); border: 0; border-radius: 25px; font-size: 12px; font-weight: 800; cursor: pointer; box-shadow: 0 7px 20px rgba(0,0,0,.2), inset 0 1px 0 rgba(255,255,255,.24); transition: transform .16s ease, filter .16s ease; }
  .play-collection:hover:not(:disabled) { filter: brightness(1.08); transform: scale(1.03); }
  .play-collection:disabled { opacity: .5; cursor: default; }
  .track-table { padding: 7px clamp(18px,3.5vw,52px) 22px; }
  .track-table-head { display: grid; grid-template-columns: 40px minmax(0,1fr) minmax(70px,.34fr) 56px 38px; align-items: center; column-gap: 12px; }
  .track-row { display: grid; grid-template-columns: minmax(0,1fr) 38px; align-items: center; column-gap: 7px; border-radius: 6px; }
  .track-row-main { width: 100%; min-width: 0; min-height: 58px; display: grid; grid-template-columns: 40px minmax(0,1fr) minmax(70px,.34fr) 56px; align-items: center; column-gap: 12px; padding: 5px 12px; color: #e9ecea; text-align: left; background: transparent; border: 0; border-radius: 5px; cursor: pointer; }
  .track-table-head { height: 38px; padding: 0 12px; color: #a2aaa5; border-bottom: 1px solid rgba(255,255,255,.11); font-size: 10px; }
  .index-col { text-align: center; }
  .title-col { min-width: 0; }
  .duration-col { display: flex; justify-content: flex-end; padding-right: 8px; }
  .plays-col { text-align: right; }
  .track-row:hover, .track-row:focus-within { background: rgba(255,255,255,.08); }
  .track-row-main:hover { background: transparent; }
  .track-row :global(.track-actions) { justify-self: center; }
  .track-index { position: relative; color: #9ca39e; font-size: 11px; }
  .row-play { position: absolute; inset: 0; display: none; place-items: center; color: #fff; }
  .track-row:hover .row-number { visibility: hidden; }
  .track-row:hover .row-play { display: grid; }
  .track-title-cell { display: flex; align-items: center; gap: 11px; }
  .track-title-cell > img { width: 40px; height: 40px; flex: 0 0 40px; object-fit: cover; border-radius: 4px; box-shadow: 0 3px 10px rgba(0,0,0,.2); }
  .track-copy { min-width: 0; display: flex; flex-direction: column; gap: 4px; }
  .track-copy strong, .track-copy small { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .track-copy strong { color: #f1f4f2; font-size: 12px; font-weight: 600; }
  .track-copy small, .track-plays, .track-duration { color: #a2aaa5; font-size: 10px; }
  .track-duration { font-variant-numeric: tabular-nums; }
  .empty-tracks { min-height: 160px; display: flex; align-items: center; justify-content: center; gap: 10px; color: #a2aaa5; font-size: 12px; }
  .library-overview { padding: 32px clamp(24px,4vw,58px) 30px; }
  .library-overview-header { display: flex; align-items: end; justify-content: space-between; gap: 24px; margin: 0 0 22px; }
  .library-overview-header h1 { margin: 5px 0 0; font-size: clamp(28px,3vw,38px); letter-spacing: -.045em; }
  .library-search { width: min(290px,42vw); height: 36px; display: flex; align-items: center; gap: 9px; padding: 0 12px; color: #bdc4bf; background: rgba(255,255,255,.08); border: 1px solid rgba(255,255,255,.09); border-radius: 20px; box-shadow: inset 0 1px 0 rgba(255,255,255,.08); backdrop-filter: blur(16px) saturate(1.25); }
  .library-search input { min-width: 0; flex: 1; color: #f4f6f4; background: none; border: 0; outline: none; font-size: 11px; }
  .library-search input::placeholder { color: #a2aaa5; }
  .library-categories { display: flex; gap: 8px; margin-bottom: 19px; }
  .library-categories button { min-height: 32px; padding: 0 14px; color: #e2e6e3; background: rgba(255,255,255,.08); border: 1px solid rgba(255,255,255,.04); border-radius: 20px; font-size: 11px; cursor: pointer; backdrop-filter: blur(12px); }
  .library-categories button.active { color: #111; background: #f3f5f3; }
  .collection-grid { display: grid; grid-template-columns: repeat(auto-fill,minmax(154px,1fr)); gap: 13px; }
  .collection-card { min-width: 0; display: flex; flex-direction: column; padding: 10px; color: #f1f4f2; background: rgba(255,255,255,.035); border: 1px solid rgba(255,255,255,.025); border-radius: 9px; text-align: left; cursor: pointer; transition: background .16s ease, transform .16s ease; }
  .collection-card:hover { background: rgba(255,255,255,.095); border-color: rgba(255,255,255,.08); transform: translateY(-2px); }
  .collection-card-art { position: relative; display: block; width: 100%; aspect-ratio: 1; margin-bottom: 11px; overflow: hidden; background: rgba(255,255,255,.08); border-radius: 6px; box-shadow: 0 8px 20px rgba(0,0,0,.2), inset 0 1px 0 rgba(255,255,255,.18); }
  .collection-card-art img { width: 100%; height: 100%; display: block; object-fit: cover; transition: transform .25s ease; }
  .collection-card:hover .collection-card-art img { transform: scale(1.035); }
  .card-play { position: absolute; right: 8px; bottom: 8px; width: 36px; height: 36px; display: grid; place-items: center; color: #fff; background: var(--accent); border-radius: 50%; opacity: 0; box-shadow: 0 8px 16px rgba(0,0,0,.4); transform: translateY(5px); transition: .16s ease; }
  .collection-card:hover .card-play { opacity: 1; transform: translateY(0); }
  .collection-card > strong { overflow: hidden; font-size: 11px; text-overflow: ellipsis; white-space: nowrap; }
  .collection-card > small { overflow: hidden; margin-top: 5px; color: #a4aca7; font-size: 10px; text-overflow: ellipsis; white-space: nowrap; }
  .library-empty, .page-state { min-height: 250px; display: flex; flex-direction: column; align-items: center; justify-content: center; color: #a4aca7; text-align: center; }
  .library-empty h2, .page-state h2 { margin: 13px 0 5px; color: #f3f5f3; font-size: 16px; }
  .library-empty p, .page-state p { max-width: 330px; margin: 0; font-size: 11px; line-height: 1.5; }
  .empty-art { width: 48px; height: 48px; display: grid; place-items: center; color: #c4cbc6; background: linear-gradient(145deg, rgba(75,117,94,.45), rgba(255,255,255,.06)); border: 1px solid rgba(255,255,255,.1); border-radius: 14px; box-shadow: inset 0 1px 0 rgba(255,255,255,.15); }
  .page-state { gap: 9px; padding: 40px 20px; }
  .page-state > p { margin: 0; }
  .page-state button { margin-top: 8px; padding: 8px 14px; color: #fff; background: var(--accent); border: 0; border-radius: 18px; font-size: 11px; font-weight: 750; cursor: pointer; }
  .library-tools { display: flex; align-items: center; justify-content: flex-end; flex-wrap: wrap; gap: 8px; }
  .playlist-create { display: flex; align-items: center; gap: 6px; padding: 3px; background: #151817; border: 1px solid rgba(255,255,255,.09); border-radius: 22px; }
  .playlist-create input { width: 142px; min-width: 0; padding: 0 9px; color: #f5f7f5; background: transparent; border: 0; outline: 0; font-size: 10px; }
  .playlist-create input::placeholder { color: #929a95; }
  .playlist-create button, .delete-playlist, .cancel-delete { min-height: 30px; display: inline-flex; align-items: center; justify-content: center; gap: 6px; padding: 0 11px; color: #101211; background: #f0f3f1; border: 0; border-radius: 17px; font-size: 10px; font-weight: 750; cursor: pointer; }
  .playlist-create button:disabled { opacity: .45; cursor: default; }
  .delete-playlist { color: #ffd8d8; background: rgba(212,73,73,.16); border: 1px solid rgba(255,100,100,.2); }
  .cancel-delete { color: #dce1de; background: rgba(255,255,255,.08); }
  .playlist-message { margin: 8px 0 0; color: #ffb6b6; font-size: 11px; }
  .spinner { width: 25px; height: 25px; border: 2px solid rgba(255,255,255,.18); border-top-color: var(--accent); border-radius: 50%; animation: spin .7s linear infinite; }
  @keyframes spin { to { transform: rotate(360deg); } }
  @media (max-width: 700px) { .collection-hero { min-height: 260px; padding-inline: 20px; } .collection-cover { width: 125px; flex-basis: 125px; } .collection-info h1 { font-size: 32px; } .track-table-head { grid-template-columns: 30px minmax(0,1fr) 48px 32px; gap: 8px; } .track-row { grid-template-columns: minmax(0,1fr) 32px; gap: 3px; } .track-row-main { grid-template-columns: 30px minmax(0,1fr) 48px; gap: 8px; } .plays-col { display: none; } .library-overview-header { align-items: stretch; flex-direction: column; } .library-tools { justify-content: stretch; } .library-search { width: 100%; } .playlist-create { flex: 1; } .playlist-create input { width: 100%; } }
</style>
