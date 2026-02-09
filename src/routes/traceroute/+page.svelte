<script lang="ts">
  import { translationsStore } from '$lib/stores/i18n';
  import { invoke } from '@tauri-apps/api/core';
  import { Search, Loader2, Trash2, Network } from 'lucide-svelte';

  let translations = $derived($translationsStore);

  function t(key: string): any {
    const keys = key.split('.');
    let value: any = translations;
    for (const k of keys) {
      value = value?.[k];
    }
    return value ?? key;
  }

  interface TracerouteHop {
    hop: number;
    hostname?: string | null;
    ip?: string | null;
    rtt_ms?: number | null;
    status: string;
  }

  interface TracerouteResponse {
    host: string;
    hops: TracerouteHop[];
    duration_ms: number;
    error?: string | null;
  }

  let host = $state('');
  let maxHops = $state(30);
  let timeout = $state(5000);
  let isTracing = $state(false);
  let result = $state<TracerouteResponse | null>(null);
  let error = $state('');

  async function trace() {
    if (!host.trim()) {
      error = t('traceroute.errors.hostRequired');
      return;
    }

    isTracing = true;
    error = '';
    result = null;

    try {
      const response = await invoke<TracerouteResponse>('traceroute', {
        request: {
          host: host.trim(),
          max_hops: maxHops,
          timeout_ms: timeout
        }
      });
      result = response;
      if (response.error) {
        error = response.error;
      }
    } catch (err) {
      error = `${t('traceroute.errors.traceFailed')}: ${err instanceof Error ? err.message : String(err)}`;
    } finally {
      isTracing = false;
    }
  }

  function clear() {
    host = '';
    maxHops = 30;
    timeout = 5000;
    result = null;
    error = '';
  }

  function getStatusColor(status: string): string {
    switch (status.toLowerCase()) {
      case 'success':
        return 'text-green-600 dark:text-green-400';
      case 'timeout':
        return 'text-yellow-600 dark:text-yellow-400';
      case 'error':
        return 'text-red-600 dark:text-red-400';
      default:
        return 'text-gray-600 dark:text-gray-400';
    }
  }
</script>

<div class="flex flex-col h-full w-full ml-0 mr-0 p-2 space-y-4 overflow-y-auto">
  <div class="card">
    <div class="space-y-4">
      <div>
        <h1 class="text-2xl font-semibold text-gray-900 dark:text-gray-100">{t('traceroute.title')}</h1>
        <p class="text-sm text-gray-600 dark:text-gray-400 mt-1">{t('traceroute.description')}</p>
      </div>

      <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
        <div>
          <label for="traceroute-host-input" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
            {t('traceroute.host')}
          </label>
          <input
            id="traceroute-host-input"
            type="text"
            bind:value={host}
            placeholder={t('traceroute.hostPlaceholder')}
            class="input w-full"
            disabled={isTracing}
            autocapitalize="off"
            autocorrect="off"
            spellcheck="false"
            onkeydown={(e) => e.key === 'Enter' && !isTracing && trace()}
          />
        </div>

        <div>
          <label for="traceroute-max-hops-input" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
            {t('traceroute.maxHops')}
          </label>
          <input
            id="traceroute-max-hops-input"
            type="number"
            bind:value={maxHops}
            min="1"
            max="64"
            class="input w-full"
            disabled={isTracing}
          />
        </div>

        <div>
          <label for="traceroute-timeout-input" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
            {t('traceroute.timeout')} (ms)
          </label>
          <input
            id="traceroute-timeout-input"
            type="number"
            bind:value={timeout}
            min="1000"
            max="30000"
            step="1000"
            class="input w-full"
            disabled={isTracing}
          />
        </div>
      </div>

      <div class="flex gap-2">
        <button
          onclick={trace}
          disabled={isTracing || !host.trim()}
          class="btn-primary disabled:opacity-50 disabled:cursor-not-allowed"
        >
          {#if isTracing}
            <Loader2 class="w-4 h-4 inline mr-2 animate-spin" />
            {t('traceroute.tracing')}
          {:else}
            <Search class="w-4 h-4 inline mr-2" />
            {t('traceroute.trace')}
          {/if}
        </button>
        {#if host || result || error}
          <button onclick={clear} class="btn-secondary">
            <Trash2 class="w-4 h-4 inline mr-2" />
            {t('common.clear')}
          </button>
        {/if}
      </div>

      {#if error}
        <div class="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg p-3">
          <p class="text-sm text-red-800 dark:text-red-200">{error}</p>
        </div>
      {/if}
    </div>
  </div>

  {#if result && result.hops.length > 0}
    <div class="card">
      <div class="flex items-center justify-between mb-4">
        <div class="flex items-center gap-2">
          <Network class="w-5 h-5 text-primary-500" />
          <h2 class="text-lg font-semibold text-gray-900 dark:text-gray-100">
            {t('traceroute.results')} - {result.host}
          </h2>
        </div>
        <span class="text-sm text-gray-500 dark:text-gray-400">
          {t('traceroute.duration')}: {result.duration_ms}ms
        </span>
      </div>

      <div class="overflow-x-auto">
        <table class="min-w-full text-sm">
          <thead>
            <tr class="border-b border-gray-200 dark:border-gray-700 text-gray-600 dark:text-gray-300">
              <th class="text-left py-2 pr-4 font-medium">{t('traceroute.hop')}</th>
              <th class="text-left py-2 pr-4 font-medium">{t('traceroute.hostname')}</th>
              <th class="text-left py-2 pr-4 font-medium">{t('traceroute.ip')}</th>
              <th class="text-left py-2 pr-4 font-medium">{t('traceroute.rtt')}</th>
              <th class="text-left py-2 pr-4 font-medium">{t('traceroute.status')}</th>
            </tr>
          </thead>
          <tbody>
            {#each result.hops as hop}
              <tr class="border-b border-gray-100 dark:border-gray-800 last:border-b-0">
                <td class="py-2 pr-4 align-top font-mono text-xs text-gray-900 dark:text-gray-100 font-semibold">
                  {hop.hop}
                </td>
                <td class="py-2 pr-4 align-top font-mono text-xs text-gray-900 dark:text-gray-100">
                  {hop.hostname || '-'}
                </td>
                <td class="py-2 pr-4 align-top font-mono text-xs text-gray-700 dark:text-gray-300">
                  {hop.ip || '-'}
                </td>
                <td class="py-2 pr-4 align-top font-mono text-xs {hop.rtt_ms !== null && hop.rtt_ms !== undefined ? 'text-gray-900 dark:text-gray-100' : 'text-gray-400 dark:text-gray-500'}">
                  {hop.rtt_ms !== null && hop.rtt_ms !== undefined ? `${hop.rtt_ms.toFixed(2)} ms` : '-'}
                </td>
                <td class="py-2 pr-4 align-top">
                  <span class="text-xs font-medium {getStatusColor(hop.status)}">
                    {hop.status}
                  </span>
                </td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    </div>
  {:else if result && result.hops.length === 0}
    <div class="card">
      <div class="flex flex-col items-center justify-center py-8 text-center">
        <Network class="w-12 h-12 text-gray-400 dark:text-gray-500 mb-4" />
        <p class="text-lg font-medium text-gray-900 dark:text-gray-100 mb-2">
          {t('traceroute.noResults')}
        </p>
        <p class="text-sm text-gray-600 dark:text-gray-400">
          {t('traceroute.noResultsDesc')}
        </p>
      </div>
    </div>
  {/if}
</div>
