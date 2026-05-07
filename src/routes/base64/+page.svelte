<script lang="ts">
  import { translationsStore } from '$lib/stores/i18n';
  import { getToolData } from '$lib/stores/deepLink';
  import { onMount } from 'svelte';
  
  let input = $state('');
  let output = $state('');
  let isEncoding = $state(true);
  let copied = $state(false);
  let urlSafe = $state(false);

  let translations = $derived($translationsStore);

  function t(key: string): string {
    const keys = key.split('.');
    let value: any = translations;
    for (const k of keys) {
      value = value?.[k];
    }
    return value || key;
  }

  onMount(() => {
    // Check for deep link data
    const deepLinkData = getToolData('base64');
    if (deepLinkData?.text) {
      input = deepLinkData.text;
      // Set action (encode/decode)
      if (deepLinkData.action === 'decode') {
        isEncoding = false;
        decode();
      } else {
        isEncoding = true;
        encode();
      }
    }
  });

  function encode() {
    if (!input.trim()) {
      output = '';
      return;
    }

    try {
      let encoded = btoa(unescape(encodeURIComponent(input)));
      if (urlSafe) {
        encoded = encoded.replace(/\+/g, '-').replace(/\//g, '_').replace(/=+$/, '');
      }
      output = encoded;
    } catch (error) {
      output = `Error: ${error instanceof Error ? error.message : 'Unknown error'}`;
    }
  }

  function decode() {
    if (!input.trim()) {
      output = '';
      return;
    }

    try {
      let decoded = input;
      // 自动检测 URL Safe 格式
      if (decoded.includes('-') || decoded.includes('_')) {
        decoded = decoded.replace(/-/g, '+').replace(/_/g, '/');
      }
      // 补齐 padding
      while (decoded.length % 4 !== 0) {
        decoded += '=';
      }
      output = decodeURIComponent(escape(atob(decoded)));
    } catch (error) {
      output = `Error: ${error instanceof Error ? error.message : 'Invalid Base64'}`;
    }
  }

  async function copyToClipboard() {
    const text = output || input;
    if (!text) return;
    
    try {
      await navigator.clipboard.writeText(text);
      copied = true;
      setTimeout(() => {
        copied = false;
      }, 2000);
    } catch (error) {
      console.error('Failed to copy:', error);
    }
  }

  function clear() {
    input = '';
    output = '';
  }

  $effect(() => {
    if (input) {
      if (isEncoding) {
        encode();
      } else {
        decode();
      }
    }
  });
</script>

<div class="max-w-4xl mx-auto p-8">
  <div class="card">
    <h2 class="text-2xl font-bold mb-6 text-gray-900 dark:text-gray-100">
      {t('nav.base64')}
    </h2>

    <div class="space-y-4">
      <div class="flex gap-2">
        <button
          class="px-4 py-2 rounded-lg transition-colors {isEncoding
            ? 'bg-primary-600 text-white'
            : 'bg-gray-200 dark:bg-gray-700 text-gray-900 dark:text-gray-100'}"
          onclick={() => { isEncoding = true; encode(); }}
        >
          Encode
        </button>
        <button
          class="px-4 py-2 rounded-lg transition-colors {!isEncoding
            ? 'bg-primary-600 text-white'
            : 'bg-gray-200 dark:bg-gray-700 text-gray-900 dark:text-gray-100'}"
          onclick={() => { isEncoding = false; decode(); }}
        >
          Decode
        </button>
      </div>

      <div>
        <label for="base64-input" class="block text-sm font-medium mb-2 text-gray-700 dark:text-gray-300">
          Input
        </label>
        <textarea
          id="base64-input"
          bind:value={input}
          placeholder={isEncoding ? 'Enter text to encode...' : 'Enter Base64 to decode...'}
          class="textarea h-32"
          oninput={isEncoding ? encode : decode}
        ></textarea>
      </div>

      {#if isEncoding}
        <div class="flex items-center gap-2">
          <input
            type="checkbox"
            id="url-safe"
            bind:checked={urlSafe}
            onchange={() => { if (input) encode(); }}
            class="w-4 h-4 text-primary-600 bg-gray-100 border-gray-300 rounded focus:ring-primary-500 dark:bg-gray-700 dark:border-gray-600"
          />
          <label for="url-safe" class="text-sm text-gray-700 dark:text-gray-300 cursor-pointer select-none">
            URL Safe (Base64URL)
          </label>
        </div>
      {/if}

      {#if output}
        <div>
          <label for="base64-output" class="block text-sm font-medium mb-2 text-gray-700 dark:text-gray-300">
            Output
          </label>
          <div class="flex gap-2">
            <textarea
              id="base64-output"
              value={output}
              readonly
              class="textarea h-32 font-mono text-sm flex-1"
            ></textarea>
            <button
              onclick={copyToClipboard}
              class="btn-secondary whitespace-nowrap"
            >
              {copied ? t('common.copied') : t('common.copy')}
            </button>
          </div>
        </div>
      {/if}

      <div class="flex gap-2">
        <button onclick={clear} class="btn-secondary">
          {t('common.clear')}
        </button>
      </div>
    </div>
  </div>
</div>

