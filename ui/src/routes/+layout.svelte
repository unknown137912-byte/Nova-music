<script lang="ts">
import { page } from '$app/stores';
import { onMount } from 'svelte';

let isAuthenticated = false;
let loading = true;

onMount(async () => {
  // Check authentication status
  try {
    // TODO: Check authentication via Tauri command
    // For now, just simulate loading
    await new Promise(resolve => setTimeout(resolve, 1000));
    loading = false;
  } catch (error) {
    console.error('Auth check failed:', error);
    loading = false;
  }
});

function handleLogin() {
  // TODO: Open login webview or show login modal
  console.log('Login clicked');
}

function handleLogout() {
  // TODO: Call logout command
  console.log('Logout clicked');
}
</script>

<div class="app-container">
  {#if loading}
    <div class="loading-screen">
      <div class="loading-spinner"></div>
      <p>Loading NOVA Music...</p>
    </div>
  {:else}
    <header class="app-header">
      <div class="header-left">
        <h1 class="app-title">NOVA Music</h1>
        <nav class="navigation">
          <a href="/" class="nav-item" class:active={$page.url.pathname === '/'}>Home</a>
          <a href="/search" class="nav-item" class:active={$page.url.pathname.startsWith('/search')}>Search</a>
          <a href="/library" class="nav-item" class:active={$page.url.pathname.startsWith('/library')}>Library</a>
          <a href="/playlist" class="nav-item" class:active={$page.url.pathname.startsWith('/playlist')}>Playlists</a>
        </nav>
      </div>
      
      <div class="header-right">
        {#if isAuthenticated}
          <div class="user-menu">
            <span class="user-email">user@example.com</span>
            <button class="logout-btn" on:click={handleLogout}>Logout</button>
          </div>
        {:else}
          <button class="login-btn" on:click={handleLogin}>Sign In</button>
        {/if}
      </div>
    </header>

    <main class="app-content">
      <slot />
    </main>

    <footer class="app-footer">
      <p>&copy; 2026 NOVA Music. Licensed under GPL-3.0-or-later.</p>
    </footer>
  {/if}
</div>

<style>
.app-container {
  display: flex;
  flex-direction: column;
  min-height: 100vh;
  background: var(--bg-primary);
  color: var(--text-primary);
}

.loading-screen {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  min-height: 100vh;
  background: var(--bg-primary);
}

.loading-spinner {
  width: 40px;
  height: 40px;
  border: 4px solid var(--border-primary);
  border-top-color: var(--accent-600);
  border-radius: 50%;
  animation: spin 1s linear infinite;
  margin-bottom: var(--space-4);
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

.app-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: var(--space-4) var(--space-6);
  background: var(--bg-secondary);
  border-bottom: 1px solid var(--border-primary);
  position: sticky;
  top: 0;
  z-index: 100;
}

.app-title {
  font-size: var(--text-2xl);
  font-weight: var(--weight-bold);
  color: var(--accent-600);
  margin: 0;
}

.navigation {
  display: flex;
  gap: var(--space-6);
  margin-top: var(--space-2);
}

.nav-item {
  color: var(--text-secondary);
  text-decoration: none;
  padding: var(--space-2) 0;
  position: relative;
  transition: color 0.2s ease;
}

.nav-item:hover {
  color: var(--text-primary);
}

.nav-item.active {
  color: var(--accent-600);
}

.nav-item.active::after {
  content: '';
  position: absolute;
  bottom: -2px;
  left: 0;
  right: 0;
  height: 2px;
  background: var(--accent-600);
  border-radius: 1px;
}

.header-right {
  display: flex;
  align-items: center;
  gap: var(--space-4);
}

.login-btn, .logout-btn {
  padding: var(--space-2) var(--space-4);
  border: none;
  border-radius: var(--radius-md);
  cursor: pointer;
  font-size: var(--text-sm);
  font-weight: var(--weight-medium);
  transition: all 0.2s ease;
}

.login-btn {
  background: var(--accent-600);
  color: white;
}

.login-btn:hover {
  background: var(--accent-700);
  transform: translateY(-1px);
}

.logout-btn {
  background: transparent;
  color: var(--text-secondary);
  border: 1px solid var(--border-primary);
}

.logout-btn:hover {
  background: var(--bg-tertiary);
  color: var(--text-primary);
}

.user-menu {
  display: flex;
  align-items: center;
  gap: var(--space-3);
}

.user-email {
  color: var(--text-secondary);
  font-size: var(--text-sm);
}

.app-content {
  flex: 1;
  padding: var(--space-6);
  max-width: 1400px;
  margin: 0 auto;
  width: 100%;
}

.app-footer {
  padding: var(--space-6);
  text-align: center;
  border-top: 1px solid var(--border-primary);
  background: var(--bg-secondary);
  color: var(--text-tertiary);
  font-size: var(--text-sm);
}

@media (max-width: 768px) {
  .app-header {
    flex-direction: column;
    gap: var(--space-4);
  }
  
  .navigation {
    flex-wrap: wrap;
    justify-content: center;
  }
  
  .app-content {
    padding: var(--space-4);
  }
}
</style>
