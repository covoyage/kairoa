<script lang="ts">
  import { translationsStore } from '$lib/stores/i18n';
  import { Copy, Check, Trash2, RefreshCw, ArrowLeftRight, FileJson, FileSpreadsheet, Info, Download } from 'lucide-svelte';
  import Papa from 'papaparse';
  import { browser } from '$app/environment';

  type Mode = 'csv2json' | 'json2csv';

  let translations = $derived($translationsStore);

  function t(key: string): string {
    const keys = key.split('.');
    let value: any = translations;
    for (const k of keys) {
      value = value?.[k];
    }
    return value || key;
  }

  let mode = $state<Mode>('csv2json');
  let delimiter = $state<'auto' | 'comma' | 'semicolon' | 'tab' | 'pipe'>('auto');
  let inputText = $state('');
  let outputText = $state('');
  let error = $state('');
  let copied = $state(false);
  let isConverting = $state(false);
  let dialogModule: typeof import('@tauri-apps/plugin-dialog') | null = null;
  let fsModule: typeof import('@tauri-apps/plugin-fs') | null = null;

  function getDelimiterChar(): string | undefined {
    switch (delimiter) {
      case 'comma':
        return ',';
      case 'semicolon':
        return ';';
      case 'tab':
        return '\t';
      case 'pipe':
        return '|';
      default:
        return undefined; // auto-detect
    }
  }

  function setMode(next: Mode) {
    mode = next;
    error = '';
    copied = false;
    outputText = '';
  }

  function getInputPlaceholder() {
    return mode === 'csv2json'
      ? t('dataConverter.inputPlaceholderCsv')
      : t('dataConverter.inputPlaceholderJson');
  }

  async function convert() {
    error = '';
    copied = false;
    outputText = '';

    if (!inputText.trim()) {
      error = t('dataConverter.errorEmpty');
      return;
    }

    isConverting = true;
    try {
      if (mode === 'csv2json') {
        const result = Papa.parse(inputText, {
          header: true,
          skipEmptyLines: true,
          dynamicTyping: true,
          delimiter: getDelimiterChar()
        });

        if (result.errors && result.errors.length > 0) {
          error = result.errors[0].message || t('dataConverter.errorInvalidCsv');
          return;
        }

        outputText = JSON.stringify(result.data, null, 2);
      } else {
        let data: unknown;
        try {
          data = JSON.parse(inputText);
        } catch (err) {
          error = t('dataConverter.errorInvalidJson');
          return;
        }

        if (!Array.isArray(data)) {
          error = t('dataConverter.errorJsonArray');
          return;
        }

        // Ensure array of objects
        const normalized = (data as any[]).map((item) =>
          typeof item === 'object' && item !== null ? item : { value: item }
        );

        const csv = Papa.unparse(normalized, {
          delimiter: getDelimiterChar()
        });
        outputText = csv;
      }
    } catch (err) {
      console.error(err);
      error = mode === 'csv2json' ? t('dataConverter.errorInvalidCsv') : t('dataConverter.errorInvalidJson');
    } finally {
      isConverting = false;
    }
  }

  function swapMode() {
    mode = mode === 'csv2json' ? 'json2csv' : 'csv2json';
    error = '';
    copied = false;
    // Keep output as next input to speed round-trip workflows
    if (outputText.trim()) {
      inputText = outputText;
    } else {
      inputText = '';
    }
    outputText = '';
  }

  async function copyOutput() {
    if (!outputText || !browser) return;
    try {
      await navigator.clipboard.writeText(outputText);
      copied = true;
      setTimeout(() => (copied = false), 1500);
    } catch (err) {
      console.error('Copy failed', err);
    }
  }

  function clearAll() {
    inputText = '';
    outputText = '';
    error = '';
    copied = false;
  }

  async function exportCsv() {
    if (!outputText || mode !== 'json2csv') return;
    const isTauri = browser && typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;

    if (isTauri) {
      try {
        if (!dialogModule) dialogModule = await import('@tauri-apps/plugin-dialog');
        if (!fsModule) fsModule = await import('@tauri-apps/plugin-fs');
        if (!dialogModule || !fsModule) throw new Error('Failed to load Tauri plugins');

        const { save } = dialogModule;
        const { writeFile } = fsModule;

        const filePath = await save({
          defaultPath: `Kairoa-data-converter-${Date.now()}.csv`,
          filters: [{ name: 'CSV File', extensions: ['csv'] }]
        });

        if (filePath) {
          const uint8Array = new TextEncoder().encode(outputText);
          await writeFile(filePath, uint8Array);
        }
      } catch (err) {
        console.error('Export CSV failed:', err);
        alert(t('dataConverter.exportCsvError') || 'Failed to export CSV.');
      }
    } else {
      const blob = new Blob([outputText], { type: 'text/csv;charset=utf-8;' });
      const url = URL.createObjectURL(blob);
      const link = document.createElement('a');
      link.href = url;
      link.download = `Kairoa-data-converter-${Date.now()}.csv`;
      document.body.appendChild(link);
      link.click();
      document.body.removeChild(link);
      URL.revokeObjectURL(url);
    }
  }
</script>

<div class="h-full w-full p-4">
  <div class="max-w-7xl mx-auto h-full flex flex-col gap-4">

    <div class="card flex flex-col gap-4">
      <div class="flex flex-wrap items-center gap-3 justify-between">
        <div class="flex flex-wrap items-center gap-3">
          <div class="flex items-center gap-2">
            <span class="text-sm text-gray-600 dark:text-gray-400">{t('dataConverter.delimiter')}</span>
            <select bind:value={delimiter} class="input w-40 text-sm">
              <option value="auto">{t('dataConverter.delimiterAuto')}</option>
              <option value="comma">{t('dataConverter.delimiterComma')}</option>
              <option value="semicolon">{t('dataConverter.delimiterSemicolon')}</option>
              <option value="tab">{t('dataConverter.delimiterTab')}</option>
              <option value="pipe">{t('dataConverter.delimiterPipe')}</option>
            </select>
          </div>
          <div class="flex gap-2">
            <button class="btn-primary" onclick={convert} disabled={isConverting}>
              {#if isConverting}
                <RefreshCw class="w-4 h-4 inline mr-1 animate-spin" />
                {t('dataConverter.converting')}
              {:else}
                <RefreshCw class="w-4 h-4 inline mr-1" />
                {t('dataConverter.convert')}
              {/if}
            </button>
            <button class="btn-secondary" onclick={swapMode}>
              <ArrowLeftRight class="w-4 h-4 inline mr-1" />
              {t('dataConverter.swap')}
            </button>
            <button class="btn-secondary" onclick={clearAll}>
              <Trash2 class="w-4 h-4 inline mr-1" />
              {t('dataConverter.clear')}
            </button>
          </div>
        </div>
        <div class="flex items-center gap-2">
          {#if outputText}
            {#if mode === 'json2csv'}
              <button class="btn-secondary text-sm" onclick={exportCsv}>
                <Download class="w-4 h-4 inline mr-1" />
                {t('dataConverter.exportCsv')}
              </button>
            {/if}
            <button class="btn-secondary text-sm" onclick={copyOutput}>
              {#if copied}
                <Check class="w-4 h-4 inline mr-1" />
                {t('common.copied')}
              {:else}
                <Copy class="w-4 h-4 inline mr-1" />
                {t('dataConverter.copyOutput')}
              {/if}
            </button>
          {/if}
        </div>
      </div>

      {#if error}
        <div class="p-3 rounded-md bg-red-50 dark:bg-red-900/30 text-red-700 dark:text-red-200 text-sm border border-red-100 dark:border-red-800 flex items-start gap-2">
          <Info class="w-4 h-4 mt-0.5" />
          <span>{error}</span>
        </div>
      {/if}

      <div class="grid grid-cols-1 lg:grid-cols-2 gap-4 flex-1 min-h-[420px]">
        <div class="flex flex-col h-full">
          <div class="flex items-center justify-between mb-2">
            <h3 class="text-sm font-semibold text-gray-800 dark:text-gray-100">
              {mode === 'csv2json' ? 'CSV' : 'JSON'}
            </h3>
          </div>
          <textarea
            bind:value={inputText}
            class="input font-mono text-sm h-full min-h-[240px]"
            placeholder={getInputPlaceholder()}
            spellcheck="false"
          ></textarea>
        </div>

        <div class="flex flex-col h-full">
          <div class="flex items-center justify-between mb-2">
            <h3 class="text-sm font-semibold text-gray-800 dark:text-gray-100">
              {mode === 'csv2json' ? 'JSON' : 'CSV'}
            </h3>
          </div>
          <textarea
            value={outputText}
            readonly
            class="input font-mono text-sm h-full min-h-[240px]"
            placeholder={t('dataConverter.outputPlaceholder')}
            spellcheck="false"
          ></textarea>
        </div>
      </div>
    </div>
  </div>
</div>

