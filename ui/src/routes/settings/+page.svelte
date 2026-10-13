<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { getVersion } from '@tauri-apps/api/app';
  import { listen } from '@tauri-apps/api/event';
  import { open } from '@tauri-apps/plugin-dialog';
  import { check } from '@tauri-apps/plugin-updater';
  import { relaunch } from '@tauri-apps/plugin-process';
  import {
    Settings2, Palette, SlidersHorizontal, Keyboard, HardDrive, Info, UserRound,
    X, History, Monitor, Bell, Power, FolderOpen, Download, ShieldCheck,
    ExternalLink, AudioLines, Database, RefreshCw, CheckCircle2, Loader2, Play,
    Music2, Home, Library, Search, Sparkles, ListMusic, Mic2, HelpCircle,
  } from 'lucide-svelte';

  export let onClose: () => void = () => {};

  type SettingsSection = 'General' | 'Appearance' | 'Playback' | 'Audio & Quality' | 'YouTube Music' | 'Library & Playlists' | 'Downloads & Local Music' | 'Lyrics' | 'Notifications' | 'Keyboard Shortcuts' | 'Privacy & Security' | 'Advanced' | 'About NOVA';
  let activeSection: SettingsSection = 'General';
  const sections: { name: SettingsSection; icon: any }[] = [
    { name: 'General', icon: Settings2 },
    { name: 'Appearance', icon: Palette },
    { name: 'Playback', icon: Music2 },
    { name: 'Audio & Quality', icon: AudioLines },
    { name: 'YouTube Music', icon: UserRound },
    { name: 'Library & Playlists', icon: Library },
    { name: 'Downloads & Local Music', icon: Download },
    { name: 'Lyrics', icon: Mic2 },
    { name: 'Notifications', icon: Bell },
    { name: 'Keyboard Shortcuts', icon: Keyboard },
    { name: 'Privacy & Security', icon: ShieldCheck },
    { name: 'Advanced', icon: SlidersHorizontal },
    { name: 'About NOVA', icon: Info },
  ];
  let settings: Record<string, string> = {};
  let isLoading = false;
  let isSuccess = false;
  let authMessage = '';
  let accountName = '';
  let accountEmail = '';
  let accountSignedIn = false;
  let googleOAuthClientId = '';
  let cookieInput = '';
  let audioQuality = 'high';
  let downloadQuality = 'high';
  let downloadFormat = 'original';
  let replayGain = true;
  let gaplessPlayback = true;
  let autoPlayNext = true;
  let playbackSpeed = 1;
  let volume = 85;
  let cosmicTheme = 'nebula';
  let accentColor = '#f46b82';
  let appearanceBackdrop = 'artwork';
  let motionPreference = 'full';
  let appearanceDensity = 'comfortable';
  let downloadDirectory = '';
  let storageInfo: any = null;
  let optimizingDatabase = false;
  let historyEnabled = true;
  let closeToTray = false;
  let songNotifications = false;
  let launchAtStartup = false;
  let startPage = 'home';
  let repeatMode = 0;
  let shuffleMode = false;
  let autoOpenLyrics = false;
  let lyricsFontSize = 19;
  let sidebarVisible = true;
  let confirmClearHistory = false;
  let appVersion = '1.3.0';
  let availableUpdate: any = null;
  let updateBusy = false;
  let updateProgress = 0;
  let updateProgressKnown = false;
  let updateBytesDownloaded = 0;
  let updateContentLength = 0;
  let updateMessage = '';
  let updateError = false;
  let disposed = false;
  let stopListening: (() => void) | undefined;

  const sectionDescriptions: Record<SettingsSection, string> = {
    General: 'Set how NOVA opens and behaves on your Mac.',
    Appearance: 'Choose the look and motion used throughout NOVA.',
    Playback: 'Set the behavior you use while listening.',
    'Audio & Quality': 'Choose stream quality and audio processing.',
    'YouTube Music': 'Manage your account and the features it exposes to NOVA.',
    'Library & Playlists': 'Manage local listening history and saved collections.',
    'Downloads & Local Music': 'Choose download settings and review local storage.',
    Lyrics: 'Set how lyrics appear while music plays.',
    Notifications: 'Choose which playback events NOVA sends to macOS.',
    'Keyboard Shortcuts': 'Review shortcuts that are active while NOVA is focused.',
    'Privacy & Security': 'Control local history and your account connection.',
    Advanced: 'Review diagnostics and maintain NOVA’s local database.',
    'About NOVA': 'Version, updates, project links, and support.',
  };

  function setAccount(name?: string, email?: string) {
    accountName = name || 'YouTube User';
    accountEmail = email || '';
    accountSignedIn = true;
  }

  function applyTheme(theme: string) {
    const shell = document.querySelector('.app-shell');
    shell?.classList.remove('theme-nebula', 'theme-cyber', 'theme-deepspace');
    shell?.classList.add(`theme-${theme}`);
  }

  onMount(() => {
    disposed = false;
    getVersion().then((version) => { if (!disposed) appVersion = version; }).catch(() => {});
    listen<any>('google-auth-complete', async ({ payload }) => {
      if (!payload?.ok) {
        isSuccess = false;
        authMessage = payload?.error || 'Google sign-in did not finish.';
        return;
      }
      setAccount(payload.name, payload.email);
      isSuccess = true;
      authMessage = 'Google account connected. Refreshing your YouTube Music library…';
      const [libraryResult, homeResult] = await Promise.allSettled([
        invoke('get_library'), invoke('get_home', { params: null }),
      ]);
      const library: any = libraryResult.status === 'fulfilled' ? libraryResult.value : null;
      const playlists = Array.isArray(library) ? library : (library?.playlists || []);
      authMessage = `Connected as ${accountName}. Loaded ${playlists.length} playlist${playlists.length === 1 ? '' : 's'}${homeResult.status === 'fulfilled' ? ' and refreshed Home recommendations' : ''}.`;
      window.dispatchEvent(new CustomEvent('nova-account-updated'));
    }).then((unlisten) => {
      if (disposed) unlisten(); else stopListening = unlisten;
    });

    (async () => {
      try {
        const [loadedSettings, account, autostart]: any[] = await Promise.all([
          invoke('get_settings'), invoke('get_account_info'), invoke('plugin:autostart|is_enabled').catch(() => false),
        ]);
        if (disposed) return;
        settings = loadedSettings || {};
        audioQuality = settings.quality || 'high';
        downloadQuality = settings.download_quality || settings.quality || 'high';
        downloadFormat = settings.download_format || 'original';
        replayGain = settings.replay_gain !== 'false';
        gaplessPlayback = settings.gapless !== 'false';
        autoPlayNext = settings.auto_play_next !== 'false';
        playbackSpeed = Number(settings.playback_speed || 1);
        volume = Number(settings.volume || 85);
        cosmicTheme = settings.cosmic_theme || 'nebula';
        accentColor = settings.accent_color || '#f46b82';
        appearanceBackdrop = settings.appearance_backdrop || 'artwork';
        motionPreference = settings.motion_preference || 'full';
        appearanceDensity = settings.appearance_density || 'comfortable';
        downloadDirectory = settings.download_directory || '';
        googleOAuthClientId = settings.google_oauth_client_id || '';
        historyEnabled = settings.history_enabled !== 'false';
        closeToTray = settings.close_to_tray === 'true';
        songNotifications = settings.song_notifications === 'true';
        launchAtStartup = Boolean(autostart);
        startPage = settings.start_page || 'home';
        repeatMode = Math.max(0, Math.min(2, Number(settings.repeat_mode || 0)));
        shuffleMode = settings.shuffle_mode === 'true';
        autoOpenLyrics = settings.auto_open_lyrics === 'true';
        lyricsFontSize = Math.max(14, Math.min(28, Number(settings.lyrics_font_size || 19)));
        sidebarVisible = settings.sidebar_visible !== 'false';
        accountSignedIn = Boolean(account?.is_logged_in);
        accountName = account?.name || '';
        accountEmail = account?.email || '';
        applyTheme(cosmicTheme);
        void refreshStorageInfo();
      } catch (error) {
        console.error('Could not load settings:', error);
      }
    })();

    return () => {
      disposed = true;
      stopListening?.();
    };
  });

  async function saveSetting(key: string, value: string | number | boolean) {
    try {
      await invoke('update_settings', { key, value: String(value) });
      settings = { ...settings, [key]: String(value) };
      if (key === 'close_to_tray') {
        window.dispatchEvent(new CustomEvent('nova-close-to-background-changed', { detail: String(value) === 'true' }));
      }
      if (key === 'auto_play_next') {
        window.dispatchEvent(new CustomEvent('nova-autoplay-changed', { detail: String(value) === 'true' }));
      }
      if (['accent_color', 'appearance_backdrop', 'motion_preference', 'cosmic_theme', 'appearance_density', 'lyrics_font_size', 'sidebar_visible'].includes(key)) {
        window.dispatchEvent(new CustomEvent('nova-appearance-changed', { detail: { key, value: String(value) } }));
      }
      if (['repeat_mode', 'shuffle_mode'].includes(key)) {
        window.dispatchEvent(new CustomEvent('nova-playback-settings-changed', {
          detail: { repeatMode, shuffleMode },
        }));
      }
      authMessage = '';
      return true;
    } catch (error: any) {
      authMessage = error?.message || String(error);
      isSuccess = false;
      return false;
    }
  }

  async function toggleStartup() {
    try {
      const next = !launchAtStartup;
      await invoke(next ? 'plugin:autostart|enable' : 'plugin:autostart|disable');
      launchAtStartup = next;
      await saveSetting('launch_at_startup', next);
      isSuccess = true;
      authMessage = next ? 'NOVA will open when you sign in to your Mac.' : 'NOVA will no longer open at login.';
    } catch (error: any) {
      isSuccess = false;
      authMessage = `Could not change startup behavior: ${error?.message || error}`;
    }
  }

  async function changeTheme() {
    applyTheme(cosmicTheme);
    await saveSetting('cosmic_theme', cosmicTheme);
  }

  async function refreshAccountData() {
    try {
      isLoading = true;
      authMessage = 'Refreshing account and library data…';
      const [account, library, home]: any[] = await Promise.all([
        invoke('get_account_info'), invoke('get_library'), invoke('get_home', { params: null }),
      ]);
      accountSignedIn = Boolean(account?.is_logged_in);
      accountName = account?.name || '';
      accountEmail = account?.email || '';
      const playlists = Array.isArray(library) ? library : (library?.playlists || []);
      window.dispatchEvent(new CustomEvent('nova-account-updated'));
      window.dispatchEvent(new CustomEvent('nova-refresh-current-view'));
      authMessage = `Account refreshed. ${playlists.length} playlist${playlists.length === 1 ? '' : 's'} available${home ? '; Home picks refreshed' : ''}.`;
      isSuccess = true;
    } catch (error: any) {
      authMessage = `Could not refresh account data: ${error?.message || error}`;
      isSuccess = false;
    } finally {
      isLoading = false;
    }
  }

  async function clearLocalPlaybackHistory() {
    try {
      await invoke('clear_local_playback_history');
      confirmClearHistory = false;
      window.dispatchEvent(new CustomEvent('nova-refresh-current-view'));
      authMessage = 'NOVA local playback history cleared. YouTube account history and playlists were not changed.';
      isSuccess = true;
    } catch (error: any) {
      authMessage = error?.message || String(error);
      isSuccess = false;
    }
  }

  async function chooseDownloadFolder() {
    try {
      const selected = await open({ directory: true, multiple: false, title: 'Choose music download folder' });
      if (typeof selected === 'string') {
        downloadDirectory = selected;
        await saveSetting('download_directory', selected);
      }
    } catch (error: any) {
      authMessage = `Could not choose that folder: ${error?.message || error}`;
      isSuccess = false;
    }
  }

  async function downloadCurrentTrack() {
    try {
      isLoading = true;
      const path: string = await invoke('download_current_track');
      authMessage = `Downloaded to ${path}`;
      isSuccess = true;
    } catch (error: any) {
      authMessage = error?.message || String(error);
      isSuccess = false;
    } finally {
      isLoading = false;
    }
  }

  async function refreshStorageInfo() {
    try { storageInfo = await invoke('get_storage_info'); }
    catch (_) { storageInfo = null; }
  }

  async function optimizeDatabase() {
    try {
      optimizingDatabase = true;
      await invoke('optimize_database');
      authMessage = 'Local database optimized. Your downloads and account are unchanged.';
      isSuccess = true;
      await refreshStorageInfo();
    } catch (error: any) {
      authMessage = `Could not optimize local data: ${error?.message || error}`;
      isSuccess = false;
    } finally {
      optimizingDatabase = false;
    }
  }

  function formatBytes(value: number) {
    if (!Number.isFinite(value) || value <= 0) return '0 B';
    const units = ['B', 'KB', 'MB', 'GB'];
    const power = Math.min(Math.floor(Math.log(value) / Math.log(1024)), units.length - 1);
    return `${(value / Math.pow(1024, power)).toFixed(power === 0 ? 0 : 1)} ${units[power]}`;
  }

  async function startGoogleSignIn() {
    try {
      isLoading = true;
      authMessage = '';
      await invoke('start_google_sign_in');
      authMessage = 'Google sign-in opened in your browser. Choose an account and allow access to return to NOVA.';
      isSuccess = true;
    } catch (error: any) {
      authMessage = `Could not start Google sign-in: ${error?.message || error}`;
      isSuccess = false;
    } finally {
      isLoading = false;
    }
  }

  async function saveGoogleOAuthClientId() {
    try {
      const saved = await saveSetting('google_oauth_client_id', googleOAuthClientId.trim());
      if (!saved) return;
      authMessage = 'OAuth client ID saved. Add your Google account as a test user in that project before signing in.';
      isSuccess = true;
    } catch (error: any) {
      authMessage = error?.message || String(error);
      isSuccess = false;
    }
  }

  async function saveCookieAuth() {
    if (!cookieInput.trim()) return;
    try {
      isLoading = true;
      authMessage = '';
      const result: any = await invoke('sign_in', { cookie: cookieInput.trim() });
      if (result?.status === 'logged_in') {
        setAccount(result.name, result.email);
        cookieInput = '';
        isSuccess = true;
        authMessage = `Connected to YouTube Music as ${accountName}.`;
        window.dispatchEvent(new CustomEvent('nova-account-updated'));
      }
    } catch (error: any) {
      authMessage = `Could not connect that session: ${error?.message || error}`;
      isSuccess = false;
    } finally {
      isLoading = false;
    }
  }

  async function signOut() {
    try {
      await invoke('sign_out');
      accountSignedIn = false;
      accountName = '';
      accountEmail = '';
      authMessage = 'Signed out of YouTube Music.';
      isSuccess = true;
      window.dispatchEvent(new CustomEvent('nova-account-updated'));
    } catch (error: any) {
      authMessage = `Could not sign out: ${error?.message || error}`;
      isSuccess = false;
    }
  }

  async function checkForUpdates() {
    updateBusy = true;
    updateMessage = 'Checking GitHub for a newer version…';
    updateError = false;
    availableUpdate = null;
    try {
      availableUpdate = await check();
      updateMessage = availableUpdate
        ? `NOVA ${availableUpdate.version} is ready to install.`
        : 'You’re up to date.';
    } catch (error: any) {
      updateError = true;
      updateMessage = `Could not check for updates: ${error?.message || error}`;
    } finally {
      updateBusy = false;
    }
  }

  async function installUpdate() {
    if (!availableUpdate) return;
    updateBusy = true;
    updateProgress = 0;
    updateProgressKnown = false;
    updateBytesDownloaded = 0;
    updateContentLength = 0;
    updateError = false;
    updateMessage = `Downloading NOVA ${availableUpdate.version}…`;
    try {
      await availableUpdate.downloadAndInstall((event: any) => {
        if (event.event === 'Started') {
          updateContentLength = event.data.contentLength || 0;
          updateProgressKnown = updateContentLength > 0;
        } else if (event.event === 'Progress') {
          updateBytesDownloaded += event.data.chunkLength;
          if (updateContentLength > 0) {
            updateProgressKnown = true;
            updateProgress = Math.min(100, Math.round((updateBytesDownloaded / updateContentLength) * 100));
          }
        } else if (event.event === 'Finished') {
          updateProgress = 100;
        }
      });
      updateMessage = 'Update installed. Restarting NOVA…';
      await relaunch();
    } catch (error: any) {
      updateError = true;
      updateMessage = `Could not install the update: ${error?.message || error}`;
    } finally {
      updateBusy = false;
    }
  }
</script>

<div class="settings-window">
  <aside class="settings-nav">
    <div class="settings-nav-head"><span class="settings-brand-mark"><Sparkles size={16} /></span><div><h2>Settings</h2><small>NOVA Music</small></div><button class="close-settings mobile-close" on:click={onClose} aria-label="Close settings"><X size={17} /></button></div>
    <nav aria-label="Settings categories">
      {#each sections as section (section.name)}
        <button class:active={activeSection === section.name} on:click={() => activeSection = section.name} aria-current={activeSection === section.name ? 'page' : undefined}>
          <svelte:component this={section.icon} size={16} strokeWidth={1.8} /><span>{section.name}</span>
        </button>
      {/each}
    </nav>
    <div class="settings-version">NOVA MUSIC <span>v{appVersion}</span></div>
  </aside>

  <main class="settings-main">
    <header class="settings-main-head">
      <div><span class="settings-eyebrow">PREFERENCES</span><h1>{activeSection}</h1><p>{sectionDescriptions[activeSection]}</p></div>
      <button class="close-settings desktop-close" on:click={onClose} aria-label="Close settings"><X size={18} /></button>
    </header>

    {#if authMessage}
      <div class="settings-message" class:success={isSuccess} role="status" aria-live="polite">{authMessage}</div>
    {/if}

    <div class="settings-scroll">
      {#if activeSection === 'General'}
        <div class="settings-group-label">STARTUP</div>
        <div class="setting-row"><div class="setting-icon"><Home size={17} /></div><div class="setting-copy"><strong>Start page</strong><small>Open NOVA to the page you use most.</small></div><select bind:value={startPage} on:change={() => saveSetting('start_page', startPage)} aria-label="Start page"><option value="home">Home</option><option value="library">Library</option><option value="search">Search</option></select></div>
        <div class="setting-row"><div class="setting-icon"><Monitor size={17} /></div><div class="setting-copy"><strong>Minimize on close</strong><small>Keep NOVA running and minimize its window to the Dock.</small></div><button class="switch" class:enabled={closeToTray} role="switch" aria-checked={closeToTray} on:click={() => { closeToTray = !closeToTray; saveSetting('close_to_tray', closeToTray); }}><span></span></button></div>
        <div class="setting-row last-row"><div class="setting-icon"><Power size={17} /></div><div class="setting-copy"><strong>Launch at login</strong><small>Open NOVA automatically when you sign in to your Mac.</small></div><button class="switch" class:enabled={launchAtStartup} role="switch" aria-checked={launchAtStartup} on:click={toggleStartup}><span></span></button></div>
        <div class="settings-group-label">APP BEHAVIOR</div>
        <div class="info-card"><HelpCircle size={18} /><p>Closing the NOVA window either hides it in the Dock or quits the app. NOVA supports English only in this build; unsupported language and menu bar controls are not shown.</p></div>

      {:else if activeSection === 'Appearance'}
        <div class="settings-group-label">THEME</div>
        <div class="theme-choice-grid">
          <button class:chosen={cosmicTheme === 'nebula'} class="theme-choice obsidian" on:click={() => { cosmicTheme = 'nebula'; void changeTheme(); }}><span class="theme-swatch"><i></i><b></b><em></em></span><strong>NOVA Obsidian</strong><small>Deep charcoal surfaces</small></button>
          <button class:chosen={cosmicTheme === 'cyber'} class="theme-choice aurora" on:click={() => { cosmicTheme = 'cyber'; void changeTheme(); }}><span class="theme-swatch"><i></i><b></b><em></em></span><strong>NOVA Aurora</strong><small>Cool blue and teal glow</small></button>
          <button class:chosen={cosmicTheme === 'deepspace'} class="theme-choice classic" on:click={() => { cosmicTheme = 'deepspace'; void changeTheme(); }}><span class="theme-swatch"><i></i><b></b><em></em></span><strong>NOVA Classic</strong><small>Quiet neutral styling</small></button>
        </div>
        <div class="settings-group-label">COLOR AND BACKDROP</div>
        <div class="setting-row"><div class="setting-icon"><Palette size={17} /></div><div class="setting-copy"><strong>Accent color</strong><small>Apply the color to buttons and playback controls.</small></div><select bind:value={accentColor} on:change={() => saveSetting('accent_color', accentColor)} aria-label="Accent color"><option value="#f46b82">Rose</option><option value="#51d6b2">Aurora green</option><option value="#5bc4ee">Arctic cyan</option><option value="#a78bfa">Violet</option><option value="#f2b75e">Amber</option><option value="#eeeeee">Monochrome</option></select></div>
        <div class="setting-row"><div class="setting-icon"><Palette size={17} /></div><div class="setting-copy"><strong>Custom accent</strong><small>Fine tune NOVA’s highlight color.</small></div><div class="accent-picker"><input type="color" bind:value={accentColor} on:change={() => saveSetting('accent_color', accentColor)} aria-label="Custom accent color" /><span>{accentColor.toUpperCase()}</span></div></div>
        <div class="setting-row"><div class="setting-icon"><Monitor size={17} /></div><div class="setting-copy"><strong>App background</strong><small>Artwork glow, Aurora gradient, or a flat dark canvas.</small></div><select bind:value={appearanceBackdrop} on:change={() => saveSetting('appearance_backdrop', appearanceBackdrop)} aria-label="Background style"><option value="artwork">Artwork glow</option><option value="gradient">Aurora gradient</option><option value="flat">Minimal dark</option></select></div>
        <div class="settings-group-label">LAYOUT</div>
        <div class="setting-row"><div class="setting-icon"><SlidersHorizontal size={17} /></div><div class="setting-copy"><strong>Interface density</strong><small>Adjust spacing in navigation and music shelves.</small></div><select bind:value={appearanceDensity} on:change={() => saveSetting('appearance_density', appearanceDensity)} aria-label="Interface density"><option value="comfortable">Comfortable</option><option value="compact">Compact</option></select></div>
        <div class="setting-row"><div class="setting-icon"><Library size={17} /></div><div class="setting-copy"><strong>Show sidebar</strong><small>Keep the music navigation visible.</small></div><button class="switch" class:enabled={sidebarVisible} role="switch" aria-checked={sidebarVisible} on:click={() => { sidebarVisible = !sidebarVisible; saveSetting('sidebar_visible', sidebarVisible); }}><span></span></button></div>
        <div class="setting-row last-row"><div class="setting-icon"><AudioLines size={17} /></div><div class="setting-copy"><strong>Motion effects</strong><small>Reduce transitions and animation across NOVA.</small></div><select bind:value={motionPreference} on:change={() => saveSetting('motion_preference', motionPreference)} aria-label="Motion effects"><option value="full">Full motion</option><option value="reduced">Reduced motion</option></select></div>

      {:else if activeSection === 'Playback'}
        <div class="settings-group-label">QUEUE BEHAVIOR</div>
        <div class="setting-row"><div class="setting-icon"><Play size={17} /></div><div class="setting-copy"><strong>Autoplay recommendations</strong><small>Continue with YouTube Music up-next picks when the queue ends.</small></div><button class="switch" class:enabled={autoPlayNext} role="switch" aria-checked={autoPlayNext} on:click={() => { autoPlayNext = !autoPlayNext; saveSetting('auto_play_next', autoPlayNext); }}><span></span></button></div>
        <div class="setting-row"><div class="setting-icon"><Sparkles size={17} /></div><div class="setting-copy"><strong>Shuffle collection</strong><small>Randomize tracks while playing a playlist or album.</small></div><button class="switch" class:enabled={shuffleMode} role="switch" aria-checked={shuffleMode} on:click={() => { shuffleMode = !shuffleMode; saveSetting('shuffle_mode', shuffleMode); }}><span></span></button></div>
        <div class="setting-row last-row"><div class="setting-icon"><ListMusic size={17} /></div><div class="setting-copy"><strong>Repeat</strong><small>{repeatMode === 1 ? 'Repeat the current track.' : repeatMode === 2 ? 'Repeat the current playlist or album.' : 'Stop at the end of the current collection.'}</small></div><select bind:value={repeatMode} on:change={() => saveSetting('repeat_mode', repeatMode)} aria-label="Repeat mode"><option value={0}>Off</option><option value={1}>Repeat track</option><option value={2}>Repeat collection</option></select></div>
        <div class="settings-group-label">PLAYER</div>
        <div class="setting-row"><div class="setting-icon"><AudioLines size={17} /></div><div class="setting-copy"><strong>Gapless playback</strong><small>Reduce silence between consecutive tracks.</small></div><button class="switch" class:enabled={gaplessPlayback} role="switch" aria-checked={gaplessPlayback} on:click={() => { gaplessPlayback = !gaplessPlayback; saveSetting('gapless', gaplessPlayback); }}><span></span></button></div>
        <div class="setting-row"><div class="setting-icon"><SlidersHorizontal size={17} /></div><div class="setting-copy"><strong>Playback speed</strong><small>Apply the speed to the current and next track.</small></div><select bind:value={playbackSpeed} on:change={() => saveSetting('playback_speed', playbackSpeed)} aria-label="Playback speed"><option value={0.5}>0.5×</option><option value={0.75}>0.75×</option><option value={1}>Normal</option><option value={1.25}>1.25×</option><option value={1.5}>1.5×</option><option value={1.75}>1.75×</option><option value={2}>2×</option></select></div>
        <div class="info-card"><HelpCircle size={18} /><p>Queue restoration after quitting and audio crossfade are not available in this build. Repeat and shuffle stay synchronized with the player controls.</p></div>

      {:else if activeSection === 'Audio & Quality'}
        <div class="settings-group-label">STREAMING</div>
        <div class="setting-row"><div class="setting-icon"><AudioLines size={17} /></div><div class="setting-copy"><strong>Audio stream quality</strong><small>Choose the preferred YouTube Music stream quality.</small></div><select bind:value={audioQuality} on:change={() => saveSetting('quality', audioQuality)} aria-label="Streaming quality"><option value="auto">Automatic</option><option value="high">High</option><option value="medium">Standard</option><option value="low">Data saver</option></select></div>
        <div class="settings-group-label">AUDIO PROCESSING</div>
        <div class="setting-row"><div class="setting-icon"><AudioLines size={17} /></div><div class="setting-copy"><strong>Volume normalization</strong><small>Use source loudness data to balance levels when available.</small></div><button class="switch" class:enabled={replayGain} role="switch" aria-checked={replayGain} on:click={() => { replayGain = !replayGain; saveSetting('replay_gain', replayGain); }}><span></span></button></div>
        <div class="setting-row last-row"><div class="setting-icon"><Music2 size={17} /></div><div class="setting-copy"><strong>Default volume</strong><small>Set the player volume now and use it on next launch.</small></div><div class="setting-range"><input type="range" min="0" max="100" bind:value={volume} on:change={() => saveSetting('volume', volume)} aria-label="Default volume" /><span>{volume}%</span></div></div>
        <div class="info-card"><HelpCircle size={18} /><p>NOVA follows the stream variants provided by YouTube Music. Audio output device selection and exclusive audio mode are not exposed by the current playback engine.</p></div>

      {:else if activeSection === 'YouTube Music'}
        <div class="settings-group-label">ACCOUNT</div>
        <section class="account-card profile-card">
          <div class="account-heading"><div class="account-icon profile-avatar-large">{#if accountSignedIn}{accountName.charAt(0).toUpperCase() || 'N'}{:else}<UserRound size={18} />{/if}</div><div><strong>{accountSignedIn ? (accountName || 'YouTube Music account') : 'Not connected'}</strong><small>{accountSignedIn ? (accountEmail || 'Account session active') : 'Connect to load your library, playlists, and account history.'}</small></div>{#if accountSignedIn}<span class="connection-status connected">CONNECTED</span>{/if}</div>
          {#if accountSignedIn}
            <div class="account-actions"><button class="secondary-button" on:click={refreshAccountData} disabled={isLoading}><RefreshCw size={14} class={isLoading ? 'spinning' : ''} /> Refresh account data</button><button class="text-button" on:click={signOut}>Disconnect</button></div>
          {:else}
            <p class="account-note">Sign-in opens Google’s supported authorization flow. NOVA does not store Google passwords; account access depends on the session and YouTube Music capabilities available to this integration.</p>
            <label class="field-label" for="oauth-id">Desktop OAuth client ID</label>
            <div class="inline-field"><input id="oauth-id" bind:value={googleOAuthClientId} placeholder="Client ID ending in .apps.googleusercontent.com" autocomplete="off" spellcheck="false" /><button class="secondary-button" on:click={saveGoogleOAuthClientId}>Save</button></div>
            <button class="primary-button oauth-button" on:click={startGoogleSignIn} disabled={isLoading}>↗ {isLoading ? 'Opening Google…' : 'Continue with Google'}</button>
            <details class="cookie-details"><summary>Connect with an existing browser session</summary><p>A browser session cookie is a credential. It stays on this device; paste it only into this local field.</p><textarea bind:value={cookieInput} placeholder="Session cookie value" rows="3" autocomplete="off" spellcheck="false"></textarea><button class="secondary-button" on:click={saveCookieAuth} disabled={isLoading || !cookieInput}>{isLoading ? 'Connecting…' : 'Connect session'}</button></details>
          {/if}
        </section>
        <div class="settings-group-label">CAPABILITIES</div>
        <div class="capability-list"><div><span class="capability-dot available"></span><strong>Search and playback</strong><small>Available through NOVA’s YouTube Music integration.</small></div><div><span class="capability-dot" class:available={accountSignedIn}></span><strong>Personal library and playlists</strong><small>{accountSignedIn ? 'Loads from the active account session.' : 'Connect an account to access your personal library.'}</small></div><div><span class="capability-dot" class:available={accountSignedIn}></span><strong>Account history</strong><small>{accountSignedIn ? 'Read when the current session grants access.' : 'Unavailable until an account is connected.'}</small></div></div>
        <div class="info-card"><ShieldCheck size={18} /><p>Google sign-in does not grant every YouTube Music feature. NOVA does not provide periodic background sync or account-side playlist editing controls in Settings.</p></div>

      {:else if activeSection === 'Library & Playlists'}
        <div class="settings-group-label">LISTENING ACTIVITY</div>
        <div class="setting-row last-row"><div class="setting-icon"><History size={17} /></div><div class="setting-copy"><strong>Listening history</strong><small>Save NOVA plays locally and use listening activity for Home picks.</small></div><button class="switch" class:enabled={historyEnabled} role="switch" aria-checked={historyEnabled} on:click={() => { historyEnabled = !historyEnabled; void saveSetting('history_enabled', historyEnabled); window.dispatchEvent(new CustomEvent('nova-refresh-current-view')); }}><span></span></button></div>
        <div class="settings-group-label">YOUR COLLECTION</div>
        <div class="setting-row"><div class="setting-icon"><Library size={17} /></div><div class="setting-copy"><strong>Library access</strong><small>{accountSignedIn ? 'Your connected library is available from the sidebar.' : 'Connect YouTube Music to load account playlists and likes.'}</small></div><span class="connection-status">{accountSignedIn ? 'ACCOUNT' : 'LOCAL'}</span></div>
        <div class="setting-row last-row"><div class="setting-icon"><ListMusic size={17} /></div><div class="setting-copy"><strong>Local playlists</strong><small>NOVA keeps playlists you create on this device.</small></div><button class="secondary-button" on:click={() => { onClose(); window.dispatchEvent(new CustomEvent('nova-open-library')); }}>Open library</button></div>
        <div class="info-card"><HelpCircle size={18} /><p>NOVA history, the YouTube account history, and your library are separate. Turning off local history stops new local records and listening-based Home recommendations.</p></div>

      {:else if activeSection === 'Downloads & Local Music'}
        <div class="settings-group-label">DOWNLOADS</div>
        <div class="setting-row"><div class="setting-icon"><FolderOpen size={17} /></div><div class="setting-copy"><strong>Download folder</strong><small class="path-text">{downloadDirectory || '~/Music/NOVA Music Downloads'}</small></div><button class="secondary-button" on:click={chooseDownloadFolder}>Choose folder</button></div>
        <div class="setting-row"><div class="setting-icon"><Download size={17} /></div><div class="setting-copy"><strong>Download quality</strong><small>Maximum stream quality requested for downloads.</small></div><select bind:value={downloadQuality} on:change={() => saveSetting('download_quality', downloadQuality)} aria-label="Download quality"><option value="high">Highest available</option><option value="medium">Standard</option><option value="low">Data saver</option></select></div>
        <div class="setting-row last-row"><div class="setting-icon"><AudioLines size={17} /></div><div class="setting-copy"><strong>Download format</strong><small>{downloadFormat === 'flac' ? 'A lossless container cannot restore detail absent from the source stream.' : 'Choose the format written when NOVA converts a download.'}</small></div><select bind:value={downloadFormat} on:change={() => saveSetting('download_format', downloadFormat)} aria-label="Download format"><option value="original">Original source</option><option value="mp3">MP3</option><option value="opus">Opus</option><option value="flac">FLAC container</option></select></div>
        <button class="primary-button download-button" on:click={downloadCurrentTrack} disabled={isLoading}><Download size={16} /> {isLoading ? 'Preparing download…' : 'Download current track'}</button>
        <div class="settings-group-label storage-label">LOCAL STORAGE</div>
        <div class="storage-summary"><div><span>Database</span><strong>{formatBytes(storageInfo?.databaseBytes || 0)}</strong></div><div><span>Temporary audio cache</span><strong>{formatBytes(storageInfo?.playbackCacheBytes || 0)}</strong></div><div><span>Downloads</span><strong>{formatBytes(storageInfo?.downloadsBytes || 0)}</strong></div></div>
        <div class="info-card"><Database size={18} /><p>Database optimization reclaims unused database space. It does not remove downloads or original music files. NOVA does not scan arbitrary folders for local music in this build.</p></div>
        <button class="secondary-button optimize-button" on:click={optimizeDatabase} disabled={optimizingDatabase}><RefreshCw size={14} class={optimizingDatabase ? 'spinning' : ''} /> {optimizingDatabase ? 'Optimizing…' : 'Optimize local database'}</button>

      {:else if activeSection === 'Lyrics'}
        <div class="settings-group-label">DISPLAY</div>
        <div class="setting-row"><div class="setting-icon"><Mic2 size={17} /></div><div class="setting-copy"><strong>Automatically open lyrics</strong><small>Show the lyrics panel when a new song starts.</small></div><button class="switch" class:enabled={autoOpenLyrics} role="switch" aria-checked={autoOpenLyrics} on:click={() => { autoOpenLyrics = !autoOpenLyrics; saveSetting('auto_open_lyrics', autoOpenLyrics); }}><span></span></button></div>
        <div class="setting-row last-row"><div class="setting-icon"><AudioLines size={17} /></div><div class="setting-copy"><strong>Lyrics text size</strong><small>Adjust synchronized and plain lyrics from 14 to 28 pixels.</small></div><div class="setting-range"><input type="range" min="14" max="28" bind:value={lyricsFontSize} on:change={() => saveSetting('lyrics_font_size', lyricsFontSize)} aria-label="Lyrics text size" /><span>{lyricsFontSize}px</span></div></div>
        <div class="settings-group-label">SYNC</div>
        <div class="info-card"><Mic2 size={18} /><p>NOVA shows synchronized lyrics only when timed lines are available. Other results remain readable as plain text; the current track controls the lyrics shown in the player and full screen.</p></div>

      {:else if activeSection === 'Notifications'}
        <div class="settings-group-label">PLAYBACK</div>
        <div class="setting-row last-row"><div class="setting-icon"><Bell size={17} /></div><div class="setting-copy"><strong>Track change notifications</strong><small>Send a macOS notification when a new track begins.</small></div><button class="switch" class:enabled={songNotifications} role="switch" aria-checked={songNotifications} on:click={() => { songNotifications = !songNotifications; saveSetting('song_notifications', songNotifications); }}><span></span></button></div>
        <div class="settings-group-label">SYSTEM PERMISSION</div>
        <div class="info-card"><HelpCircle size={18} /><p>Notifications use macOS notification permission. Playback errors and download progress remain visible inside NOVA; separate OS notifications for those events are not enabled.</p></div>

      {:else if activeSection === 'Keyboard Shortcuts'}
        <div class="settings-group-label">IN-APP SHORTCUTS</div>
        <div class="shortcut-list"><div><span><strong>Play or pause</strong><small>Toggle playback from any NOVA screen.</small></span><kbd>SPACE</kbd></div><div><span><strong>Focus search</strong><small>Focus the main search field.</small></span><kbd>⌘ K</kbd></div><div><span><strong>Close full screen</strong><small>Exit the immersive player.</small></span><kbd>ESC</kbd></div></div>
        <div class="info-card"><Keyboard size={18} /><p>These shortcuts work while NOVA is focused. Global media shortcuts and shortcut customization are not active in this build.</p></div>

      {:else if activeSection === 'Privacy & Security'}
        <div class="settings-group-label">LOCAL DATA</div>
        <div class="setting-row"><div class="setting-icon"><History size={17} /></div><div class="setting-copy"><strong>Local playback history</strong><small>Clear NOVA’s local play records. Account history and playlists are not affected.</small></div>{#if confirmClearHistory}<button class="danger-button" on:click={clearLocalPlaybackHistory}>Confirm clear</button>{:else}<button class="secondary-button" on:click={() => confirmClearHistory = true}>Clear history</button>{/if}</div>
        {#if confirmClearHistory}<div class="confirm-note" role="alert">This removes local listening records. Your music, downloads, account history, and playlists stay in place. <button on:click={() => confirmClearHistory = false}>Cancel</button></div>{/if}
        <div class="setting-row last-row"><div class="setting-icon"><UserRound size={17} /></div><div class="setting-copy"><strong>YouTube Music account</strong><small>{accountSignedIn ? `Signed in as ${accountName || accountEmail || 'your account'}.` : 'No account is connected.'} Disconnecting does not delete local history or playlists.</small></div>{#if accountSignedIn}<button class="secondary-button" on:click={signOut}>Disconnect</button>{/if}</div>
        <div class="settings-group-label">PRIVACY</div>
        <div class="info-card"><ShieldCheck size={18} /><p>Account sessions and local preferences are stored on this device. This build has no analytics or crash-report setting, and does not offer a control to erase YouTube account history.</p></div>

      {:else if activeSection === 'Advanced'}
        <div class="settings-group-label">DATABASE</div>
        <div class="storage-summary"><div><span>Database</span><strong>{formatBytes(storageInfo?.databaseBytes || 0)}</strong></div><div><span>Temporary audio cache</span><strong>{formatBytes(storageInfo?.playbackCacheBytes || 0)}</strong></div><div><span>Downloads</span><strong>{formatBytes(storageInfo?.downloadsBytes || 0)}</strong></div></div>
        <div class="info-card"><Database size={18} /><p>Database optimization reclaims unused SQLite space. Cache files are managed by the player; clearing them while audio is active is not exposed.</p></div>
        <button class="secondary-button optimize-button" on:click={optimizeDatabase} disabled={optimizingDatabase}><RefreshCw size={14} class={optimizingDatabase ? 'spinning' : ''} /> {optimizingDatabase ? 'Optimizing…' : 'Optimize local database'}</button>
        <div class="settings-group-label">SERVICE STATUS</div>
        <div class="setting-row"><div class="setting-icon"><Search size={17} /></div><div class="setting-copy"><strong>Music service</strong><small>YouTube Music metadata and stream services power search, playback, lyrics, and Home recommendations.</small></div><span class="connection-status">ONLINE WHEN AVAILABLE</span></div>
        <div class="setting-row last-row"><div class="setting-icon"><History size={17} /></div><div class="setting-copy"><strong>Retry behavior</strong><small>Requests use bounded retries and stop when the operation fails or is cancelled.</small></div><span class="connection-status">AUTOMATIC</span></div>

      {:else if activeSection === 'About NOVA'}
        <div class="about-card"><div class="about-mark"><Sparkles size={29} /></div><h2>NOVA Music</h2><p>A desktop music player for YouTube Music.</p><span class="version-pill">Version {appVersion}</span></div>
        <section class="update-card" aria-label="App updates"><div class="update-heading"><div><strong>Software updates</strong><small>Check the NOVA GitHub release channel.</small></div><span class="update-state" class:has-update={Boolean(availableUpdate)}>{availableUpdate ? 'UPDATE READY' : 'STABLE'}</span></div>
          {#if updateMessage}<p class="update-message" class:error={updateError} role="status" aria-live="polite">{updateMessage}</p>{/if}
          {#if updateBusy && updateProgressKnown}<div class="update-progress" role="progressbar" aria-valuemin="0" aria-valuemax="100" aria-valuenow={updateProgress}><span style={`width: ${updateProgress}%`}></span></div>{/if}
          <div class="update-actions">{#if availableUpdate}<button class="primary-button" on:click={installUpdate} disabled={updateBusy}>{#if updateBusy}<Loader2 size={15} class="spinning" />{:else}<Download size={15} />{/if}{updateBusy ? (updateProgressKnown ? `Downloading ${updateProgress}%` : 'Installing…') : 'Download and install'}</button><button class="secondary-button" on:click={checkForUpdates} disabled={updateBusy}><RefreshCw size={14} /> Check again</button>{:else}<button class="primary-button" on:click={checkForUpdates} disabled={updateBusy}>{#if updateBusy}<Loader2 size={15} class="spinning" />{:else if updateMessage && !updateError}<CheckCircle2 size={15} />{:else}<RefreshCw size={15} />{/if}{updateBusy ? 'Checking…' : updateMessage && !updateError ? 'Check again' : 'Check for updates'}</button>{/if}</div>
        </section>
        <div class="settings-group-label">PROJECT AND HELP</div>
        <a class="about-link" href="https://github.com/unknown137912-byte/Nova-music" target="_blank" rel="noreferrer"><span><strong>Source code</strong><small>NOVA Music on GitHub</small></span><ExternalLink size={16} /></a>
        <a class="about-link" href="https://github.com/unknown137912-byte/Nova-music/issues" target="_blank" rel="noreferrer"><span><strong>Report a bug</strong><small>Open a NOVA Music issue</small></span><ExternalLink size={16} /></a>
        <a class="about-link" href="https://github.com/unknown137912-byte/Nova-music/blob/main/LICENSE" target="_blank" rel="noreferrer"><span><strong>Open source license</strong><small>Review the project license</small></span><ExternalLink size={16} /></a>
        <div class="info-card"><ShieldCheck size={18} /><p>YouTube Music is a third-party service. Account access and playback depend on its current session and service availability.</p></div>
      {/if}
    </div>
  </main>
</div>

<style>
  .settings-window { width: 100%; height: 100%; min-width: 0; min-height: 0; display: grid; grid-template-columns: 224px minmax(0,1fr); overflow: hidden; color: #f5eeec; background: radial-gradient(ellipse at 92% 0%, rgba(157,87,77,.16), transparent 47%), linear-gradient(145deg, rgba(33,20,21,.94), rgba(15,13,15,.97)); }
  .settings-nav { min-width: 0; min-height: 0; display: flex; flex-direction: column; padding: 19px 11px 13px; background: rgba(13,11,13,.6); border-right: 1px solid rgba(255,255,255,.07); -webkit-backdrop-filter: blur(25px); backdrop-filter: blur(25px); }
  .settings-nav-head { flex: 0 0 auto; display: flex; align-items: center; gap: 10px; padding: 2px 8px 18px; }
  .settings-brand-mark { width: 32px; height: 32px; flex: 0 0 32px; display: grid; place-items: center; color: #ffe4dc; background: linear-gradient(145deg, rgba(245,174,151,.27), rgba(255,255,255,.07)); border: 1px solid rgba(255,255,255,.13); border-radius: 10px; }
  .settings-nav-head > div { min-width: 0; flex: 1; }
  .settings-nav-head h2 { margin: 0; color: #fff7f4; font-size: 15px; font-weight: 750; letter-spacing: -.025em; }
  .settings-nav-head small { display: block; margin-top: 2px; color: rgba(246,231,227,.48); font-size: 9px; letter-spacing: .1em; text-transform: uppercase; }
  .settings-nav nav { min-height: 0; flex: 1; display: flex; flex-direction: column; gap: 3px; overflow: auto; scrollbar-width: thin; scrollbar-color: rgba(255,255,255,.16) transparent; }
  .settings-nav nav button { width: 100%; min-height: 35px; flex: 0 0 auto; display: flex; align-items: center; gap: 10px; padding: 0 10px; overflow: hidden; color: rgba(248,239,236,.65); text-align: left; background: transparent; border: 1px solid transparent; border-radius: 9px; font-size: 11px; font-weight: 560; cursor: pointer; transition: color .16s ease, background .16s ease, border-color .16s ease; }
  .settings-nav nav button span { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .settings-nav nav button :global(svg) { flex: 0 0 16px; color: rgba(248,226,218,.62); }
  .settings-nav nav button:hover { color: #fff; background: rgba(255,255,255,.07); }
  .settings-nav nav button.active { color: #fff; background: linear-gradient(105deg, rgba(224,151,129,.18), rgba(255,255,255,.055)); border-color: rgba(255,255,255,.075); font-weight: 680; }
  .settings-nav nav button.active :global(svg) { color: #ffc1ab; }
  .settings-version { flex: 0 0 auto; margin-top: 12px; padding: 11px 10px 0; color: rgba(255,241,237,.46); border-top: 1px solid rgba(255,255,255,.07); font-size: 8px; font-weight: 800; letter-spacing: .13em; }
  .settings-version span { display: block; margin-top: 4px; color: rgba(255,241,237,.62); font-size: 10px; font-weight: 550; letter-spacing: 0; }
  .settings-main { min-width: 0; min-height: 0; display: flex; flex-direction: column; }
  .settings-main-head { min-height: 81px; flex: 0 0 auto; display: flex; justify-content: space-between; align-items: center; gap: 16px; padding: 13px 24px 12px; background: rgba(255,255,255,.018); border-bottom: 1px solid rgba(255,255,255,.065); }
  .settings-eyebrow { display: block; margin-bottom: 4px; color: rgba(255,228,219,.48); font-size: 8px; font-weight: 800; letter-spacing: .17em; }
  .settings-main-head h1 { margin: 0 0 3px; color: #fff7f4; font-size: 19px; font-weight: 760; letter-spacing: -.04em; }
  .settings-main-head p { margin: 0; color: rgba(245,231,227,.58); font-size: 10px; line-height: 1.4; }
  .close-settings { width: 32px; height: 32px; flex: 0 0 32px; display: grid; place-items: center; color: rgba(255,245,241,.66); background: rgba(255,255,255,.055); border: 1px solid rgba(255,255,255,.08); border-radius: 50%; cursor: pointer; transition: background .16s ease, color .16s ease; }
  .close-settings:hover { color: #fff; background: rgba(255,255,255,.13); }
  .mobile-close { display: none; }
  .settings-message { flex: 0 0 auto; margin: 11px 20px 0; padding: 9px 12px; color: #ffd0cb; background: rgba(190,62,55,.14); border: 1px solid rgba(246,112,103,.2); border-radius: 9px; font-size: 10px; line-height: 1.45; overflow-wrap: anywhere; }
  .settings-message.success { color: #cbf6d9; background: rgba(39,155,82,.12); border-color: rgba(67,190,103,.2); }
  .settings-scroll { min-width: 0; min-height: 0; flex: 1; overflow: auto; padding: 18px 20px 26px; scrollbar-width: thin; scrollbar-color: rgba(255,255,255,.19) transparent; }
  .settings-group-label { margin: 17px 3px 8px; color: rgba(255,231,224,.48); font-size: 8px; font-weight: 800; letter-spacing: .15em; }
  .settings-group-label:first-child { margin-top: 2px; }
  .setting-row { min-height: 62px; display: flex; align-items: center; gap: 11px; margin: 5px 0; padding: 10px 12px; background: linear-gradient(115deg, rgba(255,255,255,.055), rgba(255,255,255,.026)); border: 1px solid rgba(255,255,255,.075); border-radius: 11px; box-shadow: inset 0 1px 0 rgba(255,255,255,.035); }
  .setting-row.last-row { margin-bottom: 0; }
  .setting-icon { width: 30px; height: 30px; flex: 0 0 30px; display: grid; place-items: center; color: rgba(255,221,207,.7); background: rgba(255,255,255,.05); border: 1px solid rgba(255,255,255,.055); border-radius: 9px; }
  .setting-copy { min-width: 0; flex: 1; display: flex; flex-direction: column; gap: 4px; }
  .setting-copy strong { color: rgba(255,247,244,.94); font-size: 11px; font-weight: 660; }
  .setting-copy small { color: rgba(245,229,224,.56); font-size: 9px; line-height: 1.42; }
  .setting-row select { width: min(177px, 42%); min-width: 100px; min-height: 32px; flex: 0 0 auto; padding: 0 28px 0 10px; color: #fff4f0; background: rgba(16,14,15,.72); border: 1px solid rgba(255,255,255,.1); border-radius: 9px; font-size: 10px; outline: none; }
  .setting-row select:focus, .inline-field input:focus, .cookie-details textarea:focus { border-color: rgba(255,202,182,.5); box-shadow: 0 0 0 3px rgba(244,145,121,.09); }
  .switch { width: 39px; height: 23px; flex: 0 0 39px; display: flex; align-items: center; padding: 3px; background: rgba(255,255,255,.19); border: 1px solid rgba(255,255,255,.08); border-radius: 16px; cursor: pointer; transition: background .18s ease; }
  .switch span { width: 15px; height: 15px; display: block; background: #fff8f5; border-radius: 50%; box-shadow: 0 1px 4px #0006; transition: transform .18s cubic-bezier(.2,.8,.2,1); }
  .switch.enabled { background: var(--accent, #eaa08e); }
  .switch.enabled span { transform: translateX(16px); }
  .account-card { padding: 15px; background: linear-gradient(130deg, rgba(255,255,255,.065), rgba(255,255,255,.025)); border: 1px solid rgba(255,255,255,.09); border-radius: 13px; box-shadow: inset 0 1px 0 rgba(255,255,255,.045); }
  .account-heading { display: flex; align-items: center; gap: 11px; }
  .account-heading > div:nth-child(2) { min-width: 0; flex: 1; display: flex; flex-direction: column; gap: 4px; }
  .account-heading strong { overflow: hidden; color: #fff6f2; font-size: 12px; text-overflow: ellipsis; white-space: nowrap; }
  .account-heading small { color: rgba(245,229,224,.55); font-size: 9px; line-height: 1.4; }
  .account-icon { width: 36px; height: 36px; flex: 0 0 36px; display: grid; place-items: center; color: #f5e9e5; background: rgba(255,255,255,.09); border-radius: 50%; }
  .account-icon.profile-avatar-large { color: #301d1d; background: linear-gradient(145deg, #f3d0bf, #d8887c); font-size: 15px; font-weight: 800; }
  .profile-card { margin-bottom: 12px; }
  .text-button { min-height: 29px; padding: 0 10px; color: rgba(255,240,235,.8); background: rgba(255,255,255,.075); border: 1px solid rgba(255,255,255,.07); border-radius: 8px; font-size: 9px; cursor: pointer; }
  .text-button:hover { color: #fff; background: rgba(255,255,255,.13); }
  .account-note { margin: 13px 0; color: rgba(245,229,224,.64); font-size: 10px; line-height: 1.55; }
  .account-actions { display: flex; flex-wrap: wrap; align-items: center; gap: 8px; margin-top: 13px; }
  .connected { color: #c3f2ce; background: rgba(70,166,92,.16); }
  .field-label { display: block; margin: 12px 0 6px; color: rgba(255,237,230,.72); font-size: 9px; }
  .inline-field { display: flex; gap: 8px; }
  .inline-field input, .cookie-details textarea { min-width: 0; flex: 1; color: #fff6f3; background: rgba(9,8,9,.5); border: 1px solid rgba(255,255,255,.12); border-radius: 8px; font-size: 10px; outline: none; }
  .inline-field input { height: 34px; padding: 0 10px; }
  .primary-button, .secondary-button, .danger-button { min-height: 33px; display: inline-flex; align-items: center; justify-content: center; gap: 7px; padding: 0 12px; border: 1px solid transparent; border-radius: 9px; font-size: 10px; font-weight: 700; white-space: nowrap; cursor: pointer; transition: filter .15s ease, background .15s ease, transform .15s ease; }
  .primary-button { color: #251413; background: var(--accent, #eaa08e); }
  .primary-button:hover:not(:disabled) { filter: brightness(1.08); transform: translateY(-1px); }
  .primary-button:disabled, .secondary-button:disabled { opacity: .55; cursor: default; }
  .secondary-button { color: rgba(255,243,238,.88); background: rgba(255,255,255,.065); border-color: rgba(255,255,255,.075); }
  .secondary-button:hover:not(:disabled) { color: #fff; background: rgba(255,255,255,.12); }
  .danger-button { color: #ffe4e1; background: rgba(202,64,65,.25); border-color: rgba(255,129,123,.22); }
  .danger-button:hover { background: rgba(202,64,65,.38); }
  .oauth-button { margin-top: 10px; }
  .cookie-details { margin-top: 13px; padding-top: 11px; border-top: 1px solid rgba(255,255,255,.08); }
  .cookie-details summary { color: rgba(255,240,235,.8); font-size: 10px; cursor: pointer; }
  .cookie-details p { color: rgba(245,229,224,.52); font-size: 9px; line-height: 1.45; }
  .cookie-details textarea { width: 100%; padding: 9px; resize: vertical; }
  .cookie-details button { margin-top: 8px; }
  .version-pill, .connection-status { flex: 0 0 auto; padding: 5px 8px; color: rgba(255,241,236,.72); background: rgba(255,255,255,.075); border: 1px solid rgba(255,255,255,.055); border-radius: 13px; font-size: 8px; font-weight: 750; letter-spacing: .05em; }
  .accent-picker { min-width: 118px; display: inline-flex; align-items: center; gap: 8px; padding: 4px 8px; color: rgba(255,243,238,.78); background: rgba(10,9,10,.48); border: 1px solid rgba(255,255,255,.1); border-radius: 9px; }
  .accent-picker input { width: 26px; height: 25px; padding: 0; overflow: hidden; background: transparent; border: 0; border-radius: 6px; cursor: pointer; }
  .accent-picker span { font-size: 9px; letter-spacing: .04em; }
  .theme-choice-grid { display: grid; grid-template-columns: repeat(3,minmax(0,1fr)); gap: 9px; }
  .theme-choice { min-width: 0; display: flex; flex-direction: column; padding: 8px; color: #fff2ed; text-align: left; background: rgba(255,255,255,.035); border: 1px solid rgba(255,255,255,.08); border-radius: 11px; cursor: pointer; transition: transform .16s ease, border-color .16s ease, background .16s ease; }
  .theme-choice:hover { transform: translateY(-1px); background: rgba(255,255,255,.065); }
  .theme-choice.chosen { border-color: rgba(255,194,171,.5); box-shadow: 0 0 0 2px rgba(255,176,151,.08); }
  .theme-choice strong { margin-top: 8px; font-size: 9px; font-weight: 700; }
  .theme-choice small { margin-top: 3px; color: rgba(245,229,224,.52); font-size: 8px; line-height: 1.35; }
  .theme-swatch { height: 52px; position: relative; display: block; overflow: hidden; background: #111214; border: 1px solid rgba(255,255,255,.12); border-radius: 7px; }
  .theme-swatch i { position: absolute; inset: 0 auto 0 0; width: 22%; background: rgba(0,0,0,.38); }
  .theme-swatch b { position: absolute; top: 11px; left: 32%; width: 43%; height: 6px; background: rgba(255,255,255,.72); border-radius: 5px; }
  .theme-swatch em { position: absolute; bottom: 8px; left: 32%; width: 28%; height: 16px; background: #e5a08d; border-radius: 4px; }
  .aurora .theme-swatch { background: radial-gradient(ellipse at 74% 20%, #236c79, #15212d 65%); }
  .aurora .theme-swatch em { background: #56d3bd; }
  .classic .theme-swatch { background: linear-gradient(140deg,#383334,#171719); }
  .classic .theme-swatch em { background: #d6d0cc; }
  .setting-range { min-width: 144px; display: flex; align-items: center; gap: 9px; color: rgba(255,243,238,.76); font-size: 9px; }
  .setting-range input { width: 112px; accent-color: var(--accent, #eaa08e); }
  .capability-list { display: grid; gap: 7px; }
  .capability-list > div, .shortcut-list > div { min-width: 0; display: flex; align-items: center; gap: 10px; padding: 11px 12px; background: rgba(255,255,255,.035); border: 1px solid rgba(255,255,255,.065); border-radius: 10px; }
  .capability-list strong { min-width: 135px; color: rgba(255,246,242,.88); font-size: 10px; }
  .capability-list small { margin-left: auto; color: rgba(245,229,224,.5); font-size: 9px; text-align: right; }
  .capability-dot { width: 8px; height: 8px; flex: 0 0 8px; background: rgba(255,255,255,.23); border-radius: 50%; }
  .capability-dot.available { background: #69d89c; box-shadow: 0 0 10px rgba(105,216,156,.4); }
  .storage-summary { display: grid; grid-template-columns: repeat(3,minmax(0,1fr)); gap: 8px; margin: 9px 0; }
  .storage-summary > div { min-width: 0; display: flex; flex-direction: column; gap: 6px; padding: 12px; background: rgba(255,255,255,.045); border: 1px solid rgba(255,255,255,.07); border-radius: 10px; }
  .storage-summary span { color: rgba(245,229,224,.53); font-size: 9px; }
  .storage-summary strong { overflow: hidden; color: #fff4f0; font-size: 13px; text-overflow: ellipsis; }
  .storage-label { margin-top: 22px; }
  .optimize-button { margin: 5px 0 12px; }
  .download-button { margin: 12px 0 3px; }
  .path-text { max-width: 235px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .info-card { display: flex; align-items: flex-start; gap: 10px; margin: 12px 0 3px; padding: 12px; color: rgba(248,232,226,.61); background: rgba(255,255,255,.035); border: 1px solid rgba(255,255,255,.065); border-radius: 10px; }
  .info-card :global(svg) { flex: 0 0 auto; color: rgba(255,211,193,.73); }
  .info-card p { margin: 1px 0 0; font-size: 9px; line-height: 1.55; }
  .shortcut-list { display: grid; gap: 6px; }
  .shortcut-list > div { justify-content: space-between; }
  .shortcut-list span { min-width: 0; display: flex; flex-direction: column; gap: 4px; }
  .shortcut-list strong { color: rgba(255,246,242,.9); font-size: 10px; }
  .shortcut-list small { color: rgba(245,229,224,.52); font-size: 9px; }
  .shortcut-list kbd { padding: 6px 9px; color: #fff5f1; background: rgba(8,8,9,.48); border: 1px solid rgba(255,255,255,.11); border-radius: 7px; font-size: 9px; font-weight: 650; }
  .confirm-note { display: flex; flex-wrap: wrap; align-items: center; gap: 5px; padding: 10px 12px; color: #ffdcd7; background: rgba(177,52,51,.15); border: 1px solid rgba(255,128,120,.17); border-radius: 9px; font-size: 9px; line-height: 1.5; }
  .confirm-note button { padding: 0; color: #fff; background: transparent; border: 0; text-decoration: underline; cursor: pointer; }
  .about-card { display: flex; flex-direction: column; align-items: center; padding: 14px 12px 18px; text-align: center; }
  .about-mark { width: 58px; height: 58px; display: grid; place-items: center; color: #291816; background: linear-gradient(145deg,#f8c2a8,#d9717d); border-radius: 18px; box-shadow: 0 10px 30px rgba(238,145,132,.22); }
  .about-card h2 { margin: 9px 0 3px; color: #fff7f4; font-size: 17px; letter-spacing: -.03em; }
  .about-card p { margin: 0 0 9px; color: rgba(245,229,224,.53); font-size: 10px; }
  .update-card { margin: 0 0 18px; padding: 14px; background: linear-gradient(130deg,rgba(255,255,255,.06),rgba(255,255,255,.025)); border: 1px solid rgba(255,255,255,.085); border-radius: 12px; }
  .update-heading { display: flex; align-items: center; justify-content: space-between; gap: 12px; }
  .update-heading > div { display: flex; flex-direction: column; gap: 5px; }
  .update-heading strong { color: #fff4f0; font-size: 11px; }
  .update-heading small { color: rgba(245,229,224,.52); font-size: 9px; }
  .update-state { padding: 5px 8px; color: rgba(255,243,238,.65); background: rgba(255,255,255,.08); border-radius: 12px; font-size: 7px; font-weight: 800; letter-spacing: .08em; white-space: nowrap; }
  .update-state.has-update { color: #211313; background: var(--accent, #eaa08e); }
  .update-message { margin: 10px 0 0; color: #b9f3c9; font-size: 9px; line-height: 1.5; overflow-wrap: anywhere; }
  .update-message.error { color: #ffc0c0; }
  .update-progress { height: 4px; overflow: hidden; margin-top: 10px; background: rgba(255,255,255,.12); border-radius: 5px; }
  .update-progress span { height: 100%; display: block; background: var(--accent, #eaa08e); border-radius: inherit; transition: width .18s ease; }
  .update-actions { display: flex; flex-wrap: wrap; gap: 7px; margin-top: 12px; }
  :global(.spinning) { animation: update-spin .9s linear infinite; }
  @keyframes update-spin { to { transform: rotate(360deg); } }
  .about-link { display: flex; justify-content: space-between; align-items: center; gap: 12px; margin: 6px 0; padding: 12px 13px; color: #fff1ec; text-decoration: none; background: rgba(255,255,255,.04); border: 1px solid rgba(255,255,255,.07); border-radius: 10px; transition: background .16s ease; }
  .about-link:hover { background: rgba(255,255,255,.08); }
  .about-link span { display: flex; flex-direction: column; gap: 4px; }
  .about-link strong { font-size: 10px; }
  .about-link small { color: rgba(245,229,224,.52); font-size: 9px; }
  @media (max-width: 740px) {
    .settings-window { grid-template-columns: 177px minmax(0,1fr); }
    .settings-nav { padding-inline: 7px; }
    .settings-nav nav button { gap: 7px; padding-inline: 7px; font-size: 10px; }
    .settings-main-head { padding-inline: 15px; }
    .settings-scroll { padding-inline: 13px; }
    .setting-row { gap: 7px; padding-inline: 8px; }
    .setting-icon { display: none; }
    .setting-row select { width: 42%; min-width: 92px; }
    .theme-choice-grid { grid-template-columns: 1fr; }
    .theme-swatch { height: 38px; }
    .capability-list > div { flex-wrap: wrap; }
    .capability-list small { width: 100%; margin-left: 18px; text-align: left; }
    .setting-range { min-width: 112px; }
    .setting-range input { width: 78px; }
  }
  @media (max-height: 590px) {
    .settings-nav { padding-top: 12px; }
    .settings-nav-head { padding-bottom: 9px; }
    .settings-nav nav button { min-height: 31px; }
    .settings-main-head { min-height: 68px; }
    .settings-scroll { padding-top: 12px; }
  }
</style>
