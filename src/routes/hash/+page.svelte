<script lang="ts">
  import { translationsStore } from '$lib/stores/i18n';
  import CryptoJS from 'crypto-js';
  import { onDestroy, onMount } from 'svelte';
  import { browser } from '$app/environment';
  
  type Algorithm = 'md5' | 'sha1' | 'sha224' | 'sha256' | 'sha384' | 'sha512' | 'ripemd160';
  type InputType = 'text' | 'file';
  
  let input = $state('');
  let inputType = $state<InputType>('text');
  let selectedFile = $state<File | null>(null);
  let isCalculating = $state(false);
  let results = $state<Record<Algorithm, string>>({
    md5: '',
    sha1: '',
    sha224: '',
    sha256: '',
    sha384: '',
    sha512: '',
    ripemd160: ''
  });
  let copiedAlgorithms = $state<Set<Algorithm>>(new Set());
  let textareaRefs = $state<Record<Algorithm, HTMLTextAreaElement | null>>({
    md5: null,
    sha1: null,
    sha224: null,
    sha256: null,
    sha384: null,
    sha512: null,
    ripemd160: null
  });

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
    if (browser && typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window) {
      import('@tauri-apps/api/event').then((eventModule) => {
        eventModule.listen('tauri://drag-drop', async (event: any) => {
          const paths = event.payload.paths as string[];
          if (paths && paths.length > 0) {
            await handleTauriFileDrop(paths[0]);
          }
        });
      });
    }
  });
  
  async function handleTauriFileDrop(path: string) {
    try {
      const { readFile } = await import('@tauri-apps/plugin-fs');
      const contents = await readFile(path);
      const fileName = path.split('/').pop() || path.split('\\').pop() || 'file';
      const blob = new Blob([contents]);
      const file = new File([blob], fileName);
      selectedFile = file;
      inputType = 'file';
      calculateFileHashes(file);
    } catch (err) {
      console.error('Failed to read dropped file:', err);
    }
  }

  function adjustTextareaHeight(textarea: HTMLTextAreaElement | null) {
    if (!textarea) return;
    textarea.style.height = 'auto';
    textarea.style.height = `${textarea.scrollHeight}px`;
  }

  function calculateTextHashes() {
    if (!input.trim()) {
      results = {
        md5: '',
        sha1: '',
        sha224: '',
        sha256: '',
        sha384: '',
        sha512: '',
        ripemd160: ''
      };
      return;
    }

    try {
      results = {
        md5: CryptoJS.MD5(input).toString(),
        sha1: CryptoJS.SHA1(input).toString(),
        sha224: CryptoJS.SHA224(input).toString(),
        sha256: CryptoJS.SHA256(input).toString(),
        sha384: CryptoJS.SHA384(input).toString(),
        sha512: CryptoJS.SHA512(input).toString(),
        ripemd160: CryptoJS.RIPEMD160(input).toString()
      };
    } catch (error) {
      const errorMsg = `Error: ${error instanceof Error ? error.message : 'Unknown error'}`;
      results = {
        md5: errorMsg,
        sha1: errorMsg,
        sha224: errorMsg,
        sha256: errorMsg,
        sha384: errorMsg,
        sha512: errorMsg,
        ripemd160: errorMsg
      };
    }
  }

  async function calculateFileHashes(file: File) {
    // 如果文件太大（超过 100MB），给出警告
    const maxSize = 100 * 1024 * 1024; // 100MB
    if (file.size > maxSize) {
      const errorMsg = `Error: File is too large (${(file.size / 1024 / 1024).toFixed(2)}MB). Maximum size is 100MB.`;
      results = {
        md5: errorMsg,
        sha1: errorMsg,
        sha224: errorMsg,
        sha256: errorMsg,
        sha384: errorMsg,
        sha512: errorMsg,
        ripemd160: errorMsg
      };
      isCalculating = false;
      return;
    }

    isCalculating = true;
    results = {
      md5: '',
      sha1: '',
      sha224: '',
      sha256: '',
      sha384: '',
      sha512: '',
      ripemd160: ''
    };

    try {
      const arrayBuffer = await file.arrayBuffer();
      const wordArray = CryptoJS.lib.WordArray.create(arrayBuffer);

      results = {
        md5: CryptoJS.MD5(wordArray).toString(),
        sha1: CryptoJS.SHA1(wordArray).toString(),
        sha224: CryptoJS.SHA224(wordArray).toString(),
        sha256: CryptoJS.SHA256(wordArray).toString(),
        sha384: CryptoJS.SHA384(wordArray).toString(),
        sha512: CryptoJS.SHA512(wordArray).toString(),
        ripemd160: CryptoJS.RIPEMD160(wordArray).toString()
      };
    } catch (error) {
      const errorMsg = `Error: ${error instanceof Error ? error.message : 'Unknown error'}`;
      results = {
        md5: errorMsg,
        sha1: errorMsg,
        sha224: errorMsg,
        sha256: errorMsg,
        sha384: errorMsg,
        sha512: errorMsg,
        ripemd160: errorMsg
      };
    } finally {
      isCalculating = false;
    }
  }

  async function calculateAllHashes() {
    if (inputType === 'file' && selectedFile) {
      await calculateFileHashes(selectedFile);
    } else {
      calculateTextHashes();
    }
  }

  function handleFileSelect(event: Event) {
    const target = event.target as HTMLInputElement;
    const file = target.files?.[0];
    if (file) {
      // 清理旧的文件数据
      if (selectedFile) {
        // 清空旧的结果，释放内存
        results = {
          md5: '',
          sha1: '',
          sha224: '',
          sha256: '',
          sha384: '',
          sha512: '',
          ripemd160: ''
        };
      }
      selectedFile = file;
      calculateFileHashes(file);
    }
  }

  function handleFileDrop(event: DragEvent) {
    event.preventDefault();
    const file = event.dataTransfer?.files[0];
    if (file) {
      // 清理旧的文件数据
      if (selectedFile) {
        // 清空旧的结果，释放内存
        results = {
          md5: '',
          sha1: '',
          sha224: '',
          sha256: '',
          sha384: '',
          sha512: '',
          ripemd160: ''
        };
      }
      selectedFile = file;
      calculateFileHashes(file);
    }
  }

  function handleDragOver(event: DragEvent) {
    event.preventDefault();
  }

  function openFileDialog() {
    const fileInput = document.getElementById('file-input') as HTMLInputElement;
    fileInput?.click();
  }

  function switchInputType(type: InputType) {
    inputType = type;
    if (type === 'file') {
      input = '';
      selectedFile = null;
      // 重置文件输入
      const fileInput = document.getElementById('file-input') as HTMLInputElement;
      if (fileInput) {
        fileInput.value = '';
      }
    } else {
      selectedFile = null;
    }
    results = {
      md5: '',
      sha1: '',
      sha224: '',
      sha256: '',
      sha384: '',
      sha512: '',
      ripemd160: ''
    };
  }

  async function copyToClipboard(algorithm: Algorithm, hash: string) {
    if (!hash) return;
    
    try {
      await navigator.clipboard.writeText(hash);
      // 立即更新状态，触发视觉反馈
      copiedAlgorithms.add(algorithm);
      copiedAlgorithms = new Set(copiedAlgorithms);
      
      // 1秒后恢复
      setTimeout(() => {
        copiedAlgorithms.delete(algorithm);
        copiedAlgorithms = new Set(copiedAlgorithms);
      }, 1000);
    } catch (error) {
      console.error('Failed to copy:', error);
    }
  }

  function clear() {
    input = '';
    selectedFile = null;
    // 重置文件输入
    const fileInput = document.getElementById('file-input') as HTMLInputElement;
    if (fileInput) {
      fileInput.value = '';
    }
    results = {
      md5: '',
      sha1: '',
      sha224: '',
      sha256: '',
      sha384: '',
      sha512: '',
      ripemd160: ''
    };
  }

  // 组件卸载时清理资源
  onDestroy(() => {
    input = '';
    selectedFile = null;
    results = {
      md5: '',
      sha1: '',
      sha224: '',
      sha256: '',
      sha384: '',
      sha512: '',
      ripemd160: ''
    };
  });

  const algorithms: Algorithm[] = ['md5', 'sha1', 'sha224', 'sha256', 'sha384', 'sha512', 'ripemd160'];

  // 当 hash 值变化时，自动调整 textarea 高度
  $effect(() => {
    algorithms.forEach(algorithm => {
      if (results[algorithm]) {
        adjustTextareaHeight(textareaRefs[algorithm]);
      }
    });
  });
</script>

<div class="w-full ml-0 mr-0 p-2 space-y-6">
  <!-- 输入区域卡片 -->
  <div class="card">
    <div class="space-y-4">
      <!-- 输入类型切换 -->
      <div class="border-b border-gray-200 dark:border-gray-700">
        <div class="flex gap-6">
          <button
            onclick={() => switchInputType('text')}
            class="px-4 py-2 relative transition-colors font-medium {inputType === 'text'
              ? 'text-primary-600 dark:text-primary-400'
              : 'text-gray-500 dark:text-gray-400 hover:text-gray-700 dark:hover:text-gray-300'}"
          >
            {t('hash.input')}
            {#if inputType === 'text'}
              <span class="absolute bottom-0 left-0 right-0 h-0.5 bg-primary-600 dark:bg-primary-400"></span>
            {/if}
          </button>
          <button
            onclick={() => switchInputType('file')}
            class="px-4 py-2 relative transition-colors font-medium {inputType === 'file'
              ? 'text-primary-600 dark:text-primary-400'
              : 'text-gray-500 dark:text-gray-400 hover:text-gray-700 dark:hover:text-gray-300'}"
          >
            {t('hash.file')}
            {#if inputType === 'file'}
              <span class="absolute bottom-0 left-0 right-0 h-0.5 bg-primary-600 dark:bg-primary-400"></span>
            {/if}
          </button>
        </div>
      </div>

      {#if inputType === 'text'}
        <div>
          <textarea
            id="hash-input"
            bind:value={input}
            placeholder={t('hash.placeholder')}
            class="textarea h-32"
          ></textarea>
        </div>
      {:else}
        <div>
          <input
            id="file-input"
            type="file"
            onchange={handleFileSelect}
            class="hidden"
          />
          <div 
            role="button"
            tabindex="0"
            onclick={openFileDialog}
            onkeydown={(e) => e.key === 'Enter' && openFileDialog()}
            ondragover={handleDragOver}
            ondrop={handleFileDrop}
            class="border-2 border-dashed border-gray-300 dark:border-gray-600 rounded-lg p-8 text-center cursor-pointer hover:border-gray-400 dark:hover:border-gray-500 transition-colors"
          >
            {#if selectedFile}
              <p class="text-sm text-gray-900 dark:text-gray-100 font-medium">{selectedFile.name}</p>
              <p class="text-xs text-gray-500 dark:text-gray-400 mt-1">
                {(selectedFile.size / 1024 / 1024).toFixed(2)} MB
              </p>
            {:else}
              <p class="text-sm text-gray-600 dark:text-gray-400">
                {t('hash.filePlaceholder')}
              </p>
            {/if}
          </div>
        </div>
      {/if}

      <div class="flex gap-2">
        <button 
          onclick={calculateAllHashes} 
          disabled={isCalculating || (inputType === 'text' && !input.trim()) || (inputType === 'file' && !selectedFile)}
          class="px-4 py-2 text-white rounded-lg transition-colors font-medium hover:opacity-90 disabled:opacity-50 disabled:cursor-not-allowed"
          style="background-color: #030213;"
        >
          {isCalculating ? t('hash.calculating') : t('hash.calculate')}
        </button>
        <button onclick={clear} class="btn-secondary">
          {t('hash.clear')}
        </button>
      </div>
    </div>
  </div>

  <!-- 哈希结果卡片网格 -->
  <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
    {#each algorithms as algorithm}
      {@const hash = results[algorithm]}
      {@const hasHash = hash && hash.length > 0 && !hash.startsWith('Error')}
      {@const isCopied = copiedAlgorithms.has(algorithm)}
      <div class="card relative p-2 {isCopied ? 'ring-2 ring-green-500 ring-offset-2' : ''} transition-all duration-300">
        <div class="space-y-4">
          <div class="flex items-start justify-between">
            <div class="flex-1">
              <h3 class="text-xl font-bold text-gray-900 dark:text-gray-100">
                {t(`hash.algorithms.${algorithm}`)}
              </h3>
            </div>
            {#if hasHash}
              <button
                onclick={() => copyToClipboard(algorithm, hash)}
                class="btn-secondary text-xs px-3 py-1.5 whitespace-nowrap ml-2 transition-all duration-200 {isCopied ? 'bg-green-500 hover:bg-green-600 text-white' : ''}"
              >
                {#if isCopied}
                  <span class="flex items-center gap-1">
                    <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7"></path>
                    </svg>
                    {t('common.copied')}
                  </span>
                {:else}
                  {t('common.copy')}
                {/if}
              </button>
            {/if}
          </div>
          
          {#if hasHash}
            <textarea
              bind:this={textareaRefs[algorithm]}
              readonly
              value={hash}
              class="textarea font-mono text-sm min-h-16 resize-none overflow-hidden {isCopied ? 'bg-green-50 dark:bg-green-900/20 border-green-300 dark:border-green-700' : ''} transition-colors duration-300"
            ></textarea>
          {:else}
            <div class="min-h-16 flex items-center justify-center text-gray-400 dark:text-gray-500 text-sm">
              {hash && hash.startsWith('Error') ? hash : t('hash.placeholder')}
            </div>
          {/if}
        </div>
      </div>
    {/each}
  </div>
</div>

