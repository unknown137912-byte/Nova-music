<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { writeText } from '@tauri-apps/plugin-clipboard-manager';
  import { listen } from '@tauri-apps/api/event';
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import { exit } from '@tauri-apps/plugin-process';
  import { fade, fly } from 'svelte/transition';
  import {
    Home as HomeIcon, Search, ChevronRight, ChevronLeft,
    UserRound, ListMusic, Mic2, Heart, Play, Pause, SkipBack,
    SkipForward, Volume2, VolumeX, PanelRight, Download, MoreHorizontal, Shuffle, Repeat2,
    Plus, Music2, Sparkles, X, ChevronDown, Maximize2, Minimize2, ArrowDownToLine,
    LayoutGrid, Radio, CalendarDays, Pin, Clock3, Disc, Clapperboard, Podcast, Star, Settings2,
    PanelLeftClose, PanelLeftOpen,
  } from 'lucide-svelte';

  import Home from './routes/+page.svelte';
  import SearchPage from './routes/search/+page.svelte';
  import LibraryPage from './routes/library/+page.svelte';
  import Playlists from './routes/playlists/+page.svelte';
  import ArtistsPage from './routes/artists/+page.svelte';
  import Settings from './routes/settings/+page.svelte';
  import TrackActions from './lib/TrackActions.svelte';
  import { artworkFor, artworkUrl as highResArtwork, recoverArtwork } from './lib/artwork';

  type Tab = 'home' | 'search' | 'library' | 'playlists' | 'artist';
  type NavigationEntry = { tab: Tab; playlistId: string | null; artistId: string; query: string };
  type SidePanel = 'now-playing' | 'queue' | 'lyrics' | null;

  let activeTab: Tab = 'home';
  let activePlaylistId: string | null = null;
  let activeArtistId = '';
  let historyStack: NavigationEntry[] = [{ tab: 'home', playlistId: null, artistId: '', query: '' }];
  let historyIndex = 0;
  let lastTrackpadNavigationAt = 0;
  let settingsOpen = false;
  let sidebarHidden = false;
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
  let libraryLoading = true;
  let libraryLoaded = false;
  let libraryLoadingTimer: ReturnType<typeof setTimeout> | undefined;
  let likedTrackIds = new Set<string>();
  let downloadJobs: any[] = [];
  let activeDownloadJobs: any[] = [];
  let downloadsOpen = false;
  let appearanceBackdrop = 'artwork';
  let motionPreference = 'full';
  let appearanceDensity = 'comfortable';
  let lyricsFontSize = 19;
  let libraryCategory = 'playlists';
  let playlistCategory = 'Playlists';
  let queueItems: any[] = [];
  let accountName = '';
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
  let pageArtwork = '';
  let playbackCollection: any = null;
  let shuffleMode = false;
  let repeatMode = 0;

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

  function updatePlaylistContext(detail: any) {
    if (!detail || (!detail.title && !detail?.items?.length)) {
      playbackCollection = null;
      pageArtwork = '';
      return;
    }
    playbackCollection = {
      title: String(detail.title || ''),
      thumbnail: String(detail.thumbnail || detail.cover || ''),
      items: Array.isArray(detail.items) ? detail.items : [],
    };
    pageArtwork = String(detail.thumbnail || detail.cover || '');
    if (pageArtwork) sampleArtworkPalette(pageArtwork);
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
    let stopDownloadUpdates: (() => void) | undefined;
    const refreshLikedTracks = () => void loadLikedTracks();
    const requestShuffle = () => { shuffleMode = true; };
    const handlePlaylistContext = (event: Event) => updatePlaylistContext((event as CustomEvent).detail);
    const handlePageArtwork = (event: Event) => {
      pageArtwork = String((event as CustomEvent<string>).detail || '');
      if (pageArtwork) sampleArtworkPalette(pageArtwork);
    };
    window.addEventListener('nova-liked-changed', refreshLikedTracks);
    window.addEventListener('nova-shuffle-request', requestShuffle);
    const openHistory = () => navigate('library');
    const openSettingsFromPage = () => openSettings();
    const openLibraryFromSettings = () => openLibraryCategory('Playlists');
    const refreshAccountLibrary = () => void loadAccountAndLibrary();
    const closeSearchOnOutsideClick = (event: MouseEvent) => {
      const target = event.target as HTMLElement | null;
      if (!target?.closest('.global-search-wrap')) searchPopoverOpen = false;
      if (!target?.closest('.downloads-wrap')) downloadsOpen = false;
    };
    window.addEventListener('nova-open-history', openHistory);
    window.addEventListener('nova-open-library', openLibraryFromSettings);
    window.addEventListener('nova-playlist-context', handlePlaylistContext);
    window.addEventListener('nova-page-artwork', handlePageArtwork);
    window.addEventListener('nova-open-settings', openSettingsFromPage);
    window.addEventListener('nova-local-playlists-changed', refreshAccountLibrary);
    window.addEventListener('pointerdown', closeSearchOnOutsideClick);
    const handleCloseSettingChange = (event: Event) => {
      closeToBackgroundEnabled = (event as CustomEvent<boolean>).detail;
    };
    const handleAutoPlayChange = (event: Event) => {
      autoPlayNext = (event as CustomEvent<boolean>).detail;
    };
    const handlePlaybackSettingsChange = (event: Event) => {
      const detail = (event as CustomEvent<{ repeatMode?: number; shuffleMode?: boolean }>).detail;
      if (typeof detail?.repeatMode === 'number') repeatMode = detail.repeatMode;
      if (typeof detail?.shuffleMode === 'boolean') shuffleMode = detail.shuffleMode;
    };
    const applyAppearanceSetting = (event: Event) => {
      const detail = (event as CustomEvent<{ key: string; value: string }>).detail;
      if (!detail) return;
      const shell = document.querySelector<HTMLElement>('.app-shell');
      if (detail.key === 'accent_color') shell?.style.setProperty('--user-accent', detail.value);
      if (detail.key === 'appearance_backdrop') appearanceBackdrop = detail.value;
      if (detail.key === 'motion_preference') motionPreference = detail.value;
      if (detail.key === 'cosmic_theme') {
        shell?.classList.remove('theme-nebula', 'theme-cyber', 'theme-deepspace');
        if (['nebula', 'cyber', 'deepspace'].includes(detail.value)) shell?.classList.add(`theme-${detail.value}`);
      }
      if (detail.key === 'appearance_density') {
        appearanceDensity = detail.value;
        shell?.classList.toggle('compact-interface', detail.value === 'compact');
      }
      if (detail.key === 'lyrics_font_size') {
        lyricsFontSize = Math.max(14, Math.min(28, Number(detail.value) || 19));
        shell?.style.setProperty('--lyrics-size', `${lyricsFontSize}px`);
      }
      if (detail.key === 'sidebar_visible') sidebarHidden = detail.value !== 'true';
    };
    window.addEventListener('nova-close-to-background-changed', handleCloseSettingChange);
    window.addEventListener('nova-autoplay-changed', handleAutoPlayChange);
    window.addEventListener('nova-appearance-changed', applyAppearanceSetting);
    window.addEventListener('nova-playback-settings-changed', handlePlaybackSettingsChange);

    listen('playback-started', async () => {
      playbackError = null;
      isPlaying = true;
      isLoadingTrack = false;
      try {
        const [settings, track]: [any, any] = await Promise.all([
          invoke('get_settings'), invoke('get_now_playing'),
        ]);
        const trackId = String(track?.id || '');
        if (settings?.auto_open_lyrics === 'true' && !showNowPlaying) {
          sidePanel = 'lyrics';
          window.setTimeout(() => void loadLyrics(), 80);
        }
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
        if (repeatMode === 1 && nowPlaying) {
          await handlePlayTrack(nowPlaying);
          return;
        }
        const collection = playbackCollection?.items || [];
        if (collection.length) {
          const currentIndex = activeImmersiveTrackIndex;
          let nextIndex = currentIndex + 1;
          if (shuffleMode && collection.length > 1) {
            do { nextIndex = Math.floor(Math.random() * collection.length); } while (nextIndex === currentIndex);
          } else if (nextIndex >= collection.length && repeatMode === 2) {
            nextIndex = 0;
          }
          if (nextIndex >= 0 && nextIndex < collection.length) {
            await handlePlayTrack(collection[nextIndex]);
            await loadQueue();
            return;
          }
        }
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
      if (!libraryLoaded) {
        libraryLoading = true;
        if (libraryLoadingTimer) clearTimeout(libraryLoadingTimer);
        // A slow remote library must not leave the Library route looking frozen.
        // The request continues and can still populate the page when it completes.
        libraryLoadingTimer = setTimeout(() => {
          if (!disposed && !libraryLoaded) {
            libraryLoaded = true;
            libraryLoading = false;
          }
        }, 8000);
      }
      try {
        const [account, library]: [any, any] = await Promise.all([
          invoke('get_account_info'), invoke('get_library'),
        ]);
        if (disposed) return;
        if (libraryLoadingTimer) clearTimeout(libraryLoadingTimer);
        accountSignedIn = Boolean(account?.is_logged_in);
        accountName = account?.name || account?.email || '';
        libraryCollections = {
          playlists: (Array.isArray(library) ? library : (library?.playlists || []))
            .filter((item: any) => !['LM', 'VLLM'].includes(String(item?.id || item?.browseId || item?.browse_id || ''))
              && !/^(liked music|liked songs)$/i.test(String(item?.title || item?.name || ''))),
          albums: library?.albums || [],
          artists: library?.artists || [],
        };
        libraryLoaded = true;
        libraryLoading = false;
      } catch (_) {
        // The app can still play/search while account or library data is unavailable.
        if (libraryLoadingTimer) clearTimeout(libraryLoadingTimer);
        libraryLoaded = true;
        libraryLoading = false;
      }
    };

    const loadLikedTracks = async () => {
      try {
        const tracks: any[] = await invoke('get_liked_tracks');
        if (!disposed) likedTrackIds = new Set((Array.isArray(tracks) ? tracks : []).map((item) => String(item.id || item.videoId || '')).filter(Boolean));
      } catch (_) { /* Likes remain available in the local database when the account is offline. */ }
    };

    invoke<any>('get_settings').then((settings) => {
      if (!disposed) {
        closeToBackgroundEnabled = settings?.close_to_tray === 'true';
        autoPlayNext = settings?.auto_play_next !== 'false';
        shuffleMode = settings?.shuffle_mode === 'true';
        repeatMode = Math.max(0, Math.min(2, Number(settings?.repeat_mode || 0)));
        const theme = String(settings?.cosmic_theme || 'nebula');
        const shell = document.querySelector<HTMLElement>('.app-shell');
        shell?.classList.remove('theme-nebula', 'theme-cyber', 'theme-deepspace');
        shell?.classList.add(`theme-${theme}`);
        shell?.style.setProperty('--user-accent', settings?.accent_color || '');
        appearanceBackdrop = settings?.appearance_backdrop || 'artwork';
        motionPreference = settings?.motion_preference || 'full';
        appearanceDensity = settings?.appearance_density || 'comfortable';
        shell?.classList.toggle('compact-interface', appearanceDensity === 'compact');
        lyricsFontSize = Math.max(14, Math.min(28, Number(settings?.lyrics_font_size || 19)));
        shell?.style.setProperty('--lyrics-size', `${lyricsFontSize}px`);
        if (settings?.sidebar_visible === 'false') sidebarHidden = true;
        const startPage = settings?.start_page;
        if (startPage === 'home' || startPage === 'library' || startPage === 'search') {
          activeTab = startPage;
          activePlaylistId = null;
          activeArtistId = '';
          historyStack = [{ tab: startPage, playlistId: null, artistId: '', query: '' }];
          historyIndex = 0;
        }
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
          if (!sameTrack) sampleArtworkPalette(artworkFor(track, 1600, playbackCollection?.thumbnail || pageArtwork));
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
    loadLikedTracks();
    void loadQueue();
    listen<any>('download-updated', ({ payload }) => {
      if (!payload?.id) return;
      downloadJobs = [payload, ...downloadJobs.filter((job) => job.id !== payload.id)].slice(0, 30);
    }).then((unlisten) => { if (disposed) unlisten(); else stopDownloadUpdates = unlisten; });
    pollPlayback();
    const playbackTimer = setInterval(pollPlayback, 900);
    const accountTimer = setInterval(loadAccountAndLibrary, 45000);
    return () => {
      disposed = true;
      clearInterval(playbackTimer);
      clearInterval(accountTimer);
      if (libraryLoadingTimer) clearTimeout(libraryLoadingTimer);
      if (appNoticeTimer) clearTimeout(appNoticeTimer);
      stopStart?.();
      stopError?.();
      stopEnded?.();
      stopCloseRequest?.();
      stopDownloadUpdates?.();
      window.removeEventListener('nova-liked-changed', refreshLikedTracks);
      window.removeEventListener('nova-shuffle-request', requestShuffle);
      window.removeEventListener('nova-open-history', openHistory);
      window.removeEventListener('nova-open-library', openLibraryFromSettings);
      window.removeEventListener('nova-playlist-context', handlePlaylistContext);
      window.removeEventListener('nova-page-artwork', handlePageArtwork);
      window.removeEventListener('nova-open-settings', openSettingsFromPage);
      window.removeEventListener('nova-local-playlists-changed', refreshAccountLibrary);
      window.removeEventListener('pointerdown', closeSearchOnOutsideClick);
      window.removeEventListener('nova-close-to-background-changed', handleCloseSettingChange);
      window.removeEventListener('nova-autoplay-changed', handleAutoPlayChange);
      window.removeEventListener('nova-appearance-changed', applyAppearanceSetting);
      window.removeEventListener('nova-playback-settings-changed', handlePlaybackSettingsChange);
      if (globalSearchTimer) clearTimeout(globalSearchTimer);
    };
  });

  function navigate(tab: Tab) {
    searchPopoverOpen = false;
    if (tab === 'home' || tab === 'search') sidePanel = null;
    if (tab !== 'playlists' || !activePlaylistId) {
      playbackCollection = null;
      pageArtwork = '';
    }
    const destination: NavigationEntry = {
      tab,
      playlistId: tab === 'playlists' ? activePlaylistId : null,
      artistId: tab === 'artist' ? activeArtistId : '',
      query: tab === 'search' ? globalSearchQuery : '',
    };
    const current = historyStack[historyIndex];
    if (current && current.tab === destination.tab && current.playlistId === destination.playlistId
      && current.artistId === destination.artistId && current.query === destination.query) {
      if (showNowPlaying) void closeImmersivePlayer();
      return;
    }
    historyStack = [...historyStack.slice(0, historyIndex + 1), destination];
    historyIndex = historyStack.length - 1;
    activeTab = tab;
    if (showNowPlaying) void closeImmersivePlayer();
  }

  function openSearchFor(query = '') {
    globalSearchQuery = query;
    navigate('search');
    if (!query) setTimeout(() => globalSearchInput?.focus(), 0);
  }

  function openLibraryCategory(category: 'Playlists' | 'Albums' | 'Artists') {
    libraryCategory = category.toLowerCase();
    playlistCategory = category;
    activePlaylistId = null;
    navigate('playlists');
  }

  function navigateHistory(direction: number) {
    const target = historyIndex + direction;
    if (target < 0 || target >= historyStack.length) return;
    historyIndex = target;
    const destination = historyStack[historyIndex];
    activeTab = destination.tab;
    activePlaylistId = destination.playlistId;
    activeArtistId = destination.artistId;
    globalSearchQuery = destination.query;
    if (destination.tab === 'home' || destination.tab === 'search') sidePanel = null;
    else if (sidePanel === null && (destination.tab === 'playlists' && destination.playlistId || destination.tab === 'artist' && destination.artistId)) sidePanel = 'queue';
    if (showNowPlaying) void closeImmersivePlayer();
  }

  function handleTrackpadNavigation(event: WheelEvent) {
    if (Math.abs(event.deltaX) < 48 || Math.abs(event.deltaX) < Math.abs(event.deltaY) * 1.35) return;
    const target = event.target as HTMLElement | null;
    if (target?.closest('.home-chips, .shelf-track, .shelf-scroller, .downloads-list, .global-search-wrap, input, textarea, [contenteditable="true"]')) return;
    const now = Date.now();
    if (now - lastTrackpadNavigationAt < 650) return;
    lastTrackpadNavigationAt = now;
    event.preventDefault();
    // A rightward two-finger swipe goes back; a leftward swipe goes forward.
    navigateHistory(event.deltaX > 0 ? -1 : 1);
  }

  async function startWindowDrag(event: MouseEvent) {
    if (event.button !== 0 || event.target !== event.currentTarget) return;
    try { await getCurrentWindow().startDragging(); } catch (_) { /* Web preview has no native window to drag. */ }
  }

  function openSettings() {
    settingsOpen = true;
  }

  function openSongActionsOnDoubleClick(event: MouseEvent) {
    const target = event.target;
    if (!(target instanceof Element) || target.closest('.track-actions, input, textarea, [contenteditable="true"]')) return;
    const row = target.closest('.search-result-row, .recent-card-wrap, .shelf-card-wrap, .history-row-wrap, .song-row, .track-row, .artist-song-row, .immersive-song-row-wrap, .player-track');
    const actions = row?.querySelector('.track-actions');
    actions?.dispatchEvent(new Event('nova-track-actions-open'));
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
    if (activeTab === 'search' && historyStack[historyIndex]?.tab === 'search') {
      historyStack = historyStack.map((entry, index) => index === historyIndex ? { ...entry, query: globalSearchQuery } : entry);
    }
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
    const collectionArtwork = playbackCollection?.thumbnail || playbackCollection?.cover || pageArtwork || '';
    nowPlaying = {
      ...track,
      id: trackId,
      title: track?.title || track?.name || 'Unknown track',
      artist: track?.artist || track?.artists || track?.subtitle || 'YouTube Music',
      thumbnail: track?.thumbnail || track?.thumbnail_url || track?.cover || '',
      fallbackArtwork: collectionArtwork,
      duration: duration || undefined,
    };
    sampleArtworkPalette(artworkFor(nowPlaying, 1600, collectionArtwork));
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
        thumbnail: resolved?.thumbnail || resolved?.thumbnail_url || nowPlaying.thumbnail,
        fallbackArtwork: collectionArtwork,
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
    sidePanel = 'queue';
    navigate('playlists');
  }

  async function loadLikedTracks() {
    try {
      const tracks: any[] = await invoke('get_liked_tracks');
      likedTrackIds = new Set((Array.isArray(tracks) ? tracks : []).map((item) => String(item.id || item.videoId || '')).filter(Boolean));
    } catch (_) { /* Keep the last local snapshot if the database is temporarily busy. */ }
  }

  function handleOpenArtist(id: string) {
    if (!id) return;
    activeArtistId = id;
    sidePanel = 'queue';
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

  function toggleShuffleMode() {
    shuffleMode = !shuffleMode;
    void invoke('update_settings', { key: 'shuffle_mode', value: String(shuffleMode) })
      .catch((error: any) => showNotice(error?.message || 'Could not save shuffle preference'));
  }

  function cycleRepeatMode() {
    repeatMode = (repeatMode + 1) % 3;
    void invoke('update_settings', { key: 'repeat_mode', value: String(repeatMode) })
      .catch((error: any) => showNotice(error?.message || 'Could not save repeat preference'));
  }

  async function skipTrack() {
    try {
      const tracks = playbackCollection?.items || [];
      if (tracks.length) {
        let nextIndex = activeImmersiveTrackIndex + 1;
        if (repeatMode === 1 && activeImmersiveTrackIndex >= 0) nextIndex = activeImmersiveTrackIndex;
        else if (shuffleMode && tracks.length > 1) {
          do { nextIndex = Math.floor(Math.random() * tracks.length); } while (nextIndex === activeImmersiveTrackIndex);
        } else if (nextIndex >= tracks.length && repeatMode === 2) nextIndex = 0;
        if (nextIndex >= 0 && nextIndex < tracks.length) {
          await handlePlayTrack(tracks[nextIndex]);
          return;
        }
      }
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
        await invoke('rate_track', { trackId, liked, track });
        track.liked = liked;
        track.rating = liked ? 'like' : 'indifferent';
        if (liked) likedTrackIds = new Set([...likedTrackIds, trackId]);
        else { const next = new Set(likedTrackIds); next.delete(trackId); likedTrackIds = next; }
        window.dispatchEvent(new CustomEvent('nova-liked-changed'));
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
      const message = typeof error === 'string' ? error : (error?.message || 'Could not complete that song action');
      if (message !== 'Download cancelled') showNotice(message);
      return false;
    }
  }

  async function prevTrack() {
    if (playbackCollection?.items?.length && activeImmersiveTrackIndex > 0) {
      await handlePlayTrack(playbackCollection.items[activeImmersiveTrackIndex - 1]);
      return;
    }
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
      const message = typeof error === 'string' ? error : (error?.message || 'Download failed');
      if (message !== 'Download cancelled') showNotice(message);
    }
  }

  async function cancelDownload(jobId: string) {
    try {
      await invoke('cancel_download', { jobId });
    } catch (error: any) {
      showNotice(typeof error === 'string' ? error : (error?.message || 'Could not cancel download'));
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
    const id = String(track?.videoId || track?.video_id || track?.track_id || track?.id || '');
    return likedTrackIds.has(id) || track?.liked === true || String(track?.rating || '').toLowerCase() === 'like';
  }

  function recoverNowPlayingArtwork(event: Event) {
    recoverArtwork(event, nowPlaying ? {
      ...nowPlaying,
      fallbackArtwork: playbackCollection?.thumbnail || playbackCollection?.cover || pageArtwork || nowPlaying.fallbackArtwork || '',
    } : null);
  }

  function recoverCollectionArtwork(event: Event) {
    recoverArtwork(event, {
      ...(playbackCollection || nowPlaying || {}),
      fallbackArtwork: collectionFallbackArtwork || currentArtwork || '',
    });
  }

  async function toggleLikeTrack() {
    const trackId = String(nowPlaying?.id || '').trim();
    if (!trackId) return;
    const wasLiked = trackIsLiked(nowPlaying);
    nowPlaying = { ...nowPlaying, liked: !wasLiked, rating: wasLiked ? 'indifferent' : 'like' };
    try {
      await invoke('rate_track', { trackId, liked: !wasLiked, track: nowPlaying });
      if (wasLiked) { const next = new Set(likedTrackIds); next.delete(trackId); likedTrackIds = next; }
      else likedTrackIds = new Set([...likedTrackIds, trackId]);
      window.dispatchEvent(new CustomEvent('nova-liked-changed'));
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

  function formatTrackDuration(track: any) {
    const value = track?.duration_seconds ?? track?.duration;
    return formatTime(durationInSeconds(value) ?? Number(value) ?? 0);
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
  $: immersiveTracks = playbackCollection?.items?.length
    ? playbackCollection.items
    : (nowPlaying ? [nowPlaying, ...queueItems] : queueItems);
  $: activeImmersiveTrackId = String(nowPlaying?.id || nowPlaying?.videoId || '');
  $: activeImmersiveTrackIndex = immersiveTracks.findIndex((item: any) => String(item?.videoId || item?.video_id || item?.track_id || item?.id || '') === activeImmersiveTrackId);
  $: queuePreviewItems = playbackCollection?.items?.length && activeImmersiveTrackIndex >= 0
    ? playbackCollection.items.slice(activeImmersiveTrackIndex + 1)
    : queueItems;
  $: currentArtwork = artworkFor(nowPlaying, 1600, playbackCollection?.thumbnail || playbackCollection?.cover || pageArtwork || nowPlaying?.fallbackArtwork || '');
  $: collectionFallbackArtwork = artworkFor(playbackCollection?.items?.[0] || nowPlaying, 1600, currentArtwork);
  $: collectionArtwork = artworkFor(playbackCollection, 1600, collectionFallbackArtwork || currentArtwork);
  $: immersiveArtwork = currentArtwork;
  $: activeDownloadJobs = downloadJobs.filter((job) => ['starting', 'downloading', 'converting'].includes(job.status));
  $: if (activeDownloadJobs.length === 0 && downloadsOpen) downloadsOpen = false;
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

<svelte:window on:keydown={handleHotkey} on:dblclick={openSongActionsOnDoubleClick} on:contextmenu|preventDefault />

<div class="app-shell theme-nebula" class:compact-interface={appearanceDensity === 'compact'} class:search-expanded={searchPopoverOpen && globalSearchQuery.trim().length >= 2} class:panel-hidden={sidePanel === null} class:sidebar-hidden={sidebarHidden} class:reduced-motion={motionPreference === 'reduced'} class:flat-backdrop={appearanceBackdrop === 'flat'} class:gradient-backdrop={appearanceBackdrop === 'gradient'} style={`--cover-rgb:${coverRgb};--cover-deep-rgb:${coverDeepRgb};--lyrics-size:${lyricsFontSize}px;--ambient-image:url("${immersiveArtwork.replaceAll('"', '%22')}")`}>
  <div class="app-backdrop" aria-hidden="true">
    {#key immersiveArtwork}<img src={immersiveArtwork || '/placeholder.svg'} alt="" in:fade={{ duration: 520 }} out:fade={{ duration: 360 }} on:error={recoverNowPlayingArtwork} />{/key}
  </div>
  <header class="topbar">
    {#if historyIndex > 0 && activeTab !== 'home' && !showNowPlaying}
      <div class="topbar-history">
        <button class="chrome-button history-back" on:click={() => navigateHistory(-1)} aria-label="Go back" title="Go back">
          <ChevronLeft size={17} />
        </button>
      </div>
    {/if}
    <div class="window-drag-space" on:mousedown={startWindowDrag} aria-hidden="true"></div>
    <div class="topbar-actions">
      <button class="chrome-button sidebar-toggle" on:click={() => sidebarHidden = !sidebarHidden} aria-label={sidebarHidden ? 'Show sidebar' : 'Hide sidebar'} title={sidebarHidden ? 'Show sidebar' : 'Hide sidebar'} aria-pressed={sidebarHidden}>
        {#if sidebarHidden}<PanelLeftOpen size={17} />{:else}<PanelLeftClose size={17} />{/if}
      </button>
      {#if activeDownloadJobs.length}
      <div class="downloads-wrap">
        <button class="chrome-button download-top-button" class:active={downloadsOpen} class:downloading={activeDownloadJobs.length > 0} on:click|stopPropagation={() => downloadsOpen = !downloadsOpen} aria-label="Active downloads" aria-expanded={downloadsOpen} title="Active downloads">
          <ArrowDownToLine size={17} />
          <span class="download-pulse"></span>
        </button>
        {#if downloadsOpen}
          <section class="downloads-popover" aria-label="Downloads">
            <header><strong>Downloads</strong><span>{activeDownloadJobs.length} active</span><button class="download-popover-close" on:click|stopPropagation={() => downloadsOpen = false} aria-label="Close downloads"><X size={15} /></button></header>
            {#if downloadJobs.length}
              <div class="downloads-list">
                {#each downloadJobs as job (job.id)}
                  <div class="download-job">
                    <span class="download-job-icon" class:done={job.status === 'completed'} class:failed={job.status === 'failed'}><Download size={15} /></span>
                    <div class="download-job-copy"><strong>{job.title || 'NOVA Music download'}</strong><small>{job.status === 'completed' ? 'Download complete' : job.status === 'cancelled' ? 'Download cancelled' : job.status === 'failed' ? (job.error || 'Download failed') : job.status === 'starting' ? 'Finding the best audio stream…' : job.status === 'converting' ? 'Converting audio…' : job.progress == null ? 'Downloading…' : `${Math.round(job.progress)}% · Downloading`}</small>
                      {#if ['starting', 'downloading', 'converting'].includes(job.status)}<span class="download-progress"><i style={`width:${job.progress == null ? 28 : Math.max(3, job.progress)}%`}></i></span>{/if}
                    </div>
                    {#if job.status === 'completed'}<span class="download-check">✓</span>{:else if job.status === 'cancelled'}<span class="download-cancelled">Cancelled</span>{:else if ['starting', 'downloading', 'converting'].includes(job.status)}<button class="download-cancel" on:click|stopPropagation={() => void cancelDownload(job.id)} aria-label={`Cancel download of ${job.title || 'song'}`} title="Cancel download"><X size={13} /><span>Cancel</span></button>{/if}
                  </div>
                {/each}
              </div>
            {:else}<p class="downloads-empty">Your downloads will appear here.</p>{/if}
          </section>
        {/if}
      </div>
      {/if}
    </div>
  </header>

  <div class="workarea">
      <aside class="library-sidebar" aria-label="Music navigation">
        <div class="sidebar-window-space" on:mousedown={startWindowDrag} aria-hidden="true"></div>
        <button class="sidebar-brand" on:click={() => navigate('home')} aria-label="NOVA Music home"><img class="sidebar-brand-icon" src="/icon.png" alt="" /><strong>NOVA MUSIC</strong></button>

        <div class="global-search-wrap">
          <div class="global-search" class:focused={searchPopoverOpen}>
            <Search size={15} />
            <input
              bind:this={globalSearchInput}
              bind:value={globalSearchQuery}
              on:input={handleGlobalSearchInput}
              on:focus={() => { searchPopoverOpen = true; }}
              on:keydown={handleGlobalSearchKeydown}
              placeholder="Search"
              aria-label="Search YouTube Music"
              aria-expanded={searchPopoverOpen}
              aria-controls="global-search-results"
            />
            <kbd>⌘K</kbd>
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
                        <button class="search-result-main" on:click={() => chooseGlobalSearchResult(item, 'song')}><img src={artworkFor(item, 900)} alt="" on:error={(event) => recoverArtwork(event, item)} /><span class="search-result-copy"><strong>{item.title || 'Untitled track'}</strong><small>{item.artist || item.artists || 'YouTube Music'}</small></span><Play size={15} fill="currentColor" /></button>
                        <TrackActions track={item} playlists={libraryCollections.playlists} onAction={handleTrackAction} />
                      </div>
                    {/each}
                  </div>
                {/if}
                {#if quickSearchCollections.length}
                  <div class="search-popover-section collection-results"><span class="search-popover-label">MORE RESULTS</span>
                    {#each quickSearchCollections as item (`${item.resultKind}:${item.id}`)}
                      <button class="search-collection-result" on:click={() => chooseGlobalSearchResult(item, item.resultKind)}><img src={highResArtwork(item.thumbnail || item.thumbnail_url || '', 700) || '/placeholder.svg'} alt="" on:error={(event) => recoverArtwork(event, item)} /><span><strong>{item.title || 'Untitled'}</strong><small>{item.resultKind}</small></span></button>
                    {/each}
                  </div>
                {/if}
                {#if !quickSearchSongs.length && !quickSearchCollections.length}<div class="search-popover-state">No results found for “{globalSearchQuery.trim()}”.</div>{/if}
                <button class="search-view-all" on:click={openFullSearch}>Show all results for “{globalSearchQuery.trim()}” <ChevronRight size={15} /></button>
              {:else}<div class="search-popover-state">Search songs, artists, albums, and playlists.</div>{/if}
            </div>
          {/if}
        </div>

        <nav class="sidebar-primary" aria-label="Browse">
          <button class:active={activeTab === 'home'} on:click={() => navigate('home')}><HomeIcon size={15} /><span>Home</span></button>
          <button class:active={activeTab === 'search'} on:click={() => openSearchFor('new releases')}><LayoutGrid size={15} /><span>New</span></button>
          <button on:click={() => openSearchFor('radio')}><Radio size={15} /><span>Radio</span></button>
          <button on:click={() => openSearchFor('concerts')}><CalendarDays size={15} /><span>Concerts</span></button>
        </nav>

        <div class="sidebar-library-nav">
          <button class="sidebar-section-label" on:click={() => openLibraryCategory('Playlists')}><span>Library</span><ChevronDown size={13} /></button>
          <button on:click={() => openLibraryCategory('Playlists')}><Pin size={15} /><span>Pins</span><ChevronRight size={13} /></button>
          <button on:click={() => navigate('library')}><Clock3 size={15} /><span>Recently Added</span></button>
          <button on:click={() => handleOpenPlaylist('NOVA_LIKED')}><Music2 size={15} /><span>Songs</span></button>
          <button class:active={libraryCategory === 'albums'} on:click={() => openLibraryCategory('Albums')}><Disc size={15} /><span>Albums</span></button>
          <button class:active={libraryCategory === 'artists'} on:click={() => openLibraryCategory('Artists')}><UserRound size={15} /><span>Artists</span></button>
          <button on:click={() => openSearchFor('music videos')}><Clapperboard size={15} /><span>Music Videos</span></button>
          <button on:click={() => openSearchFor('podcasts')}><Podcast size={15} /><span>Podcasts</span></button>
          <button on:click={() => openLibraryCategory('Playlists')}><ListMusic size={15} /><span>Apple Music Playlists</span><ChevronRight size={13} /></button>
        </div>

        <div class="sidebar-playlists">
          <div class="sidebar-section-label"><span>Playlists</span><span class="sidebar-section-actions"><button class="sidebar-add" on:click={() => openLibraryCategory('Playlists')} aria-label="More playlist options"><MoreHorizontal size={14} /></button><button class="sidebar-add" on:click={() => openLibraryCategory('Playlists')} aria-label="Collapse playlists"><ChevronDown size={13} /></button></span></div>
          <button class="sidebar-playlist-entry" on:click={() => openLibraryCategory('Playlists')}><span class="sidebar-playlist-art"><Plus size={14} /></span><span>Create New</span></button>
          <button class="sidebar-playlist-entry" class:active={activeTab === 'playlists' && !activePlaylistId} on:click={() => openLibraryCategory('Playlists')}><span class="sidebar-playlist-art"><LayoutGrid size={14} /></span><span>All Playlists</span></button>
          <button class="sidebar-playlist-entry" class:active={activePlaylistId === 'NOVA_LIKED'} on:click={() => handleOpenPlaylist('NOVA_LIKED')}><span class="sidebar-playlist-art liked-art"><Star size={14} fill="currentColor" /></span><span>Favorite Songs</span></button>
          {#each sidebarLibraryItems as item (item.id || item.playlist_id || item.title)}
            <button class="sidebar-playlist-entry" on:click={() => playLibraryItem(item)}>
              <span class="sidebar-playlist-art">{#if item.thumbnail || item.cover || item.thumbnail_url}<img src={highResArtwork(item.thumbnail || item.cover || item.thumbnail_url, 500)} alt="" on:error={(event) => recoverArtwork(event, item)} />{:else}<Music2 size={14} />{/if}</span>
              <span>{item.title || item.name || 'Untitled playlist'}</span><ChevronRight size={12} />
            </button>
          {/each}
          {#if sidebarLibraryItems.length === 0}<p class="sidebar-empty">Your saved playlists appear here.</p>{/if}
        </div>

        <button class="sidebar-settings" on:click={openSettings} aria-label="Open settings" title="Settings">
          <Settings2 size={18} />
        </button>
      </aside>

    <main class="main-content" class:panel-open={sidePanel !== null} on:wheel|nonpassive={handleTrackpadNavigation}>
      {#key `${activeTab}:${activePlaylistId || ''}:${activeArtistId}:${activeTab === 'search' ? globalSearchQuery : ''}`}
      <div class="route-layer" in:fly={{ y: 8, duration: 230 }} out:fade={{ duration: 115 }}>
      {#if activeTab === 'home'}
        <Home accountName={accountName} currentTrackId={String(nowPlaying?.id || '')} fallbackArtwork={currentArtwork} onPlayTrack={handlePlayTrack} onOpenPlaylist={handleOpenPlaylist} onOpenArtist={handleOpenArtist} onTrackAction={handleTrackAction} playlistOptions={libraryCollections.playlists} />
      {:else if activeTab === 'search'}
        <SearchPage initialQuery={globalSearchQuery} onPlayTrack={handlePlayTrack} onOpenPlaylist={handleOpenPlaylist} onOpenArtist={handleOpenArtist} onTrackAction={handleTrackAction} playlistOptions={libraryCollections.playlists} />
      {:else if activeTab === 'library'}
        <LibraryPage libraryCollections={libraryCollections} libraryLoading={libraryLoading} accountInfo={{ is_logged_in: accountSignedIn, name: accountName }} fallbackArtwork={currentArtwork} onPlayTrack={handlePlayTrack} onOpenPlaylist={handleOpenPlaylist} onOpenArtist={handleOpenArtist} onTrackAction={handleTrackAction} playlistOptions={libraryCollections.playlists} />
      {:else if activeTab === 'playlists'}
        <Playlists bind:activePlaylistId bind:category={playlistCategory} onPlayTrack={handlePlayTrack} onOpenArtist={handleOpenArtist} onTrackAction={handleTrackAction} playlistOptions={libraryCollections.playlists} />
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
            <button class:active={sidePanel === 'now-playing'} on:click={() => sidePanel = 'now-playing'} aria-label="Now playing" title="Now playing"><Music2 size={15} /></button>
            <button class:active={sidePanel === 'queue'} on:click={() => { sidePanel = 'queue'; void loadQueue(); }} aria-label="Queue" title="Queue"><ListMusic size={15} /></button>
            <button class:active={sidePanel === 'lyrics'} on:click={() => toggleSidePanel('lyrics')} aria-label="Lyrics" title="Lyrics"><Mic2 size={15} /></button>
            <button class:active={shuffleMode} on:click={toggleShuffleMode} aria-label={shuffleMode ? 'Turn shuffle off' : 'Turn shuffle on'} title="Shuffle"><Shuffle size={14} /></button>
          </div>
          {#if sidePanel === 'queue'}<span class="queue-item-count">{queuePreviewItems.length + (nowPlaying ? 1 : 0)} items</span>{/if}
          <button class="icon-button" on:click|stopPropagation={() => sidePanel = null} aria-label="Close panel" title="Close panel"><X size={17} /></button>
        </div>
        {#if sidePanel === 'now-playing'}
          <div class="now-playing-panel">
            {#if nowPlaying}
              <div class="panel-now-art"><img src={currentArtwork} alt="" on:error={recoverNowPlayingArtwork} /></div>
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
                  <img src={artworkFor(playbackCollection, 1000, collectionFallbackArtwork || currentArtwork)} alt="" on:error={recoverCollectionArtwork} />
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
            <div class="queue-context"><strong>Now Playing</strong></div>
            {#if nowPlaying}
              <button class="queue-track current" on:click={openImmersivePlayer}>
                <span class="queue-number"><span class="playing-bars"><i></i><i></i><i></i></span></span>
                <img src={currentArtwork} alt="" on:error={recoverNowPlayingArtwork} />
                <span class="queue-track-copy"><strong>{nowPlaying.title}</strong><small>{nowPlaying.artist}</small></span>
                <span class="queue-time"><Heart size={13} fill={trackIsLiked(nowPlaying) ? 'currentColor' : 'none'} /></span>
              </button>
            {/if}
            <div class="queue-next-heading"><span><strong>Playing Next</strong><small>{playbackCollection?.title || nowPlaying?.playlist || 'From your queue'}</small></span><em>{queuePreviewItems.length}</em></div>
            {#each queuePreviewItems as item, index (item.track_id || item.videoId || item.video_id || item.id || index)}
              <button class="queue-track" on:click={() => handlePlayTrack(item)}>
                <span class="queue-number">{index + 1}</span>
                <img src={artworkFor(item, 1000, playbackCollection?.thumbnail || playbackCollection?.cover || pageArtwork)} alt="" on:error={(event) => recoverArtwork(event, { ...item, fallbackArtwork: playbackCollection?.thumbnail || playbackCollection?.cover || pageArtwork })} />
                <span class="queue-track-copy"><strong>{item.title || 'Untitled track'}</strong><small>{item.artist || item.subtitle || 'YouTube Music'}</small></span>
                <span class="queue-time"><MoreHorizontal size={15} /></span>
              </button>
            {/each}
            {#if queuePreviewItems.length === 0 && !nowPlaying}
              <div class="panel-empty"><ListMusic size={25} /><p>Your queue is empty.</p><button on:click={() => navigate('search')}>Find something to play</button></div>
            {/if}
          </div>
        {:else}
          <div class="lyrics-content">
            {#if nowPlaying}
              <div class="lyrics-track-art"><img src={currentArtwork} alt="" on:error={recoverNowPlayingArtwork} /></div>
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
      <div class="immersive-backdrop" aria-hidden="true">
        {#key immersiveArtwork}<img src={immersiveArtwork || '/placeholder.svg'} alt="" in:fade={{ duration: 500 }} out:fade={{ duration: 360 }} on:error={recoverNowPlayingArtwork} />{/key}
      </div>
      <div class="immersive-glass-wash"></div>
      <header class="immersive-header">
        <button class="immersive-close" on:click={closeImmersivePlayer} aria-label="Close expanded player" title="Back to NOVA"><ChevronDown size={21} /></button>
        <div class="immersive-header-title"><span>NOW PLAYING</span><strong>NOVA MUSIC</strong></div>
        <button class="immersive-close fullscreen-exit" on:click={closeImmersivePlayer} aria-label="Exit full-screen player" title="Exit full-screen player"><Minimize2 size={17} /><span>Exit full screen</span></button>
      </header>

      <div class="immersive-content">
        <section class="immersive-playlist" aria-label="Playlist tracks">
          <div class="immersive-playlist-summary">
            <img src={collectionArtwork} alt="" on:error={recoverCollectionArtwork} />
            <div><span>PLAYING FROM</span><h1>{playbackCollection?.title || nowPlaying.album || 'Up Next'}</h1><p>{playbackCollection?.items?.length || immersiveTracks.length} songs · {nowPlaying.artist || 'YouTube Music'}</p></div>
          </div>
          <div class="immersive-playlist-tools">
            <button class="immersive-playlist-play" on:click={() => playbackCollection?.items?.length ? handlePlayTrack(playbackCollection.items[0]) : togglePlay()}><Play size={14} fill="currentColor" /> Play</button>
            <button class="immersive-tool-button" class:active={shuffleMode} on:click={toggleShuffleMode} aria-label={shuffleMode ? 'Turn shuffle off' : 'Turn shuffle on'} title="Shuffle"><Shuffle size={15} /></button>
            <button class="immersive-tool-button" on:click={downloadCurrentTrack} aria-label="Download current song" title="Download"><Download size={15} /></button>
          </div>
          <div class="immersive-track-scroll">
            {#each immersiveTracks as item, index (String(item?.videoId || item?.video_id || item?.track_id || item?.id || index))}
              {@const itemId = String(item?.videoId || item?.video_id || item?.track_id || item?.id || '')}
              <div class="immersive-song-row-wrap">
                <button class="immersive-song-row" class:playing={itemId === activeImmersiveTrackId} on:click={() => handlePlayTrack(item)}>
                  <span class="immersive-song-index">{itemId === activeImmersiveTrackId && isPlaying ? '♫' : index + 1}</span>
                  <span class="immersive-song-copy"><strong>{item?.title || item?.name || 'Untitled track'}</strong><small>{item?.artist || item?.artists || item?.subtitle || 'YouTube Music'}</small></span>
                  <span class="immersive-song-duration">{formatTrackDuration(item)}</span>
                </button>
                <TrackActions track={item} playlists={libraryCollections.playlists} onAction={handleTrackAction} />
              </div>
            {/each}
          </div>
        </section>

        <section class="immersive-art-column" aria-label="Track artwork and details">
          <div class="immersive-overline"><span><i></i> LIVE SESSION</span><span>{formatTime(playbackPosition)} <b>/</b> {formatTime(playbackDuration)}</span></div>
          <div class="immersive-track-info">
            <div><span class="immersive-label">PLAYING FROM YOUR MUSIC</span><h1>{nowPlaying.title || 'Unknown track'}</h1><p>{nowPlaying.artist || 'YouTube Music'}{nowPlaying.album ? ` · ${nowPlaying.album}` : ''}</p></div>
            <button class="immersive-like" class:liked={trackIsLiked(nowPlaying)} on:click={toggleLikeTrack} aria-label={trackIsLiked(nowPlaying) ? 'Remove from Liked Music' : 'Add to Liked Music'} title={trackIsLiked(nowPlaying) ? 'Remove from Liked Music' : 'Add to Liked Music'}><Heart size={22} fill={trackIsLiked(nowPlaying) ? 'currentColor' : 'none'} /></button>
          </div>
          <div class="immersive-next-up">
            <div><span class="immersive-label">UP NEXT</span><strong>{queuePreviewItems[0]?.title || 'End of queue'}</strong><small>{queuePreviewItems[0]?.artist || 'Add songs to keep listening'}</small></div>
            <button class="immersive-secondary-action" on:click={async () => { await closeImmersivePlayer(); sidePanel = 'queue'; await loadQueue(); }} aria-label={`Open queue, ${queueItems.length} tracks`} title="Open queue"><ListMusic size={16} /><span>{queueItems.length}</span></button>
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
            <button class="transport-button immersive-mode-button" class:active={shuffleMode} on:click={toggleShuffleMode} aria-label="Toggle shuffle" title="Shuffle"><Shuffle size={16} /></button>
            <button class="transport-button" on:click={prevTrack} aria-label="Previous track"><SkipBack size={19} fill="currentColor" /></button>
            <button class="transport-play immersive-play" on:click={togglePlay} aria-label={isPlaying ? 'Pause' : 'Play'} disabled={isLoadingTrack}>{#if isLoadingTrack}<span class="loader-dot"></span>{:else if isPlaying}<Pause size={23} fill="currentColor" />{:else}<Play size={23} fill="currentColor" />{/if}</button>
            <button class="transport-button" on:click={skipTrack} aria-label="Next track"><SkipForward size={19} fill="currentColor" /></button>
            <button class="transport-button immersive-mode-button" class:active={repeatMode > 0} on:click={cycleRepeatMode} aria-label="Change repeat mode" title={repeatMode === 1 ? 'Repeat one' : repeatMode === 2 ? 'Repeat all' : 'Repeat off'}><Repeat2 size={16} /><span>{repeatMode === 1 ? '1' : ''}</span></button>
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
          <img src={currentArtwork} alt="" on:error={recoverNowPlayingArtwork} />
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
        <button class="player-mode-button" class:active={shuffleMode} on:click={toggleShuffleMode} aria-label="Toggle shuffle" title="Shuffle"><Shuffle size={14} /></button>
        <button class="transport-button" on:click={prevTrack} aria-label="Previous track" title="Previous track"><SkipBack size={17} fill="currentColor" /></button>
        <button class="transport-play" on:click={togglePlay} aria-label={isPlaying ? 'Pause' : 'Play'} title={isPlaying ? 'Pause' : 'Play'} disabled={!nowPlaying || isLoadingTrack}>
          {#if isLoadingTrack}<span class="loader-dot"></span>{:else if isPlaying}<Pause size={19} fill="currentColor" />{:else}<Play size={19} fill="currentColor" />{/if}
        </button>
        <button class="transport-button" on:click={skipTrack} aria-label="Next track" title="Next track"><SkipForward size={17} fill="currentColor" /></button>
        <button class="player-mode-button" class:active={repeatMode > 0} on:click={cycleRepeatMode} aria-label="Change repeat mode" title={repeatMode === 1 ? 'Repeat one' : repeatMode === 2 ? 'Repeat all' : 'Repeat off'}><Repeat2 size={14} /><span>{repeatMode === 1 ? '1' : ''}</span></button>
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
      <button class="player-small-action" on:click={openImmersivePlayer} aria-label="Open expanded player" title="Expanded player"><Maximize2 size={16} /></button>
      <button class="player-small-action" class:selected={sidePanel === 'now-playing'} on:click={() => toggleSidePanel('now-playing')} aria-label="Now playing" title="Now playing"><PanelRight size={17} /></button>
      <button class="player-small-action" class:selected={sidePanel === 'lyrics'} on:click={() => toggleSidePanel('lyrics')} aria-label="Lyrics" title="Lyrics"><Mic2 size={17} /></button>
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
    --accent: var(--user-accent, rgb(var(--cover-rgb)));
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
  :global(.app-shell.theme-cyber), :global(.app-shell.theme-deepspace) { --accent: var(--user-accent, rgb(var(--cover-rgb))); --accent-soft: color-mix(in srgb, var(--accent) 16%, transparent); }
  :global(.app-shell.flat-backdrop) { background: #080909; }
  :global(.app-shell.gradient-backdrop) { background: radial-gradient(ellipse at 12% 0%, rgba(93,92,190,.22), transparent 45%), radial-gradient(ellipse at 100% 100%, rgba(20,143,146,.18), transparent 42%), #080909; }
  :global(.app-shell.reduced-motion *), :global(.app-shell.reduced-motion *::before), :global(.app-shell.reduced-motion *::after) { scroll-behavior: auto !important; animation-duration: .01ms !important; animation-iteration-count: 1 !important; transition-duration: .01ms !important; }

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
    box-shadow: inset 0 1px 0 rgba(255,255,255,.09), 0 8px 24px rgba(0,0,0,.1);
    -webkit-backdrop-filter: blur(26px) saturate(1.3);
    backdrop-filter: blur(26px) saturate(1.3);
  }
  .topbar-history, .topbar-actions { display: flex; align-items: center; gap: 8px; }
  .topbar-history { flex: 0 0 auto; }
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
  .downloads-wrap { position: relative; display: inline-flex; }
  .download-top-button { position: relative; }
  .download-top-button.active, .download-top-button.downloading { color: var(--accent); }
  .download-pulse { position: absolute; top: 4px; right: 4px; width: 7px; height: 7px; background: var(--accent); border: 1px solid #151616; border-radius: 50%; animation: download-pulse 1.25s ease-in-out infinite; }
  @keyframes download-pulse { 50% { box-shadow: 0 0 0 4px color-mix(in srgb, var(--accent) 20%, transparent); transform: scale(.82); } }
  .downloads-popover { position: absolute; z-index: 40; top: calc(100% + 12px); right: -95px; width: min(360px, calc(100vw - 30px)); max-height: min(460px, calc(100vh - 95px)); overflow: hidden; color: #f5f6f6; background: rgba(23,25,26,.97); border: 1px solid rgba(255,255,255,.11); border-radius: 14px; box-shadow: 0 22px 60px #0009; backdrop-filter: blur(28px) saturate(1.35); animation: download-panel-in .18s ease-out both; }
  .downloads-popover > header { height: 48px; display: flex; align-items: center; justify-content: space-between; padding: 0 15px; border-bottom: 1px solid rgba(255,255,255,.08); }
  .downloads-popover > header strong { font-size: 13px; }
  .downloads-popover > header span { color: #9da3a3; font-size: 10px; }
  .downloads-list { max-height: min(410px, calc(100vh - 145px)); overflow: auto; padding: 5px; }
  .download-job { min-height: 63px; display: flex; align-items: center; gap: 11px; padding: 9px; border-radius: 9px; }
  .download-job:hover { background: rgba(255,255,255,.05); }
  .download-job-icon { width: 34px; height: 34px; flex: 0 0 34px; display: grid; place-items: center; color: var(--accent); background: color-mix(in srgb, var(--accent) 14%, transparent); border-radius: 9px; }
  .download-job-icon.done { color: #70d8a2; background: rgba(112,216,162,.13); }
  .download-job-icon.failed { color: #ff8585; background: rgba(255,100,100,.13); }
  .download-job-copy { min-width: 0; flex: 1; display: flex; flex-direction: column; gap: 5px; }
  .download-job-copy strong { overflow: hidden; font-size: 11px; font-weight: 650; text-overflow: ellipsis; white-space: nowrap; }
  .download-job-copy small { overflow: hidden; color: #a1a7a7; font-size: 9px; text-overflow: ellipsis; white-space: nowrap; }
  .download-progress { height: 3px; overflow: hidden; background: #3a3e3e; border-radius: 4px; }
  .download-progress i { height: 100%; display: block; background: var(--accent); border-radius: inherit; transition: width .22s ease; }
  .download-check { color: #70d8a2; font-size: 16px; }
  .downloads-empty { margin: 0; padding: 28px 16px; color: #969d9d; text-align: center; font-size: 11px; }
  @keyframes download-panel-in { from { opacity: 0; transform: translateY(-5px) scale(.985); } to { opacity: 1; transform: none; } }
  .global-search-wrap { position: relative; width: min(460px, 42vw); margin: 0 auto; transition: width .38s cubic-bezier(.2,.8,.2,1), transform .38s cubic-bezier(.2,.8,.2,1); }
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
    transition: border-color .16s ease, background .16s ease, box-shadow .16s ease, height .32s cubic-bezier(.2,.8,.2,1), padding .32s cubic-bezier(.2,.8,.2,1);
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
  .topbar-actions { width: auto; max-width: 390px; min-width: 225px; flex: 0 1 auto; justify-content: flex-end; overflow: hidden; }
  .downloads-wrap { flex: 0 0 auto; }

  .workarea { min-height: 0; flex: 1; display: flex; overflow: hidden; gap: 0; }
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
  .icon-button { width: 30px; height: 30px; }
  .liked-art { color: white; background: linear-gradient(135deg, #f7838c, #ad5fce); }
  .queue-track-copy { min-width: 0; flex: 1; display: flex; flex-direction: column; gap: 3px; }
  .queue-track-copy strong { overflow: hidden; color: #e7e9e9; font-size: 12px; font-weight: 650; text-overflow: ellipsis; white-space: nowrap; }
  .queue-track-copy small { overflow: hidden; color: #929696; font-size: 10px; text-overflow: ellipsis; white-space: nowrap; }
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
  .queue-context strong { color: #f1f3f3; font-size: 13px; }
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
  .lyric-line { width: 100%; padding: 7px 8px; color: #858b89; background: transparent; border: 0; border-radius: 6px; font-size: var(--lyrics-size, 19px); line-height: 1.4; text-align: center; cursor: pointer; transition: color .2s ease, background .2s ease; }
  .lyric-line:hover, .lyric-line.current { color: #fff; background: rgba(255,255,255,.055); }
  .lyric-line.current { color: var(--accent); font-weight: 700; }
  .lyrics-plain { width: 100%; min-height: 0; flex: 1; overflow: auto; margin-top: 18px; color: #d4d7d6; font-size: var(--lyrics-size, 19px); line-height: 1.55; white-space: pre-wrap; }
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
  .immersive-close.fullscreen-exit { width: auto; min-width: 40px; gap: 8px; padding: 0 13px; border-radius: 22px; }
  .fullscreen-exit span { font-size: 10px; white-space: nowrap; }
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
  .immersive-track-info { width: min(100%, 500px); display: flex; align-items: center; gap: 16px; padding: 22px 2px 0; }
  .immersive-track-info > div { min-width: 0; flex: 1; }
  .immersive-track-info h1 { overflow: hidden; margin: 7px 0 5px; color: #fff; font-size: clamp(23px, 2.6vw, 34px); line-height: 1.1; letter-spacing: -.045em; text-overflow: ellipsis; white-space: nowrap; }
  .immersive-track-info p { overflow: hidden; margin: 0; color: rgba(240,235,238,.68); font-size: 14px; text-overflow: ellipsis; white-space: nowrap; }
  .immersive-like { width: 43px; height: 43px; flex: 0 0 43px; display: grid; place-items: center; color: rgba(255,255,255,.78); background: rgba(255,255,255,.08); border: 1px solid rgba(255,255,255,.12); border-radius: 50%; cursor: pointer; transition: color .18s ease, background .18s ease, transform .18s ease; }
  .immersive-like:hover { color: #fff; background: rgba(255,255,255,.16); transform: scale(1.04); }
  .immersive-like.liked { color: var(--accent); }
  .immersive-like.compact { width: 37px; height: 37px; flex-basis: 37px; background: transparent; border-color: transparent; }
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
  .immersive-plain-lyrics { color: rgba(255,255,255,.82); font-size: var(--lyrics-size, 19px); line-height: 1.65; white-space: pre-wrap; }
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
  .player-progress input, .volume-control input { height: 4px; appearance: none; -webkit-appearance: none; border: 0; border-radius: 6px; outline: none; cursor: pointer; background: linear-gradient(to right, #f2f4f4 var(--progress), #444848 var(--progress)); transition: height .18s ease, filter .18s ease; }
  .player-progress input:hover { height: 6px; filter: drop-shadow(0 0 5px rgba(255,255,255,.16)); }
  .player-progress input { min-width: 0; flex: 1; }
  .player-progress input::-webkit-slider-thumb, .volume-control input::-webkit-slider-thumb { width: 0; height: 0; appearance: none; -webkit-appearance: none; background: #f7f8f8; border: 0; border-radius: 50%; box-shadow: 0 0 0 3px color-mix(in srgb, var(--accent) 25%, transparent); transition: width .18s cubic-bezier(.2,.8,.2,1), height .18s cubic-bezier(.2,.8,.2,1), opacity .16s ease; opacity: 0; }
  .player-progress input:hover::-webkit-slider-thumb, .player-progress input:focus-visible::-webkit-slider-thumb, .volume-control input:hover::-webkit-slider-thumb { width: 11px; height: 11px; opacity: 1; }
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
    .topbar-actions { min-width: 100px; }
    .player-bar { grid-template-columns: minmax(145px,1fr) minmax(260px,1.25fr) minmax(110px,1fr); padding-inline: 10px; }
    .player-extras { gap: 3px; }
    .volume-control input { display: none; }
  }
  @media (max-height: 700px) {
    .immersive-header { height: 56px; flex-basis: 56px; }
    .immersive-content { padding-top: 4px; padding-bottom: 8px; gap: 26px; }
    .immersive-lyrics { padding: 20px 24px; }
    .immersive-lyric { padding: 7px 0; font-size: clamp(16px, 1.8vw, 22px); }
    .immersive-controls { padding-bottom: max(12px, env(safe-area-inset-bottom)); }
  }
  @media (prefers-reduced-motion: reduce) {
    :global(*), :global(*::before), :global(*::after) { scroll-behavior: auto !important; animation-duration: .01ms !important; transition-duration: .01ms !important; }
  }
</style>
