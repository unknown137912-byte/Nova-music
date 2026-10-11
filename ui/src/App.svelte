<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { writeText } from '@tauri-apps/plugin-clipboard-manager';
  import { listen } from '@tauri-apps/api/event';
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import { exit } from '@tauri-apps/plugin-process';
  import { fade, fly } from 'svelte/transition';
  import {
    Home as HomeIcon, Search, Library, Settings2, ChevronLeft, ChevronRight,
    RotateCw, UserRound, ListMusic, Mic2, Heart, Play, Pause, SkipBack,
    SkipForward, Volume2, VolumeX, PanelRight, Download, MoreHorizontal,
    Plus, Music2, Sparkles, X, ListEnd, ChevronDown, Maximize2,
  } from 'lucide-svelte';

  import Home from './routes/+page.svelte';
  import SearchPage from './routes/search/+page.svelte';
  import LibraryPage from './routes/library/+page.svelte';
  import Playlists from './routes/playlists/+page.svelte';
  import ArtistsPage from './routes/artists/+page.svelte';
  import Settings from './routes/settings/+page.svelte';
  import TrackActions from './lib/TrackActions.svelte';

  type Tab = 'home' | 'search' | 'library' | 'playlists' | 'artist';
  type SidePanel = 'now-playing' | 'queue' | 'lyrics' | null;

  let activeTab: Tab = 'home';
  let activePlaylistId: string | null = null;
  let activeArtistId = '';
  let historyStack: Tab[] = ['home'];
  let historyIndex = 0;
  let librarySidebarOpen = true;
  let settingsOpen = false;
  let showNowPlaying = false;
  let sidePanel: SidePanel = null;
  let nowPlaying: any = null;
  let isPlaying = false;
  let isLoadingTrack = false;
  let playbackError: string | null = null;
  let playbackPosition = 0;
  let playbackDuration = 0;
  let isSeeking = false;
  let currentVolume = 85;
  let libraryCollections: Record<string, any[]> = { playlists: [], albums: [], artists: [] };
  let libraryCategory = 'playlists';
  let queueItems: any[] = [];
  let accountName = '';
  let accountEmail = '';
  let accountSignedIn = false;
  let appNotice = '';
  let appNoticeTimer: ReturnType<typeof setTimeout> | undefined;
  let closeToBackgroundEnabled = false;
  let autoPlayNext = true;
  let lastNotifiedTrackId = '';
  let lastNonzeroVolume = 85;
  let globalSearchQuery = '';
  let globalSearchResults: any = null;
  let globalSearchLoading = false;
  let globalSearchError = '';
  let searchPopoverOpen = false;
  let globalSearchInput: HTMLInputElement | undefined;
  let globalSearchTimer: ReturnType<typeof setTimeout> | undefined;
  let globalSearchRequest = 0;
  let lyricsTrackId = '';
  let lyricsData: any = null;
  let lyricsLoading = false;
  let lyricsError = '';
  let lyricsRequestSerial = 0;
  let activeLyricIndex = -1;
  let lastScrolledLyric = '';
  let coverRgb = '244 107 130';
  let coverDeepRgb = '30 17 30';

  function fallbackArtworkPalette(source: string) {
    let hash = 2166136261;
    for (let index = 0; index < source.length; index += 1) {
      hash = Math.imul(hash ^ source.charCodeAt(index), 16777619);
    }
    const hue = (hash >>> 0) % 360;
    const saturation = 0.46;
    const lightness = 0.52;
    const chroma = (1 - Math.abs(2 * lightness - 1)) * saturation;
    const x = chroma * (1 - Math.abs((hue / 60) % 2 - 1));
    const match = lightness - chroma / 2;
    const channels = hue < 60 ? [chroma, x, 0]
      : hue < 120 ? [x, chroma, 0]
      : hue < 180 ? [0, chroma, x]
      : hue < 240 ? [0, x, chroma]
      : hue < 300 ? [x, 0, chroma]
      : [chroma, 0, x];
    const [r, g, b] = channels.map((channel) => Math.round((channel + match) * 255));
    coverRgb = `${r} ${g} ${b}`;
    coverDeepRgb = `${Math.round(r * .2 + 7)} ${Math.round(g * .2 + 8)} ${Math.round(b * .22 + 10)}`;
  }

  function sampleArtworkPalette(source: string) {
    if (!source) return;
    fallbackArtworkPalette(source);
    const image = new Image();
    image.crossOrigin = 'anonymous';
    image.onload = () => {
      try {
        const canvas = document.createElement('canvas');
        canvas.width = 32;
        canvas.height = 32;
        const context = canvas.getContext('2d', { willReadFrequently: true });
        if (!context) return;
        context.drawImage(image, 0, 0, canvas.width, canvas.height);
        const pixels = context.getImageData(0, 0, canvas.width, canvas.height).data;
        const buckets = new Map<string, { rgb: number[]; score: number }>();
        for (let index = 0; index < pixels.length; index += 16) {
          const r = pixels[index], g = pixels[index + 1], b = pixels[index + 2], alpha = pixels[index + 3];
          if (alpha < 150) continue;
          const max = Math.max(r, g, b), min = Math.min(r, g, b), light = (max + min) / 2;
          const saturation = max === 0 ? 0 : (max - min) / max;
          if (light < 24 || light > 246) continue;
          const key = `${Math.round(r / 32)}-${Math.round(g / 32)}-${Math.round(b / 32)}`;
          const current = buckets.get(key) || { rgb: [r, g, b], score: 0 };
          current.rgb = current.rgb.map((value, i) => (value * current.score + [r, g, b][i]) / (current.score + 1));
          current.score += 0.35 + saturation * 1.8;
          buckets.set(key, current);
        }
        const best = [...buckets.values()].sort((a, b) => b.score - a.score)[0];
        if (!best) return;
        const [r, g, b] = best.rgb.map((value) => Math.round(value));
        coverRgb = `${r} ${g} ${b}`;
        coverDeepRgb = `${Math.round(r * .18 + 8)} ${Math.round(g * .18 + 8)} ${Math.round(b * .2 + 10)}`;
      } catch (_) {
        // Remote artwork can block canvas sampling; keep the last usable palette.
      }
    };
    image.src = source;
  }

  onMount(() => {
    let disposed = false;
    let stopStart: (() => void) | undefined;
    let stopError: (() => void) | undefined;
    let stopEnded: (() => void) | undefined;
    let stopCloseRequest: (() => void) | undefined;
    const openHistory = () => navigate('library');
    const openSettingsFromPage = () => openSettings();
    const refreshAccountLibrary = () => void loadAccountAndLibrary();
    const closeSearchOnOutsideClick = (event: MouseEvent) => {
      const target = event.target as HTMLElement | null;
      if (!target?.closest('.global-search-wrap')) searchPopoverOpen = false;
    };
    window.addEventListener('nova-open-history', openHistory);
    window.addEventListener('nova-open-settings', openSettingsFromPage);
    window.addEventListener('nova-local-playlists-changed', refreshAccountLibrary);
    window.addEventListener('pointerdown', closeSearchOnOutsideClick);
    const handleCloseSettingChange = (event: Event) => {
      closeToBackgroundEnabled = (event as CustomEvent<boolean>).detail;
    };
    const handleAutoPlayChange = (event: Event) => {
      autoPlayNext = (event as CustomEvent<boolean>).detail;
    };
    window.addEventListener('nova-close-to-background-changed', handleCloseSettingChange);
    window.addEventListener('nova-autoplay-changed', handleAutoPlayChange);

    listen('playback-started', async () => {
      playbackError = null;
      isPlaying = true;
      isLoadingTrack = false;
      try {
        const [settings, track]: [any, any] = await Promise.all([
          invoke('get_settings'), invoke('get_now_playing'),
        ]);
        const trackId = String(track?.id || '');
        if (settings?.song_notifications === 'true' && trackId && trackId !== lastNotifiedTrackId) {
          lastNotifiedTrackId = trackId;
          await invoke('show_song_notification', {
            title: track?.title || 'Now playing',
            artist: track?.artist || 'YouTube Music',
          });
        }
      } catch (_) {
        // Notifications are optional; playback must continue if the OS blocks them.
      }
    }).then((unlisten) => {
      if (disposed) unlisten(); else stopStart = unlisten;
    });

    getCurrentWindow().onCloseRequested((event) => {
      if (closeToBackgroundEnabled) {
        event.preventDefault();
        void getCurrentWindow().minimize();
        return;
      }
      event.preventDefault();
      void exit(0);
    }).then((unlisten) => {
      if (disposed) unlisten(); else stopCloseRequest = unlisten;
    });

    listen<string>('playback-error', (event) => {
      playbackError = event.payload || 'Playback failed';
      isPlaying = false;
      isLoadingTrack = false;
    }).then((unlisten) => {
      if (disposed) unlisten(); else stopError = unlisten;
    });

    listen('playback-ended', async () => {
      try {
        let next: any = await invoke('dequeue_next_track');
        if (!next && autoPlayNext) next = await invoke('get_autoplay_track');
        if (next && !disposed) {
          await handlePlayTrack(next);
          await loadQueue();
        }
      } catch (error: any) {
        showNotice(typeof error === 'string' ? error : (error?.message || 'Could not continue the queue'));
      }
    }).then((unlisten) => {
      if (disposed) unlisten(); else stopEnded = unlisten;
    });

    const loadAccountAndLibrary = async () => {
      try {
        const [account, library]: [any, any] = await Promise.all([
          invoke('get_account_info'), invoke('get_library'),
        ]);
        if (disposed) return;
        accountSignedIn = Boolean(account?.is_logged_in);
        accountName = account?.name || account?.email || '';
        accountEmail = account?.email || '';
        libraryCollections = {
          playlists: Array.isArray(library) ? library : (library?.playlists || []),
          albums: library?.albums || [],
          artists: library?.artists || [],
        };
      } catch (_) {
        // The app can still play/search while account or library data is unavailable.
      }
    };

    invoke<any>('get_settings').then((settings) => {
      if (!disposed) {
        closeToBackgroundEnabled = settings?.close_to_tray === 'true';
        autoPlayNext = settings?.auto_play_next !== 'false';
        const theme = String(settings?.cosmic_theme || 'nebula');
        const shell = document.querySelector('.app-shell');
        shell?.classList.remove('theme-nebula', 'theme-cyber', 'theme-deepspace');
        shell?.classList.add(`theme-${theme}`);
      }
    }).catch(() => {});

    const pollPlayback = async () => {
      try {
        const [track, state]: [any, any] = await Promise.all([
          invoke('get_now_playing'), invoke('get_playback_state'),
        ]);
        if (disposed) return;
        if (track?.id) {
          const sameTrack = nowPlaying?.id === track.id;
          if (!sameTrack) resetLyricsForTrack(String(track.id));
          if (!sameTrack) sampleArtworkPalette(String(track.thumbnail || ''));
          nowPlaying = {
            ...track,
            ...(sameTrack && nowPlaying?.album ? { album: nowPlaying.album } : {}),
            ...(sameTrack && nowPlaying?.albumId ? { albumId: nowPlaying.albumId } : {}),
            ...(sameTrack && nowPlaying?.artistId ? { artistId: nowPlaying.artistId } : {}),
            ...(sameTrack && track.liked === undefined && nowPlaying?.liked !== undefined ? { liked: nowPlaying.liked, rating: nowPlaying.rating } : {}),
          };
          if (!sameTrack && (sidePanel === 'lyrics' || showNowPlaying)) void loadLyrics();
        }
        if (state) {
          if (!isLoadingTrack) isPlaying = !state.paused && !state.idle;
          if (!isSeeking) playbackPosition = Number(state.position || 0);
          if (Number(state.duration) > 0) playbackDuration = Number(state.duration);
          if (typeof state.volume === 'number') {
            currentVolume = state.volume;
            if (state.volume > 0) lastNonzeroVolume = state.volume;
          }
        }
      } catch (_) {
        // Idle playback state is expected on first launch.
      }
    };

    loadAccountAndLibrary();
    pollPlayback();
    const playbackTimer = setInterval(pollPlayback, 900);
    const accountTimer = setInterval(loadAccountAndLibrary, 45000);
    return () => {
      disposed = true;
      clearInterval(playbackTimer);
      clearInterval(accountTimer);
      if (appNoticeTimer) clearTimeout(appNoticeTimer);
      stopStart?.();
      stopError?.();
      stopEnded?.();
      stopCloseRequest?.();
      window.removeEventListener('nova-open-history', openHistory);
      window.removeEventListener('nova-open-settings', openSettingsFromPage);
      window.removeEventListener('nova-local-playlists-changed', refreshAccountLibrary);
      window.removeEventListener('pointerdown', closeSearchOnOutsideClick);
      window.removeEventListener('nova-close-to-background-changed', handleCloseSettingChange);
      window.removeEventListener('nova-autoplay-changed', handleAutoPlayChange);
      if (globalSearchTimer) clearTimeout(globalSearchTimer);
    };
  });

  function navigate(tab: Tab) {
    searchPopoverOpen = false;
    if (activeTab === tab) {
      if (showNowPlaying) void closeImmersivePlayer();
      return;
    }
    historyStack = [...historyStack.slice(0, historyIndex + 1), tab];
    historyIndex = historyStack.length - 1;
    activeTab = tab;
    if (showNowPlaying) void closeImmersivePlayer();
  }

  function navigateHistory(direction: number) {
    const target = historyIndex + direction;
    if (target < 0 || target >= historyStack.length) return;
    historyIndex = target;
    activeTab = historyStack[historyIndex];
    if (showNowPlaying) void closeImmersivePlayer();
  }

  function openSettings() {
    settingsOpen = true;
  }

  function showNotice(message: string) {
    appNotice = message;
    if (appNoticeTimer) clearTimeout(appNoticeTimer);
    appNoticeTimer = setTimeout(() => appNotice = '', 4000);
  }

  async function runGlobalSearch() {
    const query = globalSearchQuery.trim();
    if (!query) return;
    const request = ++globalSearchRequest;
    globalSearchLoading = true;
    globalSearchError = '';
    searchPopoverOpen = true;
    try {
      const result: any = await invoke('search', { query });
      if (request === globalSearchRequest) globalSearchResults = result;
    } catch (error: any) {
      if (request === globalSearchRequest) {
        globalSearchError = typeof error === 'string' ? error : (error?.message || 'Search failed. Try again.');
        globalSearchResults = null;
      }
    } finally {
      if (request === globalSearchRequest) globalSearchLoading = false;
    }
  }

  function handleGlobalSearchInput() {
    searchPopoverOpen = true;
    globalSearchRequest += 1;
    globalSearchResults = null;
    globalSearchError = '';
    globalSearchLoading = false;
    if (globalSearchTimer) clearTimeout(globalSearchTimer);
    const query = globalSearchQuery.trim();
    if (query.length < 2) {
      globalSearchRequest += 1;
      globalSearchResults = null;
      globalSearchError = '';
      globalSearchLoading = false;
      return;
    }
    globalSearchTimer = setTimeout(() => void runGlobalSearch(), 320);
  }

  function handleGlobalSearchKeydown(event: KeyboardEvent) {
    if (event.key === 'Escape') {
      searchPopoverOpen = false;
      globalSearchInput?.blur();
    } else if (event.key === 'Enter') {
      event.preventDefault();
      if (globalSearchTimer) clearTimeout(globalSearchTimer);
      if (globalSearchResults) navigate('search');
      else void runGlobalSearch();
    }
  }

  function openFullSearch() {
    if (!globalSearchQuery.trim()) return;
    if (globalSearchTimer) clearTimeout(globalSearchTimer);
    searchPopoverOpen = false;
    navigate('search');
  }

  function chooseGlobalSearchResult(item: any, kind: 'song' | 'artist' | 'album' | 'playlist') {
    searchPopoverOpen = false;
    if (kind === 'song') {
      handlePlayTrack({
        ...item,
        id: item.videoId || item.video_id || item.id,
        artist: item.artist || item.artists || item.subtitle || 'YouTube Music',
        thumbnail: item.thumbnail || item.thumbnail_url || '',
      });
    } else if (kind === 'artist') {
      handleOpenArtist(item.id || item.browseId || item.browse_id || '');
    } else {
      handleOpenPlaylist(item.id || item.browseId || item.browse_id || '');
    }
  }

  $: quickSearchSongs = (globalSearchResults?.songs || []).slice(0, 6);
  $: quickSearchCollections = [
    ...(globalSearchResults?.artists || []).slice(0, 2).map((item: any) => ({ ...item, resultKind: 'artist' })),
    ...(globalSearchResults?.albums || []).slice(0, 2).map((item: any) => ({ ...item, resultKind: 'album' })),
    ...(globalSearchResults?.playlists || []).slice(0, 2).map((item: any) => ({ ...item, resultKind: 'playlist' })),
  ];

  function resetLyricsForTrack(trackId: string) {
    if (lyricsTrackId === trackId) return;
    lyricsRequestSerial += 1;
    lyricsTrackId = '';
    lyricsData = null;
    lyricsLoading = false;
    lyricsError = '';
    activeLyricIndex = -1;
    lastScrolledLyric = '';
  }

  async function loadLyrics(force = false) {
    const track = nowPlaying;
    const trackId = String(track?.id || '').trim();
    if (!trackId) return;
    if (!force && lyricsTrackId === trackId && (lyricsLoading || lyricsData)) return;

    lyricsTrackId = trackId;
    lyricsData = null;
    lyricsError = '';
    lyricsLoading = true;
    const requestSerial = ++lyricsRequestSerial;
    const artist = Array.isArray(track.artist)
      ? track.artist.map((item: any) => item?.name || item?.text || '').filter(Boolean).join(', ')
      : String(track.artist || track.artists || track.subtitle || '');
    try {
      const result: any = await invoke('get_lyrics', {
        videoId: trackId,
        title: String(track.title || 'Unknown track'),
        artist,
        album: track.album || null,
        duration: playbackDuration || Number(track.duration || 0) || null,
      });
      if (requestSerial !== lyricsRequestSerial || trackId !== String(nowPlaying?.id || '')) return;
      lyricsData = result;
      if (!result?.lines?.length && !String(result?.plain_text || '').trim()) {
        lyricsError = 'Lyrics are not available for this song yet.';
      }
    } catch (error: any) {
      if (requestSerial !== lyricsRequestSerial) return;
      lyricsError = typeof error === 'string' ? error : (error?.message || 'Could not load lyrics. Check your connection and retry.');
    } finally {
      if (requestSerial === lyricsRequestSerial) lyricsLoading = false;
    }
  }

  function getActiveLyric(lines: any[], position: number) {
    let active = -1;
    const timeMs = position * 1000;
    for (let index = 0; index < lines.length; index += 1) {
      if (Number(lines[index]?.time_ms) > timeMs) break;
      active = index;
    }
    return active;
  }

  function openImmersivePlayer() {
    if (!nowPlaying) return;
    showNowPlaying = true;
    void loadLyrics();
  }

  function closeImmersivePlayer() {
    showNowPlaying = false;
  }

  async function seekToLyric(timeMs: number) {
    const position = Math.max(0, timeMs / 1000);
    playbackPosition = position;
    try { await invoke('seek', { position }); }
    catch (error: any) { showNotice(typeof error === 'string' ? error : (error?.message || 'Could not seek')); }
  }

  function durationInSeconds(value: any): number | null {
    if (typeof value === 'number' && Number.isFinite(value)) return value;
    if (typeof value !== 'string' || !value.trim()) return null;
    const parts = value.split(':').map((part) => Number(part));
    if (parts.some((part) => !Number.isFinite(part))) return null;
    if (parts.length === 2) return parts[0] * 60 + parts[1];
    if (parts.length === 3) return parts[0] * 3600 + parts[1] * 60 + parts[2];
    return null;
  }

  async function handlePlayTrack(track: any) {
    const trackId = String(track?.videoId || track?.video_id || track?.trackId || track?.track_id || track?.id || '').trim();
    if (!trackId) {
      playbackError = 'This item is missing its YouTube track ID. Refresh the page and try again.';
      showNotice(playbackError);
      return;
    }
    const duration = durationInSeconds(track?.duration_seconds ?? track?.duration);
    resetLyricsForTrack(trackId);
    playbackError = null;
    isLoadingTrack = true;
    isPlaying = true;
    playbackPosition = 0;
    playbackDuration = duration || 0;
    nowPlaying = {
      ...track,
      id: trackId,
      title: track?.title || track?.name || 'Unknown track',
      artist: track?.artist || track?.artists || track?.subtitle || 'YouTube Music',
      thumbnail: track?.thumbnail || track?.thumbnail_url || track?.cover || '',
      duration: duration || undefined,
    };
    sampleArtworkPalette(String(nowPlaying.thumbnail || ''));
    if (sidePanel === 'lyrics' || showNowPlaying) void loadLyrics();

    try {
      const resolved: any = await invoke('play_track', {
        trackId,
        title: nowPlaying.title,
        artist: nowPlaying.artist,
        thumbnail: nowPlaying.thumbnail,
        duration,
      });
      if (resolved) nowPlaying = {
        ...nowPlaying,
        ...resolved,
        ...(track?.album ? { album: track.album } : {}),
        ...(track?.albumId || track?.album_id ? { albumId: track.albumId || track.album_id } : {}),
        ...(track?.artistId || track?.artist_id ? { artistId: track.artistId || track.artist_id } : {}),
        liked: trackIsLiked(track),
      };
    } catch (error: any) {
      playbackError = typeof error === 'string' ? error : (error?.message || 'Playback failed');
      isPlaying = false;
      showNotice(playbackError);
    } finally {
      isLoadingTrack = false;
    }
  }

  function handleOpenPlaylist(id: string) {
    if (!id) return;
    activePlaylistId = id;
    navigate('playlists');
  }

  function handleOpenArtist(id: string) {
    if (!id) return;
    activeArtistId = id;
    navigate('artist');
  }

  async function togglePlay() {
    try {
      const paused: boolean = await invoke('pause_playback');
      isPlaying = !paused;
    } catch (error: any) {
      playbackError = typeof error === 'string' ? error : (error?.message || 'Playback control failed');
      showNotice(playbackError);
    }
  }

  async function skipTrack() {
    try {
      let next: any = await invoke('dequeue_next_track');
      let autoplayError: any = null;
      if (!next && autoPlayNext) {
        try { next = await invoke('get_autoplay_track'); }
        catch (error) { autoplayError = error; }
      }
      if (next) {
        await handlePlayTrack(next);
        await loadQueue();
      } else {
        await invoke('skip_track');
        if (autoplayError) showNotice(typeof autoplayError === 'string' ? autoplayError : (autoplayError?.message || 'Autoplay recommendations are unavailable right now'));
      }
    }
    catch (error: any) { showNotice(typeof error === 'string' ? error : (error?.message || 'Could not skip track')); }
  }

  async function handleTrackAction(track: any, action: string): Promise<boolean | void> {
    const trackId = String(track?.videoId || track?.video_id || track?.trackId || track?.track_id || track?.id || '').trim();
    if (!trackId) { showNotice('This song is missing its YouTube track ID.'); return false; }
    try {
      if (action === 'play-next' || action === 'queue') {
        const count: number = await invoke('enqueue_track', { track, playNext: action === 'play-next' });
        await loadQueue();
        showNotice(action === 'play-next' ? 'Added as the next song' : `Added to queue · ${count} queued`);
      } else if (action.startsWith('playlist:')) {
        const playlistId = action.slice('playlist:'.length);
        if (playlistId.startsWith('NOVA_LOCAL_')) {
          await invoke('add_track_to_local_playlist', { track, playlistId });
        } else {
          await invoke('add_track_to_playlist', { trackId, playlistId });
        }
        window.dispatchEvent(new CustomEvent('nova-playlist-updated', { detail: playlistId }));
        window.dispatchEvent(new CustomEvent('nova-local-playlists-changed'));
        showNotice('Added to playlist');
      } else if (action === 'download') {
        const path: string = await invoke('download_track', { trackId, title: track?.title || track?.name || 'NOVA Music download' });
        showNotice(`Saved to ${path}`);
      } else if (action === 'like') {
        const liked = !trackIsLiked(track);
        await invoke('rate_track', { trackId, liked });
        track.liked = liked;
        track.rating = liked ? 'like' : 'indifferent';
        if (String(nowPlaying?.id || '') === trackId) nowPlaying = { ...nowPlaying, liked, rating: track.rating };
        showNotice(liked ? 'Added to Liked Music' : 'Removed from Liked Music');
        return liked;
      } else if (action === 'copy') {
        await writeText(`https://music.youtube.com/watch?v=${encodeURIComponent(trackId)}`);
        showNotice('Song link copied');
      } else if (action === 'artist') {
        const artistId = track?.artistId || track?.artist_id;
        if (artistId) handleOpenArtist(String(artistId));
      }
    } catch (error: any) {
      showNotice(typeof error === 'string' ? error : (error?.message || 'Could not complete that song action'));
      return false;
    }
  }

  async function prevTrack() {
    try { await invoke('previous_track'); }
    catch (error: any) { showNotice(typeof error === 'string' ? error : (error?.message || 'Could not play previous track')); }
  }

  function previewSeek(event: Event) {
    isSeeking = true;
    playbackPosition = Number((event.currentTarget as HTMLInputElement).value);
  }

  async function commitSeek(event: Event) {
    const position = Number((event.currentTarget as HTMLInputElement).value);
    playbackPosition = position;
    try { await invoke('seek', { position }); }
    catch (error: any) { showNotice(typeof error === 'string' ? error : (error?.message || 'Could not seek')); }
    finally { isSeeking = false; }
  }

  async function changeVolume(event: Event) {
    const volume = Number((event.currentTarget as HTMLInputElement).value);
    currentVolume = volume;
    if (volume > 0) lastNonzeroVolume = volume;
    try { await invoke('set_volume', { volume }); }
    catch (error: any) { showNotice(typeof error === 'string' ? error : (error?.message || 'Could not change volume')); }
  }

  async function toggleMute() {
    const next = currentVolume > 0 ? 0 : lastNonzeroVolume;
    if (currentVolume > 0) lastNonzeroVolume = currentVolume;
    currentVolume = next;
    try { await invoke('set_volume', { volume: next }); }
    catch (error: any) { showNotice(typeof error === 'string' ? error : (error?.message || 'Could not change volume')); }
  }

  async function downloadCurrentTrack() {
    try {
      const path: string = await invoke('download_current_track');
      showNotice(`Saved to ${path}`);
    } catch (error: any) {
      showNotice(typeof error === 'string' ? error : (error?.message || 'Download failed'));
    }
  }

  async function loadQueue() {
    try {
      const result: any = await invoke('get_queue');
      queueItems = Array.isArray(result) ? result : (result?.items || []);
    } catch (_) {
      queueItems = [];
    }
  }

  function toggleSidePanel(panel: Exclude<SidePanel, null>) {
    sidePanel = sidePanel === panel ? null : panel;
    if (sidePanel === 'queue') loadQueue();
    if (sidePanel === 'lyrics') void loadLyrics();
  }

  function trackIsLiked(track: any) {
    return track?.liked === true || String(track?.rating || '').toLowerCase() === 'like';
  }

  async function toggleLikeTrack() {
    const trackId = String(nowPlaying?.id || '').trim();
    if (!trackId) return;
    const wasLiked = trackIsLiked(nowPlaying);
    nowPlaying = { ...nowPlaying, liked: !wasLiked, rating: wasLiked ? 'indifferent' : 'like' };
    try {
      await invoke('rate_track', { trackId, liked: !wasLiked });
      showNotice(wasLiked ? 'Removed from Liked Music' : 'Added to Liked Music');
    } catch (error: any) {
      nowPlaying = { ...nowPlaying, liked: wasLiked, rating: wasLiked ? 'like' : 'indifferent' };
      showNotice(typeof error === 'string' ? error : (error?.message || 'Could not update Liked Music'));
    }
  }

  function formatTime(seconds: number) {
    if (!Number.isFinite(seconds) || seconds < 0) return '0:00';
    const mins = Math.floor(seconds / 60);
    return `${mins}:${Math.floor(seconds % 60).toString().padStart(2, '0')}`;
  }

  function playLibraryItem(item: any) {
    const id = item?.playlistId || item?.playlist_id || item?.browseId || item?.browse_id || item?.id || '';
    if (libraryCategory === 'artists') handleOpenArtist(id);
    else handleOpenPlaylist(id);
  }

  $: sidebarLibraryItems = libraryCollections[libraryCategory] || [];

  function handleHotkey(event: KeyboardEvent) {
    const target = event.target as HTMLElement | null;
    if (event.key === 'Escape' && showNowPlaying) {
      event.preventDefault();
      void closeImmersivePlayer();
      return;
    }
    if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === 'k') {
      event.preventDefault();
      globalSearchInput?.focus();
      searchPopoverOpen = true;
      return;
    }
    if (event.code !== 'Space' || target?.matches('input, textarea, button, [contenteditable="true"]')) return;
    event.preventDefault();
    togglePlay();
  }

  $: activeLyricIndex = getActiveLyric(lyricsData?.lines || [], playbackPosition);
  $: if ((sidePanel === 'lyrics' || showNowPlaying) && activeLyricIndex >= 0) {
    const scrollKey = `${lyricsTrackId}:${activeLyricIndex}`;
    if (scrollKey !== lastScrolledLyric) {
      lastScrolledLyric = scrollKey;
      setTimeout(() => {
        document.querySelectorAll(`[data-lyric-index="${activeLyricIndex}"]`).forEach((line) => {
          line.scrollIntoView({ behavior: 'smooth', block: 'center' });
        });
      }, 50);
    }
  }
</script>

<svelte:window on:keydown={handleHotkey} />

<div class="app-shell theme-nebula" style={`--cover-rgb:${coverRgb};--cover-deep-rgb:${coverDeepRgb}`}>
  <header class="topbar" data-tauri-drag-region>
    <div class="traffic-light-clearance" data-tauri-drag-region></div>
    <div class="topbar-history">
      <button class="chrome-button" on:click={() => navigateHistory(-1)} disabled={historyIndex === 0} aria-label="Back">
        <ChevronLeft size={18} />
      </button>
      <button class="chrome-button" on:click={() => navigateHistory(1)} disabled={historyIndex >= historyStack.length - 1} aria-label="Forward">
        <ChevronRight size={18} />
      </button>
      <button class="chrome-button refresh-button" on:click={() => window.dispatchEvent(new CustomEvent('nova-refresh-current-view'))} aria-label="Refresh">
        <RotateCw size={16} />
      </button>
    </div>

    <div class="global-search-wrap">
      <div class="global-search" class:focused={searchPopoverOpen}>
        <Search size={17} />
        <input
          bind:this={globalSearchInput}
          bind:value={globalSearchQuery}
          on:input={handleGlobalSearchInput}
          on:focus={() => { searchPopoverOpen = true; }}
          on:keydown={handleGlobalSearchKeydown}
          placeholder="What do you want to play?"
          aria-label="Search YouTube Music"
          aria-expanded={searchPopoverOpen}
          aria-controls="global-search-results"
        />
        <kbd>⌘ K</kbd>
      </div>
      {#if searchPopoverOpen && globalSearchQuery.trim().length >= 2}
        <div id="global-search-results" class="global-search-popover" in:fly={{ y: -7, duration: 170 }}>
          <div class="search-popover-heading"><span>SEARCH YOUTUBE MUSIC</span><button on:click={() => searchPopoverOpen = false} aria-label="Close search results"><X size={15} /></button></div>
          {#if globalSearchLoading}
            <div class="search-popover-state"><span class="search-popover-spinner"></span><span>Finding music…</span></div>
          {:else if globalSearchError}
            <div class="search-popover-state error">{globalSearchError}</div>
          {:else if globalSearchResults}
            {#if quickSearchSongs.length}
              <div class="search-popover-section"><span class="search-popover-label">SONGS</span>
                {#each quickSearchSongs as item (item.id || item.videoId)}
                  <div class="search-result-row">
                    <button class="search-result-main" on:click={() => chooseGlobalSearchResult(item, 'song')}>
                      <img src={item.thumbnail || '/placeholder.jpg'} alt="" />
                      <span class="search-result-copy"><strong>{item.title || 'Untitled track'}</strong><small>{item.artist || item.artists || 'YouTube Music'}</small></span>
                      <Play size={15} fill="currentColor" />
                    </button>
                    <TrackActions track={item} playlists={libraryCollections.playlists} onAction={handleTrackAction} />
                  </div>
                {/each}
              </div>
            {/if}
            {#if quickSearchCollections.length}
              <div class="search-popover-section collection-results"><span class="search-popover-label">MORE RESULTS</span>
                {#each quickSearchCollections as item (`${item.resultKind}:${item.id}`)}
                  <button class="search-collection-result" on:click={() => chooseGlobalSearchResult(item, item.resultKind)}>
                    <img src={item.thumbnail || '/placeholder.jpg'} alt="" />
                    <span><strong>{item.title || 'Untitled'}</strong><small>{item.resultKind}</small></span>
                  </button>
                {/each}
              </div>
            {/if}
            {#if !quickSearchSongs.length && !quickSearchCollections.length}
              <div class="search-popover-state">No results found for “{globalSearchQuery.trim()}”.</div>
            {/if}
            <button class="search-view-all" on:click={openFullSearch}>Show all results for “{globalSearchQuery.trim()}” <ChevronRight size={15} /></button>
          {:else}
            <div class="search-popover-state">Search songs, artists, albums, and playlists.</div>
          {/if}
        </div>
      {/if}
    </div>

    <div class="topbar-actions" data-tauri-drag-region>
      <button class="chrome-button library-toggle" class:selected={librarySidebarOpen} on:click={() => librarySidebarOpen = !librarySidebarOpen} aria-label="Toggle your library">
        <Library size={18} />
      </button>
      <button class="profile-button" on:click={openSettings} aria-label="Account and settings">
        <span class="profile-avatar">{accountSignedIn ? (accountName.charAt(0).toUpperCase() || 'N') : 'N'}</span>
        <span class="profile-label">{accountName || 'NOVA Music'}</span>
        <ChevronDown size={14} />
      </button>
      <button class="chrome-button settings-top-button" on:click={openSettings} aria-label="Settings">
        <Settings2 size={17} />
      </button>
    </div>
  </header>

  <div class="workarea">
    <aside class="nav-rail" aria-label="Primary navigation">
      <button class="brand-mark" on:click={() => navigate('home')} aria-label="NOVA Music home">
        <Sparkles size={19} />
      </button>
      <div class="rail-separator"></div>
      <button class="rail-button" class:selected={activeTab === 'home' && !showNowPlaying} on:click={() => navigate('home')} aria-label="Home" title="Home">
        <HomeIcon size={19} />
      </button>
      <button class="rail-button" class:selected={activeTab === 'library'} on:click={() => { navigate('library'); librarySidebarOpen = true; }} aria-label="Library" title="Library">
        <Library size={19} />
      </button>
      <button class="rail-button" class:selected={sidePanel === 'queue' || sidePanel === 'now-playing'} on:click={() => toggleSidePanel('queue')} aria-label="Queue" title="Queue">
        <ListMusic size={19} />
      </button>
      <div class="rail-spacer"></div>
      {#if nowPlaying}
        <button class="rail-art-button" on:click={openImmersivePlayer} aria-label="Open now playing" title="Now playing">
          <img src={nowPlaying.thumbnail || '/placeholder.jpg'} alt="" />
        </button>
      {/if}
      <button class="rail-button rail-settings" on:click={openSettings} aria-label="Settings" title="Settings">
        <Settings2 size={19} />
      </button>
    </aside>

    {#if librarySidebarOpen}
      <aside class="library-sidebar">
        <div class="sidebar-heading">
          <button class="library-title" on:click={() => navigate('library')}>
            <Library size={17} /> <span>Your Library</span>
          </button>
          <button class="icon-button" on:click={() => navigate('playlists')} aria-label="Browse playlists" title="Browse playlists">
            <Plus size={17} />
          </button>
          <button class="icon-button" on:click={() => librarySidebarOpen = false} aria-label="Close library sidebar" title="Close sidebar">
            <PanelRight size={16} />
          </button>
        </div>
        <div class="library-filters">
          <button class="filter-chip" class:active={libraryCategory === 'playlists'} on:click={() => libraryCategory = 'playlists'}>Playlists</button>
          <button class="filter-chip" class:active={libraryCategory === 'albums'} on:click={() => libraryCategory = 'albums'}>Albums</button>
          <button class="filter-chip" class:active={libraryCategory === 'artists'} on:click={() => libraryCategory = 'artists'}>Artists</button>
        </div>
        <div class="library-tools">
          <button class="sort-button" on:click={() => navigate('library')}>Recently added <ListEnd size={14} /></button>
        </div>
        <div class="library-list">
          <button class="library-entry" on:click={() => handleOpenPlaylist('LM')}>
            <span class="library-art liked-art"><Heart size={19} fill="currentColor" /></span>
            <span class="library-copy"><strong>Liked Music</strong><small>Auto playlist</small></span>
          </button>
          {#each sidebarLibraryItems as item (item.id || item.playlist_id || item.title)}
            <button class="library-entry" on:click={() => playLibraryItem(item)}>
              <span class="library-art">
                {#if item.thumbnail || item.cover || item.thumbnail_url}
                  <img src={item.thumbnail || item.cover || item.thumbnail_url} alt="" />
                {:else}
                  <Music2 size={18} />
                {/if}
              </span>
              <span class="library-copy"><strong>{item.title || item.name || 'Untitled playlist'}</strong><small>{item.subtitle || item.type || 'Playlist'}</small></span>
            </button>
          {/each}
          {#if sidebarLibraryItems.length === 0}
            <div class="sidebar-empty">Your saved {libraryCategory} will show up here.</div>
          {/if}
        </div>
      </aside>
    {/if}

    <main class="main-content" class:panel-open={sidePanel !== null}>
      {#key `${activeTab}:${activePlaylistId || ''}:${activeArtistId}:${activeTab === 'search' ? globalSearchQuery : ''}`}
      <div class="route-layer" in:fly={{ y: 8, duration: 230 }} out:fade={{ duration: 115 }}>
      {#if activeTab === 'home'}
        <Home accountName={accountName} onPlayTrack={handlePlayTrack} onOpenPlaylist={handleOpenPlaylist} onOpenArtist={handleOpenArtist} onTrackAction={handleTrackAction} playlistOptions={libraryCollections.playlists} />
      {:else if activeTab === 'search'}
        <SearchPage initialQuery={globalSearchQuery} onPlayTrack={handlePlayTrack} onOpenPlaylist={handleOpenPlaylist} onOpenArtist={handleOpenArtist} onTrackAction={handleTrackAction} playlistOptions={libraryCollections.playlists} />
      {:else if activeTab === 'library'}
        <LibraryPage onPlayTrack={handlePlayTrack} onOpenPlaylist={handleOpenPlaylist} onOpenArtist={handleOpenArtist} onTrackAction={handleTrackAction} playlistOptions={libraryCollections.playlists} />
      {:else if activeTab === 'playlists'}
        <Playlists bind:activePlaylistId onPlayTrack={handlePlayTrack} onOpenArtist={handleOpenArtist} onTrackAction={handleTrackAction} playlistOptions={libraryCollections.playlists} />
      {:else if activeTab === 'artist'}
        <ArtistsPage artistId={activeArtistId} onPlayTrack={handlePlayTrack} onOpenPlaylist={handleOpenPlaylist} onOpenArtist={handleOpenArtist} onTrackAction={handleTrackAction} playlistOptions={libraryCollections.playlists} />
      {/if}
      </div>
      {/key}
    </main>

    {#if sidePanel}
      <aside class="right-panel">
        <div class="right-panel-header">
          <div class="panel-tabs">
            <button class:active={sidePanel === 'now-playing'} on:click={() => toggleSidePanel('now-playing')}>Now playing</button>
            <button class:active={sidePanel === 'queue'} on:click={() => toggleSidePanel('queue')}>Queue</button>
            <button class:active={sidePanel === 'lyrics'} on:click={() => toggleSidePanel('lyrics')}>Lyrics</button>
          </div>
          <button class="icon-button" on:click={() => sidePanel = null} aria-label="Close panel"><X size={17} /></button>
        </div>
        {#if sidePanel === 'now-playing'}
          <div class="now-playing-panel">
            {#if nowPlaying}
              <div class="panel-now-art"><img src={nowPlaying.thumbnail || '/placeholder.jpg'} alt="" /></div>
              <div class="panel-now-track">
                <div class="panel-now-copy">
                  <strong>{nowPlaying.title || 'Unknown track'}</strong>
                  <button disabled={!nowPlaying.artistId} on:click={() => nowPlaying.artistId && handleOpenArtist(nowPlaying.artistId)}>{nowPlaying.artist || 'YouTube Music'}</button>
                </div>
                <button class="like-track-button" class:liked={trackIsLiked(nowPlaying)} on:click={toggleLikeTrack} aria-label={trackIsLiked(nowPlaying) ? 'Remove from Liked Music' : 'Add to Liked Music'} title={trackIsLiked(nowPlaying) ? 'Remove from Liked Music' : 'Add to Liked Music'}>
                  <Heart size={19} fill={trackIsLiked(nowPlaying) ? 'currentColor' : 'none'} />
                </button>
              </div>
              <div class="panel-album-card">
                <div class="panel-album-heading"><strong>Playing from</strong><MoreHorizontal size={18} /></div>
                <button class="panel-album-link" disabled={!nowPlaying.albumId} on:click={() => nowPlaying.albumId && handleOpenPlaylist(nowPlaying.albumId)}>
                  <img src={nowPlaying.thumbnail || '/placeholder.jpg'} alt="" />
                  <span><strong>{nowPlaying.album || 'YouTube Music'}</strong><small>{nowPlaying.album ? 'Album' : 'Current source'}</small></span>
                </button>
              </div>
              <div class="panel-artist-card">
                <div class="panel-artist-art"><UserRound size={32} /></div>
                <div class="panel-artist-info"><strong>About the artist</strong><span>{nowPlaying.artist || 'YouTube Music'}</span></div>
              </div>
            {:else}
              <div class="panel-empty"><Music2 size={25} /><p>Play a track to see what’s playing.</p></div>
            {/if}
          </div>
        {:else if sidePanel === 'queue'}
          <div class="queue-content">
            <div class="queue-context">
              <span>PLAYING FROM</span>
              <strong>{nowPlaying?.playlist || 'Your queue'}</strong>
              <small>{queueItems.length} {queueItems.length === 1 ? 'track' : 'tracks'} queued</small>
            </div>
            {#if nowPlaying}
              <button class="queue-track current" on:click={openImmersivePlayer}>
                <span class="queue-number"><span class="playing-bars"><i></i><i></i><i></i></span></span>
                <img src={nowPlaying.thumbnail || '/placeholder.jpg'} alt="" />
                <span class="queue-track-copy"><strong>{nowPlaying.title}</strong><small>{nowPlaying.artist}</small></span>
                <span class="queue-time">{formatTime(playbackDuration)}</span>
              </button>
            {/if}
            {#each queueItems as item, index (item.track_id || item.id || index)}
              <button class="queue-track" on:click={() => handlePlayTrack(item)}>
                <span class="queue-number">{index + 1}</span>
                <img src={item.thumbnail_url || item.thumbnail || '/placeholder.jpg'} alt="" />
                <span class="queue-track-copy"><strong>{item.title || 'Untitled track'}</strong><small>{item.artist || item.subtitle || 'YouTube Music'}</small></span>
                <span class="queue-time">{formatTime(Number(item.duration_seconds || item.duration || 0))}</span>
              </button>
            {/each}
            {#if queueItems.length === 0 && !nowPlaying}
              <div class="panel-empty"><ListMusic size={25} /><p>Your queue is empty.</p><button on:click={() => navigate('search')}>Find something to play</button></div>
            {/if}
          </div>
        {:else}
          <div class="lyrics-content">
            {#if nowPlaying}
              <div class="lyrics-track-art"><img src={nowPlaying.thumbnail || '/placeholder.jpg'} alt="" /></div>
              <h2>{nowPlaying.title}</h2>
              <p>{nowPlaying.artist}</p>
              {#if lyricsLoading}
                <div class="lyrics-placeholder"><span class="lyrics-spinner"></span><span>Finding lyrics…</span></div>
              {:else if lyricsData?.lines?.length}
                <div class="lyrics-lines" aria-label="Synchronized lyrics">
                  {#each lyricsData.lines as line, index (index)}
                    <button class="lyric-line" class:current={index === activeLyricIndex} data-lyric-index={index} on:click={() => seekToLyric(Number(line.time_ms || 0))}>{line.text}</button>
                  {/each}
                </div>
                {#if lyricsData.attribution}<small class="lyrics-attribution">{lyricsData.attribution}</small>{/if}
              {:else if lyricsData?.plain_text}
                <div class="lyrics-plain">{#each String(lyricsData.plain_text).split(/\r?\n/) as line, index (index)}<p>{line || '\u00a0'}</p>{/each}</div>
                {#if lyricsData.attribution}<small class="lyrics-attribution">{lyricsData.attribution}</small>{/if}
              {:else}
                <div class="lyrics-placeholder"><Mic2 size={22} /><span>{lyricsError || 'Lyrics will appear here when available.'}</span><button on:click={() => loadLyrics(true)}>Try again</button></div>
              {/if}
            {:else}
              <div class="panel-empty"><Mic2 size={25} /><p>Play a track to see lyrics here.</p></div>
            {/if}
          </div>
        {/if}
      </aside>
    {/if}
  </div>

  {#if showNowPlaying && nowPlaying}
    <section class="immersive-player" aria-label="Expanded now playing" in:fade={{ duration: 240 }}>
      <div class="immersive-backdrop" style={`background-image: url("${nowPlaying.thumbnail || ''}")`}></div>
      <div class="immersive-glass-wash"></div>
      <header class="immersive-header">
        <button class="immersive-close" on:click={closeImmersivePlayer} aria-label="Close expanded player" title="Back to NOVA"><ChevronDown size={21} /></button>
        <div class="immersive-header-title"><span>NOW PLAYING</span><strong>NOVA MUSIC</strong></div>
        <span aria-hidden="true"></span>
      </header>

      <div class="immersive-content">
        <section class="immersive-art-column" aria-label="Track artwork and details">
          <div class="immersive-overline"><span><i></i> LIVE SESSION</span><span>{formatTime(playbackPosition)} <b>/</b> {formatTime(playbackDuration)}</span></div>
          <div class="immersive-artwork"><img src={nowPlaying.thumbnail || '/placeholder.jpg'} alt={`${nowPlaying.title || 'Track'} artwork`} /></div>
          <div class="immersive-track-info">
            <div><span class="immersive-label">PLAYING FROM YOUR MUSIC</span><h1>{nowPlaying.title || 'Unknown track'}</h1><p>{nowPlaying.artist || 'YouTube Music'}{nowPlaying.album ? ` · ${nowPlaying.album}` : ''}</p></div>
            <button class="immersive-like" class:liked={trackIsLiked(nowPlaying)} on:click={toggleLikeTrack} aria-label={trackIsLiked(nowPlaying) ? 'Remove from Liked Music' : 'Add to Liked Music'} title={trackIsLiked(nowPlaying) ? 'Remove from Liked Music' : 'Add to Liked Music'}><Heart size={22} fill={trackIsLiked(nowPlaying) ? 'currentColor' : 'none'} /></button>
          </div>
          <div class="immersive-art-actions">
            <button class="immersive-secondary-action" on:click={downloadCurrentTrack}><Download size={16} /> Download</button>
            <button class="immersive-secondary-action" on:click={async () => { await closeImmersivePlayer(); sidePanel = 'queue'; await loadQueue(); }}><ListMusic size={16} /> Up Next <span>{queueItems.length}</span></button>
            <TrackActions track={nowPlaying} liked={trackIsLiked(nowPlaying)} playlists={libraryCollections.playlists} onAction={handleTrackAction} />
          </div>
        </section>

        <section class="immersive-lyrics" aria-label="Synchronized lyrics">
          <div class="immersive-lyrics-header"><div><span class="immersive-label">SING ALONG</span><h2>Lyrics</h2></div><span class="lyrics-source">{lyricsData?.source || 'NOVA lyrics'}</span></div>
          {#if lyricsLoading}
            <div class="immersive-lyrics-empty"><span class="lyrics-spinner"></span><strong>Finding lyrics</strong><p>Checking YouTube Music and synced lyrics sources…</p></div>
          {:else if lyricsData?.lines?.length}
            <div class="immersive-lyric-scroll">
              {#each lyricsData.lines as line, index (index)}
                <button class="immersive-lyric" class:active={index === activeLyricIndex} class:nearby={Math.abs(index - activeLyricIndex) === 1} data-lyric-index={index} on:click={() => seekToLyric(Number(line.time_ms || 0))}>{line.text}</button>
              {/each}
            </div>
            {#if lyricsData.attribution}<small class="immersive-attribution">{lyricsData.attribution}</small>{/if}
          {:else if lyricsData?.plain_text}
            <div class="immersive-lyric-scroll immersive-plain-lyrics">{#each String(lyricsData.plain_text).split(/\r?\n/) as line, index (index)}<p>{line || '\u00a0'}</p>{/each}</div>
            {#if lyricsData.attribution}<small class="immersive-attribution">{lyricsData.attribution}</small>{/if}
          {:else}
            <div class="immersive-lyrics-empty"><Mic2 size={28} /><strong>{lyricsError || 'No lyrics found for this track'}</strong><p>Try again, or keep listening while NOVA checks another source.</p><button class="lyrics-retry" on:click={() => loadLyrics(true)}>Try again</button></div>
          {/if}
        </section>
      </div>

      <footer class="immersive-controls">
        <div class="immersive-timeline">
          <span>{formatTime(playbackPosition)}</span>
          <input type="range" min="0" max={playbackDuration || 180} step="0.5" value={playbackPosition} on:input={previewSeek} on:change={commitSeek} aria-label="Seek through the current song" style={`--progress:${playbackDuration ? Math.min(100, playbackPosition / playbackDuration * 100) : 0}%`} />
          <span>{formatTime(playbackDuration)}</span>
        </div>
        <div class="immersive-transport-row">
          <div class="immersive-control-spacer"><button class="immersive-like compact" class:liked={trackIsLiked(nowPlaying)} on:click={toggleLikeTrack} aria-label="Toggle liked song"><Heart size={19} fill={trackIsLiked(nowPlaying) ? 'currentColor' : 'none'} /></button></div>
          <div class="immersive-transport">
            <button class="transport-button" on:click={prevTrack} aria-label="Previous track"><SkipBack size={19} fill="currentColor" /></button>
            <button class="transport-play immersive-play" on:click={togglePlay} aria-label={isPlaying ? 'Pause' : 'Play'} disabled={isLoadingTrack}>{#if isLoadingTrack}<span class="loader-dot"></span>{:else if isPlaying}<Pause size={23} fill="currentColor" />{:else}<Play size={23} fill="currentColor" />{/if}</button>
            <button class="transport-button" on:click={skipTrack} aria-label="Next track"><SkipForward size={19} fill="currentColor" /></button>
          </div>
          <div class="immersive-volume"><button class="immersive-like compact" on:click={toggleMute} aria-label="Mute or unmute">{#if currentVolume === 0}<VolumeX size={18} />{:else}<Volume2 size={18} />{/if}</button><input type="range" min="0" max="100" value={currentVolume} on:input={changeVolume} aria-label="Volume" style={`--progress:${currentVolume}%`} /></div>
        </div>
      </footer>
    </section>
  {/if}

  <footer class="player-bar">
    <div class="player-track">
      {#if nowPlaying}
        <button class="player-art-button" on:click={openImmersivePlayer} aria-label="Open now playing">
          <img src={nowPlaying.thumbnail || '/placeholder.jpg'} alt="" />
        </button>
        <div class="player-track-copy">
          <strong title={nowPlaying.title}>{nowPlaying.title || 'Unknown track'}</strong>
          <span title={nowPlaying.artist}>{playbackError || nowPlaying.artist || 'YouTube Music'}</span>
        </div>
        <button class="player-small-action like-track-button" class:liked={trackIsLiked(nowPlaying)} on:click={toggleLikeTrack} aria-label={trackIsLiked(nowPlaying) ? 'Remove from Liked Music' : 'Add to Liked Music'} title={trackIsLiked(nowPlaying) ? 'Remove from Liked Music' : 'Add to Liked Music'}>
          <Heart size={17} fill={trackIsLiked(nowPlaying) ? 'currentColor' : 'none'} />
        </button>
        <TrackActions track={nowPlaying} liked={trackIsLiked(nowPlaying)} playlists={libraryCollections.playlists} onAction={handleTrackAction} />
      {:else}
        <div class="player-art-empty"><Music2 size={19} /></div>
        <div class="player-track-copy"><strong>NOVA Music</strong><span>Choose a song to get started</span></div>
      {/if}
    </div>

    <div class="player-center">
      <div class="transport-controls">
        <button class="transport-button" on:click={prevTrack} aria-label="Previous track" title="Previous track"><SkipBack size={17} fill="currentColor" /></button>
        <button class="transport-play" on:click={togglePlay} aria-label={isPlaying ? 'Pause' : 'Play'} title={isPlaying ? 'Pause' : 'Play'} disabled={!nowPlaying || isLoadingTrack}>
          {#if isLoadingTrack}<span class="loader-dot"></span>{:else if isPlaying}<Pause size={19} fill="currentColor" />{:else}<Play size={19} fill="currentColor" />{/if}
        </button>
        <button class="transport-button" on:click={skipTrack} aria-label="Next track" title="Next track"><SkipForward size={17} fill="currentColor" /></button>
      </div>
      <div class="player-progress">
        <span>{formatTime(playbackPosition)}</span>
        <input
          type="range"
          min="0"
          max={playbackDuration || 180}
          step="0.5"
          value={playbackPosition}
          on:input={previewSeek}
          on:change={commitSeek}
          aria-label="Playback position"
          style={`--progress:${playbackDuration ? Math.min(100, playbackPosition / playbackDuration * 100) : 0}%`}
        />
        <span>{formatTime(playbackDuration)}</span>
      </div>
    </div>

    <div class="player-extras">
      <button class="player-small-action" class:selected={sidePanel === 'now-playing'} on:click={() => toggleSidePanel('now-playing')} aria-label="Now playing" title="Now playing"><PanelRight size={17} /></button>
      <button class="player-small-action" on:click={openImmersivePlayer} aria-label="Open expanded player" title="Expanded player"><Maximize2 size={16} /></button>
      <button class="player-small-action" class:selected={sidePanel === 'lyrics'} on:click={() => toggleSidePanel('lyrics')} aria-label="Lyrics" title="Lyrics"><Mic2 size={17} /></button>
      <button class="player-small-action" class:selected={sidePanel === 'queue'} on:click={() => toggleSidePanel('queue')} aria-label="Queue" title="Queue"><ListMusic size={17} /></button>
      <div class="volume-control">
        <button class="player-small-action" on:click={toggleMute} aria-label="Mute or unmute" title="Mute or unmute">
          {#if currentVolume === 0}<VolumeX size={17} />{:else}<Volume2 size={17} />{/if}
        </button>
        <input type="range" min="0" max="100" value={currentVolume} on:input={changeVolume} aria-label="Volume" style={`--progress:${currentVolume}%`} />
      </div>
      <button class="player-small-action more-button" on:click={openSettings} aria-label="More options" title="Settings"><MoreHorizontal size={18} /></button>
    </div>
  </footer>

  {#if appNotice}
    <div class="app-notice" role="status">{appNotice}</div>
  {/if}

  {#if settingsOpen}
    <div class="settings-scrim" role="presentation" on:click|self={() => settingsOpen = false} on:keydown={(event) => { if (event.key === 'Escape') settingsOpen = false; }} tabindex="-1">
      <section class="settings-dialog" role="dialog" aria-modal="true" aria-label="NOVA Music settings">
        <Settings onClose={() => settingsOpen = false} />
      </section>
    </div>
  {/if}
</div>

<style>
  :global(*) { box-sizing: border-box; }
  :global(html), :global(body), :global(#svelte) { width: 100%; height: 100%; margin: 0; background-color: #080909; }
  :global(body) {
    overflow: hidden;
    background: #050606;
    color: #f5f5f5;
    font-family: "DM Sans", Inter, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
    -webkit-font-smoothing: antialiased;
  }
  :global(button), :global(input), :global(select), :global(textarea) { font: inherit; }

  .app-shell {
    --accent: rgb(var(--cover-rgb));
    --accent-soft: rgba(var(--cover-rgb), .16);
    --glass-fill: rgba(26, 29, 29, .76);
    --glass-edge: rgba(255, 255, 255, .11);
    --surface: #121313;
    --surface-2: #181a1a;
    --line: rgba(255,255,255,.08);
    width: 100vw;
    height: 100vh;
    min-width: 840px;
    min-height: 620px;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    color: #f6f6f6;
    background: radial-gradient(ellipse at 30% -15%, rgba(var(--cover-rgb), .16), transparent 45%), radial-gradient(ellipse at 90% 105%, rgba(var(--cover-deep-rgb), .38), transparent 48%), #080909;
  }
  :global(.app-shell.theme-cyber), :global(.app-shell.theme-deepspace) { --accent: rgb(var(--cover-rgb)); --accent-soft: rgba(var(--cover-rgb), .16); }

  .topbar {
    position: relative;
    z-index: 30;
    height: 56px;
    flex: 0 0 56px;
    display: flex;
    align-items: center;
    gap: 22px;
    padding: 0 18px 0 9px;
    background: linear-gradient(110deg, rgba(34, 38, 37, .78), rgba(11, 12, 12, .82) 42%, rgba(17, 19, 19, .74));
    border-bottom: 1px solid var(--glass-edge);
    box-shadow: inset 0 1px 0 rgba(255,255,255,.09), 0 12px 28px rgba(0,0,0,.13);
    -webkit-backdrop-filter: blur(26px) saturate(1.3);
    backdrop-filter: blur(26px) saturate(1.3);
  }
  .traffic-light-clearance { width: 66px; height: 100%; flex: 0 0 66px; }
  .topbar-history, .topbar-actions { display: flex; align-items: center; gap: 8px; }
  .chrome-button, .icon-button, .player-small-action, .transport-button {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    color: #b7b9b9;
    background: transparent;
    border: 0;
    border-radius: 50%;
    cursor: pointer;
    transition: color .16s ease, background .16s ease, transform .16s ease;
  }
  .chrome-button { width: 32px; height: 32px; }
  .chrome-button:hover, .icon-button:hover, .player-small-action:hover, .transport-button:hover { color: #fff; background: rgba(255,255,255,.09); }
  .chrome-button:disabled { opacity: .35; cursor: default; }
  .refresh-button { margin-left: 5px; }
  .global-search-wrap { position: relative; width: min(460px, 42vw); margin: 0 auto; }
  .global-search {
    height: 38px;
    width: 100%;
    display: flex;
    align-items: center;
    gap: 11px;
    padding: 0 14px;
    border: 1px solid transparent;
    border-radius: 22px;
    background: linear-gradient(145deg, rgba(80, 84, 83, .35), rgba(30, 32, 32, .73));
    color: #bfc1c1;
    box-shadow: inset 0 1px 0 rgba(255,255,255,.1), inset 0 0 0 1px rgba(255,255,255,.025);
    -webkit-backdrop-filter: blur(18px) saturate(1.25);
    backdrop-filter: blur(18px) saturate(1.25);
    transition: border-color .16s ease, background .16s ease, box-shadow .16s ease;
  }
  .global-search:hover, .global-search.focused { background: #202222; border-color: rgba(255,255,255,.2); box-shadow: 0 0 0 3px rgba(var(--cover-rgb),.08), inset 0 1px 0 rgba(255,255,255,.1); }
  .global-search input { min-width: 0; flex: 1; color: #f0f2f2; background: transparent; border: 0; outline: 0; font-size: 13px; }
  .global-search input::placeholder { color: #bfc1c1; opacity: 1; }
  .global-search kbd { color: #8c8f8f; font-size: 10px; border: 1px solid #454747; border-radius: 4px; padding: 2px 5px; }
  .global-search-popover { position: absolute; z-index: 100; top: calc(100% + 10px); left: 50%; width: min(580px, calc(100vw - 32px)); max-height: min(72vh, 650px); overflow: auto; padding: 11px; color: #f2f3f3; background: #111313; border: 1px solid rgba(255,255,255,.13); border-radius: 14px; box-shadow: 0 24px 70px rgba(0,0,0,.72), inset 0 1px rgba(255,255,255,.05); transform: translateX(-50%); }
  .search-popover-heading { display: flex; align-items: center; justify-content: space-between; padding: 5px 7px 10px; color: #858b8b; font-size: 9px; font-weight: 800; letter-spacing: .12em; }
  .search-popover-heading button { display: grid; place-items: center; width: 26px; height: 26px; color: #9da2a2; background: transparent; border: 0; border-radius: 50%; cursor: pointer; }
  .search-popover-heading button:hover { color: #fff; background: #252828; }
  .search-popover-section { display: flex; flex-direction: column; gap: 2px; padding: 4px 0 7px; }
  .search-popover-label { padding: 5px 8px; color: #777e7e; font-size: 9px; font-weight: 800; letter-spacing: .1em; }
  .search-result-row { position: relative; display: flex; align-items: center; border-radius: 8px; }
  .search-result-row:hover { background: #1b1e1e; }
  .search-result-main { min-width: 0; flex: 1; display: flex; align-items: center; gap: 11px; padding: 7px 10px; color: #e9ebeb; text-align: left; background: transparent; border: 0; border-radius: 8px; cursor: pointer; }
  .search-result-main > img, .search-collection-result > img { width: 42px; height: 42px; flex: 0 0 42px; object-fit: cover; background: #202323; border-radius: 6px; }
  .search-result-copy, .search-collection-result > span { min-width: 0; flex: 1; display: flex; flex-direction: column; gap: 4px; }
  .search-result-copy strong, .search-collection-result strong { overflow: hidden; color: #f0f2f2; font-size: 12px; font-weight: 650; text-overflow: ellipsis; white-space: nowrap; }
  .search-result-copy small, .search-collection-result small { overflow: hidden; color: #949a9a; font-size: 10px; text-overflow: ellipsis; white-space: nowrap; }
  .search-result-main > :global(svg) { margin: 0 5px; color: var(--accent); opacity: 0; transition: opacity .15s ease; }
  .search-result-row:hover .search-result-main > :global(svg) { opacity: 1; }
  .search-collection-result { min-width: 0; display: flex; align-items: center; gap: 10px; padding: 7px 9px; color: #eee; text-align: left; background: transparent; border: 0; border-radius: 8px; cursor: pointer; }
  .search-collection-result:hover { background: #1b1e1e; }
  .search-collection-result > img { width: 36px; height: 36px; flex-basis: 36px; border-radius: 50%; }
  .search-popover-state { min-height: 58px; display: flex; align-items: center; gap: 10px; padding: 12px; color: #9ca2a2; font-size: 12px; }
  .search-popover-state.error { color: #ffb6b6; }
  .search-popover-spinner { width: 15px; height: 15px; border: 2px solid #454a4a; border-top-color: #f0f2f2; border-radius: 50%; animation: search-spin .7s linear infinite; }
  @keyframes search-spin { to { transform: rotate(360deg); } }
  .search-view-all { width: 100%; min-height: 38px; display: flex; align-items: center; justify-content: center; gap: 6px; margin-top: 5px; color: #e6e8e8; background: #1b1e1e; border: 1px solid rgba(255,255,255,.07); border-radius: 9px; font-size: 11px; font-weight: 650; cursor: pointer; }
  .search-view-all:hover { background: #252929; }
  .topbar-actions { min-width: 225px; justify-content: flex-end; }
  .library-toggle.selected { color: var(--accent); }
  .profile-button {
    max-width: 220px;
    height: 34px;
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 3px 11px 3px 4px;
    color: #f5f5f5;
    background: #1a1b1b;
    border: 1px solid rgba(255,255,255,.08);
    border-radius: 20px;
    cursor: pointer;
  }
  .profile-button:hover { background: #282a2a; }
  .profile-avatar { width: 26px; height: 26px; display: grid; place-items: center; color: #111; background: var(--accent); border-radius: 50%; font-size: 12px; font-weight: 800; }
  .profile-label { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; font-size: 12px; font-weight: 650; }
  .settings-top-button { margin-left: 3px; }

  .workarea { min-height: 0; flex: 1; display: flex; overflow: hidden; gap: 0; }
  .nav-rail {
    width: 68px;
    flex: 0 0 68px;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8px;
    padding: 14px 9px 12px;
    background: linear-gradient(180deg, rgba(31, 35, 34, .66), rgba(10, 11, 11, .88));
    border-right: 1px solid rgba(255,255,255,.075);
    box-shadow: inset -1px 0 0 rgba(255,255,255,.025);
    -webkit-backdrop-filter: blur(24px) saturate(1.35);
    backdrop-filter: blur(24px) saturate(1.35);
  }
  .brand-mark, .rail-button {
    width: 44px;
    height: 44px;
    flex: 0 0 44px;
    display: grid;
    place-items: center;
    color: #a7aaaa;
    background: transparent;
    border: 0;
    border-radius: 14px;
    cursor: pointer;
    transition: color .16s ease, background .16s ease;
  }
  .brand-mark { color: #111; background: var(--accent); margin-bottom: 2px; }
  .brand-mark:hover { filter: brightness(1.08); }
  .rail-separator { width: 26px; height: 1px; margin: 2px 0 7px; background: rgba(255,255,255,.11); }
  .rail-button:hover { color: #fff; background: #1a1c1c; }
  .rail-button.selected { color: #fff; background: #202222; box-shadow: inset 3px 0 0 var(--accent); }
  .rail-spacer { flex: 1; }
  .rail-art-button { width: 42px; height: 42px; padding: 0; overflow: hidden; background: transparent; border: 0; border-radius: 9px; cursor: pointer; }
  .rail-art-button img { display: block; width: 100%; height: 100%; object-fit: cover; }
  .rail-settings { margin-top: 3px; }

  .library-sidebar {
    width: 255px;
    flex: 0 0 255px;
    min-width: 0;
    display: flex;
    flex-direction: column;
    padding: 17px 12px 12px;
    background: linear-gradient(160deg, rgba(34, 37, 36, .78), rgba(14, 15, 15, .84) 40%, rgba(19, 21, 20, .75));
    border-right: 1px solid var(--glass-edge);
    box-shadow: inset 1px 0 0 rgba(255,255,255,.035), inset -1px 0 0 rgba(255,255,255,.03), 10px 0 30px rgba(0,0,0,.15);
    -webkit-backdrop-filter: blur(30px) saturate(1.35);
    backdrop-filter: blur(30px) saturate(1.35);
    overflow: hidden;
  }
  .sidebar-heading { display: flex; align-items: center; gap: 6px; padding: 0 5px 12px; }
  .library-title { min-width: 0; flex: 1; display: flex; gap: 10px; align-items: center; color: #b7baba; background: none; border: 0; font-size: 14px; font-weight: 700; cursor: pointer; text-align: left; }
  .library-title:hover { color: #fff; }
  .icon-button { width: 30px; height: 30px; }
  .library-filters { display: flex; flex-wrap: wrap; gap: 6px; padding: 3px 4px 12px; }
  .filter-chip { padding: 6px 10px; color: #d1d3d3; background: #242626; border: 0; border-radius: 15px; font-size: 11px; cursor: pointer; }
  .filter-chip:hover, .filter-chip.active { color: #101212; background: #f4f5f5; }
  .library-tools { display: flex; align-items: center; justify-content: space-between; padding: 0 2px 9px; color: #8f9393; }
  .sort-button { display: inline-flex; align-items: center; gap: 7px; color: inherit; background: none; border: 0; font-size: 11px; cursor: pointer; }
  .library-list { min-height: 0; flex: 1; overflow: auto; scrollbar-width: thin; scrollbar-color: #414444 transparent; }
  .library-entry { width: 100%; display: flex; align-items: center; gap: 10px; padding: 7px 6px; text-align: left; color: #e4e6e6; background: transparent; border: 0; border-radius: 7px; cursor: pointer; }
  .library-entry:hover { background: #202222; }
  .library-art { width: 42px; height: 42px; flex: 0 0 42px; display: grid; place-items: center; overflow: hidden; color: #b9bcbc; background: #282a2a; border-radius: 5px; }
  .library-art img { width: 100%; height: 100%; object-fit: cover; }
  .liked-art { color: white; background: linear-gradient(135deg, #f7838c, #ad5fce); }
  .library-copy, .queue-track-copy { min-width: 0; flex: 1; display: flex; flex-direction: column; gap: 3px; }
  .library-copy strong, .queue-track-copy strong { overflow: hidden; color: #e7e9e9; font-size: 12px; font-weight: 650; text-overflow: ellipsis; white-space: nowrap; }
  .library-copy small, .queue-track-copy small { overflow: hidden; color: #929696; font-size: 10px; text-overflow: ellipsis; white-space: nowrap; }
  .sidebar-empty { padding: 16px 8px; color: #858989; font-size: 12px; line-height: 1.5; }

  .main-content { min-width: 0; min-height: 0; flex: 1; overflow: auto; background: radial-gradient(ellipse at 48% -10%, rgba(var(--cover-rgb),.14), transparent 45%), radial-gradient(ellipse at 90% 100%, rgba(var(--cover-deep-rgb),.26), transparent 48%), rgba(8,9,9,.96); scrollbar-width: thin; scrollbar-color: #3d4040 transparent; }
  .route-layer { min-height: 100%; transform-origin: 50% 30%; }
  .main-content.panel-open { border-right: 1px solid rgba(255,255,255,.07); }
  .right-panel { width: 355px; flex: 0 0 355px; min-width: 0; display: flex; flex-direction: column; overflow: hidden; background: linear-gradient(155deg, rgba(36,40,39,.82), rgba(14,15,15,.83) 42%, rgba(21,23,22,.76)); border-left: 1px solid var(--glass-edge); box-shadow: inset 1px 0 0 rgba(255,255,255,.05), -12px 0 30px rgba(0,0,0,.18); -webkit-backdrop-filter: blur(30px) saturate(1.4); backdrop-filter: blur(30px) saturate(1.4); }
  .right-panel-header { height: 52px; flex: 0 0 52px; display: flex; justify-content: space-between; align-items: center; padding: 0 15px; border-bottom: 1px solid rgba(255,255,255,.075); box-shadow: inset 0 1px 0 rgba(255,255,255,.045); }
  .panel-tabs { display: flex; align-items: center; gap: 22px; height: 100%; }
  .panel-tabs button { position: relative; height: 100%; padding: 0; color: #939797; background: none; border: 0; font-size: 13px; font-weight: 650; cursor: pointer; }
  .panel-tabs button:hover, .panel-tabs button.active { color: #fff; }
  .panel-tabs button.active::after { position: absolute; bottom: 0; left: 0; right: 0; height: 2px; background: #fff; border-radius: 2px; content: ''; }
  .queue-content { min-height: 0; flex: 1; overflow: auto; padding: 15px 10px; }
  .queue-context { display: flex; flex-direction: column; gap: 5px; margin: 1px 7px 16px; }
  .queue-context span { color: #858989; font-size: 9px; font-weight: 750; letter-spacing: .1em; }
  .queue-context strong { color: #f1f3f3; font-size: 13px; }
  .queue-context small { color: #898d8d; font-size: 11px; }
  .queue-track { width: 100%; display: flex; align-items: center; gap: 10px; padding: 8px 7px; color: #e7e9e9; text-align: left; background: transparent; border: 0; border-radius: 7px; cursor: pointer; }
  .queue-track:hover { background: #202222; }
  .queue-track.current { background: rgba(255,255,255,.045); }
  .queue-track.current strong { color: var(--accent); }
  .queue-track img { width: 42px; height: 42px; flex: 0 0 42px; object-fit: cover; border-radius: 5px; }
  .queue-number { width: 18px; flex: 0 0 18px; text-align: center; color: #777c7c; font-size: 10px; }
  .queue-time { color: #8b9090; font-size: 10px; font-variant-numeric: tabular-nums; }
  .playing-bars { height: 16px; display: flex; align-items: center; justify-content: center; gap: 2px; }
  .playing-bars i { width: 2px; height: 8px; display: block; background: var(--accent); animation: bars .7s ease-in-out infinite alternate; }
  .playing-bars i:nth-child(2) { height: 13px; animation-delay: .2s; }
  .playing-bars i:nth-child(3) { height: 5px; animation-delay: .35s; }
  @keyframes bars { to { transform: scaleY(.45); } }
  .panel-empty { flex: 1; min-height: 220px; display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 12px; color: #858989; text-align: center; font-size: 12px; }
  .panel-empty p { margin: 0; }
  .panel-empty button { padding: 8px 14px; color: #111; background: var(--accent); border: 0; border-radius: 18px; font-size: 11px; font-weight: 700; cursor: pointer; }
  .lyrics-content { min-height: 0; flex: 1; display: flex; flex-direction: column; align-items: center; overflow: hidden; padding: 28px 22px; text-align: center; }
  .lyrics-track-art { width: 116px; height: 116px; overflow: hidden; border-radius: 7px; box-shadow: 0 9px 30px #0008; }
  .lyrics-track-art img { width: 100%; height: 100%; object-fit: cover; }
  .lyrics-content h2 { margin: 18px 0 4px; font-size: 15px; }
  .lyrics-content > p { margin: 0; color: #969a9a; font-size: 12px; }
  .lyrics-placeholder { flex: 1; display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 12px; color: #a1a5a5; max-width: 230px; font-size: 12px; line-height: 1.5; }
  .lyrics-placeholder button, .lyrics-retry { padding: 8px 14px; color: #171113; background: var(--accent); border: 0; border-radius: 18px; font-size: 11px; font-weight: 700; cursor: pointer; }
  .lyrics-spinner { width: 20px; height: 20px; border: 2px solid rgba(255,255,255,.18); border-top-color: var(--accent); border-radius: 50%; animation: spin .7s linear infinite; }
  .lyrics-lines { width: 100%; min-height: 0; flex: 1; display: flex; flex-direction: column; align-items: stretch; gap: 10px; overflow: auto; margin-top: 18px; padding: 8px 4px 22px; scrollbar-width: thin; scrollbar-color: #454a48 transparent; }
  .lyric-line { width: 100%; padding: 7px 8px; color: #858b89; background: transparent; border: 0; border-radius: 6px; font-size: 13px; line-height: 1.55; text-align: center; cursor: pointer; transition: color .2s ease, background .2s ease; }
  .lyric-line:hover, .lyric-line.current { color: #fff; background: rgba(255,255,255,.055); }
  .lyric-line.current { color: var(--accent); font-weight: 700; }
  .lyrics-plain { width: 100%; min-height: 0; flex: 1; overflow: auto; margin-top: 18px; color: #d4d7d6; font-size: 13px; line-height: 1.8; white-space: pre-wrap; }
  .lyrics-plain p { margin: 0 0 8px; }
  .lyrics-attribution { padding: 12px 0 0; color: #777e7b; font-size: 10px; }

  .now-playing-panel { min-height: 0; flex: 1; overflow: auto; padding: 20px 16px 28px; scrollbar-width: thin; scrollbar-color: #454a48 transparent; }
  .panel-now-art { width: 100%; aspect-ratio: 1; overflow: hidden; border-radius: 8px; background: rgba(255,255,255,.055); box-shadow: 0 20px 48px rgba(0,0,0,.38), inset 0 1px 0 rgba(255,255,255,.18); }
  .panel-now-art img { width: 100%; height: 100%; display: block; object-fit: cover; }
  .panel-now-track { display: flex; align-items: center; gap: 10px; padding: 16px 1px 20px; }
  .panel-now-copy { min-width: 0; flex: 1; display: flex; flex-direction: column; gap: 5px; }
  .panel-now-copy strong { overflow: hidden; color: #fff; font-size: 17px; letter-spacing: -.025em; text-overflow: ellipsis; white-space: nowrap; }
  .panel-now-copy button { max-width: 100%; overflow: hidden; padding: 0; color: #b5bbb7; background: none; border: 0; text-align: left; text-overflow: ellipsis; white-space: nowrap; font-size: 12px; cursor: pointer; }
  .panel-now-copy button:hover { color: #fff; text-decoration: underline; }
  .panel-now-copy button:disabled { color: #b5bbb7; cursor: default; text-decoration: none; }
  .like-track-button { color: #adb4b0; }
  .like-track-button.liked { color: var(--accent); }
  .panel-album-card, .panel-artist-card { overflow: hidden; margin: 0 0 15px; background: linear-gradient(140deg, rgba(255,255,255,.09), rgba(255,255,255,.045)); border: 1px solid rgba(255,255,255,.075); border-radius: 9px; box-shadow: inset 0 1px 0 rgba(255,255,255,.085), 0 12px 30px rgba(0,0,0,.13); }
  .panel-album-heading { display: flex; justify-content: space-between; align-items: center; padding: 13px 13px 10px; color: #f4f5f4; font-size: 12px; }
  .panel-album-link { width: 100%; display: flex; align-items: center; gap: 10px; padding: 3px 12px 13px; color: #fff; background: transparent; border: 0; text-align: left; cursor: pointer; }
  .panel-album-link:disabled { cursor: default; }
  .panel-album-link img { width: 44px; height: 44px; object-fit: cover; border-radius: 5px; }
  .panel-album-link span { min-width: 0; display: flex; flex-direction: column; gap: 4px; }
  .panel-album-link strong, .panel-album-link small { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .panel-album-link strong { font-size: 12px; }
  .panel-album-link small { color: #a4aaa6; font-size: 10px; }
  .panel-artist-card { position: relative; min-height: 148px; display: flex; align-items: end; padding: 14px; background: linear-gradient(155deg, rgba(109,55,78,.76), rgba(23,21,25,.86)); }
  .panel-artist-art { position: absolute; top: 13px; right: 13px; width: 54px; height: 54px; display: grid; place-items: center; color: rgba(255,255,255,.78); background: rgba(255,255,255,.12); border: 1px solid rgba(255,255,255,.14); border-radius: 50%; box-shadow: inset 0 1px 0 rgba(255,255,255,.2); }
  .panel-artist-info { position: relative; z-index: 1; display: flex; flex-direction: column; gap: 5px; }
  .panel-artist-info strong { font-size: 12px; }
  .panel-artist-info span { color: #d0d6d2; font-size: 11px; }

  .immersive-player { position: fixed; z-index: 15; inset: 0; display: flex; flex-direction: column; overflow: hidden; color: #fff; background: linear-gradient(128deg, color-mix(in srgb, rgb(var(--cover-rgb)) 23%, #111313), #0b0c0c 58%, color-mix(in srgb, rgb(var(--cover-deep-rgb)) 42%, #101111)); isolation: isolate; }
  .immersive-backdrop { position: absolute; z-index: 0; inset: -70px; background-position: center; background-size: cover; filter: blur(70px) saturate(.82); opacity: .18; transform: scale(1.12); pointer-events: none; }
  .immersive-glass-wash { position: absolute; z-index: 1; inset: 0; background: linear-gradient(112deg, rgba(7,8,8,.82), rgba(10,12,12,.71) 64%, rgba(8,9,9,.82)); pointer-events: none; transition: background 700ms ease; }
  .immersive-header, .immersive-content, .immersive-controls { position: relative; z-index: 2; }
  .immersive-header { height: 68px; flex: 0 0 68px; display: grid; grid-template-columns: 1fr auto 1fr; align-items: center; padding: 4px clamp(18px, 3vw, 48px) 0; }
  .immersive-close { width: 40px; height: 40px; display: grid; place-items: center; justify-self: start; color: rgba(255,255,255,.78); background: rgba(255,255,255,.075); border: 1px solid rgba(255,255,255,.1); border-radius: 50%; cursor: pointer; backdrop-filter: blur(18px); transition: background .18s ease, transform .18s ease; }
  .immersive-close:last-child { justify-self: end; }
  .immersive-close:hover { color: #fff; background: rgba(255,255,255,.16); transform: translateY(-1px); }
  .immersive-header-title { display: flex; flex-direction: column; align-items: center; gap: 4px; }
  .immersive-header-title span, .immersive-label { color: rgba(237,231,234,.6); font-size: 9px; font-weight: 800; letter-spacing: .18em; }
  .immersive-header-title strong { color: rgba(255,255,255,.86); font-size: 11px; font-weight: 750; letter-spacing: .12em; }
  .immersive-content { width: min(1320px, calc(100% - 96px)); min-height: 0; flex: 1; display: grid; grid-template-columns: minmax(330px, .95fr) minmax(350px, 1.05fr); align-items: center; gap: clamp(42px, 6vw, 96px); margin: 0 auto; padding: 18px 0 24px; }
  .immersive-art-column { min-width: 0; display: flex; flex-direction: column; align-items: center; }
  .immersive-overline { width: min(100%, 510px); display: flex; align-items: center; justify-content: space-between; padding: 0 2px 12px; color: rgba(241,237,239,.7); font-size: 10px; font-variant-numeric: tabular-nums; }
  .immersive-overline > span:first-child { display: flex; align-items: center; gap: 7px; font-size: 9px; font-weight: 750; letter-spacing: .12em; }
  .immersive-overline i { width: 6px; height: 6px; background: var(--accent); border-radius: 50%; box-shadow: 0 0 12px rgba(var(--cover-rgb),.8); }
  .immersive-overline b { margin: 0 3px; color: rgba(255,255,255,.32); }
  .immersive-artwork { width: min(100%, 500px, 48vh); aspect-ratio: 1; overflow: hidden; border: 1px solid rgba(255,255,255,.14); border-radius: 10px; background: rgba(255,255,255,.06); box-shadow: 0 30px 88px rgba(0,0,0,.43), 0 0 60px rgba(var(--cover-rgb),.16), inset 0 1px 0 rgba(255,255,255,.16); }
  .immersive-artwork img { width: 100%; height: 100%; display: block; object-fit: cover; }
  .immersive-track-info { width: min(100%, 500px); display: flex; align-items: center; gap: 16px; padding: 22px 2px 0; }
  .immersive-track-info > div { min-width: 0; flex: 1; }
  .immersive-track-info h1 { overflow: hidden; margin: 7px 0 5px; color: #fff; font-size: clamp(23px, 2.6vw, 34px); line-height: 1.1; letter-spacing: -.045em; text-overflow: ellipsis; white-space: nowrap; }
  .immersive-track-info p { overflow: hidden; margin: 0; color: rgba(240,235,238,.68); font-size: 14px; text-overflow: ellipsis; white-space: nowrap; }
  .immersive-like { width: 43px; height: 43px; flex: 0 0 43px; display: grid; place-items: center; color: rgba(255,255,255,.78); background: rgba(255,255,255,.08); border: 1px solid rgba(255,255,255,.12); border-radius: 50%; cursor: pointer; transition: color .18s ease, background .18s ease, transform .18s ease; }
  .immersive-like:hover { color: #fff; background: rgba(255,255,255,.16); transform: scale(1.04); }
  .immersive-like.liked { color: var(--accent); }
  .immersive-like.compact { width: 37px; height: 37px; flex-basis: 37px; background: transparent; border-color: transparent; }
  .immersive-art-actions { width: min(100%, 500px); display: flex; gap: 9px; padding-top: 17px; }
  .immersive-secondary-action { display: inline-flex; align-items: center; gap: 8px; padding: 9px 12px; color: rgba(255,255,255,.82); background: rgba(255,255,255,.075); border: 1px solid rgba(255,255,255,.1); border-radius: 20px; font-size: 11px; cursor: pointer; backdrop-filter: blur(18px); }
  .immersive-secondary-action:hover { background: rgba(255,255,255,.14); }
  .immersive-secondary-action span { min-width: 17px; padding: 2px 5px; color: #fff; background: rgba(255,255,255,.13); border-radius: 10px; text-align: center; font-size: 9px; }
  .immersive-lyrics { min-width: 0; height: min(100%, 650px); display: flex; flex-direction: column; overflow: hidden; padding: clamp(22px, 3vw, 40px); background: linear-gradient(145deg, color-mix(in srgb, rgb(var(--cover-rgb)) 17%, #181a1a), #141616 68%, color-mix(in srgb, rgb(var(--cover-deep-rgb)) 24%, #171818)); border: 1px solid rgba(255,255,255,.11); border-radius: 18px; box-shadow: inset 0 1px 0 rgba(255,255,255,.13), 0 24px 65px rgba(0,0,0,.32); }
  .immersive-lyrics-header { display: flex; align-items: center; justify-content: space-between; padding: 0 2px 15px; border-bottom: 1px solid rgba(255,255,255,.1); }
  .immersive-lyrics-header h2 { margin: 5px 0 0; color: #fff; font-size: 24px; letter-spacing: -.035em; }
  .lyrics-source { padding: 6px 9px; color: rgba(255,255,255,.66); background: rgba(255,255,255,.075); border: 1px solid rgba(255,255,255,.08); border-radius: 12px; font-size: 9px; }
  .immersive-lyric-scroll { min-height: 0; flex: 1; overflow: auto; padding: 22px 2px 32px; scrollbar-width: thin; scrollbar-color: rgba(255,255,255,.22) transparent; mask-image: linear-gradient(transparent, #000 8%, #000 92%, transparent); }
  .immersive-lyric { width: 100%; display: block; padding: 11px 0; color: rgba(255,255,255,.24); opacity: .64; background: transparent; border: 0; text-align: left; font-size: clamp(19px, 2vw, 28px); line-height: 1.28; font-weight: 700; letter-spacing: -.03em; cursor: pointer; transition: color .32s ease, transform .32s cubic-bezier(.2,.8,.2,1), opacity .32s ease, filter .32s ease; filter: blur(.25px); }
  .immersive-lyric.nearby { color: rgba(255,255,255,.65); opacity: .92; filter: none; }
  .immersive-lyric:hover { color: rgba(255,255,255,.78); }
  .immersive-lyric.active { color: #fff; opacity: 1; filter: none; transform: translateX(5px) scale(1.015); text-shadow: 0 5px 30px rgba(var(--cover-rgb),.34); }
  .immersive-plain-lyrics { color: rgba(255,255,255,.82); font-size: 18px; line-height: 1.8; white-space: pre-wrap; }
  .immersive-plain-lyrics p { margin: 0 0 8px; }
  .immersive-attribution { padding-top: 10px; color: rgba(255,255,255,.42); font-size: 10px; }
  .immersive-lyrics-empty { min-height: 0; flex: 1; display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 13px; color: rgba(255,255,255,.64); text-align: center; }
  .immersive-lyrics-empty strong { color: #fff; font-size: 15px; }
  .immersive-lyrics-empty p { max-width: 260px; margin: 0; color: rgba(255,255,255,.48); font-size: 12px; line-height: 1.6; }
  .immersive-controls { width: min(960px, calc(100% - 64px)); flex: 0 0 auto; margin: 0 auto; padding: 8px 0 max(20px, env(safe-area-inset-bottom)); }
  .immersive-timeline { display: flex; align-items: center; gap: 12px; color: rgba(255,255,255,.58); font-size: 10px; font-variant-numeric: tabular-nums; }
  .immersive-timeline span { min-width: 32px; text-align: center; }
  .immersive-timeline input, .immersive-volume input { min-width: 0; flex: 1; height: 4px; appearance: none; -webkit-appearance: none; border: 0; border-radius: 8px; outline: none; cursor: pointer; background: linear-gradient(to right, var(--accent) var(--progress), rgba(255,255,255,.23) var(--progress)); }
  .immersive-timeline input::-webkit-slider-thumb, .immersive-volume input::-webkit-slider-thumb { width: 11px; height: 11px; appearance: none; -webkit-appearance: none; background: #fff; border: 0; border-radius: 50%; box-shadow: 0 1px 5px #0008; }
  .immersive-transport-row { display: grid; grid-template-columns: 1fr 1fr 1fr; align-items: center; padding-top: 10px; }
  .immersive-control-spacer { display: flex; align-items: center; justify-content: flex-start; }
  .immersive-transport { display: flex; align-items: center; justify-content: center; gap: 26px; }
  .immersive-transport .transport-button { width: 38px; height: 38px; color: rgba(255,255,255,.8); }
  .immersive-play { width: 49px; height: 49px; color: #191417; background: #fff; }
  .immersive-play:hover:not(:disabled) { color: #111; background: #ffe6eb; }
  .immersive-volume { display: flex; align-items: center; justify-content: flex-end; gap: 9px; }
  .immersive-volume input { width: 105px; flex: 0 1 105px; }

  .player-bar {
    height: 86px;
    flex: 0 0 86px;
    display: grid;
    grid-template-columns: minmax(205px, 1fr) minmax(320px, 1.2fr) minmax(205px, 1fr);
    align-items: center;
    gap: 16px;
    padding: 0 19px;
    background: linear-gradient(110deg, rgba(29,32,31,.87), rgba(10,11,11,.91) 45%, rgba(21,23,22,.86));
    border-top: 1px solid var(--glass-edge);
    box-shadow: inset 0 1px 0 rgba(255,255,255,.095), 0 -15px 32px rgba(0,0,0,.14);
    -webkit-backdrop-filter: blur(30px) saturate(1.4);
    backdrop-filter: blur(30px) saturate(1.4);
    z-index: 4;
  }
  .player-track { min-width: 0; display: flex; align-items: center; gap: 11px; }
  .player-art-button, .player-art-empty { width: 48px; height: 48px; flex: 0 0 48px; display: grid; place-items: center; overflow: hidden; color: #c4c8c8; background: #1c1e1e; border: 0; border-radius: 5px; }
  .player-art-button { padding: 0; cursor: pointer; }
  .player-art-button img { width: 100%; height: 100%; object-fit: cover; }
  .player-track-copy { min-width: 0; display: flex; flex: 1; flex-direction: column; gap: 4px; }
  .player-track-copy strong { overflow: hidden; color: #eee; font-size: 12px; font-weight: 650; text-overflow: ellipsis; white-space: nowrap; }
  .player-track-copy span { overflow: hidden; color: #979b9b; font-size: 10px; text-overflow: ellipsis; white-space: nowrap; }
  .player-small-action { width: 30px; height: 30px; flex: 0 0 30px; }
  .player-small-action.selected { color: var(--accent); }
  .player-center { min-width: 0; width: min(100%, 580px); justify-self: center; display: flex; flex-direction: column; align-items: center; gap: 8px; }
  .transport-controls { height: 31px; display: flex; align-items: center; justify-content: center; gap: 15px; }
  .transport-button { width: 28px; height: 28px; }
  .transport-play { width: 33px; height: 33px; display: grid; place-items: center; padding-left: 1px; color: #111; background: #f5f6f6; border: 0; border-radius: 50%; cursor: pointer; transition: transform .15s ease, background .15s ease; }
  .transport-play:hover:not(:disabled) { transform: scale(1.07); background: #fff; }
  .transport-play:disabled { opacity: .45; cursor: default; }
  .loader-dot { width: 14px; height: 14px; border: 2px solid #777; border-top-color: #151515; border-radius: 50%; animation: spin .7s linear infinite; }
  @keyframes spin { to { transform: rotate(360deg); } }
  .player-progress { width: 100%; display: flex; align-items: center; gap: 8px; color: #969a9a; font-size: 9px; font-variant-numeric: tabular-nums; }
  .player-progress span { width: 31px; text-align: center; }
  .player-progress input, .volume-control input { height: 4px; appearance: none; -webkit-appearance: none; border: 0; border-radius: 6px; outline: none; cursor: pointer; background: linear-gradient(to right, #f2f4f4 var(--progress), #444848 var(--progress)); }
  .player-progress input { min-width: 0; flex: 1; }
  .player-progress input::-webkit-slider-thumb, .volume-control input::-webkit-slider-thumb { width: 10px; height: 10px; appearance: none; -webkit-appearance: none; background: #f7f8f8; border: 0; border-radius: 50%; opacity: 0; transition: opacity .12s; }
  .player-progress input:hover::-webkit-slider-thumb, .volume-control input:hover::-webkit-slider-thumb { opacity: 1; }
  .player-extras { display: flex; align-items: center; justify-content: flex-end; gap: 8px; }
  .volume-control { display: flex; align-items: center; gap: 5px; }
  .volume-control input { width: 84px; }
  .more-button { margin-left: 2px; }

  .settings-scrim { position: fixed; z-index: 20; inset: 0; display: grid; place-items: center; padding: 34px; background: rgba(0,0,0,.68); backdrop-filter: blur(4px); }
  .settings-dialog { width: min(900px, calc(100vw - 80px)); height: min(690px, calc(100vh - 86px)); min-height: 480px; overflow: hidden; background: #101111; border: 1px solid rgba(255,255,255,.1); border-radius: 15px; box-shadow: 0 28px 100px #000b; }
  .app-notice { position: fixed; z-index: 30; right: 22px; bottom: 103px; max-width: 440px; padding: 12px 15px; color: #f1f2f2; background: #242626; border: 1px solid rgba(255,255,255,.1); border-radius: 8px; box-shadow: 0 10px 35px #0009; font-size: 12px; }

  @media (max-width: 1120px) {
    .library-sidebar { width: 220px; flex-basis: 220px; }
    .right-panel { width: 310px; flex-basis: 310px; }
    .player-bar { grid-template-columns: minmax(170px,1fr) minmax(280px,1.2fr) minmax(160px,1fr); gap: 10px; }
    .volume-control input { width: 60px; }
  }
  @media (max-width: 960px) {
    .library-sidebar { display: none; }
    .right-panel { width: 290px; flex-basis: 290px; }
    .profile-label { display: none; }
    .topbar-actions { min-width: 100px; }
    .player-bar { grid-template-columns: minmax(145px,1fr) minmax(260px,1.25fr) minmax(110px,1fr); padding-inline: 10px; }
    .player-extras { gap: 3px; }
    .volume-control input { display: none; }
  }
  @media (max-height: 700px) {
    .immersive-header { height: 56px; flex-basis: 56px; }
    .immersive-content { padding-top: 4px; padding-bottom: 8px; gap: 26px; }
    .immersive-artwork { width: min(100%, 420px, 42vh); }
    .immersive-lyrics { padding: 20px 24px; }
    .immersive-lyric { padding: 7px 0; font-size: clamp(16px, 1.8vw, 22px); }
    .immersive-controls { padding-bottom: max(12px, env(safe-area-inset-bottom)); }
  }
  @media (prefers-reduced-motion: reduce) {
    :global(*), :global(*::before), :global(*::after) { scroll-behavior: auto !important; animation-duration: .01ms !important; transition-duration: .01ms !important; }
  }
</style>
