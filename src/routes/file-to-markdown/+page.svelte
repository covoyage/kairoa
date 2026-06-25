<script lang="ts">
  import { translationsStore } from '$lib/stores/i18n';
  import { Upload, FileText, Copy, Check, Download, X, Clock, Eye, FileCode } from 'lucide-svelte';
  import { onMount } from 'svelte';

  let translations = $derived($translationsStore);

  function t(key: string): string {
    const keys = key.split('.');
    let value: any = translations;
    for (const k of keys) {
      value = value?.[k];
    }
    return value || key;
  }

  let selectedFile = $state<File | null>(null);
  let markdownResult = $state('');
  let isConverting = $state(false);
  let isCopied = $state(false);
  let error = $state('');
  let isDragOver = $state(false);
  let conversionTime = $state(0);
  let fileInput: HTMLInputElement | undefined = $state();
  let showModal = $state(false);
  let showPreview = $state(false);
  let markdownIt = $state<any>(null);

  onMount(async () => {
    const mod = await import('markdown-it');
    markdownIt = new mod.default({ html: true });
  });

  let renderedHtml = $derived(showPreview && markdownIt ? markdownIt.render(markdownResult) : '');

  function formatFileSize(bytes: number): string {
    if (bytes < 1024) return bytes + ' B';
    if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(1) + ' KB';
    return (bytes / (1024 * 1024)).toFixed(2) + ' MB';
  }

  async function handleFileSelect(file: File) {
    error = '';
    markdownResult = '';
    conversionTime = 0;
    selectedFile = file;
  }

  function onFileInputChange(e: Event) {
    const target = e.target as HTMLInputElement;
    if (target.files && target.files.length > 0) {
      handleFileSelect(target.files[0]);
    }
  }

  function onDragOver(e: DragEvent) {
    e.preventDefault();
    isDragOver = true;
  }

  function onDragLeave() {
    isDragOver = false;
  }

  function onDrop(e: DragEvent) {
    e.preventDefault();
    isDragOver = false;
    if (e.dataTransfer?.files && e.dataTransfer.files.length > 0) {
      handleFileSelect(e.dataTransfer.files[0]);
    }
  }

  async function convert() {
    if (!selectedFile) {
      error = t('fileToMarkdown.errorNoFile');
      return;
    }

    isConverting = true;
    error = '';
    markdownResult = '';

    const startTime = performance.now();

    try {
      const ext = selectedFile.name.split('.').pop() || '';
      const data = await selectedFile.arrayBuffer();

      const pdfjsLib = await import('pdfjs-dist');
      const workerUrl = await import('pdfjs-dist/build/pdf.worker.min.mjs?url');
      pdfjsLib.GlobalWorkerOptions.workerSrc = workerUrl.default;

      const mod = await import('@covoyage/file2md');
      const File2MD = mod.File2MD;
      const md = new File2MD();
      const result = await md.convertStream(data, {
        streamInfo: { extension: '.' + ext, filename: selectedFile.name }
      });
      markdownResult = result.markdown || '';
      conversionTime = performance.now() - startTime;
      showModal = true;
      showPreview = false;
    } catch (err) {
      console.error('Conversion failed:', err);
      error = t('fileToMarkdown.errorConvertFailed') + ': ' + (err instanceof Error ? err.message : String(err));
    } finally {
      isConverting = false;
    }
  }

  async function copyResult() {
    if (!markdownResult) return;
    try {
      await navigator.clipboard.writeText(markdownResult);
      isCopied = true;
      setTimeout(() => isCopied = false, 2000);
    } catch (err) {
      console.error('Failed to copy:', err);
    }
  }

  function downloadResult() {
    if (!markdownResult) return;
    const blob = new Blob([markdownResult], { type: 'text/markdown' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    const baseName = selectedFile?.name?.replace(/\.[^.]+$/, '') || 'output';
    a.download = baseName + '.md';
    a.click();
    URL.revokeObjectURL(url);
  }

  function closeModal() {
    showModal = false;
    showPreview = false;
  }

  function clearAll() {
    selectedFile = null;
    markdownResult = '';
    error = '';
    conversionTime = 0;
    isCopied = false;
    showModal = false;
    showPreview = false;
    if (fileInput) fileInput.value = '';
  }

  function triggerFileUpload() {
    fileInput?.click();
  }
</script>

<div class="w-full ml-0 mr-0 p-2 space-y-6">
  <div class="card">
    <div class="space-y-4">
      <p class="text-sm text-gray-600 dark:text-gray-400">
        {t('fileToMarkdown.description')}
      </p>

      <div
        ondragover={onDragOver}
        ondragleave={onDragLeave}
        ondrop={onDrop}
        onclick={triggerFileUpload}
        role="button"
        tabindex="0"
        onkeypress={(e) => { if (e.key === 'Enter') triggerFileUpload(); }}
        class="border-2 border-dashed rounded-xl p-8 text-center cursor-pointer transition-all duration-200 {isDragOver
          ? 'border-primary-500 bg-primary-50 dark:bg-primary-900/20'
          : selectedFile
            ? 'border-green-400 dark:border-green-600 bg-green-50 dark:bg-green-900/10'
            : 'border-gray-300 dark:border-gray-600 hover:border-primary-400 dark:hover:border-primary-500 hover:bg-gray-50 dark:hover:bg-gray-800/50'}"
      >
        <input
          type="file"
          accept=".pdf,.docx,.xlsx,.xls,.pptx,.html,.htm,.txt,.csv,.json,.md,.epub,.xml,.zip"
          onchange={onFileInputChange}
          bind:this={fileInput}
          class="hidden"
        />

        {#if selectedFile}
          <div class="flex flex-col items-center gap-2" role="presentation" onclick={(e) => e.stopPropagation()} onkeypress={() => {}}>
            <FileText class="w-10 h-10 text-green-500" />
            <p class="font-medium text-gray-900 dark:text-gray-100">{selectedFile.name}</p>
            <p class="text-sm text-gray-500 dark:text-gray-400">
              {formatFileSize(selectedFile.size)}
            </p>
            <button
              onclick={(e) => { e.stopPropagation(); clearAll(); }}
              class="mt-1 text-sm text-red-500 hover:text-red-700 dark:hover:text-red-400 flex items-center gap-1"
            >
              <X class="w-3.5 h-3.5" />
              {t('fileToMarkdown.clear')}
            </button>
          </div>
        {:else}
          <div class="flex flex-col items-center gap-2">
            <Upload class="w-10 h-10 text-gray-400 dark:text-gray-500" />
            <p class="font-medium text-gray-700 dark:text-gray-300">
              {t('fileToMarkdown.dropHere')}
            </p>
            <p class="text-sm text-gray-500 dark:text-gray-400">
              {t('fileToMarkdown.clickToUpload')}
            </p>
          </div>
        {/if}
      </div>

      <p class="text-xs text-gray-400 dark:text-gray-500 text-center">
        {t('fileToMarkdown.supportedFormats')}
      </p>

      {#if selectedFile}
        <div class="flex gap-2">
          <button
            onclick={convert}
            disabled={isConverting}
            class="px-6 py-2.5 text-white rounded-lg transition-colors font-medium hover:opacity-90 disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-2"
            style="background-color: #818089;"
          >
            {#if isConverting}
              <svg class="animate-spin w-4 h-4" viewBox="0 0 24 24">
                <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" fill="none" />
                <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z" />
              </svg>
              {t('fileToMarkdown.converting')}
            {:else}
              <Upload class="w-4 h-4" />
              {t('fileToMarkdown.convert')}
            {/if}
          </button>
          <button onclick={clearAll} class="btn-secondary">
            {t('fileToMarkdown.clear')}
          </button>
        </div>
      {/if}

      {#if error}
        <div class="p-3 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg">
          <p class="text-sm text-red-600 dark:text-red-400">{error}</p>
        </div>
      {/if}
    </div>
  </div>
</div>

{#if showModal && markdownResult}
  <div
    class="fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4"
    onclick={(e) => { if (e.target === e.currentTarget) closeModal(); }}
  >
    <div class="bg-white dark:bg-gray-800 rounded-xl shadow-xl w-full max-w-4xl max-h-[90vh] flex flex-col">
      <div class="p-4 border-b border-gray-200 dark:border-gray-700 flex items-center justify-between shrink-0">
        <h3 class="text-lg font-semibold text-gray-900 dark:text-gray-100 flex items-center gap-2">
          {t('fileToMarkdown.result')}
          {#if conversionTime > 0}
            <span class="text-xs font-normal text-gray-500 dark:text-gray-400 bg-gray-100 dark:bg-gray-700 px-2 py-0.5 rounded flex items-center gap-0.5">
              <Clock class="w-3 h-3" />
              {(conversionTime / 1000).toFixed(1)}s
            </span>
          {/if}
        </h3>
        <button
          onclick={closeModal}
          class="p-1 rounded hover:bg-gray-100 dark:hover:bg-gray-700 text-gray-600 dark:text-gray-400"
        >
          <X class="w-5 h-5" />
        </button>
      </div>

      <div class="flex border-b border-gray-200 dark:border-gray-700 shrink-0">
        <button
          onclick={() => showPreview = false}
          class="px-4 py-2 text-sm font-medium border-b-2 -mb-px transition-colors {!showPreview ? 'border-primary-500 text-primary-600 dark:text-primary-400' : 'border-transparent text-gray-500 dark:text-gray-400 hover:text-gray-700 dark:hover:text-gray-300'}"
        >
          <FileCode class="w-4 h-4 inline mr-1" />
          {t('fileToMarkdown.raw')}
        </button>
        <button
          onclick={() => showPreview = true}
          class="px-4 py-2 text-sm font-medium border-b-2 -mb-px transition-colors {showPreview ? 'border-primary-500 text-primary-600 dark:text-primary-400' : 'border-transparent text-gray-500 dark:text-gray-400 hover:text-gray-700 dark:hover:text-gray-300'}"
        >
          <Eye class="w-4 h-4 inline mr-1" />
          {t('fileToMarkdown.preview')}
        </button>
      </div>

      <div class="flex-1 overflow-y-auto p-4">
        {#if showPreview}
          <div class="prose dark:prose-invert max-w-none">
            {@html renderedHtml}
          </div>
        {:else}
          <textarea
            readonly
            value={markdownResult}
            class="w-full h-[500px] p-4 font-mono text-sm bg-gray-50 dark:bg-gray-900 text-gray-800 dark:text-gray-200 resize-none focus:outline-none rounded-lg border border-gray-200 dark:border-gray-700"
          ></textarea>
        {/if}
      </div>

      <div class="flex items-center justify-end gap-2 p-4 border-t border-gray-200 dark:border-gray-700 shrink-0">
        <button
          onclick={copyResult}
          class="btn-secondary whitespace-nowrap transition-all duration-200 {isCopied ? 'bg-green-500 hover:bg-green-600 text-white border-green-500' : ''}"
        >
          {#if isCopied}
            <span class="flex items-center gap-1">
              <Check class="w-4 h-4" />
              {t('fileToMarkdown.copied')}
            </span>
          {:else}
            <span class="flex items-center gap-1">
              <Copy class="w-4 h-4" />
              {t('fileToMarkdown.copy')}
            </span>
          {/if}
        </button>
        <button onclick={downloadResult} class="btn-secondary whitespace-nowrap">
          <span class="flex items-center gap-1">
            <Download class="w-4 h-4" />
            {t('fileToMarkdown.download')}
          </span>
        </button>
        <button onclick={closeModal} class="btn-secondary whitespace-nowrap">
          {t('fileToMarkdown.close')}
        </button>
      </div>
    </div>
  </div>
{/if}
