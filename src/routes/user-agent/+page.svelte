<script lang="ts">
  import { translationsStore } from '$lib/stores/i18n';
  import { Search, Copy, Check, Trash2, Monitor, Smartphone, Globe } from 'lucide-svelte';
  import { onMount } from 'svelte';
  import { browser } from '$app/environment';

  interface UserAgentInfo {
    browser: string;
    browserVersion?: string;
    engine: string;
    engineVersion?: string;
    os: string;
    osVersion?: string;
    device?: string;
    deviceType?: 'desktop' | 'mobile' | 'tablet' | 'bot';
    raw: string;
  }

  // 常见 User-Agent 示例
  const commonUserAgents = [
    {
      name: 'Chrome (Windows)',
      value: 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36'
    },
    {
      name: 'Firefox (Windows)',
      value: 'Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:121.0) Gecko/20100101 Firefox/121.0'
    },
    {
      name: 'Safari (macOS)',
      value: 'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.2 Safari/605.1.15'
    },
    {
      name: 'Edge (Windows)',
      value: 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36 Edg/120.0.0.0'
    },
    {
      name: 'Chrome (Android)',
      value: 'Mozilla/5.0 (Linux; Android 13; SM-G991B) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Mobile Safari/537.36'
    },
    {
      name: 'Safari (iOS)',
      value: 'Mozilla/5.0 (iPhone; CPU iPhone OS 17_2 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.2 Mobile/15E148 Safari/604.1'
    },
    {
      name: 'Chrome (Linux)',
      value: 'Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36'
    },
    {
      name: 'Bot (Googlebot)',
      value: 'Mozilla/5.0 (compatible; Googlebot/2.1; +http://www.google.com/bot.html)'
    }
  ];

  let translations = $derived($translationsStore);
  let query = $state('');
  let result = $state<UserAgentInfo | null>(null);
  let copied = $state(false);
  let showCommonAgents = $state(false);

  function t(key: string): string {
    const keys = key.split('.');
    let value: any = translations;
    for (const k of keys) {
      value = value?.[k];
    }
    return value || key;
  }

  // 解析 User-Agent
  function parseUserAgent(ua: string): UserAgentInfo {
    const info: UserAgentInfo = {
      browser: 'Unknown',
      engine: 'Unknown',
      os: 'Unknown',
      raw: ua
    };

    if (!ua || !ua.trim()) {
      return info;
    }

    const uaLower = ua.toLowerCase();

    // 检测设备类型
    if (uaLower.includes('bot') || uaLower.includes('crawler') || uaLower.includes('spider')) {
      info.deviceType = 'bot';
    } else if (uaLower.includes('mobile') || uaLower.includes('android') || uaLower.includes('iphone') || uaLower.includes('ipod')) {
      info.deviceType = 'mobile';
    } else if (uaLower.includes('tablet') || uaLower.includes('ipad')) {
      info.deviceType = 'tablet';
    } else {
      info.deviceType = 'desktop';
    }

    // 检测浏览器
    if (uaLower.includes('edg/') || uaLower.includes('edg ')) {
      info.browser = 'Microsoft Edge';
      const match = ua.match(/edg[\/\s](\d+\.\d+)/i);
      if (match) info.browserVersion = match[1];
    } else if (uaLower.includes('chrome/') && !uaLower.includes('edg')) {
      info.browser = 'Google Chrome';
      const match = ua.match(/chrome[\/\s](\d+\.\d+)/i);
      if (match) info.browserVersion = match[1];
    } else if (uaLower.includes('firefox/')) {
      info.browser = 'Mozilla Firefox';
      const match = ua.match(/firefox[\/\s](\d+\.\d+)/i);
      if (match) info.browserVersion = match[1];
    } else if (uaLower.includes('safari/') && !uaLower.includes('chrome')) {
      info.browser = 'Safari';
      const match = ua.match(/version[\/\s](\d+\.\d+)/i);
      if (match) info.browserVersion = match[1];
    } else if (uaLower.includes('opera/') || uaLower.includes('opr/')) {
      info.browser = 'Opera';
      const match = ua.match(/(?:opera|opr)[\/\s](\d+\.\d+)/i);
      if (match) info.browserVersion = match[1];
    } else if (uaLower.includes('msie') || uaLower.includes('trident/')) {
      info.browser = 'Internet Explorer';
      const match = ua.match(/(?:msie|rv:)(\d+\.\d+)/i);
      if (match) info.browserVersion = match[1];
    } else if (uaLower.includes('googlebot')) {
      info.browser = 'Googlebot';
    } else if (uaLower.includes('bingbot')) {
      info.browser = 'Bingbot';
    }

    // 检测渲染引擎
    if (uaLower.includes('webkit')) {
      info.engine = 'WebKit';
      const match = ua.match(/webkit[\/\s](\d+\.\d+)/i);
      if (match) info.engineVersion = match[1];
    } else if (uaLower.includes('gecko')) {
      info.engine = 'Gecko';
      const match = ua.match(/gecko[\/\s](\d+\.\d+)/i);
      if (match) info.engineVersion = match[1];
    } else if (uaLower.includes('trident')) {
      info.engine = 'Trident';
      const match = ua.match(/trident[\/\s](\d+\.\d+)/i);
      if (match) info.engineVersion = match[1];
    } else if (uaLower.includes('presto')) {
      info.engine = 'Presto';
    }

    // 检测操作系统
    if (uaLower.includes('windows nt')) {
      const match = ua.match(/windows nt (\d+\.\d+)/i);
      if (match) {
        const version = match[1];
        const versions: Record<string, string> = {
          '10.0': 'Windows 10/11',
          '6.3': 'Windows 8.1',
          '6.2': 'Windows 8',
          '6.1': 'Windows 7',
          '6.0': 'Windows Vista',
          '5.1': 'Windows XP'
        };
        info.os = versions[version] || `Windows NT ${version}`;
        info.osVersion = version;
      } else {
        info.os = 'Windows';
      }
    } else if (uaLower.includes('mac os x') || uaLower.includes('macintosh')) {
      info.os = 'macOS';
      const match = ua.match(/mac os x[_\s](\d+)[._](\d+)/i);
      if (match) {
        info.osVersion = `${match[1]}.${match[2]}`;
      }
    } else if (uaLower.includes('linux')) {
      info.os = 'Linux';
      if (uaLower.includes('ubuntu')) {
        info.os = 'Ubuntu';
      } else if (uaLower.includes('debian')) {
        info.os = 'Debian';
      } else if (uaLower.includes('fedora')) {
        info.os = 'Fedora';
      } else if (uaLower.includes('android')) {
        info.os = 'Android';
        const match = ua.match(/android[\/\s](\d+\.\d+)/i);
        if (match) info.osVersion = match[1];
      }
    } else if (uaLower.includes('iphone os') || uaLower.includes('cpu iphone')) {
      info.os = 'iOS';
      const match = ua.match(/iphone os[_\s](\d+)[._](\d+)/i) || ua.match(/os[_\s](\d+)[._](\d+)/i);
      if (match) {
        info.osVersion = `${match[1]}.${match[2]}`;
      }
    } else if (uaLower.includes('ipad')) {
      info.os = 'iPadOS';
      const match = ua.match(/os[_\s](\d+)[._](\d+)/i);
      if (match) {
        info.osVersion = `${match[1]}.${match[2]}`;
      }
    } else if (uaLower.includes('android')) {
      info.os = 'Android';
      const match = ua.match(/android[\/\s](\d+\.\d+)/i);
      if (match) info.osVersion = match[1];
    }

    // 检测设备
    if (uaLower.includes('iphone')) {
      info.device = 'iPhone';
      const match = ua.match(/iphone[_\s](\w+)/i);
      if (match) info.device = `iPhone ${match[1]}`;
    } else if (uaLower.includes('ipad')) {
      info.device = 'iPad';
    } else if (uaLower.includes('android')) {
      const match = ua.match(/android[^;]*;\s*([^)]+)\)/i);
      if (match) {
        info.device = match[1].trim();
      }
    }

    return info;
  }

  // 查询
  function search() {
    const trimmedQuery = query.trim();
    if (!trimmedQuery) {
      result = null;
      return;
    }

    result = parseUserAgent(trimmedQuery);
  }

  // 复制结果
  async function copyResult(text: string) {
    try {
      await navigator.clipboard.writeText(text);
      copied = true;
      setTimeout(() => {
        copied = false;
      }, 2000);
    } catch (error) {
      console.error('Failed to copy:', error);
    }
  }

  // 使用当前浏览器的 User-Agent
  function useCurrentUserAgent() {
    if (browser && typeof navigator !== 'undefined') {
      query = navigator.userAgent;
      search();
    }
  }

  // 选择常见 User-Agent
  function selectCommonUA(ua: string) {
    query = ua;
    search();
    showCommonAgents = false;
  }

  // 清空
  function clear() {
    query = '';
    result = null;
  }

  // 处理键盘事件
  function handleKeyPress(event: KeyboardEvent) {
    if (event.key === 'Enter') {
      search();
    }
  }

  // 自动搜索
  $effect(() => {
    if (query.trim()) {
      search();
    } else {
      result = null;
    }
  });

  // 从 localStorage 加载
  function loadFromStorage() {
    if (typeof window !== 'undefined') {
      const saved = localStorage.getItem('user-agent-query');
      if (saved) {
        try {
          query = saved;
        } catch (e) {
          console.error('Failed to load query:', e);
        }
      }
    }
  }

  // 保存到 localStorage
  function saveToStorage() {
    if (typeof window !== 'undefined') {
      localStorage.setItem('user-agent-query', query);
    }
  }

  $effect(() => {
    query;
    saveToStorage();
  });

  onMount(() => {
    loadFromStorage();
  });
</script>

<div class="flex flex-col h-full w-full ml-0 mr-0 p-2">
  <div class="card flex-1 flex flex-col min-h-0">
    <div class="space-y-4 mb-6">
      <!-- 查询输入 -->
      <div>
        <label for="query-input" class="block text-sm font-medium mb-2 text-gray-700 dark:text-gray-300">
          {t('userAgent.userAgent')}
        </label>
        <div class="flex gap-2">
          <input
            id="query-input"
            type="text"
            bind:value={query}
            placeholder={t('userAgent.placeholder')}
            class="input flex-1 font-mono text-sm"
            onkeydown={handleKeyPress}
          />
          <button
            onclick={search}
            class="btn-secondary"
            disabled={!query.trim()}
          >
            <Search class="w-4 h-4 inline mr-1" />
            {t('userAgent.parse')}
          </button>
          {#if query || result}
            <button type="button" class="btn-secondary" onclick={clear}>
              <Trash2 class="w-4 h-4 inline mr-1" />
              {t('common.clear')}
            </button>
          {/if}
        </div>
        <div class="flex items-center gap-2 mt-2">
          <button
            onclick={useCurrentUserAgent}
            class="text-xs text-primary-600 dark:text-primary-400 hover:underline"
          >
            {t('userAgent.useCurrent')}
          </button>
          <span class="text-gray-400 dark:text-gray-500">|</span>
          <button
            onclick={() => showCommonAgents = !showCommonAgents}
            class="text-xs text-primary-600 dark:text-primary-400 hover:underline"
          >
            {showCommonAgents ? t('userAgent.hideExamples') : t('userAgent.showExamples')}
          </button>
        </div>
      </div>

      <!-- 常见 User-Agent 列表 -->
      {#if showCommonAgents}
        <div class="bg-gray-50 dark:bg-gray-700/50 rounded-lg p-4">
          <p class="text-sm font-medium text-gray-700 dark:text-gray-300 mb-3">
            {t('userAgent.commonExamples')}
          </p>
          <div class="grid grid-cols-1 md:grid-cols-2 gap-2">
            {#each commonUserAgents as ua}
              <button
                onclick={() => selectCommonUA(ua.value)}
                class="text-left p-2 rounded hover:bg-gray-100 dark:hover:bg-gray-700 text-sm text-gray-700 dark:text-gray-300 transition-colors"
              >
                <div class="font-medium">{ua.name}</div>
                <div class="text-xs text-gray-500 dark:text-gray-400 truncate font-mono">{ua.value}</div>
              </button>
            {/each}
          </div>
        </div>
      {/if}
    </div>

    <!-- 结果展示 -->
    <div class="flex-1 overflow-y-auto">
      {#if result}
        {@const uaResult = result}
        <div class="space-y-4">
          <!-- User-Agent 原始字符串 -->
          <div class="bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-lg p-4">
            <div class="flex items-start justify-between gap-4">
              <div class="flex-1 min-w-0">
                <div class="flex items-center gap-2 mb-2">
                  <span class="text-xs font-medium text-gray-500 dark:text-gray-400">
                    {t('userAgent.rawUserAgent')}
                  </span>
                  <button
                    onclick={() => copyResult(uaResult.raw)}
                    class="p-1 rounded hover:bg-gray-100 dark:hover:bg-gray-700 text-gray-600 dark:text-gray-400 transition-colors"
                    title={t('common.copy')}
                  >
                    {#if copied}
                      <Check class="w-4 h-4" />
                    {:else}
                      <Copy class="w-4 h-4" />
                    {/if}
                  </button>
                </div>
                <p class="text-sm font-mono text-gray-900 dark:text-gray-100 break-all">
                  {uaResult.raw}
                </p>
              </div>
            </div>
          </div>

          <!-- 解析结果 -->
          <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
            <!-- 浏览器信息 -->
            <div class="bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-lg p-4">
              <div class="flex items-center gap-2 mb-3">
                <Globe class="w-5 h-5 text-primary-600 dark:text-primary-400" />
                <h3 class="text-sm font-semibold text-gray-900 dark:text-gray-100">
                  {t('userAgent.browser')}
                </h3>
              </div>
              <div class="space-y-2">
                <div>
                  <span class="text-xs text-gray-500 dark:text-gray-400">{t('userAgent.name')}:</span>
                  <span class="ml-2 text-sm font-medium text-gray-900 dark:text-gray-100">
                    {uaResult.browser}
                  </span>
                </div>
                {#if uaResult.browserVersion}
                  <div>
                    <span class="text-xs text-gray-500 dark:text-gray-400">{t('userAgent.version')}:</span>
                    <span class="ml-2 text-sm text-gray-700 dark:text-gray-300">
                      {uaResult.browserVersion}
                    </span>
                  </div>
                {/if}
              </div>
            </div>

            <!-- 操作系统信息 -->
            <div class="bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-lg p-4">
              <div class="flex items-center gap-2 mb-3">
                <Monitor class="w-5 h-5 text-primary-600 dark:text-primary-400" />
                <h3 class="text-sm font-semibold text-gray-900 dark:text-gray-100">
                  {t('userAgent.operatingSystem')}
                </h3>
              </div>
              <div class="space-y-2">
                <div>
                  <span class="text-xs text-gray-500 dark:text-gray-400">{t('userAgent.name')}:</span>
                  <span class="ml-2 text-sm font-medium text-gray-900 dark:text-gray-100">
                    {uaResult.os}
                  </span>
                </div>
                {#if uaResult.osVersion}
                  <div>
                    <span class="text-xs text-gray-500 dark:text-gray-400">{t('userAgent.version')}:</span>
                    <span class="ml-2 text-sm text-gray-700 dark:text-gray-300">
                      {uaResult.osVersion}
                    </span>
                  </div>
                {/if}
              </div>
            </div>

            <!-- 渲染引擎 -->
            <div class="bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-lg p-4">
              <div class="flex items-center gap-2 mb-3">
                <Globe class="w-5 h-5 text-primary-600 dark:text-primary-400" />
                <h3 class="text-sm font-semibold text-gray-900 dark:text-gray-100">
                  {t('userAgent.engine')}
                </h3>
              </div>
              <div class="space-y-2">
                <div>
                  <span class="text-xs text-gray-500 dark:text-gray-400">{t('userAgent.name')}:</span>
                  <span class="ml-2 text-sm font-medium text-gray-900 dark:text-gray-100">
                    {uaResult.engine}
                  </span>
                </div>
                {#if uaResult.engineVersion}
                  <div>
                    <span class="text-xs text-gray-500 dark:text-gray-400">{t('userAgent.version')}:</span>
                    <span class="ml-2 text-sm text-gray-700 dark:text-gray-300">
                      {uaResult.engineVersion}
                    </span>
                  </div>
                {/if}
              </div>
            </div>

            <!-- 设备信息 -->
            <div class="bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-lg p-4">
              <div class="flex items-center gap-2 mb-3">
                <Smartphone class="w-5 h-5 text-primary-600 dark:text-primary-400" />
                <h3 class="text-sm font-semibold text-gray-900 dark:text-gray-100">
                  {t('userAgent.device')}
                </h3>
              </div>
              <div class="space-y-2">
                {#if uaResult.device}
                  <div>
                    <span class="text-xs text-gray-500 dark:text-gray-400">{t('userAgent.name')}:</span>
                    <span class="ml-2 text-sm font-medium text-gray-900 dark:text-gray-100">
                      {uaResult.device}
                    </span>
                  </div>
                {/if}
                <div>
                  <span class="text-xs text-gray-500 dark:text-gray-400">{t('userAgent.type')}:</span>
                  <span class="ml-2 text-sm text-gray-700 dark:text-gray-300 capitalize">
                    {uaResult.deviceType || 'Unknown'}
                  </span>
                </div>
              </div>
            </div>
          </div>
        </div>
      {:else if query.trim()}
        <div class="flex flex-col items-center justify-center py-16 text-center">
          <Globe class="w-12 h-12 text-gray-400 dark:text-gray-500 mb-4" />
          <p class="text-lg font-medium text-gray-900 dark:text-gray-100 mb-2">
            {t('userAgent.noResults')}
          </p>
          <p class="text-sm text-gray-600 dark:text-gray-400">
            {t('userAgent.noResultsDesc')}
          </p>
        </div>
      {:else}
        <div class="flex flex-col items-center justify-center py-16 text-center">
          <Globe class="w-12 h-12 text-gray-400 dark:text-gray-500 mb-4" />
          <p class="text-lg font-medium text-gray-900 dark:text-gray-100 mb-2">
            {t('userAgent.welcome')}
          </p>
          <p class="text-sm text-gray-600 dark:text-gray-400">
            {t('userAgent.welcomeDesc')}
          </p>
        </div>
      {/if}
    </div>
  </div>
</div>
