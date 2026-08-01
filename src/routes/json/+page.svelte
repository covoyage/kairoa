<script lang="ts">
  import { translationsStore } from '$lib/stores/i18n';
  import { getToolData } from '$lib/stores/deepLink';
  import { onMount } from 'svelte';
  import hljs from 'highlight.js';

  let input = $state('');
  let error = $state('');
  let copied = $state(false);
  let cleared = $state(false);

  let textareaRef = $state<HTMLTextAreaElement | null>(null);
  let overlayRef = $state<HTMLElement | null>(null);
  let gutterRef = $state<HTMLElement | null>(null);

  let translations = $derived($translationsStore);

  function t(key: string): string {
    const keys = key.split('.');
    let value: any = translations;
    for (const k of keys) {
      value = value?.[k];
    }
    return value || key;
  }

  // ── Syntax highlighting (skip for very large inputs to avoid freezing) ──
  let highlightedJson = $derived.by(() => {
    if (!input.trim()) return '';
    if (input.length > 100000) return input; // skip highlighting for huge content
    try {
      return hljs.highlight(input, { language: 'json' }).value;
    } catch {
      return input;
    }
  });

  // ── Line numbers ──
  let lineNumbers = $derived.by(() => {
    if (!input) return [];
    const count = input.split('\n').length;
    return Array.from({ length: count }, (_, i) => i + 1);
  });

  // ── Stats ──
  let stats = $derived.by(() => {
    const lines = input ? input.split('\n').length : 0;
    const chars = input.length;
    return { lines, chars };
  });

  // ── Scroll sync ──
  function syncScroll() {
    if (overlayRef && textareaRef) {
      overlayRef.scrollTop = textareaRef.scrollTop;
      overlayRef.scrollLeft = textareaRef.scrollLeft;
    }
    if (gutterRef && textareaRef) {
      gutterRef.scrollTop = textareaRef.scrollTop;
    }
  }

  onMount(() => {
    // Check for deep link data
    const deepLinkData = getToolData('json');
    if (deepLinkData?.text) {
      input = deepLinkData.text;
      // Auto-format after setting input
      setTimeout(() => formatJSON(), 100);
    }
  });

  function formatJSON() {
    if (!input.trim()) {
      error = '';
      return;
    }

    try {
      const parsed = JSON.parse(input);
      input = JSON.stringify(parsed, null, 2);
      error = '';
    } catch (e) {
      error = e instanceof Error ? e.message : 'Invalid JSON';
    }
  }

  function minifyJSON() {
    if (!input.trim()) {
      error = '';
      return;
    }

    try {
      const parsed = JSON.parse(input);
      input = JSON.stringify(parsed);
      error = '';
    } catch (e) {
      error = e instanceof Error ? e.message : 'Invalid JSON';
    }
  }

  async function copyToClipboard() {
    const text = input;
    if (!text) return;
    
    try {
      await navigator.clipboard.writeText(text);
      copied = true;
      setTimeout(() => {
        copied = false;
      }, 1000);
    } catch (error) {
      console.error('Failed to copy:', error);
    }
  }

  function clear() {
    input = '';
    error = '';
    cleared = true;
    setTimeout(() => {
      cleared = false;
    }, 1000);
  }

  // Auto-sync scroll when input changes (e.g. after format/minify)
  $effect(() => {
    input;
    setTimeout(() => syncScroll(), 0);
  });
</script>

<div class="flex flex-col h-full w-full ml-0 mr-0 p-2">
  <div class="card flex-1 flex flex-col min-h-0">
    <!-- Toolbar (sticky at top, always accessible) -->
    <div class="flex items-center justify-between gap-2 px-4 py-2.5 border-b border-gray-200 dark:border-gray-700 flex-shrink-0">
      <div class="flex items-center gap-2">
        <button onclick={formatJSON} class="px-3 py-1.5 rounded-lg transition-colors font-medium text-white text-sm" style="background-color: #818089;">
          Format
        </button>
        <button onclick={minifyJSON} class="px-3 py-1.5 rounded-lg transition-colors font-medium text-white text-sm" style="background-color: #030213;">
          Minify
        </button>
        <div class="w-px h-5 bg-gray-300 dark:bg-gray-600"></div>
        <button onclick={copyToClipboard} class="btn-secondary text-sm transition-all duration-200 {copied ? 'bg-green-500 hover:bg-green-600 text-white' : ''}">
          {#if copied}
            <span class="flex items-center gap-1">
              <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7"></path>
              </svg>
              {t('common.copied')}
            </span>
          {:else}
            {t('common.copy')}
          {/if}
        </button>
        <button onclick={clear} class="btn-secondary text-sm transition-all duration-200 {cleared ? 'bg-green-500 hover:bg-green-600 text-white' : ''}">
          {t('common.clear')}
        </button>
      </div>
      <div class="text-xs text-gray-400 dark:text-gray-500 font-mono">
        {stats.lines} lines · {stats.chars} chars
      </div>
    </div>

    <!-- Error bar (inline, pushes editor down) -->
    {#if error}
      <div class="px-4 py-2 bg-red-50 dark:bg-red-900/20 border-b border-red-200 dark:border-red-800 flex-shrink-0">
        <p class="text-sm text-red-800 dark:text-red-200 font-mono">{error}</p>
      </div>
    {/if}

    <!-- Editor area -->
    <div class="flex-1 min-h-0">
      <div class="code-editor-with-gutter h-full">
        <!-- Line numbers gutter -->
        <div class="line-numbers-gutter" bind:this={gutterRef} aria-hidden="true">
          {#each lineNumbers as num}
            <div class="line-number">{num}</div>
          {/each}
        </div>
        <!-- Editor content -->
        <div class="code-editor-content">
          <pre class="json-editor-overlay font-mono text-sm" bind:this={overlayRef} aria-hidden="true"><code class="hljs language-json">{@html highlightedJson}</code></pre>
          <textarea
            id="json-input"
            bind:value={input}
            bind:this={textareaRef}
            placeholder='{`{"example": "json"}`}'
            class="json-editor-textarea font-mono text-sm h-full"
            wrap={"off" as any}
            spellcheck="false"
            autocomplete="off"
            autocapitalize="off"
            onscroll={syncScroll}
            oninput={syncScroll}
          ></textarea>
        </div>
      </div>
    </div>
  </div>
</div>
