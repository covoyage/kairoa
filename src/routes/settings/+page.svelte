<script lang="ts">
  import { locale, translationsStore } from '$lib/stores/i18n';
  import { theme } from '$lib/stores/theme';
  import { get } from 'svelte/store';
  import { browser } from '$app/environment';
  import { Sun, Moon, Globe, Download, RefreshCw } from 'lucide-svelte';
  import LanguageIcon from '$lib/components/LanguageIcon.svelte';

  let translations = $derived($translationsStore);
  let currentTheme = $state<'light' | 'dark'>(get(theme));
  let currentLocale = $state<'en' | 'zh'>(get(locale));
  let showThemeMenu = $state(false);
  let showLocaleMenu = $state(false);
  let showUpdateDialog = $state(false);
  let checkingUpdate = $state(false);
  let downloadingUpdate = $state(false);
  let downloadProgress = $state(0);
  let downloadComplete = $state(false);
  let showRestartDialog = $state(false);
  let updateInfo = $state<{
    available: boolean;
    currentVersion: string;
    latestVersion: string;
    releaseUrl: string;
    releaseName: string;
    releaseBody: string;
  } | null>(null);
  let updateInstance: any = null;

  function t(key: string): string {
    const keys = key.split('.');
    let value: any = translations;
    for (const k of keys) {
      value = value?.[k];
    }
    return value || key;
  }

  // 使用 $effect 来响应主题变化
  $effect(() => {
    const unsubscribe = theme.subscribe((t) => {
      currentTheme = t;
    });
    return unsubscribe;
  });

  // 使用 $effect 来响应语言变化
  $effect(() => {
    const unsubscribe = locale.subscribe((l) => {
      currentLocale = l;
    });
    return unsubscribe;
  });

  function setTheme(themeValue: 'light' | 'dark') {
    theme.set(themeValue);
    showThemeMenu = false;
  }

  function setLocale(localeValue: 'en' | 'zh') {
    locale.set(localeValue);
    showLocaleMenu = false;
  }

  async function checkForUpdate() {
    checkingUpdate = true;
    try {
      // 使用 Tauri 官方 updater 插件
      const { check } = await import('@tauri-apps/plugin-updater');
      const update = await check();
      
      if (update) {
        // 有更新可用
        updateInfo = {
          available: true,
          currentVersion: update.currentVersion,
          latestVersion: update.version,
          releaseUrl: `https://github.com/covoyage/kairoa/releases/tag/${update.version}`,
          releaseName: update.body || '',
          releaseBody: update.body || ''
        };
        showUpdateDialog = true;
      } else {
        // 已是最新版本 - 获取当前版本号
        const { getVersion } = await import('@tauri-apps/api/app');
        const currentVersion = await getVersion();
        updateInfo = {
          available: false,
          currentVersion: currentVersion,
          latestVersion: currentVersion,
          releaseUrl: '',
          releaseName: '',
          releaseBody: ''
        };
        showUpdateDialog = true;
      }
    } catch (error) {
      console.error('Failed to check for update:', error);
      alert(t('update.error'));
    } finally {
      checkingUpdate = false;
    }
  }

  async function installUpdate() {
    try {
      const { check } = await import('@tauri-apps/plugin-updater');
      
      const update = await check();
      if (update) {
        updateInstance = update;
        downloadingUpdate = true;
        downloadProgress = 0;
        downloadComplete = false;
        showUpdateDialog = false;
        
        let totalBytes = 0;
        let downloadedBytes = 0;
        
        // 下载更新，监听进度事件
        await update.download((event) => {
          if (event.event === 'Started') {
            // 下载开始
            totalBytes = event.data.contentLength || 0;
            downloadedBytes = 0;
            downloadProgress = 0;
          } else if (event.event === 'Progress') {
            // 下载进度更新
            downloadedBytes += event.data.chunkLength;
            if (totalBytes > 0) {
              downloadProgress = Math.min(Math.round((downloadedBytes / totalBytes) * 100), 100);
            } else {
              // 如果不知道总大小，使用简单的进度估算
              downloadProgress = Math.min(downloadProgress + 1, 95);
            }
          } else if (event.event === 'Finished') {
            // 下载完成
            downloadProgress = 100;
          }
        });
        
        // 安装更新
        await update.install();
        
        downloadingUpdate = false;
        downloadComplete = true;
        downloadProgress = 100;
        
        // 显示重启确认对话框
        showRestartDialog = true;
      }
    } catch (error) {
      console.error('Failed to install update:', error);
      downloadingUpdate = false;
      downloadProgress = 0;
      alert(t('update.error'));
    }
  }

  async function restartApp() {
    try {
      const { relaunch } = await import('@tauri-apps/plugin-process');
      await relaunch();
    } catch (error) {
      console.error('Failed to restart app:', error);
      alert('Failed to restart application. Please restart manually.');
    }
  }

  function handleClickOutside(event: MouseEvent) {
    const target = event.target as HTMLElement;
    if (!target.closest('.theme-menu-container')) {
      showThemeMenu = false;
    }
    if (!target.closest('.locale-menu-container')) {
      showLocaleMenu = false;
    }
  }

  $effect(() => {
    if (showThemeMenu || showLocaleMenu) {
      document.addEventListener('click', handleClickOutside);
      return () => {
        document.removeEventListener('click', handleClickOutside);
      };
    }
  });
</script>

<div class="max-w-4xl mx-auto p-6">
  <h1 class="text-3xl font-bold text-gray-900 dark:text-gray-100 mb-8">
    {t('settings.title')}
  </h1>

  <div class="space-y-8">
    <!-- 外观设置 -->
    <section class="bg-white dark:bg-gray-800 rounded-lg shadow-sm border border-gray-200 dark:border-gray-700 p-6">
      <h2 class="text-xl font-semibold text-gray-900 dark:text-gray-100 mb-4">
        {t('settings.appearance')}
      </h2>

      <div class="space-y-6">
        <!-- 主题设置 -->
        <div>
          <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
            {t('settings.theme')}
          </label>
          <div class="theme-menu-container relative">
            <button
              onclick={() => showThemeMenu = !showThemeMenu}
              class="w-full max-w-xs flex items-center gap-3 px-4 py-3 rounded-lg bg-gray-50 dark:bg-gray-700 border border-gray-200 dark:border-gray-600 hover:bg-gray-100 dark:hover:bg-gray-600 transition-colors"
            >
              {#if currentTheme === 'light'}
                <Sun class="w-5 h-5 text-gray-900 dark:text-gray-100" />
              {:else}
                <Moon class="w-5 h-5 text-gray-900 dark:text-gray-100" />
              {/if}
              <span class="font-medium text-sm text-gray-900 dark:text-gray-100 flex-1 text-left">
                {currentTheme === 'light' ? t('settings.lightMode') : t('settings.darkMode')}
              </span>
            </button>
            
            {#if showThemeMenu}
              <div class="absolute top-full left-0 mt-2 w-full max-w-xs bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-lg shadow-lg overflow-hidden z-50">
                {#if currentTheme !== 'light'}
                  <button
                    onclick={() => setTheme('light')}
                    class="w-full flex items-center gap-3 px-4 py-3 hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors"
                  >
                    <Sun class="w-5 h-5 text-gray-900 dark:text-gray-100" />
                    <span class="font-medium text-sm text-gray-900 dark:text-gray-100">
                      {t('settings.lightMode')}
                    </span>
                  </button>
                {/if}
                {#if currentTheme !== 'dark'}
                  <button
                    onclick={() => setTheme('dark')}
                    class="w-full flex items-center gap-3 px-4 py-3 hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors"
                  >
                    <Moon class="w-5 h-5 text-gray-900 dark:text-gray-100" />
                    <span class="font-medium text-sm text-gray-900 dark:text-gray-100">
                      {t('settings.darkMode')}
                    </span>
                  </button>
                {/if}
              </div>
            {/if}
          </div>
        </div>

        <!-- 语言设置 -->
        <div>
          <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
            {t('settings.language')}
          </label>
          <p class="text-sm text-gray-500 dark:text-gray-400 mb-3">
            {t('settings.languageDescription')}
          </p>
          <div class="locale-menu-container relative">
            <button
              onclick={() => showLocaleMenu = !showLocaleMenu}
              class="w-full max-w-xs flex items-center gap-3 px-4 py-3 rounded-lg bg-gray-50 dark:bg-gray-700 border border-gray-200 dark:border-gray-600 hover:bg-gray-100 dark:hover:bg-gray-600 transition-colors"
            >
              <div class="w-5 h-5 shrink-0 flex-none flex items-center justify-center">
                <LanguageIcon />
              </div>
              <span class="font-medium text-sm text-gray-900 dark:text-gray-100 flex-1 text-left">
                {currentLocale === 'zh' ? '中文' : 'English'}
              </span>
            </button>
            
            {#if showLocaleMenu}
              <div class="absolute top-full left-0 mt-2 w-full max-w-xs bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-lg shadow-lg overflow-hidden z-50">
                {#if currentLocale !== 'zh'}
                  <button
                    onclick={() => setLocale('zh')}
                    class="w-full flex items-center gap-3 px-4 py-3 hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors"
                  >
                    <div class="w-5 h-5 shrink-0 flex-none flex items-center justify-center">
                      <LanguageIcon />
                    </div>
                    <span class="font-medium text-sm text-gray-900 dark:text-gray-100">
                      中文
                    </span>
                  </button>
                {/if}
                {#if currentLocale !== 'en'}
                  <button
                    onclick={() => setLocale('en')}
                    class="w-full flex items-center gap-3 px-4 py-3 hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors"
                  >
                    <div class="w-5 h-5 shrink-0 flex-none flex items-center justify-center">
                      <LanguageIcon />
                    </div>
                    <span class="font-medium text-sm text-gray-900 dark:text-gray-100">
                      English
                    </span>
                  </button>
                {/if}
              </div>
            {/if}
          </div>
        </div>
      </div>
    </section>

    <!-- 更新设置 -->
    <section class="bg-white dark:bg-gray-800 rounded-lg shadow-sm border border-gray-200 dark:border-gray-700 p-6">
      <h2 class="text-xl font-semibold text-gray-900 dark:text-gray-100 mb-4">
        {t('settings.updates')}
      </h2>

      <div class="space-y-4">
        <button
          onclick={checkForUpdate}
          disabled={checkingUpdate || downloadingUpdate}
          class="flex items-center gap-3 px-4 py-3 rounded-lg bg-primary-600 dark:bg-primary-500 text-white hover:bg-primary-700 dark:hover:bg-primary-600 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
        >
          <Download class="w-5 h-5" />
          <span class="font-medium text-sm">
            {checkingUpdate ? t('settings.checkingUpdates') : t('settings.checkForUpdates')}
          </span>
        </button>

        <!-- 下载进度条 -->
        {#if downloadingUpdate}
          <div class="space-y-2 p-4 bg-gray-50 dark:bg-gray-700 rounded-lg">
            <div class="flex items-center justify-between text-sm">
              <span class="text-gray-700 dark:text-gray-300 font-medium">{t('update.downloadProgress')}</span>
              <span class="text-gray-600 dark:text-gray-400">{downloadProgress}%</span>
            </div>
            <div class="w-full bg-gray-200 dark:bg-gray-600 rounded-full h-2.5 overflow-hidden">
              <div 
                class="bg-primary-600 dark:bg-primary-500 h-2.5 rounded-full transition-all duration-300 ease-out"
                style="width: {downloadProgress}%"
              ></div>
            </div>
            <p class="text-xs text-gray-500 dark:text-gray-400 text-center">
              {t('update.downloading')}
            </p>
          </div>
        {/if}
      </div>
    </section>
  </div>
</div>

<!-- 更新对话框 -->
{#if showUpdateDialog && updateInfo}
  <div 
    class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-[10000]" 
    role="dialog"
    aria-modal="true"
    aria-labelledby="update-dialog-title"
    onclick={() => showUpdateDialog = false}
    onkeydown={(e) => {
      if (e.key === 'Escape') {
        showUpdateDialog = false;
      }
    }}
    tabindex="-1"
  >
    <div 
      class="bg-white dark:bg-gray-800 rounded-lg shadow-xl max-w-md w-full mx-4 p-6" 
      onclick={(e) => e.stopPropagation()}
      role="document"
      onkeydown={(e) => e.stopPropagation()}
    >
      <h2 id="update-dialog-title" class="text-xl font-bold text-gray-900 dark:text-gray-100 mb-4">
        {updateInfo.available ? t('update.available') : t('update.latest')}
      </h2>
      
      <div class="space-y-3 mb-4">
        <div class="flex justify-between items-center">
          <span class="text-sm text-gray-600 dark:text-gray-400">{t('update.currentVersion')}:</span>
          <span class="text-sm font-medium text-gray-900 dark:text-gray-100">{updateInfo.currentVersion}</span>
        </div>
        <div class="flex justify-between items-center">
          <span class="text-sm text-gray-600 dark:text-gray-400">{t('update.latestVersion')}:</span>
          <span class="text-sm font-medium text-gray-900 dark:text-gray-100">{updateInfo.latestVersion}</span>
        </div>
      </div>

      {#if updateInfo.available && updateInfo.releaseBody}
        <div class="mb-4">
          <h3 class="text-sm font-medium text-gray-900 dark:text-gray-100 mb-2">{t('update.releaseNotes')}:</h3>
          <div class="text-sm text-gray-600 dark:text-gray-400 max-h-40 overflow-y-auto bg-gray-50 dark:bg-gray-700 p-3 rounded">
            {updateInfo.releaseBody}
          </div>
        </div>
      {/if}

      <div class="flex gap-3 justify-end">
        <button
          onclick={() => showUpdateDialog = false}
          class="px-4 py-2 text-sm font-medium text-gray-700 dark:text-gray-300 bg-gray-100 dark:bg-gray-700 rounded-lg hover:bg-gray-200 dark:hover:bg-gray-600 transition-colors"
        >
          {t('common.close')}
        </button>
        {#if updateInfo.available}
          <button
            onclick={() => installUpdate()}
            class="px-4 py-2 text-sm font-medium text-white bg-primary-600 dark:bg-primary-500 rounded-lg hover:bg-primary-700 dark:hover:bg-primary-600 transition-colors"
          >
            {t('update.download')}
          </button>
        {/if}
      </div>
    </div>
  </div>
{/if}

<!-- 重启确认对话框 -->
{#if showRestartDialog}
  <div 
    class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-[10001]" 
    role="dialog"
    aria-modal="true"
    aria-labelledby="restart-dialog-title"
    onclick={() => showRestartDialog = false}
    onkeydown={(e) => {
      if (e.key === 'Escape') {
        showRestartDialog = false;
      }
    }}
    tabindex="-1"
  >
    <div 
      class="bg-white dark:bg-gray-800 rounded-lg shadow-xl max-w-md w-full mx-4 p-6" 
      onclick={(e) => e.stopPropagation()}
      role="document"
      onkeydown={(e) => e.stopPropagation()}
    >
      <div class="flex items-center gap-3 mb-4">
        <div class="w-10 h-10 rounded-full bg-green-100 dark:bg-green-900 flex items-center justify-center flex-shrink-0">
          <RefreshCw class="w-5 h-5 text-green-600 dark:text-green-400" />
        </div>
        <h2 id="restart-dialog-title" class="text-xl font-bold text-gray-900 dark:text-gray-100">
          {t('update.downloadComplete')}
        </h2>
      </div>
      
      <p class="text-gray-700 dark:text-gray-300 mb-6">
        {t('update.restartConfirm')}
      </p>

      <div class="flex gap-3 justify-end">
        <button
          onclick={() => {
            showRestartDialog = false;
            downloadComplete = false;
            downloadProgress = 0;
          }}
          class="px-4 py-2 text-sm font-medium text-gray-700 dark:text-gray-300 bg-gray-100 dark:bg-gray-700 rounded-lg hover:bg-gray-200 dark:hover:bg-gray-600 transition-colors"
        >
          {t('update.restartLater')}
        </button>
        <button
          onclick={restartApp}
          class="px-4 py-2 text-sm font-medium text-white bg-primary-600 dark:bg-primary-500 rounded-lg hover:bg-primary-700 dark:hover:bg-primary-600 transition-colors flex items-center gap-2"
        >
          <RefreshCw class="w-4 h-4" />
          {t('update.restartToUpdate')}
        </button>
      </div>
    </div>
  </div>
{/if}

