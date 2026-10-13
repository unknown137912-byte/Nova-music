<script lang="ts">
  import { onMount } from 'svelte';
  import { ChevronRight, Copy, Download, Heart, ListEnd, ListMusic, ListPlus, MoreHorizontal, UserRound } from 'lucide-svelte';

  export let track: any;
  export let playlists: any[] = [];
  export let liked = false;
  export let onAction: (track: any, action: string) => Promise<boolean | void> | boolean | void = () => {};

  let open = false;
  let playlistsOpen = false;
  let likedState = liked;
  let previousLiked = liked;
  let dropUp = false;
  let menuEl: HTMLDivElement;
  let actionsElement: HTMLDivElement;

  onMount(() => {
    const openFromSongDoubleClick = () => openFromDoubleClick();
    actionsElement.addEventListener('nova-track-actions-open', openFromSongDoubleClick);
    return () => actionsElement.removeEventListener('nova-track-actions-open', openFromSongDoubleClick);
  });

  function positionMenu() {
    requestAnimationFrame(() => {
      if (!menuEl) return;
      const bounds = menuEl.getBoundingClientRect();
      dropUp = bounds.bottom > window.innerHeight - 12 && bounds.top > bounds.height + 12;
    });
  }

  function toggleMenu() {
    open = !open;
    playlistsOpen = false;
    if (open) positionMenu();
  }

  function openFromDoubleClick() {
    open = true;
    playlistsOpen = false;
    positionMenu();
  }

  $: if (liked !== previousLiked) {
    previousLiked = liked;
    likedState = liked;
  }

  async function choose(action: string) {
    open = false;
    playlistsOpen = false;
    const result = await onAction(track, action);
    if (action === 'like' && typeof result === 'boolean') likedState = result;
  }
</script>

<svelte:window on:click={() => open = false} />

<div class="track-actions" bind:this={actionsElement}>
  <button class="more-button" aria-label="More song actions" title="More options" aria-expanded={open} on:click|stopPropagation={toggleMenu}>
    <MoreHorizontal size={18} />
  </button>
  {#if open}
    <div class="actions-menu" class:drop-up={dropUp} role="menu" bind:this={menuEl}>
      <div class="actions-heading"><strong>{track?.title || track?.name || 'Song options'}</strong><small>{track?.artist || track?.artists || track?.subtitle || 'NOVA Music'}</small></div>
      <button role="menuitem" on:click|stopPropagation={() => choose('play-next')}><ListEnd size={15} /> Play next</button>
      <button role="menuitem" on:click|stopPropagation={() => choose('queue')}><ListPlus size={15} /> Add to queue</button>
      <button role="menuitem" on:click|stopPropagation={() => choose('like')}><Heart size={15} fill={likedState ? 'currentColor' : 'none'} /> {likedState ? 'Remove from Liked Music' : 'Add to Liked Music'}</button>
      <button role="menuitem" aria-expanded={playlistsOpen} on:click|stopPropagation={() => playlistsOpen = !playlistsOpen}><ListMusic size={15} /> Add to playlist <ChevronRight size={14} class="submenu-arrow" /></button>
      {#if playlistsOpen}
        <div class="playlist-submenu">
          {#if playlists.length}
            {#each playlists as playlist (playlist.browseId || playlist.browse_id || playlist.id || playlist.title)}
              <button role="menuitem" on:click|stopPropagation={() => choose(`playlist:${playlist.browseId || playlist.browse_id || playlist.id}`)}>{playlist.title || playlist.name || 'Playlist'}</button>
            {/each}
          {:else}
            <small>Sign in and load playlists first.</small>
          {/if}
        </div>
      {/if}
      <div class="actions-separator"></div>
      <button role="menuitem" on:click|stopPropagation={() => choose('download')}><Download size={15} /> Download</button>
      <button role="menuitem" on:click|stopPropagation={() => choose('copy')}><Copy size={15} /> Copy song link</button>
      {#if track?.artistId || track?.artist_id}
        <button role="menuitem" on:click|stopPropagation={() => choose('artist')}><UserRound size={15} /> Go to artist</button>
      {/if}
    </div>
  {/if}
</div>

<style>
  .track-actions { position: relative; display: inline-flex; flex: 0 0 auto; }
  .more-button { width: 34px; height: 34px; display: grid; place-items: center; color: rgba(255,255,255,.62); background: transparent; border: 0; border-radius: 50%; cursor: pointer; opacity: .58; transition: color .18s ease, background .18s ease, opacity .18s ease, transform .18s ease; }
  .more-button:hover, .more-button[aria-expanded="true"] { color: #fff; background: rgba(255,255,255,.12); opacity: 1; transform: scale(1.04); }
  .actions-menu { position: absolute; z-index: 30; top: calc(100% + 5px); right: 0; width: 252px; max-height: min(70vh, 460px); overflow-y: auto; padding: 7px; color: #f8f3f2; background: linear-gradient(145deg, rgba(49,35,37,.58), rgba(20,15,17,.5)); border: 1px solid rgba(255,255,255,.17); border-radius: 15px; box-shadow: 0 20px 56px rgba(0,0,0,.3), inset 0 1px 0 rgba(255,255,255,.14); -webkit-backdrop-filter: blur(34px) saturate(1.4); backdrop-filter: blur(34px) saturate(1.4); animation: menu-in .16s cubic-bezier(.2,.8,.2,1) both; scrollbar-width: thin; }
  .actions-heading { display: flex; flex-direction: column; gap: 4px; padding: 9px 10px 10px; margin-bottom: 4px; border-bottom: 1px solid rgba(255,255,255,.09); }
  .actions-heading strong, .actions-heading small { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .actions-heading strong { color: rgba(255,255,255,.96); font-size: 11px; font-weight: 700; }
  .actions-heading small { color: rgba(255,255,255,.52); font-size: 9px; }
  .actions-separator { height: 1px; margin: 5px 7px; background: rgba(255,255,255,.09); }
  .actions-menu.drop-up { top: auto; bottom: calc(100% + 5px); animation-name: menu-in-up; }
  .actions-menu button { width: 100%; min-height: 37px; display: flex; align-items: center; gap: 10px; padding: 0 10px; color: rgba(255,255,255,.88); text-align: left; background: transparent; border: 0; border-radius: 9px; font-size: 11px; cursor: pointer; transition: background .14s ease, color .14s ease, transform .14s ease; }
  .actions-menu button:hover { color: #fff; background: rgba(var(--cover-rgb, 244 107 130),.2); transform: translateX(1px); }
  .actions-menu button :global(svg) { flex: 0 0 auto; color: rgba(255,255,255,.68); }
  .actions-menu button :global(.submenu-arrow) { margin-left: auto; }
  .playlist-submenu { max-height: 190px; overflow: auto; margin: 1px 0 4px 15px; padding-left: 5px; border-left: 1px solid rgba(255,255,255,.14); }
  .playlist-submenu button { min-height: 31px; padding-left: 9px; font-size: 10px; }
  .playlist-submenu small { display: block; padding: 7px 8px; color: rgba(255,255,255,.48); font-size: 9px; line-height: 1.5; }
  @keyframes menu-in { from { opacity: 0; transform: translateY(-4px) scale(.98); } to { opacity: 1; transform: translateY(0) scale(1); } }
  @keyframes menu-in-up { from { opacity: 0; transform: translateY(4px) scale(.98); } to { opacity: 1; transform: translateY(0) scale(1); } }
</style>
