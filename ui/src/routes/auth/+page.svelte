<script lang="ts">
import { onMount } from 'svelte';
import { invoke } from '@tauri-apps/api/core';

let loading = true;
let error = null;
let signedIn = false;
let accountEmail = '';
let lastSignIn = null;

onMount(async () => {
  try {
    loading = true;
    // Check authentication status
    // For now, use dummy data since auth is not yet implemented
    signedIn = false;
    accountEmail = '';
    loading = false;
  } catch (err: any) {
    error = err.message || 'Failed to check authentication';
    loading = false;
  }
});

function signIn() {
  // TODO: implement actual YouTube Music sign-in
}

function signOut() {
  // TODO: implement sign out
}
</script>

<div class="auth-page">
  {#if loading}
    <div class="loading-section">
      <div class="loading-spinner"></div>
      <p>Checking authentication...</p>
    </div>
  {:else if error}
    <div class="error-section">
      <div class="error-message">
        <h3>Authentication Error</h3>
        <p>{error}</p>
        <button on:click={() => window.location.reload()}>Try Again</button>
      </div>
    </div>
  {:else}
    <div class="auth-container">
      {#if signedIn}
        <div class="signed-in">
          <h2>Signed in as {accountEmail}</h2>
          <p>You are signed in to your YouTube Music account.</p>
          <button on:click={signOut}>Sign out</button>
        </div>
      {:else}
        <div class="signed-out">
          <h2>Sign in to NOVA Music</h2>
          <p>Connect your YouTube Music account to play music, access your library, and more.</p>
          
          <div class="sign-in-methods">
            <div class="method-card">
              <h3>Google Account</h3>
              <p>Sign in with your Google account to access YouTube Music.</p>
              <button class="primary-btn" on:click={signIn}>Sign in with Google</button>
            </div>
            
            <div class="method-card">
              <h3>Paste Cookie</h3>
              <p>Paste your YouTube Music authentication cookie to sign in.</p>
              <input type="text" placeholder="Enter cookie..." bind:value={accountEmail} />
              <button class="secondary-btn" on:click={signIn}>Sign in</button>
            </div>
          </div>
          
          <p class="auth-note">NOVA Music uses secure cookie-based authentication. Your credentials are stored securely in macOS Keychain.</p>
        </div>
      {/if}
    </div>
  {/if}
</div>

<style>
.auth-page {
  max-width: 1200px;
  margin: 0 auto;
  padding: var(--space-8) 0;
  background: var(--bg-primary);
  color: var(--text-primary);
}

.auth-container {
  max-width: 600px;
  margin: 0 auto;
  padding: var(--space-8);
}

.auth-container.signed-in {
  text-align: center;
}

.auth-container.signed-out {
  background: var(--bg-secondary);
  border-radius: var(--radius-lg);
  padding: var(--space-6);
  border: 1px solid var(--border-primary);
}

.auth-container h2 {
  font-size: var(--text-2xl);
  margin-bottom: var(--space-4);
  color: var(--text-primary);
}

.auth-container p {
  color: var(--text-secondary);
  margin-bottom: var(--space-6);
  font-size: var(--text-lg);
}

.sign-in-methods {
  display: grid;
  gap: var(--space-4);
  margin-top: var(--space-4);
}

.method-card {
  background: var(--bg-tertiary);
  border-radius: var(--radius-md);
  padding: var(--space-4);
  border: 1px solid var(--border-primary);
}

.method-card h3 {
  font-size: var(--text-base);
  margin-bottom: var(--space-2);
  color: var(--text-primary);
}

.method-card p {
  color: var(--text-tertiary);
  font-size: var(--text-sm);
  margin-bottom: var(--space-4);
}

.method-card button {
  width: 100%;
}

.auth-note {
  margin-top: var(--space-6);
  color: var(--text-tertiary);
  font-size: var(--text-sm);
  line-height: 1.5;
}
</style>