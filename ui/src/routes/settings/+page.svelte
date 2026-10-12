<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { getVersion } from '@tauri-apps/api/app';
  import { listen } from '@tauri-apps/api/event';
  import { open } from '@tauri-apps/plugin-dialog';
  import { check } from '@tauri-apps/plugin-updater';
  import { relaunch } from '@tauri-apps/plugin-process';
  import {
    Settings2, Palette, SlidersHorizontal, Keyboard, Gamepad2, Radio,
    HardDrive, Info, UserRound, X, Globe, History, Monitor, Bell, Power,
    FolderOpen, Download, ShieldCheck, ExternalLink, AudioLines, Database,
    RefreshCw, CheckCircle2, Loader2, Play,
  } from 'lucide-svelte';

  export let onClose: () => void = () => {};

  type SettingsSection = 'General' | 'Appearance' | 'Playback' | 'Global Hotkeys' | 'Discord' | 'Scrobbling' | 'Data & storage' | 'About';
  let activeSection: SettingsSection = 'General';
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
  let downloadDirectory = '';
  let storageInfo: any = null;
  let optimizingDatabase = false;
  let historyEnabled = true;
  let closeToTray = false;
  let songNotifications = false;
  let launchAtStartup = false;
  let appVersion = '1.2.0';
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
    General: 'History, integrations and how NOVA starts.',
    Appearance: 'Make NOVA feel like your space.',
    Playback: 'Tune the sound and playback behavior.',
    'Global Hotkeys': 'Keyboard controls that work while you browse NOVA.',
    Discord: 'Share what you are listening to with friends.',
    Scrobbling: 'Keep a listening history with Last.fm.',
    'Data & storage': 'Downloads and local app data.',
    About: 'Version information and NOVA details.',
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
        downloadDirectory = settings.download_directory || '';
        googleOAuthClientId = settings.google_oauth_client_id || '';
        historyEnabled = settings.history_enabled !== 'false';
        closeToTray = settings.close_to_tray === 'true';
        songNotifications = settings.song_notifications === 'true';
        launchAtStartup = Boolean(autostart);
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
      if (['accent_color', 'appearance_backdrop', 'motion_preference'].includes(key)) {
        window.dispatchEvent(new CustomEvent('nova-appearance-changed', { detail: { key, value: String(value) } }));
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
    <div class="settings-nav-head"><h2>Settings</h2><button class="close-settings mobile-close" on:click={onClose} aria-label="Close settings"><X size={17} /></button></div>
    <nav aria-label="Settings categories">
      <button class:active={activeSection === 'General'} on:click={() => activeSection = 'General'}><Settings2 size={16} /><span>General</span></button>
      <button class:active={activeSection === 'Appearance'} on:click={() => activeSection = 'Appearance'}><Palette size={16} /><span>Appearance</span></button>
      <button class:active={activeSection === 'Playback'} on:click={() => activeSection = 'Playback'}><SlidersHorizontal size={16} /><span>Playback</span></button>
      <button class:active={activeSection === 'Global Hotkeys'} on:click={() => activeSection = 'Global Hotkeys'}><Keyboard size={16} /><span>Global Hotkeys</span></button>
      <button class:active={activeSection === 'Discord'} on:click={() => activeSection = 'Discord'}><Gamepad2 size={16} /><span>Discord</span></button>
      <button class:active={activeSection === 'Scrobbling'} on:click={() => activeSection = 'Scrobbling'}><Radio size={16} /><span>Scrobbling</span></button>
      <button class:active={activeSection === 'Data & storage'} on:click={() => activeSection = 'Data & storage'}><HardDrive size={16} /><span>Data &amp; storage</span></button>
      <button class:active={activeSection === 'About'} on:click={() => activeSection = 'About'}><Info size={16} /><span>About</span></button>
    </nav>
    <div class="settings-version">NOVA MUSIC <span>v{appVersion}</span></div>
  </aside>

  <main class="settings-main">
    <header class="settings-main-head">
      <div><h1>{activeSection}</h1><p>{sectionDescriptions[activeSection]}</p></div>
      <button class="close-settings desktop-close" on:click={onClose} aria-label="Close settings"><X size={18} /></button>
    </header>

    {#if authMessage}
      <div class="settings-message" class:success={isSuccess} role="status">{authMessage}</div>
    {/if}

    <div class="settings-scroll">
      {#if activeSection === 'General'}
        <div class="settings-group-label">LANGUAGE</div>
        <div class="setting-row">
          <div class="setting-icon"><Globe size={17} /></div>
          <div class="setting-copy"><strong>Interface language</strong><small>Choose the language used throughout NOVA.</small></div>
          <select value="English" disabled aria-label="Interface language"><option>English</option></select>
        </div>

        <div class="settings-group-label">ACTIVITY</div>
        <div class="setting-row">
          <div class="setting-icon"><History size={17} /></div>
          <div class="setting-copy"><strong>Listening history</strong><small>Show recently played music on Home and in your Library.</small></div>
          <button class="switch" class:enabled={historyEnabled} role="switch" aria-checked={historyEnabled} on:click={() => { historyEnabled = !historyEnabled; saveSetting('history_enabled', historyEnabled); window.dispatchEvent(new CustomEvent('nova-refresh-current-view')); }}><span></span></button>
        </div>

        <div class="settings-group-label">SYSTEM</div>
        <div class="setting-row">
          <div class="setting-icon"><Monitor size={17} /></div>
          <div class="setting-copy"><strong>Minimize on close</strong><small>Keep NOVA running and move its window to the Dock when closed.</small></div>
          <button class="switch" class:enabled={closeToTray} role="switch" aria-checked={closeToTray} on:click={() => { closeToTray = !closeToTray; saveSetting('close_to_tray', closeToTray); }}><span></span></button>
        </div>
        <div class="setting-row">
          <div class="setting-icon"><Bell size={17} /></div>
          <div class="setting-copy"><strong>Song notifications</strong><small>Show a system notification when a new song starts.</small></div>
          <button class="switch" class:enabled={songNotifications} role="switch" aria-checked={songNotifications} on:click={() => { songNotifications = !songNotifications; saveSetting('song_notifications', songNotifications); }}><span></span></button>
        </div>
        <div class="setting-row last-row">
          <div class="setting-icon"><Power size={17} /></div>
          <div class="setting-copy"><strong>Launch at startup</strong><small>Open NOVA automatically when you sign in to your Mac.</small></div>
          <button class="switch" class:enabled={launchAtStartup} role="switch" aria-checked={launchAtStartup} on:click={toggleStartup}><span></span></button>
        </div>

        <div class="settings-group-label account-label">YOUTUBE MUSIC ACCOUNT</div>
        <section class="account-card">
          <div class="account-heading"><div class="account-icon"><UserRound size={18} /></div><div><strong>{accountSignedIn ? (accountName || 'Connected account') : 'Connect your account'}</strong><small>{accountSignedIn ? (accountEmail || 'Your YouTube Music library is available') : 'Sync playlists, likes and personalized recommendations.'}</small></div>{#if accountSignedIn}<button class="text-button" on:click={signOut}>Sign out</button>{/if}</div>
          {#if !accountSignedIn}
            <p class="account-note">Google currently requires your account to be added as a test user for NOVA’s OAuth project. You can also enter a client ID from a project you control.</p>
            <label class="field-label" for="oauth-id">Desktop OAuth client ID</label>
            <div class="inline-field"><input id="oauth-id" bind:value={googleOAuthClientId} placeholder="Client ID ending in .apps.googleusercontent.com" autocomplete="off" spellcheck="false" /><button class="secondary-button" on:click={saveGoogleOAuthClientId}>Save</button></div>
            <button class="primary-button oauth-button" on:click={startGoogleSignIn} disabled={isLoading}>↗ {isLoading ? 'Opening Google…' : 'Continue with Google'}</button>
            <details class="cookie-details">
              <summary>Use a YouTube Music browser session</summary>
              <p>Session cookies are account credentials. Paste them only in this local field, never in a chat or screenshot.</p>
              <textarea bind:value={cookieInput} placeholder="Cookie value containing SAPISID=…" rows="3" autocomplete="off" spellcheck="false"></textarea>
              <button class="secondary-button" on:click={saveCookieAuth} disabled={isLoading || !cookieInput}>{isLoading ? 'Connecting…' : 'Connect session'}</button>
            </details>
          {/if}
        </section>
      {:else if activeSection === 'Appearance'}
        <div class="settings-group-label">COLOR THEME</div>
        <div class="setting-row">
          <div class="setting-icon"><Palette size={17} /></div>
          <div class="setting-copy"><strong>Accent color</strong><small>Choose the color used for active controls, buttons, and playback.</small></div>
          <select bind:value={accentColor} on:change={() => saveSetting('accent_color', accentColor)} aria-label="Accent color">
            <option value="#f46b82">Rose</option><option value="#51d6b2">Aurora green</option><option value="#5bc4ee">Arctic cyan</option><option value="#a78bfa">Violet</option><option value="#f2b75e">Amber</option><option value="#eeeeee">Monochrome</option>
          </select>
        </div>
        <div class="setting-row"><div class="setting-icon"><Palette size={17} /></div><div class="setting-copy"><strong>Custom accent</strong><small>Fine tune NOVA’s highlight color.</small></div><div class="accent-picker"><input type="color" bind:value={accentColor} on:change={() => saveSetting('accent_color', accentColor)} aria-label="Custom accent color" /><span>{accentColor.toUpperCase()}</span></div></div>
        <div class="settings-group-label">INTERFACE</div>
        <div class="setting-row"><div class="setting-icon"><Monitor size={17} /></div><div class="setting-copy"><strong>Background style</strong><small>Set the glow behind NOVA’s pages.</small></div><select bind:value={appearanceBackdrop} on:change={() => saveSetting('appearance_backdrop', appearanceBackdrop)} aria-label="Background style"><option value="artwork">Artwork glow</option><option value="gradient">Aurora gradient</option><option value="flat">Minimal dark</option></select></div>
        <div class="setting-row"><div class="setting-icon"><AudioLines size={17} /></div><div class="setting-copy"><strong>Motion effects</strong><small>Choose smooth transitions or reduce animation.</small></div><select bind:value={motionPreference} on:change={() => saveSetting('motion_preference', motionPreference)} aria-label="Motion effects"><option value="full">Full motion</option><option value="reduced">Reduced motion</option></select></div>
        <div class="theme-preview-row"><div class="theme-preview green-preview"><span></span><i></i><b></b></div><div class="theme-preview cyan-preview"><span></span><i></i><b></b></div><div class="theme-preview mono-preview"><span></span><i></i><b></b></div></div>
      {:else if activeSection === 'Playback'}
        <div class="settings-group-label">STREAMING</div>
        <div class="setting-row"><div class="setting-icon"><AudioLines size={17} /></div><div class="setting-copy"><strong>Audio stream quality</strong><small>Choose the maximum quality for YouTube Music playback.</small></div><select bind:value={audioQuality} on:change={() => saveSetting('quality', audioQuality)}><option value="auto">Automatic</option><option value="high">High</option><option value="medium">Standard</option><option value="low">Data saver</option></select></div>
        <div class="settings-group-label">PLAYER</div>
        <div class="setting-row"><div class="setting-icon"><SlidersHorizontal size={17} /></div><div class="setting-copy"><strong>Volume normalization</strong><small>Balance volume between tracks when source loudness is available.</small></div><button class="switch" class:enabled={replayGain} role="switch" aria-checked={replayGain} on:click={() => { replayGain = !replayGain; saveSetting('replay_gain', replayGain); }}><span></span></button></div>
        <div class="setting-row"><div class="setting-icon"><AudioLines size={17} /></div><div class="setting-copy"><strong>Gapless playback</strong><small>Reduce silence between consecutive tracks.</small></div><button class="switch" class:enabled={gaplessPlayback} role="switch" aria-checked={gaplessPlayback} on:click={() => { gaplessPlayback = !gaplessPlayback; saveSetting('gapless', gaplessPlayback); }}><span></span></button></div>
        <div class="setting-row"><div class="setting-icon"><Play size={17} /></div><div class="setting-copy"><strong>Autoplay recommendations</strong><small>Continue with YouTube Music’s up-next picks when your queue is empty.</small></div><button class="switch" class:enabled={autoPlayNext} role="switch" aria-checked={autoPlayNext} on:click={() => { autoPlayNext = !autoPlayNext; saveSetting('auto_play_next', autoPlayNext); }}><span></span></button></div>
        <div class="setting-row"><div class="setting-icon"><SlidersHorizontal size={17} /></div><div class="setting-copy"><strong>Playback speed</strong><small>Set the speed for the current and next tracks.</small></div><select bind:value={playbackSpeed} on:change={() => saveSetting('playback_speed', playbackSpeed)}><option value={0.5}>0.5×</option><option value={0.75}>0.75×</option><option value={1}>Normal</option><option value={1.25}>1.25×</option><option value={1.5}>1.5×</option><option value={1.75}>1.75×</option><option value={2}>2×</option></select></div>
        <div class="setting-row"><div class="setting-icon"><AudioLines size={17} /></div><div class="setting-copy"><strong>Default volume</strong><small>Volume used when NOVA starts.</small></div><div class="setting-range"><input type="range" min="0" max="100" bind:value={volume} on:change={() => saveSetting('volume', volume)} /><span>{volume}%</span></div></div>
      {:else if activeSection === 'Global Hotkeys'}
        <div class="settings-group-label">AVAILABLE SHORTCUTS</div>
        <div class="setting-row hotkey-row"><div class="setting-copy"><strong>Play / pause</strong><small>Toggle music playback from any NOVA screen.</small></div><kbd>SPACE</kbd></div>
        <div class="setting-row hotkey-row"><div class="setting-copy"><strong>Open search</strong><small>Jump to music search from anywhere in NOVA.</small></div><kbd>⌘ K</kbd></div>
        <div class="info-card"><Keyboard size={18} /><p>These shortcuts work while NOVA is focused. System-wide shortcuts are not enabled yet.</p></div>
      {:else if activeSection === 'Discord'}
        <div class="integration-card"><div class="integration-icon discord-icon"><Gamepad2 size={23} /></div><div><h2>Discord Rich Presence</h2><p>Show your current track and playback status on your Discord profile.</p><span class="connection-status">Not connected</span></div></div>
        <div class="info-card"><ShieldCheck size={18} /><p>Discord presence isn’t available in this NOVA build yet. Your account and playback remain private.</p></div>
      {:else if activeSection === 'Scrobbling'}
        <div class="integration-card"><div class="integration-icon"><Radio size={23} /></div><div><h2>Last.fm scrobbling</h2><p>Save tracks you listen to in your Last.fm profile.</p><span class="connection-status">Not connected</span></div></div>
        <div class="info-card"><ShieldCheck size={18} /><p>Last.fm sign-in and scrobbling aren’t available in this NOVA build yet.</p></div>
      {:else if activeSection === 'Data & storage'}
        <div class="settings-group-label">OFFLINE MUSIC</div>
        <div class="setting-row"><div class="setting-icon"><FolderOpen size={17} /></div><div class="setting-copy"><strong>Download folder</strong><small class="path-text">{downloadDirectory || '~/Music/NOVA Music Downloads'}</small></div><button class="secondary-button" on:click={chooseDownloadFolder}>Choose folder</button></div>
        <div class="setting-row"><div class="setting-icon"><Download size={17} /></div><div class="setting-copy"><strong>Download quality</strong><small>Maximum quality used when saving a track.</small></div><select bind:value={downloadQuality} on:change={() => saveSetting('download_quality', downloadQuality)}><option value="high">Highest available</option><option value="medium">Standard</option><option value="low">Data saver</option></select></div>
        <div class="setting-row"><div class="setting-icon"><AudioLines size={17} /></div><div class="setting-copy"><strong>Download format</strong><small>{downloadFormat === 'flac' ? 'FLAC preserves the downloaded stream during lossless encoding; it cannot restore detail absent from the source.' : 'Choose the file type saved for future downloads.'}</small></div><select bind:value={downloadFormat} on:change={() => saveSetting('download_format', downloadFormat)} aria-label="Download format"><option value="original">Original source</option><option value="mp3">MP3</option><option value="opus">Opus</option><option value="flac">FLAC (lossless file)</option></select></div>
        <button class="primary-button download-button" on:click={downloadCurrentTrack} disabled={isLoading}><Download size={16} /> {isLoading ? 'Downloading…' : 'Download current track'}</button>
        <div class="settings-group-label storage-label">LOCAL DATA</div>
        <div class="storage-summary">
          <div><span>Database</span><strong>{formatBytes(storageInfo?.databaseBytes || 0)}</strong></div>
          <div><span>Playback cache</span><strong>{formatBytes(storageInfo?.playbackCacheBytes || 0)}</strong></div>
          <div><span>Downloads</span><strong>{formatBytes(storageInfo?.downloadsBytes || 0)}</strong></div>
        </div>
        <div class="info-card"><Database size={18} /><p>NOVA keeps playback history and settings in a compact local database. This tool removes expired database cache entries and reclaims unused database space. It never deletes downloaded music.</p></div>
        <button class="secondary-button optimize-button" on:click={optimizeDatabase} disabled={optimizingDatabase}><RefreshCw size={14} class={optimizingDatabase ? 'spinning' : ''} /> {optimizingDatabase ? 'Optimizing…' : 'Optimize local database'}</button>
      {:else if activeSection === 'About'}
        <div class="about-card"><div class="about-mark">✦</div><h2>NOVA Music</h2><p>A focused desktop player for your YouTube Music library.</p><span class="version-pill">Version {appVersion}</span></div>
        <section class="update-card" aria-label="App updates">
          <div class="update-heading"><div><strong>Software updates</strong><small>Check GitHub and install the latest NOVA release.</small></div><span class="update-state" class:has-update={Boolean(availableUpdate)}>{availableUpdate ? 'UPDATE READY' : 'STABLE'}</span></div>
          {#if updateMessage}<p class="update-message" class:error={updateError} role="status" aria-live="polite">{updateMessage}</p>{/if}
          {#if updateBusy && updateProgressKnown}<div class="update-progress" role="progressbar" aria-valuemin="0" aria-valuemax="100" aria-valuenow={updateProgress}><span style={`width: ${updateProgress}%`}></span></div>{/if}
          <div class="update-actions">
            {#if availableUpdate}
              <button class="primary-button" on:click={installUpdate} disabled={updateBusy}>{#if updateBusy}<Loader2 size={15} class="spinning" />{:else}<Download size={15} />{/if}{updateBusy ? (updateProgressKnown ? `Downloading ${updateProgress}%` : 'Installing…') : 'Download and install'}</button>
              <button class="secondary-button" on:click={checkForUpdates} disabled={updateBusy}><RefreshCw size={14} /> Check again</button>
            {:else}
              <button class="primary-button" on:click={checkForUpdates} disabled={updateBusy}>{#if updateBusy}<Loader2 size={15} class="spinning" />{:else if updateMessage && !updateError}<CheckCircle2 size={15} />{:else}<RefreshCw size={15} />{/if}{updateBusy ? 'Checking…' : updateMessage && !updateError ? 'Check again' : 'Check for updates'}</button>
            {/if}
          </div>
        </section>
        <div class="settings-group-label">PROJECT</div>
        <a class="about-link" href="https://music.youtube.com" target="_blank" rel="noreferrer"><span><strong>YouTube Music</strong><small>Browse your music service</small></span><ExternalLink size={16} /></a>
        <div class="info-card"><ShieldCheck size={18} /><p>NOVA uses your account only when you choose to connect it. You can sign out at any time in General settings.</p></div>
      {/if}
    </div>
  </main>
</div>

<style>
  .settings-window { width: 100%; height: 100%; display: grid; grid-template-columns: 220px minmax(0,1fr); color: #f3f4f4; background: #101111; }
  .settings-nav { min-width: 0; display: flex; flex-direction: column; padding: 21px 12px 14px; background: #0b0c0c; border-right: 1px solid rgba(255,255,255,.08); }
  .settings-nav-head { display: flex; align-items: center; justify-content: space-between; padding: 0 10px 18px; }
  .settings-nav-head h2 { margin: 0; color: #f4f5f5; font-size: 16px; letter-spacing: -.02em; }
  .settings-nav nav { display: flex; flex-direction: column; gap: 3px; }
  .settings-nav nav button { min-height: 39px; display: flex; align-items: center; gap: 10px; padding: 0 11px; color: #aeb2b2; text-align: left; background: transparent; border: 1px solid transparent; border-radius: 7px; font-size: 12px; cursor: pointer; }
  .settings-nav nav button:hover { color: #fff; background: #1a1c1c; }
  .settings-nav nav button.active { color: #f8f8f8; background: #1a1c1c; border-color: rgba(255,255,255,.1); font-weight: 650; }
  .settings-nav nav button.active :global(svg) { color: var(--accent, #1ed760); }
  .settings-version { margin-top: auto; padding: 12px 10px 0; color: #7f8484; font-size: 9px; font-weight: 750; letter-spacing: .1em; }
  .settings-version span { display: block; margin-top: 5px; color: #686d6d; font-size: 10px; font-weight: 500; letter-spacing: 0; }
  .settings-main { min-width: 0; min-height: 0; display: flex; flex-direction: column; }
  .settings-main-head { min-height: 64px; flex: 0 0 64px; display: flex; justify-content: space-between; align-items: center; gap: 16px; padding: 0 24px; border-bottom: 1px solid rgba(255,255,255,.075); }
  .settings-main-head h1 { margin: 0 0 2px; color: #f3f5f5; font-size: 15px; font-weight: 750; }
  .settings-main-head p { margin: 0; color: #9ca1a1; font-size: 10px; }
  .close-settings { display: grid; place-items: center; width: 31px; height: 31px; color: #a7abab; background: transparent; border: 0; border-radius: 50%; cursor: pointer; }
  .close-settings:hover { color: #fff; background: rgba(255,255,255,.1); }
  .mobile-close { display: none; }
  .settings-message { flex: 0 0 auto; margin: 12px 22px 0; padding: 9px 11px; color: #ffd3d3; background: rgba(189,44,44,.14); border: 1px solid rgba(245,90,90,.2); border-radius: 6px; font-size: 10px; line-height: 1.45; }
  .settings-message.success { color: #c7f8d8; background: rgba(35,160,80,.12); border-color: rgba(67,190,103,.19); }
  .settings-scroll { min-height: 0; flex: 1; overflow: auto; padding: 20px 23px 24px; scrollbar-width: thin; scrollbar-color: #404444 transparent; }
  .settings-group-label { margin: 5px 2px 8px; color: #858a8a; font-size: 9px; font-weight: 800; letter-spacing: .12em; }
  .account-label { margin-top: 22px; }
  .setting-row { min-height: 68px; display: flex; align-items: center; gap: 12px; padding: 11px 12px; border: 1px solid rgba(255,255,255,.08); border-bottom: 0; background: rgba(255,255,255,.012); }
  .setting-row:first-of-type { border-radius: 8px 8px 0 0; }
  .setting-row.last-row { border-bottom: 1px solid rgba(255,255,255,.08); border-radius: 0 0 8px 8px; }
  .setting-icon { width: 30px; height: 30px; flex: 0 0 30px; display: grid; place-items: center; color: #9da2a2; }
  .setting-copy { min-width: 0; flex: 1; display: flex; flex-direction: column; gap: 4px; }
  .setting-copy strong { color: #e7e9e9; font-size: 11px; font-weight: 650; }
  .setting-copy small { color: #989d9d; font-size: 9px; line-height: 1.4; }
  .setting-row select { width: min(188px, 43%); min-height: 33px; padding: 0 30px 0 10px; color: #e8eaea; background: #1a1c1c; border: 1px solid rgba(255,255,255,.11); border-radius: 16px; font-size: 10px; }
  .setting-row select:disabled { opacity: .8; }
  .switch { width: 36px; height: 20px; flex: 0 0 36px; display: flex; align-items: center; padding: 2px; background: #424646; border: 0; border-radius: 15px; cursor: pointer; transition: background .15s ease; }
  .switch span { width: 16px; height: 16px; display: block; background: #f5f6f6; border-radius: 50%; transition: transform .15s ease; }
  .switch.enabled { background: #e8eeee; }
  .switch.enabled span { background: #111414; transform: translateX(16px); }
  .account-card { padding: 14px; background: #151717; border: 1px solid rgba(255,255,255,.09); border-radius: 8px; }
  .account-heading { display: flex; align-items: center; gap: 10px; }
  .account-heading > div:nth-child(2) { min-width: 0; flex: 1; display: flex; flex-direction: column; gap: 4px; }
  .account-heading strong { overflow: hidden; color: #eff1f1; font-size: 11px; text-overflow: ellipsis; white-space: nowrap; }
  .account-heading small { color: #929797; font-size: 9px; }
  .account-icon { width: 34px; height: 34px; display: grid; place-items: center; color: #d7dada; background: #252828; border-radius: 50%; }
  .text-button { padding: 6px 9px; color: #e4e6e6; background: #272a2a; border: 0; border-radius: 5px; font-size: 9px; cursor: pointer; }
  .account-note { margin: 12px 0; color: #9da1a1; font-size: 9px; line-height: 1.5; }
  .field-label { display: block; margin: 12px 0 6px; color: #b7bbbb; font-size: 9px; }
  .inline-field { display: flex; gap: 7px; }
  .inline-field input, .cookie-details textarea { min-width: 0; flex: 1; color: #f3f5f5; background: #0d0f0f; border: 1px solid rgba(255,255,255,.12); border-radius: 6px; font-size: 10px; outline: none; }
  .inline-field input { height: 33px; padding: 0 9px; }
  .inline-field input:focus, .cookie-details textarea:focus { border-color: rgba(255,255,255,.32); }
  .primary-button, .secondary-button { display: inline-flex; align-items: center; justify-content: center; gap: 7px; min-height: 32px; padding: 0 12px; border: 0; border-radius: 5px; font-size: 10px; font-weight: 700; cursor: pointer; }
  .primary-button { color: #111414; background: var(--accent, #1ed760); }
  .primary-button:hover:not(:disabled) { filter: brightness(1.08); }
  .primary-button:disabled { opacity: .55; cursor: default; }
  .secondary-button { color: #e6e8e8; background: #2a2d2d; border: 1px solid rgba(255,255,255,.07); }
  .secondary-button:hover:not(:disabled) { background: #363a3a; }
  .oauth-button { margin-top: 9px; }
  .cookie-details { margin-top: 13px; padding-top: 11px; border-top: 1px solid rgba(255,255,255,.08); }
  .cookie-details summary { color: #c1c5c5; font-size: 9px; cursor: pointer; }
  .cookie-details p { color: #8e9393; font-size: 9px; line-height: 1.4; }
  .cookie-details textarea { width: 100%; padding: 8px; resize: vertical; }
  .cookie-details button { margin-top: 7px; }
  .version-pill { padding: 5px 8px; color: #c6caca; background: #272a2a; border-radius: 12px; font-size: 8px; font-weight: 750; letter-spacing: .06em; }
  .accent-picker { display: inline-flex; align-items: center; gap: 8px; min-width: 118px; padding: 5px 8px; color: #c4c8c8; background: #1b1d1d; border: 1px solid rgba(255,255,255,.1); border-radius: 8px; }
  .accent-picker input { width: 25px; height: 25px; padding: 0; overflow: hidden; background: transparent; border: 0; border-radius: 6px; cursor: pointer; }
  .accent-picker span { font-size: 9px; letter-spacing: .04em; }
  .theme-preview-row { display: flex; gap: 10px; padding: 14px 4px; }
  .theme-preview { position: relative; height: 92px; min-width: 0; flex: 1; overflow: hidden; background: #0c0d0d; border: 1px solid rgba(255,255,255,.09); border-radius: 8px; }
  .theme-preview span { position: absolute; left: 12px; top: 13px; width: 44%; height: 6px; background: #dfe3e3; border-radius: 4px; }
  .theme-preview i { position: absolute; left: 12px; bottom: 14px; width: 28px; height: 28px; background: #1ed760; border-radius: 6px; }
  .theme-preview b { position: absolute; left: 48px; right: 12px; bottom: 17px; height: 7px; background: #484c4c; border-radius: 4px; }
  .cyan-preview i { background: #5ce1e6; }
  .mono-preview i { background: #eee; }
  .setting-range { display: flex; align-items: center; gap: 9px; color: #bec2c2; font-size: 9px; }
  .setting-range input { width: 120px; accent-color: var(--accent, #1ed760); }
  .storage-summary { display: grid; grid-template-columns: repeat(3, minmax(0,1fr)); gap: 8px; margin: 11px 0; }
  .storage-summary > div { min-width: 0; display: flex; flex-direction: column; gap: 6px; padding: 11px 12px; background: #171919; border: 1px solid rgba(255,255,255,.075); border-radius: 8px; }
  .storage-summary span { color: #929898; font-size: 9px; }
  .storage-summary strong { overflow: hidden; color: #f0f2f2; font-size: 13px; text-overflow: ellipsis; }
  .optimize-button { margin: 0 0 12px; }
  .hotkey-row { min-height: 58px; justify-content: space-between; border-bottom: 1px solid rgba(255,255,255,.08); }
  .hotkey-row kbd { padding: 5px 8px; color: #d9dddd; background: #282b2b; border: 1px solid rgba(255,255,255,.1); border-radius: 4px; font-size: 9px; }
  .info-card { display: flex; align-items: flex-start; gap: 10px; margin-top: 14px; padding: 12px; color: #9ca1a1; background: #171919; border: 1px solid rgba(255,255,255,.06); border-radius: 7px; }
  .info-card :global(svg) { flex: 0 0 auto; color: #bfc4c4; }
  .info-card p { margin: 1px 0 0; font-size: 9px; line-height: 1.55; }
  .integration-card { display: flex; align-items: flex-start; gap: 14px; padding: 17px; background: #151717; border: 1px solid rgba(255,255,255,.08); border-radius: 8px; }
  .integration-icon { width: 43px; height: 43px; display: grid; place-items: center; color: #b9bebe; background: #252929; border-radius: 10px; }
  .discord-icon { color: #c5b9ff; background: #353052; }
  .integration-card h2 { margin: 0 0 4px; font-size: 12px; }
  .integration-card p { margin: 0 0 10px; color: #9da2a2; font-size: 9px; line-height: 1.45; }
  .connection-status { display: inline-block; padding: 4px 8px; color: #b1b6b6; background: #2a2d2d; border-radius: 10px; font-size: 8px; }
  .download-button { margin-top: 13px; }
  .storage-label { margin-top: 23px; }
  .path-text { max-width: 240px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .about-card { display: flex; flex-direction: column; align-items: center; padding: 25px 12px 22px; text-align: center; }
  .about-mark { width: 56px; height: 56px; display: grid; place-items: center; color: #101313; background: var(--accent, #1ed760); border-radius: 16px; font-size: 28px; }
  .about-card h2 { margin: 12px 0 3px; font-size: 17px; }
  .about-card p { margin: 0 0 12px; color: #999e9e; font-size: 10px; }
  .update-card { margin: 0 0 19px; padding: 14px; background: linear-gradient(135deg, rgba(255,255,255,.045), rgba(255,255,255,.015)); border: 1px solid rgba(255,255,255,.09); border-radius: 9px; }
  .update-heading { display: flex; align-items: center; justify-content: space-between; gap: 12px; }
  .update-heading > div { display: flex; flex-direction: column; gap: 5px; }
  .update-heading strong { color: #eef0f0; font-size: 11px; }
  .update-heading small { color: #999e9e; font-size: 9px; }
  .update-state { padding: 4px 7px; color: #aeb4b4; background: rgba(255,255,255,.07); border-radius: 10px; font-size: 7px; font-weight: 800; letter-spacing: .08em; white-space: nowrap; }
  .update-state.has-update { color: #121615; background: var(--accent, #1ed760); }
  .update-message { margin: 11px 0 0; color: #b9f3c9; font-size: 9px; line-height: 1.5; overflow-wrap: anywhere; }
  .update-message.error { color: #ffc0c0; }
  .update-progress { height: 4px; overflow: hidden; margin-top: 10px; background: #333838; border-radius: 5px; }
  .update-progress span { height: 100%; display: block; background: var(--accent, #1ed760); border-radius: inherit; transition: width .18s ease; }
  .update-actions { display: flex; flex-wrap: wrap; gap: 7px; margin-top: 12px; }
  :global(.spinning) { animation: update-spin .9s linear infinite; }
  @keyframes update-spin { to { transform: rotate(360deg); } }
  .about-link { display: flex; justify-content: space-between; align-items: center; padding: 13px; color: #e7e9e9; text-decoration: none; background: #151717; border: 1px solid rgba(255,255,255,.08); border-radius: 7px; }
  .about-link span { display: flex; flex-direction: column; gap: 4px; }
  .about-link strong { font-size: 10px; }
  .about-link small { color: #999e9e; font-size: 9px; }
  @media (max-width: 700px) { .settings-window { grid-template-columns: 150px minmax(0,1fr); } .settings-nav { padding-inline: 7px; } .settings-nav nav button { gap: 7px; padding-inline: 8px; font-size: 10px; } .settings-main-head { padding-inline: 15px; } .settings-scroll { padding-inline: 13px; } .setting-row { gap: 7px; padding-inline: 8px; } .setting-icon { display: none; } .setting-row select { width: 42%; } }
</style>
