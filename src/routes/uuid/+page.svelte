<script lang="ts">
  import { translationsStore } from '$lib/stores/i18n';
  
  import CryptoJS from 'crypto-js';
  
  type GeneratorType = 'uuid' | 'ulid';
  type UUIDVersion = 'v1' | 'v3' | 'v4' | 'v5';
  
  let generatorType = $state<GeneratorType>('uuid');
  let uuidVersion = $state<UUIDVersion>('v4');
  let namespaceName = $state('');
  let customName = $state('');
  let count = $state(1);
  let uuids = $state<string[]>([]);
  let copiedUuids = $state<Set<number>>(new Set());
  let allCopied = $state(false);
  let includeHyphens = $state(true);
  
  // 预定义的命名空间
  const NAMESPACES = {
    DNS: '6ba7b810-9dad-11d1-80b4-00c04fd430c8',
    URL: '6ba7b811-9dad-11d1-80b4-00c04fd430c8',
    OID: '6ba7b812-9dad-11d1-80b4-00c04fd430c8',
    X500: '6ba7b814-9dad-11d1-80b4-00c04fd430c8',
  };

  let translations = $derived($translationsStore);

  function t(key: string): string {
    const keys = key.split('.');
    let value: any = translations;
    for (const k of keys) {
      value = value?.[k];
    }
    return value || key;
  }

  // Base32 字符集（ULID 使用，排除 I, L, O, U）
  const BASE32_CHARS = '0123456789ABCDEFGHJKMNPQRSTVWXYZ';

  function encodeBase32(value: number, length: number): string {
    let result = '';
    for (let i = 0; i < length; i++) {
      const remainder = value % 32;
      result = BASE32_CHARS[remainder] + result;
      value = Math.floor(value / 32);
    }
    return result;
  }

  function generateULID(): string {
    // 时间戳部分（48 位，10 个 Base32 字符）
    const timestamp = Date.now();
    
    // ULID 使用 48 位时间戳（从 Unix 纪元开始的毫秒数）
    // 当前时间戳约为 13 位数字，远小于 48 位最大值
    // 48 位最大值：281474976710655 (可以表示到 8925 年)
    // 直接使用时间戳，不需要截断
    const timePart = encodeBase32(timestamp, 10);

    // 随机部分（80 位，16 个 Base32 字符）
    let randomPart = '';
    const randomBytes = new Uint8Array(10);
    crypto.getRandomValues(randomBytes);
    
    // 将 10 字节（80 位）转换为 16 个 Base32 字符
    // 每次处理 5 位（Base32）
    let randomValue = 0;
    let bitCount = 0;
    for (let i = 0; i < 10; i++) {
      randomValue = (randomValue << 8) | randomBytes[i];
      bitCount += 8;
      
      while (bitCount >= 5) {
        randomPart += BASE32_CHARS[randomValue & 0x1f];
        randomValue = randomValue >>> 5;
        bitCount -= 5;
      }
    }
    
    // 处理剩余的位数（确保有 16 个字符）
    if (bitCount > 0) {
      randomPart += BASE32_CHARS[randomValue & 0x1f];
    }
    
    // 确保随机部分是 16 个字符
    while (randomPart.length < 16) {
      randomPart = BASE32_CHARS[Math.floor(Math.random() * 32)] + randomPart;
    }
    randomPart = randomPart.slice(-16);

    return timePart + randomPart;
  }

  function generateUUIDv1() {
    // UUID v1: 基于时间戳和节点 ID
    const now = Date.now();
    const timestamp = now * 10000 + 122192928000000000; // Convert to 100-nanosecond intervals since 1582-10-15
    
    const timeLow = (timestamp & 0xffffffff).toString(16).padStart(8, '0');
    const timeMid = ((timestamp / 0x100000000) & 0xffff).toString(16).padStart(4, '0');
    const timeHi = (((timestamp / 0x1000000000000) & 0x0fff) | 0x1000).toString(16).padStart(4, '0');
    
    const clockSeq = ((Math.random() * 0x3fff) | 0x8000).toString(16).padStart(4, '0');
    
    // 生成随机节点 ID（6 字节）
    const node = Array.from({ length: 6 }, () => 
      Math.floor(Math.random() * 256).toString(16).padStart(2, '0')
    ).join('');
    
    const uuid = `${timeLow}-${timeMid}-${timeHi}-${clockSeq}-${node}`;
    return includeHyphens ? uuid : uuid.replace(/-/g, '');
  }

  function generateUUIDv4() {
    // UUID v4: 随机生成
    const uuid = 'xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx'.replace(/[xy]/g, (c) => {
      const r = (Math.random() * 16) | 0;
      const v = c === 'x' ? r : (r & 0x3) | 0x8;
      return v.toString(16);
    });
    return includeHyphens ? uuid : uuid.replace(/-/g, '');
  }

  function generateUUIDv3(namespace: string, name: string) {
    // UUID v3: 使用 MD5 哈希
    const namespaceBytes = namespace.replace(/-/g, '');
    const hash = CryptoJS.MD5(namespaceBytes + name).toString();
    
    const uuid = [
      hash.substring(0, 8),
      hash.substring(8, 12),
      '3' + hash.substring(13, 16), // 版本号 3
      ((parseInt(hash.substring(16, 18), 16) & 0x3f) | 0x80).toString(16) + hash.substring(18, 20),
      hash.substring(20, 32)
    ].join('-');
    
    return includeHyphens ? uuid : uuid.replace(/-/g, '');
  }

  function generateUUIDv5(namespace: string, name: string) {
    // UUID v5: 使用 SHA-1 哈希
    const namespaceBytes = namespace.replace(/-/g, '');
    const hash = CryptoJS.SHA1(namespaceBytes + name).toString();
    
    const uuid = [
      hash.substring(0, 8),
      hash.substring(8, 12),
      '5' + hash.substring(13, 16), // 版本号 5
      ((parseInt(hash.substring(16, 18), 16) & 0x3f) | 0x80).toString(16) + hash.substring(18, 20),
      hash.substring(20, 32)
    ].join('-');
    
    return includeHyphens ? uuid : uuid.replace(/-/g, '');
  }

  function generateUUID() {
    if (uuidVersion === 'v1') {
      return generateUUIDv1();
    } else if (uuidVersion === 'v3') {
      const namespace = namespaceName || NAMESPACES.DNS;
      const name = customName || 'example.com';
      return generateUUIDv3(namespace, name);
    } else if (uuidVersion === 'v5') {
      const namespace = namespaceName || NAMESPACES.DNS;
      const name = customName || 'example.com';
      return generateUUIDv5(namespace, name);
    } else {
      return generateUUIDv4();
    }
  }

  function generate() {
    uuids = [];
    copiedUuids = new Set();
    for (let i = 0; i < count; i++) {
      if (generatorType === 'uuid') {
        uuids.push(generateUUID());
      } else {
        uuids.push(generateULID());
      }
    }
  }

  async function copyToClipboard(index: number, uuid: string) {
    try {
      await navigator.clipboard.writeText(uuid);
      copiedUuids = new Set(copiedUuids);
      copiedUuids.add(index);
      setTimeout(() => {
        copiedUuids = new Set(copiedUuids);
        copiedUuids.delete(index);
      }, 1000);
    } catch (error) {
      console.error('Failed to copy:', error);
    }
  }

  async function copyAll() {
    if (uuids.length === 0) return;
    
    try {
      const allUuids = uuids.join('\n');
      await navigator.clipboard.writeText(allUuids);
      allCopied = true;
      setTimeout(() => {
        allCopied = false;
      }, 1000);
    } catch (error) {
      console.error('Failed to copy:', error);
    }
  }

  function clear() {
    uuids = [];
    copiedUuids = new Set();
    allCopied = false;
  }
</script>

<div class="w-full ml-0 mr-0 p-2 space-y-6">
  <!-- 输入区域卡片 -->
  <div class="card">
    <div class="space-y-4">
      <!-- 类型切换 -->
      <div class="border-b border-gray-200 dark:border-gray-700">
        <div class="flex gap-6">
          <button
            onclick={() => { generatorType = 'uuid'; clear(); }}
            class="px-4 py-2 relative transition-colors font-medium {generatorType === 'uuid'
              ? 'text-primary-600 dark:text-primary-400'
              : 'text-gray-500 dark:text-gray-400 hover:text-gray-700 dark:hover:text-gray-300'}"
          >
            UUID
            {#if generatorType === 'uuid'}
              <span class="absolute bottom-0 left-0 right-0 h-0.5 bg-primary-600 dark:bg-primary-400"></span>
            {/if}
          </button>
          <button
            onclick={() => { generatorType = 'ulid'; clear(); }}
            class="px-4 py-2 relative transition-colors font-medium {generatorType === 'ulid'
              ? 'text-primary-600 dark:text-primary-400'
              : 'text-gray-500 dark:text-gray-400 hover:text-gray-700 dark:hover:text-gray-300'}"
          >
            ULID
            {#if generatorType === 'ulid'}
              <span class="absolute bottom-0 left-0 right-0 h-0.5 bg-primary-600 dark:bg-primary-400"></span>
            {/if}
          </button>
        </div>
      </div>

      <div>
        <label for="uuid-count" class="block text-sm font-medium mb-2 text-gray-700 dark:text-gray-300">
          {t('uuid.count')}
        </label>
        <input
          id="uuid-count"
          type="number"
          bind:value={count}
          min="1"
          max="100"
          class="input w-32"
        />
      </div>

      {#if generatorType === 'uuid'}
        <div>
          <label for="uuid-version" class="block text-sm font-medium mb-2 text-gray-700 dark:text-gray-300">
            {t('uuid.version')}
          </label>
          <select
            id="uuid-version"
            bind:value={uuidVersion}
            class="input w-48"
          >
            <option value="v1">{t('uuid.v1')}</option>
            <option value="v3">{t('uuid.v3')}</option>
            <option value="v4">{t('uuid.v4')}</option>
            <option value="v5">{t('uuid.v5')}</option>
          </select>
        </div>
        
        {#if uuidVersion === 'v3' || uuidVersion === 'v5'}
          <div>
            <label for="namespace" class="block text-sm font-medium mb-2 text-gray-700 dark:text-gray-300">
              {t('uuid.namespace')}
            </label>
            <select
              id="namespace"
              bind:value={namespaceName}
              class="input"
            >
              <option value="">{t('uuid.namespaceDNS')}</option>
              <option value={NAMESPACES.DNS}>{t('uuid.namespaceDNS')} - {NAMESPACES.DNS}</option>
              <option value={NAMESPACES.URL}>{t('uuid.namespaceURL')} - {NAMESPACES.URL}</option>
              <option value={NAMESPACES.OID}>{t('uuid.namespaceOID')} - {NAMESPACES.OID}</option>
              <option value={NAMESPACES.X500}>{t('uuid.namespaceX500')} - {NAMESPACES.X500}</option>
            </select>
          </div>
          
          <div>
            <label for="custom-name" class="block text-sm font-medium mb-2 text-gray-700 dark:text-gray-300">
              {t('uuid.name')}
            </label>
            <input
              id="custom-name"
              type="text"
              bind:value={customName}
              placeholder={t('uuid.namePlaceholder')}
              class="input"
            />
          </div>
        {/if}
        
        <div class="flex items-center">
          <input
            id="include-hyphens"
            type="checkbox"
            bind:checked={includeHyphens}
            class="w-4 h-4 text-primary-600 bg-gray-100 border-gray-300 rounded focus:ring-primary-500 dark:focus:ring-primary-600 dark:ring-offset-gray-800 focus:ring-2 dark:bg-gray-700 dark:border-gray-600"
          />
          <label for="include-hyphens" class="ml-2 text-sm text-gray-700 dark:text-gray-300">
            {t('uuid.includeHyphens')}
          </label>
        </div>
      {/if}

      <div class="flex gap-2">
        <button 
          onclick={generate} 
          disabled={count < 1 || count > 100}
          class="px-4 py-2 text-white rounded-lg transition-colors font-medium hover:opacity-90 disabled:opacity-50 disabled:cursor-not-allowed"
          style="background-color: #818089;"
        >
          {t('uuid.generate')}
        </button>
        <button onclick={clear} class="btn-secondary">
          {t('uuid.clear')}
        </button>
      </div>
    </div>
  </div>

  <!-- UUID 结果卡片 -->
  {#if uuids.length > 0}
    <div class="card">
      <div class="space-y-4">
        <div class="flex items-center justify-between">
          <h3 class="text-xl font-bold text-gray-900 dark:text-gray-100 mb-1">
            {generatorType === 'uuid' ? t('uuid.generated') : t('uuid.generatedULID')}
          </h3>
          {#if uuids.length > 1}
            <button
              onclick={copyAll}
              class="btn-secondary whitespace-nowrap transition-all duration-200 {allCopied ? 'bg-green-500 hover:bg-green-600 text-white' : ''}"
            >
              {#if allCopied}
                <span class="flex items-center gap-1">
                  <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7"></path>
                  </svg>
                  {t('common.copied')}
                </span>
              {:else}
                {t('uuid.copyAll')}
              {/if}
            </button>
          {/if}
        </div>
        
        <div class="space-y-2">
          {#each uuids as uuidItem, index}
            {@const isCopied = copiedUuids.has(index)}
            <div class="flex gap-2 items-center {isCopied ? 'ring-2 ring-green-500 ring-offset-2 rounded-lg p-2 -m-2' : ''} transition-all duration-300">
              <input
                type="text"
                value={uuidItem}
                readonly
                class="input font-mono text-sm flex-1 {isCopied ? 'bg-green-50 dark:bg-green-900/20 border-green-300 dark:border-green-700' : ''} transition-colors duration-300"
              />
              <button
                onclick={() => copyToClipboard(index, uuidItem)}
                class="btn-secondary whitespace-nowrap transition-all duration-200 {isCopied ? 'bg-green-500 hover:bg-green-600 text-white' : ''}"
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
            </div>
          {/each}
        </div>
      </div>
    </div>
  {/if}
</div>

