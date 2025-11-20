<script lang="ts">
  import { translationsStore } from '$lib/stores/i18n';
  import { onDestroy, onMount } from 'svelte';
  import { Copy, Check, Trash2 } from 'lucide-svelte';
  import { page } from '$app/stores';
  import { browser } from '$app/environment';
  
  type EncodeType = 'base64' | 'image-base64' | 'url' | 'ascii' | 'jwt' | 'html';
  
  let encodeType = $state<EncodeType>('base64');
  
  // Check URL parameter for type
  $effect(() => {
    const typeParam = $page.url.searchParams.get('type');
    if (typeParam === 'base64' || typeParam === 'image-base64' || typeParam === 'url' || typeParam === 'ascii' || typeParam === 'jwt' || typeParam === 'html') {
      encodeType = typeParam as EncodeType;
    }
  });
  let input = $state('');
  let output = $state('');
  let isEncoding = $state(true);
  let copied = $state(false);
  let selectedImageFile = $state<File | null>(null);
  
  // JWT specific state
  let jwtToken = $state('');
  let decodedHeader = $state<string>('');
  let decodedPayload = $state<string>('');
  let signature = $state<string>('');
  let jwtError = $state<string>('');
  let jwtCopied = $state<{ header: boolean; payload: boolean; signature: boolean }>({
    header: false,
    payload: false,
    signature: false
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
          if (paths && paths.length > 0 && encodeType === 'image-base64') {
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
      const fileName = path.split('/').pop() || path.split('\\').pop() || 'image';
      const ext = fileName.split('.').pop()?.toLowerCase();
      const mimeTypes: Record<string, string> = {
        'jpg': 'image/jpeg',
        'jpeg': 'image/jpeg',
        'png': 'image/png',
        'gif': 'image/gif',
        'webp': 'image/webp',
        'bmp': 'image/bmp',
        'svg': 'image/svg+xml'
      };
      const mimeType = mimeTypes[ext || ''] || 'image/jpeg';
      const blob = new Blob([contents], { type: mimeType });
      const file = new File([blob], fileName, { type: mimeType });
      selectedImageFile = file;
      handleImageFileSelect({ target: { files: [file] } } as any);
    } catch (err) {
      console.error('Failed to read dropped file:', err);
    }
  }

  function encodeBase64() {
    if (!input.trim()) {
      output = '';
      return;
    }

    try {
      output = btoa(unescape(encodeURIComponent(input)));
    } catch (error) {
      output = `Error: ${error instanceof Error ? error.message : 'Unknown error'}`;
    }
  }

  function decodeBase64() {
    if (!output.trim()) {
      input = '';
      return;
    }

    try {
      input = decodeURIComponent(escape(atob(output)));
    } catch (error) {
      input = `Error: ${error instanceof Error ? error.message : 'Invalid Base64'}`;
    }
  }

  function encodeURL() {
    if (!input.trim()) {
      output = '';
      return;
    }

    try {
      output = encodeURIComponent(input);
    } catch (error) {
      output = `Error: ${error instanceof Error ? error.message : 'Unknown error'}`;
    }
  }

  function encodeImageBase64() {
    if (!input.trim()) {
      output = '';
      return;
    }

    try {
      // 将 data URI 转换为带前缀的 Base64 字符串
      const dataUri = input.trim();
      if (dataUri.startsWith('data:image/')) {
        // 如果包含 data URI 前缀，提取 MIME 类型和 Base64 部分
        const [prefix, base64] = dataUri.split(',');
        const mimeType = prefix.replace('data:', '').replace(';base64', '');
        // 输出带 data URI 前缀的 Base64
        output = `${prefix},${base64}`;
      } else {
        // 如果不是 data URI，检测图片类型并添加前缀
        let mimeType = 'image/png';
        if (dataUri.startsWith('iVBORw0KGgo')) {
          // PNG
          mimeType = 'image/png';
        } else if (dataUri.startsWith('/9j/')) {
          // JPEG
          mimeType = 'image/jpeg';
        } else if (dataUri.startsWith('R0lGODlh')) {
          // GIF
          mimeType = 'image/gif';
        } else if (dataUri.startsWith('UklGR')) {
          // WebP
          mimeType = 'image/webp';
        } else if (dataUri.startsWith('SUkq')) {
          // WebP (alternative)
          mimeType = 'image/webp';
        }
        output = `data:${mimeType};base64,${dataUri}`;
      }
    } catch (error) {
      output = `Error: ${error instanceof Error ? error.message : 'Invalid image data'}`;
    }
  }

  function decodeImageBase64() {
    if (!output.trim()) {
      input = '';
      return;
    }

    try {
      // 将 Base64 字符串转换为 data URI
      const base64 = output.trim();
      let dataUri = '';
      
      // 检查是否已经包含 data URI 前缀
      if (base64.startsWith('data:image/')) {
        dataUri = base64;
      } else {
        // 检测图片类型（简单检测，默认使用 png）
        let mimeType = 'image/png';
        if (base64.startsWith('iVBORw0KGgo')) {
          // PNG
          mimeType = 'image/png';
        } else if (base64.startsWith('/9j/')) {
          // JPEG
          mimeType = 'image/jpeg';
        } else if (base64.startsWith('R0lGODlh')) {
          // GIF
          mimeType = 'image/gif';
        } else if (base64.startsWith('UklGR')) {
          // WebP
          mimeType = 'image/webp';
        } else if (base64.startsWith('SUkq')) {
          // WebP (alternative)
          mimeType = 'image/webp';
        }
        dataUri = `data:${mimeType};base64,${base64}`;
      }
      
      input = dataUri;
    } catch (error) {
      input = `Error: ${error instanceof Error ? error.message : 'Invalid Base64'}`;
    }
  }

  function decodeURL() {
    if (!output.trim()) {
      input = '';
      return;
    }

    try {
      input = decodeURIComponent(output);
    } catch (error) {
      input = `Error: ${error instanceof Error ? error.message : 'Invalid URL encoding'}`;
    }
  }

  function encodeASCII() {
    if (!input.trim()) {
      output = '';
      return;
    }

    try {
      const result: string[] = [];
      for (let i = 0; i < input.length; i++) {
        const code = input.charCodeAt(i);
        // 转换为 \uXXXX 格式（4位十六进制，不足4位前面补0）
        result.push('\\u' + code.toString(16).padStart(4, '0'));
      }
      output = result.join('');
    } catch (error) {
      output = `Error: ${error instanceof Error ? error.message : 'Unknown error'}`;
    }
  }

  function decodeASCII() {
    if (!output.trim()) {
      input = '';
      return;
    }

    try {
      // 解析 \uXXXX 格式的 Unicode 转义序列
      const unicodeRegex = /\\u([0-9a-fA-F]{4})/g;
      input = output.replace(unicodeRegex, (match, hex) => {
        return String.fromCharCode(parseInt(hex, 16));
      });
      
      // 如果没有匹配到 \u 格式，尝试直接解析
      if (input === output) {
        throw new Error('Invalid Unicode escape sequence format');
      }
    } catch (error) {
      input = `Error: ${error instanceof Error ? error.message : 'Invalid Unicode encoding'}`;
    }
  }

  function encodeHTML() {
    if (!input.trim()) {
      output = '';
      return;
    }

    try {
      const div = document.createElement('div');
      div.textContent = input;
      output = div.innerHTML;
    } catch (error) {
      output = `Error: ${error instanceof Error ? error.message : 'Unknown error'}`;
    }
  }

  function decodeHTML() {
    if (!output.trim()) {
      input = '';
      return;
    }

    try {
      const div = document.createElement('div');
      div.innerHTML = output;
      input = div.textContent || div.innerText || '';
    } catch (error) {
      input = `Error: ${error instanceof Error ? error.message : 'Invalid HTML entities'}`;
    }
  }

  function process() {
    if (encodeType === 'base64') {
      if (isEncoding) {
        encodeBase64();
      } else {
        decodeBase64();
      }
    } else if (encodeType === 'image-base64') {
      if (isEncoding) {
        encodeImageBase64();
      } else {
        decodeImageBase64();
      }
    } else if (encodeType === 'url') {
      if (isEncoding) {
        encodeURL();
      } else {
        decodeURL();
      }
    } else if (encodeType === 'ascii') {
      if (isEncoding) {
        encodeASCII();
      } else {
        decodeASCII();
      }
    } else if (encodeType === 'html') {
      if (isEncoding) {
        encodeHTML();
      } else {
        decodeHTML();
      }
    }
  }

  async function copyToClipboard() {
    const text = isEncoding ? output : input;
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
    output = '';
    selectedImageFile = null;
    // 重置文件输入
    const fileInput = document.getElementById('image-file-input') as HTMLInputElement;
    if (fileInput) {
      fileInput.value = '';
    }
    // 强制垃圾回收提示（浏览器会自动处理，但显式清理有助于内存释放）
  }

  function handleImageFileSelect(event: Event) {
    const target = event.target as HTMLInputElement;
    const file = target.files?.[0];
    if (file && file.type.startsWith('image/')) {
      // 清理旧的图片数据
      if (selectedImageFile) {
        // 清空旧的 input 和 output，释放内存
        input = '';
        output = '';
      }
      selectedImageFile = file;
      readImageFile(file);
    }
  }

  function handleImageFileDrop(event: DragEvent) {
    event.preventDefault();
    const file = event.dataTransfer?.files[0];
    if (file && file.type.startsWith('image/')) {
      // 清理旧的图片数据
      if (selectedImageFile) {
        // 清空旧的 input 和 output，释放内存
        input = '';
        output = '';
      }
      selectedImageFile = file;
      readImageFile(file);
    }
  }

  function handleDragOver(event: DragEvent) {
    event.preventDefault();
  }

  function readImageFile(file: File) {
    // 如果文件太大（超过 10MB），给出警告
    const maxSize = 10 * 1024 * 1024; // 10MB
    if (file.size > maxSize) {
      input = `Error: Image file is too large (${(file.size / 1024 / 1024).toFixed(2)}MB). Maximum size is 10MB.`;
      return;
    }

    const reader = new FileReader();
    reader.onload = (e) => {
      const result = e.target?.result as string;
      if (result) {
        input = result;
      }
    };
    reader.onerror = () => {
      input = `Error: Failed to read image file`;
    };
    reader.readAsDataURL(file);
    
    // 清理 FileReader 引用，帮助垃圾回收
    // FileReader 会在读取完成后自动清理，但我们可以显式处理
  }

  function openImageFileDialog() {
    const fileInput = document.getElementById('image-file-input') as HTMLInputElement;
    fileInput?.click();
  }

  function isValidImageDataUri(str: string): boolean {
    return str.trim().startsWith('data:image/') && str.includes(';base64,');
  }

  function downloadImage() {
    if (!input || !isValidImageDataUri(input)) return;

    try {
      // 从 data URI 中提取 MIME 类型和 Base64 数据
      const [prefix, base64] = input.split(',');
      const mimeType = prefix.match(/data:([^;]+)/)?.[1] || 'image/png';
      const extension = mimeType.split('/')[1] || 'png';
      
      // 创建下载链接
      const link = document.createElement('a');
      link.href = input;
      link.download = `image.${extension}`;
      document.body.appendChild(link);
      link.click();
      document.body.removeChild(link);
    } catch (error) {
      console.error('Failed to download image:', error);
    }
  }

  function switchEncodeType(type: EncodeType) {
    encodeType = type;
    input = '';
    output = '';
    selectedImageFile = null;
    // JWT state reset
    jwtToken = '';
    decodedHeader = '';
    decodedPayload = '';
    signature = '';
    jwtError = '';
    jwtCopied = { header: false, payload: false, signature: false };
    // 清理文件输入
    const fileInput = document.getElementById('image-file-input') as HTMLInputElement;
    if (fileInput) {
      fileInput.value = '';
    }
  }
  
  // JWT decoding functions
  function base64UrlDecode(str: string): string {
    // 添加 padding 如果需要
    let base64 = str.replace(/-/g, '+').replace(/_/g, '/');
    while (base64.length % 4) {
      base64 += '=';
    }
    try {
      return decodeURIComponent(escape(atob(base64)));
    } catch (e) {
      throw new Error('Invalid base64url encoding');
    }
  }

  function decodeJWT() {
    jwtError = '';
    decodedHeader = '';
    decodedPayload = '';
    signature = '';

    if (!jwtToken.trim()) {
      return;
    }

    try {
      // 移除可能的 Bearer 前缀
      let token = jwtToken.trim();
      if (token.startsWith('Bearer ') || token.startsWith('bearer ')) {
        token = token.substring(7).trim();
      }

      // 分割 JWT 的三个部分
      const parts = token.split('.');
      if (parts.length !== 3) {
        throw new Error('Invalid JWT format. JWT should have three parts separated by dots.');
      }

      const [headerPart, payloadPart, signaturePart] = parts;

      // 解码 Header
      try {
        const headerJson = base64UrlDecode(headerPart);
        const headerObj = JSON.parse(headerJson);
        decodedHeader = JSON.stringify(headerObj, null, 2);
      } catch (e) {
        throw new Error(`Failed to decode header: ${e instanceof Error ? e.message : 'Unknown error'}`);
      }

      // 解码 Payload
      try {
        const payloadJson = base64UrlDecode(payloadPart);
        const payloadObj = JSON.parse(payloadJson);
        
        // 格式化日期字段（如果存在）
        if (payloadObj.exp) {
          payloadObj.exp_formatted = new Date(payloadObj.exp * 1000).toISOString();
        }
        if (payloadObj.iat) {
          payloadObj.iat_formatted = new Date(payloadObj.iat * 1000).toISOString();
        }
        if (payloadObj.nbf) {
          payloadObj.nbf_formatted = new Date(payloadObj.nbf * 1000).toISOString();
        }
        
        decodedPayload = JSON.stringify(payloadObj, null, 2);
      } catch (e) {
        throw new Error(`Failed to decode payload: ${e instanceof Error ? e.message : 'Unknown error'}`);
      }

      // 保存签名（不解码，因为它是签名的哈希值）
      signature = signaturePart;

    } catch (err) {
      jwtError = err instanceof Error ? err.message : 'Unknown error occurred';
    }
  }

  async function copyJWTToClipboard(text: string, type: 'header' | 'payload' | 'signature') {
    if (!text) return;
    
    try {
      await navigator.clipboard.writeText(text);
      jwtCopied = { ...jwtCopied, [type]: true };
      setTimeout(() => {
        jwtCopied = { ...jwtCopied, [type]: false };
      }, 2000);
    } catch (error) {
      console.error('Failed to copy:', error);
    }
  }

  function clearJWT() {
    jwtToken = '';
    decodedHeader = '';
    decodedPayload = '';
    signature = '';
    jwtError = '';
    jwtCopied = { header: false, payload: false, signature: false };
  }

  // 组件卸载时清理资源
  onDestroy(() => {
    input = '';
    output = '';
    selectedImageFile = null;
    jwtToken = '';
    decodedHeader = '';
    decodedPayload = '';
    signature = '';
    jwtError = '';
  });
  
  // JWT auto decode
  $effect(() => {
    if (encodeType === 'jwt' && jwtToken.trim()) {
      decodeJWT();
    } else if (encodeType === 'jwt' && !jwtToken.trim()) {
      decodedHeader = '';
      decodedPayload = '';
      signature = '';
      jwtError = '';
    }
  });

  $effect(() => {
    if (isEncoding) {
      if (input) {
        process();
      } else {
        output = '';
      }
    } else {
      if (output) {
        process();
      } else {
        input = '';
      }
    }
  });
</script>

<div class="flex flex-col h-full w-full ml-0 mr-0 p-2">
  <!-- 输入区域卡片 -->
  <div class="card flex-1 flex flex-col">
    <div class="flex-1 flex flex-col space-y-4">
      <!-- 编解码类型切换 -->
      <div class="border-b border-gray-200 dark:border-gray-700">
        <div class="flex gap-6">
          <button
            onclick={() => switchEncodeType('base64')}
            class="px-4 py-2 relative transition-colors font-medium {encodeType === 'base64'
              ? 'text-primary-600 dark:text-primary-400'
              : 'text-gray-500 dark:text-gray-400 hover:text-gray-700 dark:hover:text-gray-300'}"
          >
            Base64
            {#if encodeType === 'base64'}
              <span class="absolute bottom-0 left-0 right-0 h-0.5 bg-primary-600 dark:bg-primary-400"></span>
            {/if}
          </button>
          <button
            onclick={() => switchEncodeType('image-base64')}
            class="px-4 py-2 relative transition-colors font-medium {encodeType === 'image-base64'
              ? 'text-primary-600 dark:text-primary-400'
              : 'text-gray-500 dark:text-gray-400 hover:text-gray-700 dark:hover:text-gray-300'}"
          >
            {t('encodeDecode.imageBase64')}
            {#if encodeType === 'image-base64'}
              <span class="absolute bottom-0 left-0 right-0 h-0.5 bg-primary-600 dark:bg-primary-400"></span>
            {/if}
          </button>
          <button
            onclick={() => switchEncodeType('url')}
            class="px-4 py-2 relative transition-colors font-medium {encodeType === 'url'
              ? 'text-primary-600 dark:text-primary-400'
              : 'text-gray-500 dark:text-gray-400 hover:text-gray-700 dark:hover:text-gray-300'}"
          >
            URL
            {#if encodeType === 'url'}
              <span class="absolute bottom-0 left-0 right-0 h-0.5 bg-primary-600 dark:bg-primary-400"></span>
            {/if}
          </button>
          <button
            onclick={() => switchEncodeType('ascii')}
            class="px-4 py-2 relative transition-colors font-medium {encodeType === 'ascii'
              ? 'text-primary-600 dark:text-primary-400'
              : 'text-gray-500 dark:text-gray-400 hover:text-gray-700 dark:hover:text-gray-300'}"
          >
            ASCII
            {#if encodeType === 'ascii'}
              <span class="absolute bottom-0 left-0 right-0 h-0.5 bg-primary-600 dark:bg-primary-400"></span>
            {/if}
          </button>
          <button
            onclick={() => switchEncodeType('jwt')}
            class="px-4 py-2 relative transition-colors font-medium {encodeType === 'jwt'
              ? 'text-primary-600 dark:text-primary-400'
              : 'text-gray-500 dark:text-gray-400 hover:text-gray-700 dark:hover:text-gray-300'}"
          >
            JWT
            {#if encodeType === 'jwt'}
              <span class="absolute bottom-0 left-0 right-0 h-0.5 bg-primary-600 dark:text-primary-400"></span>
            {/if}
          </button>
          <button
            onclick={() => switchEncodeType('html')}
            class="px-4 py-2 relative transition-colors font-medium {encodeType === 'html'
              ? 'text-primary-600 dark:text-primary-400'
              : 'text-gray-500 dark:text-gray-400 hover:text-gray-700 dark:hover:text-gray-300'}"
          >
            {t('encodeDecode.html')}
            {#if encodeType === 'html'}
              <span class="absolute bottom-0 left-0 right-0 h-0.5 bg-primary-600 dark:text-primary-400"></span>
            {/if}
          </button>
        </div>
      </div>

      <!-- JWT Decoder UI -->
      {#if encodeType === 'jwt'}
        <div class="flex-1 flex flex-col space-y-4 min-h-0">
          <!-- 输入区域 -->
          <div class="flex-shrink-0">
            <span class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
              {t('jwt.input')}
            </span>
            <textarea
              bind:value={jwtToken}
              placeholder={t('jwt.placeholder')}
              class="textarea font-mono text-sm min-h-[120px]"
            ></textarea>
          </div>

          <div class="flex gap-2 flex-shrink-0">
            <button
              onclick={decodeJWT}
              class="btn-secondary"
            >
              {t('jwt.decode')}
            </button>
            <button
              onclick={clearJWT}
              class="btn-secondary"
            >
              <Trash2 class="w-4 h-4 inline mr-1" />
              {t('common.clear')}
            </button>
          </div>

          {#if jwtError}
            <div class="p-4 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg flex-shrink-0">
              <p class="text-sm text-red-800 dark:text-red-200">{jwtError}</p>
            </div>
          {/if}

          <!-- 解码结果 -->
          {#if decodedHeader || decodedPayload || signature}
            <div class="grid grid-cols-1 md:grid-cols-3 gap-3 flex-1 min-h-0 overflow-y-auto">
              <!-- Header -->
              <div class="card flex flex-col h-full">
                <div class="flex items-center justify-between mb-2 flex-shrink-0">
                  <h3 class="text-base font-semibold text-gray-900 dark:text-gray-100">
                    {t('jwt.header')}
                  </h3>
                  {#if decodedHeader}
                    <button
                      onclick={() => copyJWTToClipboard(decodedHeader, 'header')}
                      class="btn-secondary text-sm transition-all duration-200 {jwtCopied.header ? 'bg-green-500 hover:bg-green-600 text-white' : ''}"
                    >
                      {#if jwtCopied.header}
                        <Check class="w-4 h-4 inline mr-1" />
                        {t('common.copied')}
                      {:else}
                        <Copy class="w-4 h-4 inline mr-1" />
                        {t('common.copy')}
                      {/if}
                    </button>
                  {/if}
                </div>
                <div class="flex-1 flex flex-col min-h-0">
                  {#if decodedHeader}
                    <textarea
                      value={decodedHeader}
                      readonly
                      class="textarea font-mono text-sm flex-1 resize-none overflow-y-auto {jwtCopied.header ? 'bg-green-50 dark:bg-green-900/20 border-green-300 dark:border-green-700' : ''} transition-colors duration-300"
                    ></textarea>
                  {:else}
                    <div class="bg-gray-50 dark:bg-gray-900 border border-gray-300 dark:border-gray-600 rounded-lg px-3 py-2 flex-1 flex items-center justify-center min-h-[150px]">
                      <span class="text-sm text-gray-400 dark:text-gray-500">{t('jwt.noHeader')}</span>
                    </div>
                  {/if}
                </div>
              </div>

              <!-- Payload -->
              <div class="card flex flex-col h-full">
                <div class="flex items-center justify-between mb-2 flex-shrink-0">
                  <h3 class="text-base font-semibold text-gray-900 dark:text-gray-100">
                    {t('jwt.payload')}
                  </h3>
                  {#if decodedPayload}
                    <button
                      onclick={() => copyJWTToClipboard(decodedPayload, 'payload')}
                      class="btn-secondary text-sm transition-all duration-200 {jwtCopied.payload ? 'bg-green-500 hover:bg-green-600 text-white' : ''}"
                    >
                      {#if jwtCopied.payload}
                        <Check class="w-4 h-4 inline mr-1" />
                        {t('common.copied')}
                      {:else}
                        <Copy class="w-4 h-4 inline mr-1" />
                        {t('common.copy')}
                      {/if}
                    </button>
                  {/if}
                </div>
                <div class="flex-1 flex flex-col min-h-0">
                  {#if decodedPayload}
                    <textarea
                      value={decodedPayload}
                      readonly
                      class="textarea font-mono text-sm flex-1 resize-none overflow-y-auto {jwtCopied.payload ? 'bg-green-50 dark:bg-green-900/20 border-green-300 dark:border-green-700' : ''} transition-colors duration-300"
                    ></textarea>
                  {:else}
                    <div class="bg-gray-50 dark:bg-gray-900 border border-gray-300 dark:border-gray-600 rounded-lg px-3 py-2 flex-1 flex items-center justify-center min-h-[150px]">
                      <span class="text-sm text-gray-400 dark:text-gray-500">{t('jwt.noPayload')}</span>
                    </div>
                  {/if}
                </div>
              </div>

              <!-- Signature -->
              <div class="card flex flex-col h-full">
                <div class="flex items-center justify-between mb-2 flex-shrink-0">
                  <h3 class="text-base font-semibold text-gray-900 dark:text-gray-100">
                    {t('jwt.signature')}
                  </h3>
                  {#if signature}
                    <button
                      onclick={() => copyJWTToClipboard(signature, 'signature')}
                      class="btn-secondary text-sm transition-all duration-200 {jwtCopied.signature ? 'bg-green-500 hover:bg-green-600 text-white' : ''}"
                    >
                      {#if jwtCopied.signature}
                        <Check class="w-4 h-4 inline mr-1" />
                        {t('common.copied')}
                      {:else}
                        <Copy class="w-4 h-4 inline mr-1" />
                        {t('common.copy')}
                      {/if}
                    </button>
                  {/if}
                </div>
                <div class="flex-1 flex flex-col min-h-0">
                  {#if signature}
                    <div class="bg-gray-50 dark:bg-gray-900 border border-gray-300 dark:border-gray-600 rounded-lg px-3 py-2 flex-1 flex flex-col overflow-y-auto min-h-[150px]">
                      <code class="text-sm font-mono text-gray-900 dark:text-gray-100 break-all">
                        {signature}
                      </code>
                      <p class="text-xs text-gray-500 dark:text-gray-400 mt-1.5">
                        {t('jwt.signatureHint')}
                      </p>
                    </div>
                  {:else}
                    <div class="bg-gray-50 dark:bg-gray-900 border border-gray-300 dark:border-gray-600 rounded-lg px-3 py-2 flex-1 flex items-center justify-center min-h-[150px]">
                      <span class="text-sm text-gray-400 dark:text-gray-500">{t('jwt.noSignature')}</span>
                    </div>
                  {/if}
                </div>
              </div>
            </div>
          {/if}
        </div>
      {:else}
      <!-- 左右布局：输入 - 按钮 - 输出 -->
      <div class="flex-1 grid grid-cols-12 gap-4 items-stretch">
        <!-- 左侧输入区域 -->
        <div class="col-span-5 flex flex-col space-y-2">
          <div class="flex items-center justify-between">
            <div class="block text-base font-bold text-gray-700 dark:text-gray-300">
              {encodeType === 'base64' 
                ? t('encodeDecode.plaintext')
                : encodeType === 'image-base64'
                ? t('encodeDecode.image')
                : t('encodeDecode.plaintext')}
            </div>
            {#if !isEncoding && input}
              <div class="flex items-center gap-2">
                {#if encodeType === 'image-base64' && isValidImageDataUri(input)}
                  <button
                    onclick={downloadImage}
                    class="btn-secondary text-xs px-3 py-1.5 transition-all duration-200 flex items-center gap-1"
                    title={t('encodeDecode.downloadImage')}
                  >
                    <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4"></path>
                    </svg>
                    {t('encodeDecode.download')}
                  </button>
                {/if}
                <button
                  onclick={copyToClipboard}
                  class="btn-secondary text-xs px-3 py-1.5 transition-all duration-200 {copied ? 'bg-green-500 hover:bg-green-600 text-white' : ''}"
                >
                  {#if copied}
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
              </div>
            {/if}
          </div>
          <div class="relative flex-1">
            {#if encodeType === 'image-base64' && isEncoding}
              <!-- 图片文件选择区域 -->
              <input
                type="file"
                id="image-file-input"
                accept="image/*"
                onchange={handleImageFileSelect}
                class="hidden"
              />
              <div
                role="button"
                tabindex="0"
                aria-label={t('encodeDecode.selectImage')}
                onclick={openImageFileDialog}
                onkeydown={(e) => {
                  if (e.key === 'Enter' || e.key === ' ') {
                    e.preventDefault();
                    openImageFileDialog();
                  }
                }}
                ondragover={handleDragOver}
                ondrop={handleImageFileDrop}
                class="border-2 border-dashed border-gray-300 dark:border-gray-600 rounded-lg h-full flex flex-col items-center justify-center cursor-pointer hover:border-primary-500 dark:hover:border-primary-400 transition-colors {input ? 'border-primary-500 dark:border-primary-400 bg-primary-50 dark:bg-primary-900/20' : ''}"
              >
                {#if selectedImageFile}
                  <div class="text-center space-y-2 p-4">
                    <svg class="w-12 h-12 mx-auto text-primary-600 dark:text-primary-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16l4.586-4.586a2 2 0 012.828 0L16 16m-2-2l1.586-1.586a2 2 0 012.828 0L20 14m-6-6h.01M6 20h12a2 2 0 002-2V6a2 2 0 00-2-2H6a2 2 0 00-2 2v12a2 2 0 002 2z"></path>
                    </svg>
                    <p class="text-sm font-medium text-gray-700 dark:text-gray-300">{selectedImageFile.name}</p>
                    <p class="text-xs text-gray-500 dark:text-gray-400">{t('encodeDecode.clickToSelectImage')}</p>
                  </div>
                {:else}
                  <div class="text-center space-y-2 p-4">
                    <svg class="w-12 h-12 mx-auto text-gray-400 dark:text-gray-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16l4.586-4.586a2 2 0 012.828 0L16 16m-2-2l1.586-1.586a2 2 0 012.828 0L20 14m-6-6h.01M6 20h12a2 2 0 002-2V6a2 2 0 00-2-2H6a2 2 0 00-2 2v12a2 2 0 002 2z"></path>
                    </svg>
                    <p class="text-sm font-medium text-gray-700 dark:text-gray-300">{t('encodeDecode.selectImage')}</p>
                    <p class="text-xs text-gray-500 dark:text-gray-400">{t('encodeDecode.dragDropImage')}</p>
                  </div>
                {/if}
              </div>
            {:else if encodeType === 'image-base64' && !isEncoding && isValidImageDataUri(input)}
              <!-- 图片预览区域（解码模式） -->
              <div class="border-2 border-gray-300 dark:border-gray-600 rounded-lg h-full flex items-center justify-center bg-gray-50 dark:bg-gray-800/50 overflow-hidden">
                <img
                  src={input}
                  alt=""
                  class="max-w-full max-h-full object-contain"
                  onerror={(e) => {
                    // 如果图片加载失败，显示错误信息
                    e.currentTarget.style.display = 'none';
                  }}
                />
              </div>
              <!-- 隐藏的 textarea 用于存储 data URI -->
              <textarea
                id="encode-decode-input"
                bind:value={input}
                readonly={true}
                class="hidden"
              ></textarea>
            {:else}
              <!-- 普通文本输入区域 -->
              <textarea
                id="encode-decode-input"
                bind:value={input}
                readonly={!isEncoding}
                placeholder={isEncoding 
                  ? (encodeType === 'base64' ? t('encodeDecode.encodeBase64Placeholder') : encodeType === 'image-base64' ? t('encodeDecode.encodeImageBase64Placeholder') : encodeType === 'url' ? t('encodeDecode.encodeURLPlaceholder') : encodeType === 'html' ? t('encodeDecode.encodeHTMLPlaceholder') : t('encodeDecode.encodeASCIIPlaceholder'))
                  : ''}
                class="textarea h-full resize-none {!isEncoding ? 'bg-gray-50 dark:bg-gray-800/50 cursor-not-allowed' : ''} {!isEncoding && input ? 'bg-green-50 dark:bg-green-900/20 border-green-300 dark:border-green-700' : ''} transition-colors duration-300"
              ></textarea>
            {/if}
          </div>
        </div>

        <!-- 中间按钮区域 -->
        <div class="col-span-2 flex flex-col justify-center gap-3 px-2">
          <button
            onclick={() => { isEncoding = true; process(); }}
            class="px-4 py-3 rounded-lg transition-colors font-medium flex items-center justify-center gap-2 {isEncoding
              ? 'text-white'
              : 'bg-gray-200 dark:bg-gray-700 text-gray-900 dark:text-gray-100'}"
            style={isEncoding ? 'background-color: #818089;' : ''}
          >
            {t('encodeDecode.encode')}
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7"></path>
            </svg>
          </button>
          <button
            onclick={() => { isEncoding = false; process(); }}
            class="px-4 py-3 rounded-lg transition-colors font-medium flex items-center justify-center gap-2 {!isEncoding
              ? 'text-white'
              : 'bg-gray-200 dark:bg-gray-700 text-gray-900 dark:text-gray-100'}"
            style={!isEncoding ? 'background-color: #818089;' : ''}
          >
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7"></path>
            </svg>
            {t('encodeDecode.decode')}
          </button>
          <button onclick={clear} class="btn-secondary px-4 py-3 mt-2">
            {t('encodeDecode.clear')}
          </button>
        </div>

        <!-- 右侧输出区域 -->
        <div class="col-span-5 flex flex-col space-y-2">
          <div class="flex items-center justify-between">
            <div class="block text-base font-bold text-gray-700 dark:text-gray-300">
              {encodeType === 'base64' 
                ? t('encodeDecode.base64')
                : encodeType === 'image-base64'
                ? t('encodeDecode.base64')
                : encodeType === 'url'
                ? t('encodeDecode.urlEncoded')
                : encodeType === 'html'
                ? t('encodeDecode.html')
                : t('encodeDecode.ascii')}
            </div>
            {#if isEncoding && output}
              <button
                onclick={copyToClipboard}
                class="btn-secondary text-xs px-3 py-1.5 transition-all duration-200 {copied ? 'bg-green-500 hover:bg-green-600 text-white' : ''}"
              >
                {#if copied}
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
          <div class="relative flex-1">
            <textarea
              bind:value={output}
              readonly={isEncoding}
              placeholder={!isEncoding 
                ? (encodeType === 'base64' ? t('encodeDecode.decodeBase64Placeholder') : encodeType === 'image-base64' ? t('encodeDecode.decodeImageBase64Placeholder') : encodeType === 'url' ? t('encodeDecode.decodeURLPlaceholder') : encodeType === 'html' ? t('encodeDecode.decodeHTMLPlaceholder') : t('encodeDecode.decodeASCIIPlaceholder'))
                : ''}
              class="textarea h-full resize-none font-mono text-sm {copied ? 'bg-green-50 dark:bg-green-900/20 border-green-300 dark:border-green-700' : ''} {isEncoding ? 'bg-gray-50 dark:bg-gray-800/50 cursor-not-allowed' : ''} transition-colors duration-300"
            ></textarea>
          </div>
        </div>
      </div>
      {/if}
    </div>
  </div>
</div>

