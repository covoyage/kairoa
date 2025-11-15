<script lang="ts">
  import { translationsStore } from '$lib/stores/i18n';
  import { Copy, Check, RefreshCw, Eye, EyeOff } from 'lucide-svelte';
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

  type OtpType = 'TOTP' | 'HOTP';
  type HashAlgorithm = 'SHA-1' | 'SHA-256' | 'SHA-512';

  let otpType = $state<OtpType>('TOTP');
  let secret = $state('');
  let showSecret = $state(false);
  let algorithm = $state<HashAlgorithm>('SHA-1');
  let digits = $state<6 | 7 | 8>(6);
  let period = $state<number>(30); // TOTP 时间步长（秒）
  let counter = $state<number>(0); // HOTP 计数器
  let otpCode = $state('');
  let copied = $state(false);
  let error = $state('');
  let autoRefresh = $state(false);
  let refreshInterval: ReturnType<typeof setInterval> | null = null;
  let timeRemaining = $state<number>(0);

  // 将密钥字符串转换为字节数组
  function secretToBytes(secretStr: string): Uint8Array {
    // 移除空格，支持 Base32 格式
    const cleanSecret = secretStr.replace(/\s+/g, '').toUpperCase();
    
    // 尝试 Base32 解码
    try {
      return base32Decode(cleanSecret);
    } catch {
      // 如果不是 Base32，则使用 UTF-8 编码
      return new TextEncoder().encode(secretStr);
    }
  }

  // Base32 解码（简化版）
  function base32Decode(str: string): Uint8Array {
    const base32Chars = 'ABCDEFGHIJKLMNOPQRSTUVWXYZ234567';
    const bits: number[] = [];
    
    for (let i = 0; i < str.length; i++) {
      const char = str[i];
      if (char === '=') break; // Padding
      const val = base32Chars.indexOf(char);
      if (val === -1) throw new Error('Invalid Base32 character');
      
      // 将每个字符转换为 5 位二进制
      for (let j = 4; j >= 0; j--) {
        bits.push((val >> j) & 1);
      }
    }
    
    // 将位转换为字节
    const bytes: number[] = [];
    for (let i = 0; i < bits.length; i += 8) {
      if (i + 8 > bits.length) break;
      let byte = 0;
      for (let j = 0; j < 8; j++) {
        byte = (byte << 1) | bits[i + j];
      }
      bytes.push(byte);
    }
    
    return new Uint8Array(bytes);
  }

  // 生成 HMAC
  async function generateHMAC(key: Uint8Array, data: Uint8Array): Promise<Uint8Array> {
    const cryptoKey = await crypto.subtle.importKey(
      'raw',
      key,
      { name: 'HMAC', hash: algorithm },
      false,
      ['sign']
    );
    
    const signature = await crypto.subtle.sign('HMAC', cryptoKey, data);
    return new Uint8Array(signature);
  }

  // 动态截断
  function dynamicTruncate(hash: Uint8Array): number {
    const offset = hash[hash.length - 1] & 0x0f;
    const binary = ((hash[offset] & 0x7f) << 24) |
                   ((hash[offset + 1] & 0xff) << 16) |
                   ((hash[offset + 2] & 0xff) << 8) |
                   (hash[offset + 3] & 0xff);
    return binary;
  }

  // 生成 TOTP
  async function generateTOTP(): Promise<string> {
    if (!secret.trim()) {
      return '';
    }

    try {
      const key = secretToBytes(secret);
      const time = Math.floor(Date.now() / 1000 / period);
      
      // 将时间转换为 8 字节大端序
      const timeBuffer = new ArrayBuffer(8);
      const timeView = new DataView(timeBuffer);
      timeView.setUint32(4, time, false); // 大端序
      
      const hmac = await generateHMAC(key, new Uint8Array(timeBuffer));
      const code = dynamicTruncate(hmac) % Math.pow(10, digits);
      
      return code.toString().padStart(digits, '0');
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
      return '';
    }
  }

  // 生成 HOTP
  async function generateHOTP(): Promise<string> {
    if (!secret.trim()) {
      return '';
    }

    try {
      const key = secretToBytes(secret);
      
      // 将计数器转换为 8 字节大端序
      const counterBuffer = new ArrayBuffer(8);
      const counterView = new DataView(counterBuffer);
      counterView.setUint32(4, counter, false); // 大端序
      
      const hmac = await generateHMAC(key, new Uint8Array(counterBuffer));
      const code = dynamicTruncate(hmac) % Math.pow(10, digits);
      
      return code.toString().padStart(digits, '0');
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
      return '';
    }
  }

  // 生成 OTP
  async function generateOTP() {
    error = '';
    if (otpType === 'TOTP') {
      otpCode = await generateTOTP();
    } else {
      otpCode = await generateHOTP();
    }
  }

  // 自动刷新 TOTP
  function updateTimeRemaining() {
    if (otpType === 'TOTP' && period > 0) {
      const elapsed = Math.floor(Date.now() / 1000) % period;
      timeRemaining = period - elapsed;
    }
  }

  onMount(() => {
    // 初始生成
    generateOTP();
    
    // 设置定时器更新剩余时间
    const timeInterval = setInterval(() => {
      updateTimeRemaining();
      if (autoRefresh && otpType === 'TOTP') {
        generateOTP();
      }
    }, 1000);
    
    updateTimeRemaining();
    
    return () => {
      clearInterval(timeInterval);
      if (refreshInterval) {
        clearInterval(refreshInterval);
      }
    };
  });

  // 监听相关状态变化
  $effect(() => {
    generateOTP();
  });

  $effect(() => {
    updateTimeRemaining();
  });

  // 复制到剪贴板
  async function copyToClipboard() {
    if (!otpCode) return;
    try {
      await navigator.clipboard.writeText(otpCode);
      copied = true;
      setTimeout(() => {
        copied = false;
      }, 2000);
    } catch (error) {
      console.error('Failed to copy:', error);
    }
  }

  // 增加 HOTP 计数器
  function incrementCounter() {
    counter++;
    generateOTP();
  }

  // 清空
  function clearAll() {
    secret = '';
    otpCode = '';
    counter = 0;
    error = '';
  }
</script>

<div class="flex flex-col h-full w-full ml-0 mr-0 p-2">
  <div class="card flex-1 flex flex-col">
    <div class="flex-1 flex flex-col space-y-6">
      <!-- 配置区域 -->
      <div class="space-y-4">
        <!-- OTP 类型选择 -->
        <div>
          <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
            {t('otp.type')}
          </label>
          <div class="flex gap-2">
            <button
              onclick={() => { otpType = 'TOTP'; generateOTP(); }}
              class="flex-1 px-4 py-2 rounded-lg border transition-colors {otpType === 'TOTP' ? 'bg-primary-100 dark:bg-primary-900/30 border-primary-500 dark:border-primary-400 text-primary-700 dark:text-primary-300' : 'bg-white dark:bg-gray-800 border-gray-300 dark:border-gray-600 text-gray-700 dark:text-gray-300'}"
            >
              TOTP
            </button>
            <button
              onclick={() => { otpType = 'HOTP'; generateOTP(); }}
              class="flex-1 px-4 py-2 rounded-lg border transition-colors {otpType === 'HOTP' ? 'bg-primary-100 dark:bg-primary-900/30 border-primary-500 dark:border-primary-400 text-primary-700 dark:text-primary-300' : 'bg-white dark:bg-gray-800 border-gray-300 dark:border-gray-600 text-gray-700 dark:text-gray-300'}"
            >
              HOTP
            </button>
          </div>
        </div>

        <!-- 密钥输入 -->
        <div>
          <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
            {t('otp.secret')}
          </label>
          <div class="relative">
            <input
              type={showSecret ? 'text' : 'password'}
              bind:value={secret}
              placeholder={t('otp.secretPlaceholder')}
              class="input w-full pr-10 font-mono"
              oninput={generateOTP}
            />
            <button
              type="button"
              onclick={() => showSecret = !showSecret}
              class="absolute right-3 top-1/2 -translate-y-1/2 text-gray-500 dark:text-gray-400 hover:text-gray-700 dark:hover:text-gray-300"
            >
              {#if showSecret}
                <EyeOff class="w-5 h-5" />
              {:else}
                <Eye class="w-5 h-5" />
              {/if}
            </button>
          </div>
          <p class="text-xs text-gray-500 dark:text-gray-400 mt-1">
            {t('otp.secretHint')}
          </p>
        </div>

        <!-- 算法选择 -->
        <div>
          <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
            {t('otp.algorithm')}
          </label>
          <select
            bind:value={algorithm}
            onchange={generateOTP}
            class="input w-full"
          >
            <option value="SHA-1">SHA-1</option>
            <option value="SHA-256">SHA-256</option>
            <option value="SHA-512">SHA-512</option>
          </select>
        </div>

        <!-- 密码长度 -->
        <div>
          <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
            {t('otp.digits')}
          </label>
          <select
            bind:value={digits}
            onchange={generateOTP}
            class="input w-full"
          >
            <option value={6}>6 {t('otp.digits')}</option>
            <option value={7}>7 {t('otp.digits')}</option>
            <option value={8}>8 {t('otp.digits')}</option>
          </select>
        </div>

        <!-- TOTP 特定设置 -->
        {#if otpType === 'TOTP'}
          <div>
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
              {t('otp.period')}
            </label>
            <input
              type="number"
              bind:value={period}
              min="1"
              max="300"
              oninput={generateOTP}
              class="input w-full"
            />
            <p class="text-xs text-gray-500 dark:text-gray-400 mt-1">
              {t('otp.periodHint')}
            </p>
          </div>

          <!-- 自动刷新 -->
          <div class="flex items-center gap-2">
            <input
              type="checkbox"
              bind:checked={autoRefresh}
              class="w-4 h-4 text-primary-600 border-gray-300 rounded focus:ring-primary-500 dark:focus:ring-primary-400 dark:ring-offset-gray-800 focus:ring-2 dark:bg-gray-700 dark:border-gray-600"
            />
            <label class="text-sm text-gray-700 dark:text-gray-300">
              {t('otp.autoRefresh')}
            </label>
          </div>
        {/if}

        <!-- HOTP 特定设置 -->
        {#if otpType === 'HOTP'}
          <div>
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
              {t('otp.counter')}
            </label>
            <div class="flex gap-2">
              <input
                type="number"
                bind:value={counter}
                min="0"
                oninput={generateOTP}
                class="input flex-1"
              />
              <button
                onclick={incrementCounter}
                class="btn-secondary px-4"
                title={t('otp.increment')}
              >
                <RefreshCw class="w-4 h-4" />
              </button>
            </div>
            <p class="text-xs text-gray-500 dark:text-gray-400 mt-1">
              {t('otp.counterHint')}
            </p>
          </div>
        {/if}

        <!-- 清空按钮 -->
        {#if secret}
          <button
            onclick={clearAll}
            class="btn-secondary text-sm"
          >
            {t('otp.clear')}
          </button>
        {/if}
      </div>

      <!-- OTP 代码显示区域 -->
      {#if otpCode}
        <div class="border-t border-gray-200 dark:border-gray-700 pt-6 space-y-4">
          <div>
            <div class="flex items-center justify-between mb-2">
              <label class="block text-sm font-medium text-gray-700 dark:text-gray-300">
                {t('otp.code')}
              </label>
              <button
                onclick={copyToClipboard}
                class="btn-secondary text-xs px-2 py-1"
              >
                {#if copied}
                  <Check class="w-3 h-3" />
                {:else}
                  <Copy class="w-3 h-3" />
                {/if}
              </button>
            </div>
            <div class="font-mono text-3xl font-bold text-gray-900 dark:text-gray-100 bg-gray-50 dark:bg-gray-900 rounded px-4 py-6 border border-gray-200 dark:border-gray-700 text-center tracking-wider">
              {otpCode}
            </div>
            
            {#if otpType === 'TOTP' && timeRemaining > 0}
              <div class="mt-2 text-center">
                <div class="text-xs text-gray-500 dark:text-gray-400">
                  {t('otp.timeRemaining')}: {timeRemaining}s
                </div>
                <div class="mt-1 w-full bg-gray-200 dark:bg-gray-700 rounded-full h-1">
                  <div
                    class="bg-primary-600 dark:bg-primary-400 h-1 rounded-full transition-all duration-1000"
                    style="width: {(timeRemaining / period) * 100}%"
                  ></div>
                </div>
              </div>
            {/if}
          </div>
        </div>
      {:else if secret}
        <!-- 生成中或错误 -->
        {#if error}
          <div class="border-t border-gray-200 dark:border-gray-700 pt-6">
            <div class="p-3 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg">
              <p class="text-sm text-red-800 dark:text-red-200">
                {error}
              </p>
            </div>
          </div>
        {/if}
      {:else}
        <!-- 空状态提示 -->
        <div class="border-t border-gray-200 dark:border-gray-700 pt-6">
          <p class="text-sm text-gray-500 dark:text-gray-400 text-center">
            {t('otp.placeholder')}
          </p>
        </div>
      {/if}
    </div>
  </div>
</div>

