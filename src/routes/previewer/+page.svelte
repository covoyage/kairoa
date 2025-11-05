<script lang="ts">
  import { translationsStore } from '$lib/stores/i18n';
  import { Copy, Check, Trash2, FileImage, FileText } from 'lucide-svelte';
  import { marked, type RendererObject } from 'marked';

  let activeView = $state<'svg' | 'markdown'>('svg');
  let svgContent = $state('');
  let markdownContent = $state('');
  let copied = $state<{ svg: boolean; markdown: boolean; preview: boolean }>({ svg: false, markdown: false, preview: false });
  let previewElement: HTMLDivElement | null = null;

  let translations = $derived($translationsStore);

  function t(key: string): string {
    const keys = key.split('.');
    let value: any = translations;
    for (const k of keys) {
      value = value?.[k];
    }
    return value || key;
  }

  async function copyToClipboard(text: string, type: 'svg' | 'markdown' | 'preview') {
    if (!text) return;
    
    try {
      await navigator.clipboard.writeText(text);
      copied = { ...copied, [type]: true };
      setTimeout(() => {
        copied = { ...copied, [type]: false };
      }, 2000);
    } catch (error) {
      console.error('Failed to copy:', error);
    }
  }

  async function copyPreviewContent() {
    if (!markdownContent.trim() || !previewElement) return;
    // 从预览 DOM 元素中提取纯文本内容
    const text = previewElement.innerText || previewElement.textContent || '';
    await copyToClipboard(text, 'preview');
  }

  function clear() {
    if (activeView === 'svg') {
      svgContent = '';
    } else {
      markdownContent = '';
    }
    copied = { svg: false, markdown: false, preview: false };
  }

  // 检查 SVG 内容是否有效
  function isValidSVG(content: string): boolean {
    if (!content.trim()) return false;
    try {
      // 简单的 SVG 验证：检查是否包含 SVG 标签
      const parser = new DOMParser();
      const doc = parser.parseFromString(content, 'image/svg+xml');
      const parseError = doc.querySelector('parsererror');
      return !parseError && doc.querySelector('svg') !== null;
    } catch {
      return false;
    }
  }

  // 配置 marked 选项并自定义渲染器
  const customRenderer: Partial<RendererObject> = {
    link(href: string, title: string | null | undefined, text: string) {
      return `<a href="${href}" target="_blank" rel="noopener noreferrer"${title ? ` title="${title}"` : ''}>${text}</a>`;
    },
    image(href: string, title: string | null | undefined, text: string) {
      return `<img src="${href}" alt="${text}" class="max-w-full rounded"${title ? ` title="${title}"` : ''}>`;
    },
  };

  marked.use({
    breaks: true, // 支持换行符（单个换行符转换为 <br>）
    gfm: true, // 启用 GitHub Flavored Markdown
    renderer: customRenderer,
  });

  // 渲染 Markdown
  function renderMarkdown(content: string): string {
    if (!content.trim()) return '';
    
    try {
      return marked.parse(content) as string;
    } catch (error) {
      console.error('Markdown parsing error:', error);
      return `<p class="text-red-600 dark:text-red-400">Error parsing Markdown: ${error}</p>`;
    }
  }
</script>

<div class="w-full ml-0 mr-0 p-2 flex flex-col h-[calc(100vh-2rem)]">
  <!-- 标签页导航 -->
  <div class="card p-0 mb-4 flex-shrink-0">
    <div class="flex items-center border-b border-gray-200 dark:border-gray-700">
      <button
        onclick={() => activeView = 'svg'}
        class="flex items-center gap-2 px-4 py-3 border-b-2 transition-colors {activeView === 'svg'
          ? 'border-primary-600 dark:border-primary-400 text-primary-600 dark:text-primary-400 bg-gray-50 dark:bg-gray-800'
          : 'border-transparent text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-gray-200 hover:bg-gray-50 dark:hover:bg-gray-800'}"
      >
        <FileImage class="w-4 h-4" />
        <span class="text-sm font-medium">{t('previewer.svg')}</span>
      </button>
      <button
        onclick={() => activeView = 'markdown'}
        class="flex items-center gap-2 px-4 py-3 border-b-2 transition-colors {activeView === 'markdown'
          ? 'border-primary-600 dark:border-primary-400 text-primary-600 dark:text-primary-400 bg-gray-50 dark:bg-gray-800'
          : 'border-transparent text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-gray-200 hover:bg-gray-50 dark:hover:bg-gray-800'}"
      >
        <FileText class="w-4 h-4" />
        <span class="text-sm font-medium">{t('previewer.markdown')}</span>
      </button>
    </div>
  </div>

  <!-- 内容区域 -->
  <div class="flex-1 min-h-0 flex flex-col">
    {#if activeView === 'svg'}
      <!-- SVG Preview -->
      <div class="card flex-shrink-0 mb-4">
        <div class="space-y-4">
          <div class="flex items-center justify-between">
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300">
              {t('previewer.svgInput')}
            </label>
            <button
              onclick={() => copyToClipboard(svgContent, 'svg')}
              class="btn-secondary text-sm transition-all duration-200 {copied.svg ? 'bg-green-500 hover:bg-green-600 text-white' : ''}"
            >
              {#if copied.svg}
                <Check class="w-4 h-4 inline mr-1" />
                {t('common.copied')}
              {:else}
                <Copy class="w-4 h-4 inline mr-1" />
                {t('common.copy')}
              {/if}
            </button>
          </div>
          <textarea
            bind:value={svgContent}
            placeholder={t('previewer.svgPlaceholder')}
            class="textarea font-mono text-sm min-h-[150px]"
          ></textarea>
          <div class="flex gap-2">
            <button
              onclick={clear}
              class="btn-secondary"
            >
              <Trash2 class="w-4 h-4 inline mr-1" />
              {t('common.clear')}
            </button>
          </div>
        </div>
      </div>

      <!-- SVG 预览 -->
      {#if svgContent.trim()}
        <div class="card flex-1 min-h-0 flex flex-col">
          <div class="flex items-center justify-between mb-3 flex-shrink-0">
            <h3 class="text-base font-semibold text-gray-900 dark:text-gray-100">
              {t('previewer.preview')}
            </h3>
            {#if !isValidSVG(svgContent)}
              <span class="text-xs text-red-600 dark:text-red-400">
                {t('previewer.invalidSVG')}
              </span>
            {/if}
          </div>
          <div class="flex-1 border border-gray-300 dark:border-gray-600 rounded-lg overflow-auto bg-gray-50 dark:bg-gray-900 flex items-center justify-center p-4">
            {#if isValidSVG(svgContent)}
              <div class="max-w-full max-h-full">
                {@html svgContent}
              </div>
            {:else}
              <div class="text-center text-gray-400 dark:text-gray-500">
                <p class="text-sm">{t('previewer.invalidSVGMessage')}</p>
              </div>
            {/if}
          </div>
        </div>
      {/if}
    {:else if activeView === 'markdown'}
      <!-- Markdown Preview - 左右布局 -->
      <div class="flex-1 min-h-0 flex flex-col">
        <div class="flex items-center justify-between mb-2 flex-shrink-0">
          <div class="flex items-center gap-4">
            <button
              onclick={clear}
              class="btn-secondary text-sm"
            >
              <Trash2 class="w-4 h-4 inline mr-1" />
              {t('common.clear')}
            </button>
          </div>
          <button
            onclick={copyPreviewContent}
            class="btn-secondary text-sm transition-all duration-200 {copied.preview ? 'bg-green-500 hover:bg-green-600 text-white' : ''}"
            disabled={!markdownContent.trim()}
          >
            {#if copied.preview}
              <Check class="w-4 h-4 inline mr-1" />
              {t('common.copied')}
            {:else}
              <Copy class="w-4 h-4 inline mr-1" />
              {t('common.copy')}
            {/if}
          </button>
        </div>
        
        <div class="flex-1 min-h-0 grid grid-cols-2 gap-4">
          <!-- Markdown 输入 -->
          <div class="card flex flex-col h-full">
            <div class="flex items-center justify-between mb-2 flex-shrink-0">
              <label class="text-sm font-medium text-gray-700 dark:text-gray-300">
                {t('previewer.markdownInput')}
              </label>
            </div>
            <div class="flex-1 min-h-0">
              <textarea
                bind:value={markdownContent}
                placeholder={t('previewer.markdownPlaceholder')}
                class="textarea font-mono text-sm h-full resize-none"
              ></textarea>
            </div>
          </div>

          <!-- Markdown 预览 -->
          <div class="card flex flex-col h-full">
            <div class="flex items-center justify-between mb-2 flex-shrink-0">
              <h3 class="text-sm font-medium text-gray-700 dark:text-gray-300">
                {t('previewer.preview')}
              </h3>
            </div>
            <div class="flex-1 border border-gray-300 dark:border-gray-600 rounded-lg overflow-auto bg-white dark:bg-gray-800 p-6 min-h-0">
              {#if markdownContent.trim()}
                <div class="markdown-content" bind:this={previewElement}>
                  {@html renderMarkdown(markdownContent)}
                </div>
              {:else}
                <div class="flex items-center justify-center h-full text-gray-400 dark:text-gray-500 text-sm">
                  {t('previewer.markdownPlaceholder')}
                </div>
              {/if}
            </div>
          </div>
        </div>
      </div>
    {/if}
  </div>
</div>

