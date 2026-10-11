<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { Play, Radio, Sparkles } from 'lucide-svelte';
  import TrackActions from '../../lib/TrackActions.svelte';

  export let artistId = '';
  export let onPlayTrack: (track: any) => void = () => {};
  export let onOpenPlaylist: (id: string) => void = () => {};
  export let onOpenArtist: (id: string) => void = () => {};
  export let onTrackAction: (track: any, action: string) => void = () => {};
  export let playlistOptions: any[] = [];

  let artist: any = null;
  let loading = true;
  let error = '';

  async function loadArtist(id: string) {
    if (!id) return;
    loading = true;
    error = '';
    try {
      artist = await invoke('get_artist', { artistId: id });
    } catch (reason: any) {
      artist = null;
      error = typeof reason === 'string' ? reason : (reason?.message || 'Could not load this artist.');
    } finally {
      loading = false;
    }
  }

  let requestedArtistId = '';
  $: if (artistId && artistId !== requestedArtistId) {
    requestedArtistId = artistId;
    loadArtist(artistId);
  }

  function playSong(song: any) {
    const id = song?.videoId || song?.video_id || song?.trackId || song?.track_id || song?.id;
    if (!id) return;
    onPlayTrack({
      ...song,
      id,
      title: song.title || 'Unknown track',
      artist: song.artists || song.artist || artist?.name || 'YouTube Music',
      thumbnail: song.thumbnail || artist?.thumbnail || '',
      duration: song.duration,
    });
  }
</script>

<div class="artist-page">
  {#if loading}
    <div class="artist-state"><span class="artist-spinner"></span><p>Loading artist…</p></div>
  {:else if error}
    <div class="artist-state"><Sparkles size={24} /><h2>Artist unavailable</h2><p>{error}</p><button on:click={() => loadArtist(artistId)}>Try again</button></div>
  {:else if artist}
    <header class="artist-hero">
      {#if artist.thumbnail}<img class="artist-banner" src={artist.thumbnail} alt="" />{/if}
      <div class="artist-hero-shade"></div>
      <div class="artist-hero-copy">
        <span>ARTIST</span>
        <h1>{artist.name || 'Unknown artist'}</h1>
        <p>{[artist.monthlyListeners, artist.subscribers].filter(Boolean).join(' · ')}</p>
        {#if artist.description}<small>{artist.description}</small>{/if}
        <div class="artist-actions">
          {#if artist.radioPlaylistId}<button class="artist-play-all" on:click={() => onOpenPlaylist(artist.radioPlaylistId)}><Radio size={16} /> Start radio</button>{/if}
          {#if artist.topSongsId}<button class="artist-secondary" on:click={() => onOpenPlaylist(artist.topSongsId)}>All top songs</button>{/if}
        </div>
      </div>
    </header>

    {#if artist.topSongs?.length}
      <section class="artist-section">
        <h2>Popular</h2>
        <div class="artist-song-list">
          {#each artist.topSongs as song, index (song.videoId || song.title)}
            <div class="artist-song-row">
              <button class="artist-song-main" on:click={() => playSong(song)} aria-label={`Play ${song.title || 'track'}`}>
                <span class="artist-song-index">{index + 1}</span>
                <img src={song.thumbnail || artist.thumbnail || '/placeholder.jpg'} alt="" />
                <span class="artist-song-copy"><strong>{song.title}</strong><small>{song.artists || artist.name}</small></span>
                <span class="artist-song-play"><Play size={15} fill="currentColor" /></span>
                <span class="artist-song-duration">{song.duration || ''}</span>
              </button>
              <TrackActions track={song} playlists={playlistOptions} onAction={onTrackAction} />
            </div>
          {/each}
        </div>
      </section>
    {/if}

    {#each artist.sections || [] as section (section.title)}
      {#if section.items?.length}
        <section class="artist-section">
          <div class="artist-section-heading"><h2>{section.title}</h2>{#if section.moreBrowseId}<button on:click={() => onOpenPlaylist(section.moreBrowseId)}>Show all</button>{/if}</div>
          <div class="artist-cards">
            {#each section.items as item (item.id || item.title)}
              <button class="artist-card" on:click={() => item.kind === 'artist' ? onOpenArtist(item.id) : onOpenPlaylist(item.id)}>
                <img src={item.thumbnail || '/placeholder.jpg'} alt="" />
                <strong>{item.title}</strong><small>{item.subtitle || item.kind}</small>
              </button>
            {/each}
          </div>
        </section>
      {/if}
    {/each}
  {/if}
</div>

<style>
  .artist-page { min-height: 100%; padding: 0 0 68px; color: #f5f6f6; background: #080909; }
  .artist-hero { min-height: 320px; position: relative; display: flex; align-items: end; overflow: hidden; padding: 48px clamp(26px, 5vw, 64px) 34px; background: #171b19; }
  .artist-banner { position: absolute; inset: 0; width: 100%; height: 100%; object-fit: cover; opacity: .55; filter: blur(2px) saturate(.8); }
  .artist-hero-shade { position: absolute; inset: 0; background: linear-gradient(90deg, #080909ef 0%, #080909a8 50%, #08090942), linear-gradient(0deg, #080909 0%, transparent 80%); }
  .artist-hero-copy { position: relative; z-index: 1; max-width: 760px; }
  .artist-hero-copy > span { color: #c9cecb; font-size: 10px; font-weight: 800; letter-spacing: .15em; }
  .artist-hero h1 { margin: 8px 0; font-size: clamp(34px, 6vw, 58px); line-height: 1; letter-spacing: -.045em; }
  .artist-hero p { margin: 0; color: #d2d6d4; font-size: 13px; }
  .artist-hero-copy > small { display: block; max-width: 650px; max-height: 42px; overflow: hidden; margin-top: 12px; color: #b1b7b3; font-size: 11px; line-height: 1.5; }
  .artist-actions { display: flex; gap: 9px; margin-top: 20px; }
  .artist-play-all, .artist-secondary { min-height: 36px; display: inline-flex; align-items: center; gap: 8px; padding: 0 15px; border: 0; border-radius: 20px; font-size: 11px; font-weight: 700; cursor: pointer; }
  .artist-play-all { color: #101211; background: var(--accent); }
  .artist-secondary { color: #eee; background: #292c2b; }
  .artist-section { padding: 25px clamp(26px, 5vw, 64px) 0; }
  .artist-section h2 { margin: 0 0 14px; font-size: 19px; letter-spacing: -.025em; }
  .artist-section-heading { display: flex; align-items: center; justify-content: space-between; }
  .artist-section-heading button { color: #aeb3b0; background: transparent; border: 0; font-size: 11px; cursor: pointer; }
  .artist-song-list { max-width: 900px; }
  .artist-song-row { width: 100%; min-height: 58px; display: grid; grid-template-columns: minmax(0,1fr) 38px; align-items: center; gap: 5px; padding: 3px 5px; color: #eee; border-radius: 6px; transition: background .18s ease; }
  .artist-song-row:hover, .artist-song-row:focus-within { background: rgba(255,255,255,.07); }
  .artist-song-main { width: 100%; min-width: 0; min-height: 52px; display: flex; align-items: center; gap: 13px; padding: 3px 5px; color: #eee; background: transparent; border: 0; border-radius: 6px; text-align: left; cursor: pointer; }
  .artist-song-row :global(.track-actions) { justify-self: center; }
  .artist-song-index { width: 22px; color: #959b97; text-align: center; font-size: 11px; }
  .artist-song-main > img { width: 42px; height: 42px; object-fit: cover; border-radius: 4px; }
  .artist-song-copy { min-width: 0; flex: 1; display: flex; flex-direction: column; gap: 4px; }
  .artist-song-copy strong, .artist-card strong { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; font-size: 12px; font-weight: 600; }
  .artist-song-copy small, .artist-card small { overflow: hidden; color: #969d99; text-overflow: ellipsis; white-space: nowrap; font-size: 10px; }
  .artist-song-play { display: none; color: var(--accent); }
  .artist-song-row:hover .artist-song-play { display: inline-flex; }
  .artist-song-duration { color: #969d99; font-size: 10px; }
  .artist-cards { display: flex; gap: 17px; overflow-x: auto; scrollbar-width: none; }
  .artist-card { width: 165px; flex: 0 0 165px; display: flex; flex-direction: column; align-items: flex-start; gap: 8px; padding: 0; color: white; background: transparent; border: 0; text-align: left; cursor: pointer; }
  .artist-card img { width: 165px; aspect-ratio: 1; object-fit: cover; border-radius: 6px; }
  .artist-card:hover img { filter: brightness(1.1); }
  .artist-state { min-height: 300px; display: flex; flex-direction: column; justify-content: center; align-items: center; color: #a9aeab; text-align: center; }
  .artist-state h2 { margin: 9px 0 4px; color: #eee; font-size: 17px; }
  .artist-state p { max-width: 500px; font-size: 12px; }
  .artist-state button { padding: 8px 14px; color: #fff; background: var(--accent); border: 0; border-radius: 18px; cursor: pointer; }
  .artist-spinner { width: 28px; height: 28px; border: 3px solid #2b332e; border-top-color: var(--accent); border-radius: 50%; animation: spin .7s linear infinite; }
  @keyframes spin { to { transform: rotate(360deg); } }
</style>
