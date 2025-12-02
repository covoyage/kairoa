<script lang="ts">
  import { Copy, Check, FileJson, FileSpreadsheet, X } from 'lucide-svelte';
  import { browser } from '$app/environment';

  interface Props {
    open: boolean;
    output: string;
    onClose: () => void;
    t: (key: string) => string;
  }

  let { open = $bindable(false), output, onClose, t }: Props = $props();
  let copied = $state(false);

  // Tauri 插件模块（动态导入）
  let dialogModule: any = null;
  let fsModule: any = null;

  // 复制到剪贴板
  async function copyToClipboard() {
    if (!output) return;
    try {
      await navigator.clipboard.writeText(output);
      copied = true;
      setTimeout(() => {
        copied = false;
      }, 2000);
    } catch (error) {
      console.error('Copy failed:', error);
    }
  }

  // 导出为 JSON
  async function exportAsJson() {
    if (!output) return;
    
    const isTauri = browser && typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;

    if (isTauri) {
      try {
        if (!dialogModule) {
          dialogModule = await import('@tauri-apps/plugin-dialog');
        }
        if (!fsModule) {
          fsModule = await import('@tauri-apps/plugin-fs');
        }

        if (!dialogModule || !fsModule) {
          throw new Error('Failed to load Tauri plugins');
        }

        const { save } = dialogModule;
        if (!save || typeof save !== 'function') {
          throw new Error('save function not found in dialog module');
        }
        
        const filePath = await save({
          defaultPath: `Kairoa-mock-data-${Date.now()}.json`,
          filters: [{
            name: 'JSON File',
            extensions: ['json']
          }]
        });

        if (!filePath) return;

        const encoder = new TextEncoder();
        const uint8Array = encoder.encode(output);

        const { writeFile } = fsModule;
        if (!writeFile || typeof writeFile !== 'function') {
          throw new Error('writeFile function not found in fs module');
        }
        
        await writeFile(filePath, uint8Array);
      } catch (error) {
        console.error('Tauri export error:', error);
        const errorMessage = error instanceof Error ? error.message : String(error);
        alert(`Failed to export JSON: ${errorMessage}`);
      }
    } else {
      const blob = new Blob([output], { type: 'application/json' });
      const url = URL.createObjectURL(blob);
      const link = document.createElement('a');
      link.href = url;
      link.download = `Kairoa-mock-data-${Date.now()}.json`;
      document.body.appendChild(link);
      link.click();
      document.body.removeChild(link);
      URL.revokeObjectURL(url);
    }
  }

  // 导出为 CSV
  async function exportAsCsv() {
    if (!output) return;
    
    try {
      const data = JSON.parse(output);
      if (!Array.isArray(data) || data.length === 0) return;

      const headers = Object.keys(data[0]);
      const csvRows = [headers.join(',')];

      for (const row of data) {
        const values = headers.map(header => {
          const value = row[header];
          if (value === null || value === undefined) return '';
          if (typeof value === 'string' && value.includes(',')) {
            return `"${value.replace(/"/g, '""')}"`;
          }
          return String(value);
        });
        csvRows.push(values.join(','));
      }

      const csv = csvRows.join('\n');
      const isTauri = browser && typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;

      if (isTauri) {
        try {
          if (!dialogModule) {
            dialogModule = await import('@tauri-apps/plugin-dialog');
          }
          if (!fsModule) {
            fsModule = await import('@tauri-apps/plugin-fs');
          }

          if (!dialogModule || !fsModule) {
            throw new Error('Failed to load Tauri plugins');
          }

          const { save } = dialogModule;
          if (!save || typeof save !== 'function') {
            throw new Error('save function not found in dialog module');
          }
          
          const filePath = await save({
            defaultPath: `Kairoa-mock-data-${Date.now()}.csv`,
            filters: [{
              name: 'CSV File',
              extensions: ['csv']
            }]
          });

          if (!filePath) return;

          const encoder = new TextEncoder();
          const uint8Array = encoder.encode(csv);

          const { writeFile } = fsModule;
          if (!writeFile || typeof writeFile !== 'function') {
            throw new Error('writeFile function not found in fs module');
          }
          
          await writeFile(filePath, uint8Array);
        } catch (error) {
          console.error('Tauri export error:', error);
          const errorMessage = error instanceof Error ? error.message : String(error);
          alert(`Failed to export CSV: ${errorMessage}`);
        }
      } else {
        const blob = new Blob([csv], { type: 'text/csv;charset=utf-8;' });
        const url = URL.createObjectURL(blob);
        const link = document.createElement('a');
        link.href = url;
        link.download = `Kairoa-mock-data-${Date.now()}.csv`;
        document.body.appendChild(link);
        link.click();
        document.body.removeChild(link);
        URL.revokeObjectURL(url);
      }
    } catch (error) {
      console.error('Export CSV failed:', error);
      alert('Failed to export CSV');
    }
  }

  function handleBackdropClick(e: MouseEvent) {
    if (e.target === e.currentTarget) {
      onClose();
    }
  }
</script>

{#if open}
  <div
    class="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm p-4"
    onclick={handleBackdropClick}
    role="dialog"
    aria-modal="true"
  >
    <div class="bg-white dark:bg-gray-800 rounded-lg shadow-xl w-full max-w-4xl h-full flex flex-col">
      <!-- 标题栏 -->
      <div class="flex items-center justify-between p-4 border-b border-gray-200 dark:border-gray-700">
        <h2 class="text-xl font-semibold text-gray-900 dark:text-gray-100">
          {t('mockGenerator.output')}
        </h2>
        <button
          onclick={onClose}
          class="p-1 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors"
          aria-label="Close"
        >
          <X class="w-5 h-5 text-gray-500 dark:text-gray-400" />
        </button>
      </div>

      <!-- 内容区域 -->
      <div class="flex-1 overflow-hidden p-4">
        <textarea
          value={output}
          readonly
          placeholder={t('mockGenerator.outputPlaceholder')}
          class="input font-mono text-sm w-full h-full resize-none"
        ></textarea>
      </div>

      <!-- 操作按钮 -->
      <div class="flex items-center justify-end gap-2 p-4 border-t border-gray-200 dark:border-gray-700">
        <button
          onclick={copyToClipboard}
          class="btn-secondary text-sm"
        >
          {#if copied}
            <Check class="w-4 h-4 inline mr-1" />
            {t('common.copied')}
          {:else}
            <Copy class="w-4 h-4 inline mr-1" />
            {t('mockGenerator.copy')}
          {/if}
        </button>
        <button
          onclick={exportAsJson}
          class="btn-secondary text-sm"
        >
          <FileJson class="w-4 h-4 inline mr-1" />
          {t('mockGenerator.exportJson')}
        </button>
        <button
          onclick={exportAsCsv}
          class="btn-secondary text-sm"
        >
          <FileSpreadsheet class="w-4 h-4 inline mr-1" />
          {t('mockGenerator.exportCsv')}
        </button>
      </div>
    </div>
  </div>
{/if}
