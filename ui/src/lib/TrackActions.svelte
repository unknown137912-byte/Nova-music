<script lang="ts">
  import { ChevronRight, Copy, Download, Heart, ListEnd, ListMusic, ListPlus, MoreHorizontal, UserRound } from 'lucide-svelte';

  export let track: any;
  export let playlists: any[] = [];
  export let liked = false;
  export let onAction: (track: any, action: string) => Promise<boolean | void> | boolean | void = () => {};

  let open = false;
  let playlistsOpen = false;
  let likedState = liked;
  let previousLiked = liked;

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

<div class="track-actions">
  <button class="more-button" aria-label="More song actions" title="More options" aria-expanded={open} on:click|stopPropagation={() => open = !open}>
    <MoreHorizontal size={18} />
  </button>
  {#if open}
    <div class="actions-menu" role="menu">
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
  .actions-menu { position: absolute; z-index: 30; top: calc(100% + 5px); right: 0; width: 210px; padding: 5px; color: #f5f5f5; background: rgba(26,28,30,.94); border: 1px solid rgba(255,255,255,.12); border-radius: 12px; box-shadow: 0 16px 42px rgba(0,0,0,.48), inset 0 1px 0 rgba(255,255,255,.08); backdrop-filter: blur(24px) saturate(1.4); animation: menu-in .14s ease-out both; }
  .actions-menu button { width: 100%; min-height: 36px; display: flex; align-items: center; gap: 10px; padding: 0 10px; color: rgba(255,255,255,.86); text-align: left; background: transparent; border: 0; border-radius: 8px; font-size: 11px; cursor: pointer; transition: background .14s ease, color .14s ease; }
  .actions-menu button:hover { color: #fff; background: rgba(var(--cover-rgb, 244 107 130),.18); }
  .actions-menu button :global(svg) { color: rgba(255,255,255,.65); }
  .actions-menu button :global(.submenu-arrow) { margin-left: auto; }
  .playlist-submenu { max-height: 190px; overflow: auto; margin: 1px 0 4px 15px; padding-left: 5px; border-left: 1px solid rgba(255,255,255,.14); }
  .playlist-submenu button { min-height: 31px; padding-left: 9px; font-size: 10px; }
  .playlist-submenu small { display: block; padding: 7px 8px; color: rgba(255,255,255,.48); font-size: 9px; line-height: 1.5; }
  @keyframes menu-in { from { opacity: 0; transform: translateY(-4px) scale(.98); } to { opacity: 1; transform: translateY(0) scale(1); } }
</style>
