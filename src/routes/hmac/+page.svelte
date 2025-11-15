<script lang="ts">
  import { translationsStore } from '$lib/stores/i18n';
  import { Copy, Check } from 'lucide-svelte';

  let translations = $derived($translationsStore);

  function t(key: string): string {
    const keys = key.split('.');
    let value: any = translations;
    for (const k of keys) {
      value = value?.[k];
    }
    return value || key;
  }

  type HashAlgorithm = 'SHA-1' | 'SHA-256' | 'SHA-512';

  let key = $state('');
  let message = $state('');
  let algorithm = $state<HashAlgorithm>('SHA-256');
  let outputFormat = $state<'hex' | 'base64'>('hex');
  let hmacResult = $state('');
  let copied = $state(false);
  let error = $state('');

  // 将密钥字符串转换为字节数组
  function keyToBytes(keyStr: string): Uint8Array {
    return new TextEncoder().encode(keyStr);
  }

  // 生成 HMAC
  async function generateHMAC() {
    error = '';
    hmacResult = '';

    if (!key.trim()) {
      error = t('hmac.keyRequired');
      return;
    }

    if (!message.trim()) {
      error = t('hmac.messageRequired');
      return;
    }

    try {
      const keyBytes = keyToBytes(key);
      const messageBytes = new TextEncoder().encode(message);

      const cryptoKey = await crypto.subtle.importKey(
        'raw',
        keyBytes,
        { name: 'HMAC', hash: algorithm },
        false,
        ['sign']
      );

      const signature = await crypto.subtle.sign('HMAC', cryptoKey, messageBytes);
      const signatureArray = new Uint8Array(signature);

      if (outputFormat === 'hex') {
        // 转换为十六进制
        hmacResult = Array.from(signatureArray)
          .map(b => b.toString(16).padStart(2, '0'))
          .join('');
      } else {
        // 转换为 Base64
        const binaryString = String.fromCharCode(...signatureArray);
        hmacResult = btoa(binaryString);
      }
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    }
  }

  // 复制到剪贴板
  async function copyToClipboard() {
    if (!hmacResult) return;
    try {
      await navigator.clipboard.writeText(hmacResult);
      copied = true;
      setTimeout(() => {
        copied = false;
      }, 2000);
    } catch (error) {
      console.error('Failed to copy:', error);
    }
  }

  // 清空
  function clearAll() {
    key = '';
    message = '';
    hmacResult = '';
    error = '';
  }
</script>

<div class="flex flex-col h-full w-full ml-0 mr-0 p-2">
  <div class="card flex-1 flex flex-col">
    <div class="flex-1 flex flex-col space-y-6">
      <!-- 输入区域 -->
      <div class="space-y-4">
        <h2 class="text-lg font-semibold text-gray-900 dark:text-gray-100">
          {t('hmac.input')}
        </h2>

        <!-- 密钥 -->
        <div>
          <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
            {t('hmac.key')}
          </label>
          <textarea
            bind:value={key}
            placeholder={t('hmac.keyPlaceholder')}
            class="textarea w-full min-h-[50px] font-mono text-sm"
          ></textarea>
        </div>

        <!-- 消息 -->
        <div>
          <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
            {t('hmac.message')}
          </label>
          <textarea
            bind:value={message}
            placeholder={t('hmac.messagePlaceholder')}
            class="textarea w-full min-h-[50px] font-mono text-sm"
          ></textarea>
        </div>

        <!-- 算法选择 -->
        <div>
          <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
            {t('hmac.algorithm')}
          </label>
          <select
            bind:value={algorithm}
            class="input w-full"
          >
            <option value="SHA-1">SHA-1</option>
            <option value="SHA-256">SHA-256</option>
            <option value="SHA-512">SHA-512</option>
          </select>
        </div>

        <!-- 输出格式 -->
        <div>
          <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
            {t('hmac.outputFormat')}
          </label>
          <div class="flex gap-2">
            <button
              onclick={() => { outputFormat = 'hex'; generateHMAC(); }}
              class="flex-1 px-4 py-2 rounded-lg border transition-colors {outputFormat === 'hex' ? 'bg-primary-100 dark:bg-primary-900/30 border-primary-500 dark:border-primary-400 text-primary-700 dark:text-primary-300' : 'bg-white dark:bg-gray-800 border-gray-300 dark:border-gray-600 text-gray-700 dark:text-gray-300'}"
            >
              HEX
            </button>
            <button
              onclick={() => { outputFormat = 'base64'; generateHMAC(); }}
              class="flex-1 px-4 py-2 rounded-lg border transition-colors {outputFormat === 'base64' ? 'bg-primary-100 dark:bg-primary-900/30 border-primary-500 dark:border-primary-400 text-primary-700 dark:text-primary-300' : 'bg-white dark:bg-gray-800 border-gray-300 dark:border-gray-600 text-gray-700 dark:text-gray-300'}"
            >
              Base64
            </button>
          </div>
        </div>

        <!-- 错误提示 -->
        {#if error}
          <div class="p-3 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg">
            <p class="text-sm text-red-600 dark:text-red-400">{error}</p>
          </div>
        {/if}

        <!-- 操作按钮 -->
        <div class="flex gap-2">
          <button
            onclick={generateHMAC}
            class="btn-primary flex-1"
          >
            {t('hmac.generate')}
          </button>
          <button
            onclick={clearAll}
            class="btn-secondary"
          >
            {t('hmac.clear')}
          </button>
        </div>
      </div>

      <!-- 输出区域 -->
      {#if hmacResult}
        <div class="space-y-4">
          <div class="flex items-center justify-between">
            <h2 class="text-lg font-semibold text-gray-900 dark:text-gray-100">
              {t('hmac.result')}
            </h2>
            <button
              onclick={copyToClipboard}
              class="btn-secondary transition-all duration-200 {copied ? 'bg-green-500 hover:bg-green-600 text-white' : ''}"
            >
              {#if copied}
                <Check class="w-4 h-4 inline mr-1" />
                {t('common.copied')}
              {:else}
                <Copy class="w-4 h-4 inline mr-1" />
                {t('common.copy')}
              {/if}
            </button>
          </div>
          <div class="bg-gray-50 dark:bg-gray-900 border border-gray-300 dark:border-gray-600 rounded-lg p-4">
            <code class="text-sm font-mono text-gray-900 dark:text-gray-100 break-all">
              {hmacResult}
            </code>
          </div>
        </div>
      {:else}
        <div class="flex-1 flex items-center justify-center">
          <p class="text-sm text-gray-400 dark:text-gray-500">
            {t('hmac.placeholder')}
          </p>
        </div>
      {/if}
    </div>
  </div>
</div>

