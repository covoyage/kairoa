<script lang="ts">
  import { onMount } from 'svelte';
  import { browser } from '$app/environment';
  import { marked } from 'marked';
  import mermaid from 'mermaid';
  import { FileText, Image, GitBranch, Copy, Check } from 'lucide-svelte';
  import { t } from '$lib/stores/i18n';

  let filePath = $state('');
  let fileName = $state('');
  let fileExtension = $state('');
  let fileContent = $state('');
  let isLoading = $state(true);
  let error = $state('');
  let isTauri = $state(false);
  let copied = $state(false);

  let previewElement: HTMLElement | null = null;

  onMount(async () => {
    if (browser && typeof window !== 'undefined') {
      isTauri = '__TAURI_INTERNALS__' in window;
      
      // 从 URL 参数获取文件路径
      const params = new URLSearchParams(window.location.search);
      const path = params.get('path');
      
      if (path) {
        filePath = path;
        await loadFile(path);
      } else if (isTauri) {
        // Tauri 环境，调用 Rust 命令获取文件路径
        try {
          const { invoke } = await import('@tauri-apps/api/core');
          const openFilePath = await invoke('get_open_file_path');
          
          if (openFilePath) {
            filePath = openFilePath as string;
            await loadFile(filePath);
            // 清除已打开的文件路径
            await invoke('clear_open_file_path');
          } else {
            // 尝试从 localStorage 获取
            const savedPath = localStorage.getItem('kairoa-open-file-path');
            if (savedPath) {
              filePath = savedPath;
              await loadFile(savedPath);
              localStorage.removeItem('kairoa-open-file-path');
            } else {
              isLoading = false;
              error = t('fileViewer.errors.noFile');
            }
          }
        } catch (err) {
          console.error('Failed to get open file path:', err);
          isLoading = false;
          error = t('fileViewer.errors.loadFailed');
        }
      } else {
        isLoading = false;
        error = t('fileViewer.errors.noFile');
      }
    }
  });

  async function loadFile(path: string) {
    isLoading = true;
    error = '';
    
    try {
      if (isTauri) {
        const { invoke } = await import('@tauri-apps/api/core');
        const result = await invoke('read_file_content', { filePath: path });
        
        const fileData = result as any;
        fileName = fileData.file_name;
        fileExtension = fileData.extension;
        fileContent = fileData.content;
      } else {
        // 浏览器环境，尝试使用 File System Access API
        error = t('fileViewer.errors.notTauri');
      }
    } catch (err) {
      console.error('Failed to load file:', err);
      error = t('fileViewer.errors.loadFailed');
    } finally {
      isLoading = false;
    }
  }

  function getFileType(): 'svg' | 'markdown' | 'mermaid' | 'unknown' {
    const ext = fileExtension.toLowerCase();
    if (ext === 'svg') return 'svg';
    if (ext === 'md' || ext === 'markdown') return 'markdown';
    if (ext === 'mmd' || ext === 'mermaid') return 'mermaid';
    return 'unknown';
  }

  function isValidSVG(content: string): boolean {
    try {
      const parser = new DOMParser();
      const doc = parser.parseFromString(content, 'image/svg+xml');
      const parseError = doc.querySelector('parsererror');
      return !parseError && doc.querySelector('svg') !== null;
    } catch {
      return false;
    }
  }

  function renderMarkdown(content: string): string {
    if (!content.trim()) return '';
    try {
      return marked.parse(content) as string;
    } catch (err) {
      console.error('Markdown parsing error:', err);
      return `<p class="text-red-600 dark:text-red-400">Error parsing Markdown: ${err}</p>`;
    }
  }

  async function renderMermaid() {
    if (!fileContent.trim() || !previewElement) return;
    
    try {
      mermaid.initialize({
        startOnLoad: false,
        theme: document.documentElement.classList.contains('dark') ? 'dark' : 'default',
      });
      
      const { svg } = await mermaid.render('mermaid-diagram', fileContent);
      previewElement.innerHTML = svg;
    } catch (err) {
      console.error('Mermaid rendering error:', err);
      previewElement.innerHTML = `<p class="text-red-600 dark:text-red-400">Error rendering Mermaid diagram: ${err}</p>`;
    }
  }

  async function copyContent() {
    if (!fileContent) return;
    
    try {
      await navigator.clipboard.writeText(fileContent);
      copied = true;
      setTimeout(() => copied = false, 2000);
    } catch (err) {
      console.error('Failed to copy:', err);
    }
  }

  $effect(() => {
    if (getFileType() === 'mermaid' && fileContent && previewElement) {
      renderMermaid();
    }
  });
</script>

<div class="min-h-screen bg-gray-50 dark:bg-gray-900 p-6">
  <div class="max-w-6xl mx-auto">
    <!-- 头部 -->
    <div class="mb-6 flex items-center justify-between">
      <div class="flex items-center gap-3">
        {#if getFileType() === 'svg'}
          <Image class="w-6 h-6 text-blue-600" />
        {:else if getFileType() === 'markdown'}
          <FileText class="w-6 h-6 text-green-600" />
        {:else if getFileType() === 'mermaid'}
          <GitBranch class="w-6 h-6 text-purple-600" />
        {/if}
        <div>
          <h1 class="text-2xl font-bold text-gray-900 dark:text-gray-100">{fileName || t('fileViewer.title')}</h1>
          {#if filePath}
            <p class="text-sm text-gray-500 dark:text-gray-400 mt-1">{filePath}</p>
          {/if}
        </div>
      </div>
      
      <button
        type="button"
        class="btn-secondary flex items-center gap-2"
        onclick={copyContent}
      >
        {#if copied}
          <Check class="w-4 h-4" />
        {:else}
          <Copy class="w-4 h-4" />
        {/if}
        {copied ? t('fileViewer.copied') : t('fileViewer.copy')}
      </button>
    </div>

    <!-- 加载状态 -->
    {#if isLoading}
      <div class="flex items-center justify-center h-96">
        <div class="text-center">
          <div class="animate-spin rounded-full h-12 w-12 border-b-2 border-primary-600 mx-auto"></div>
          <p class="mt-4 text-gray-600 dark:text-gray-400">{t('fileViewer.loading')}</p>
        </div>
      </div>
    {:else if error}
      <!-- 错误状态 -->
      <div class="card bg-red-50 dark:bg-red-900/20 border-red-200 dark:border-red-800">
        <p class="text-red-600 dark:text-red-400">{error}</p>
      </div>
    {:else}
      <!-- 内容显示 -->
      <div class="card">
        {#if getFileType() === 'svg'}
          <!-- SVG 预览 -->
          <div class="flex-1 border border-gray-300 dark:border-gray-600 rounded-lg overflow-auto bg-gray-50 dark:bg-gray-900 flex items-center justify-center p-4 min-h-[600px]">
            {#if isValidSVG(fileContent)}
              <div class="max-w-full max-h-full">
                {@html fileContent}
              </div>
            {:else}
              <div class="text-center text-gray-400 dark:text-gray-500">
                <p class="text-sm">{t('fileViewer.invalidSVG')}</p>
              </div>
            {/if}
          </div>
        {:else if getFileType() === 'markdown'}
          <!-- Markdown 预览 -->
          <div class="flex-1 border border-gray-300 dark:border-gray-600 rounded-lg overflow-auto bg-white dark:bg-gray-800 p-6 min-h-[600px]">
            <div class="markdown-content" bind:this={previewElement}>
              {@html renderMarkdown(fileContent)}
            </div>
          </div>
        {:else if getFileType() === 'mermaid'}
          <!-- Mermaid 预览 -->
          <div class="flex-1 border border-gray-300 dark:border-gray-600 rounded-lg overflow-auto bg-white dark:bg-gray-800 p-6 min-h-[600px]">
            <div bind:this={previewElement}></div>
          </div>
        {:else}
          <!-- 原始内容 -->
          <pre class="bg-gray-50 dark:bg-gray-900 p-4 rounded-lg overflow-auto max-h-[600px]"><code class="text-sm font-mono">{fileContent}</code></pre>
        {/if}
      </div>
    {/if}
  </div>
</div>

<style>
  :global(.markdown-content) {
    line-height: 1.7;
  }
  :global(.markdown-content h1) {
    font-size: 2em;
    margin-top: 0;
    margin-bottom: 0.5em;
    border-bottom: 1px solid #eaecef;
    padding-bottom: 0.3em;
  }
  :global(.markdown-content h2) {
    font-size: 1.5em;
    margin-top: 1em;
    margin-bottom: 0.5em;
    border-bottom: 1px solid #eaecef;
    padding-bottom: 0.3em;
  }
  :global(.markdown-content h3) {
    font-size: 1.25em;
    margin-top: 1em;
    margin-bottom: 0.5em;
  }
  :global(.markdown-content p) {
    margin-top: 0;
    margin-bottom: 1em;
  }
  :global(.markdown-content code) {
    background-color: rgba(175, 184, 193, 0.2);
    padding: 0.2em 0.4em;
    border-radius: 3px;
    font-family: ui-monospace, SFMono-Regular, SF Mono, Menlo, Consolas, Liberation Mono, monospace;
    font-size: 85%;
  }
  :global(.markdown-content pre) {
    background-color: #f6f8fa;
    padding: 16px;
    border-radius: 6px;
    overflow: auto;
  }
  :global(.markdown-content pre code) {
    background-color: transparent;
    padding: 0;
    font-size: 100%;
  }
  :global(.markdown-content blockquote) {
    border-left: 4px solid #d0d7de;
    padding-left: 1em;
    margin-left: 0;
    color: #57606a;
  }
  :global(.markdown-content table) {
    border-collapse: collapse;
    width: 100%;
    margin-bottom: 1em;
  }
  :global(.markdown-content table th),
  :global(.markdown-content table td) {
    border: 1px solid #d0d7de;
    padding: 6px 13px;
  }
  :global(.markdown-content table tr) {
    background-color: #ffffff;
    border-top: 1px solid #d0d7de;
  }
  :global(.markdown-content table tr:nth-child(2n)) {
    background-color: #f6f8fa;
  }
  :global(.markdown-content img) {
    max-width: 100%;
    height: auto;
  }
  :global(.markdown-content a) {
    color: #0969da;
    text-decoration: none;
  }
  :global(.markdown-content a:hover) {
    text-decoration: underline;
  }
  :global(.markdown-content ul),
  :global(.markdown-content ol) {
    padding-left: 2em;
    margin-bottom: 1em;
  }
  :global(.markdown-content li) {
    margin-bottom: 0.25em;
  }
</style>
