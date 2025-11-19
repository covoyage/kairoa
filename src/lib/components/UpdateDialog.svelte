<script lang="ts">
  import { translationsStore } from '$lib/stores/i18n';
  import { X, Download, RefreshCw, ExternalLink } from 'lucide-svelte';
  import { browser } from '$app/environment';

  let translations = $derived($translationsStore);
  let showDialog = $state(false);
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

  // 监听检查更新事件
  if (browser && typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window) {
    import('@tauri-apps/api/event').then((module) => {
      module.listen('check-for-updates', () => {
        console.log('Received check-for-updates event');
        show();
      });
    });
  }

  function closeDialog() {
    if (!downloadingUpdate) {
      showDialog = false;
      updateInfo = null;
    }
  }

  async function checkForUpdate() {
    checkingUpdate = true;
    try {
      const { check } = await import('@tauri-apps/plugin-updater');
      const update = await check();
      
      if (update) {
        updateInfo = {
          available: true,
          currentVersion: update.currentVersion,
          latestVersion: update.version,
          releaseUrl: `https://github.com/covoyage/kairoa/releases/tag/${update.version}`,
          releaseName: update.body || '',
          releaseBody: update.body || ''
        };
      } else {
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
      }
    } catch (error) {
      console.error('Failed to check for update:', error);
      updateInfo = {
        available: false,
        currentVersion: 'Unknown',
        latestVersion: 'Unknown',
        releaseUrl: '',
        releaseName: '',
        releaseBody: t('update.error')
      };
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
        
        let totalBytes = 0;
        let downloadedBytes = 0;
        
        await update.download((event) => {
          if (event.event === 'Started') {
            totalBytes = event.data.contentLength || 0;
            downloadedBytes = 0;
            downloadProgress = 0;
          } else if (event.event === 'Progress') {
            downloadedBytes += event.data.chunkLength;
            if (totalBytes > 0) {
              downloadProgress = Math.min(Math.round((downloadedBytes / totalBytes) * 100), 100);
            } else {
              downloadProgress = Math.min(downloadProgress + 1, 95);
            }
          } else if (event.event === 'Finished') {
            downloadProgress = 100;
          }
        });
        
        await update.install();
        downloadingUpdate = false;
        downloadComplete = true;
        downloadProgress = 100;
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

  async function openReleaseUrl() {
    if (updateInfo?.releaseUrl) {
      if (browser && typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window) {
        try {
          const { openUrl } = await import('@tauri-apps/plugin-opener');
          await openUrl(updateInfo.releaseUrl);
        } catch (error) {
          console.error('Failed to open URL:', error);
          window.open(updateInfo.releaseUrl, '_blank');
        }
      } else {
        window.open(updateInfo.releaseUrl, '_blank');
      }
    }
  }

  // 导出函数供外部调用
  export function show() {
    showDialog = true;
    updateInfo = null;
    checkForUpdate();
  }
</script>

{#if showDialog}
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/50" onclick={closeDialog}>
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div 
      class="bg-white dark:bg-gray-800 rounded-lg shadow-xl max-w-md w-full mx-4 p-6 relative"
      onclick={(e) => e.stopPropagation()}
    >
      <!-- Close Button -->
      {#if !downloadingUpdate}
        <div class="absolute top-4 right-4">
          <button
            onclick={closeDialog}
            class="p-1 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors"
            aria-label="Close"
          >
            <X class="w-5 h-5 text-gray-600 dark:text-gray-400" />
          </button>
        </div>
      {/if}

      {#if checkingUpdate}
        <!-- Checking - 整体居中 -->
        <div class="flex flex-col items-center justify-center text-center py-12">
          <div class="w-12 h-12 rounded-full bg-primary-100 dark:bg-primary-900/30 flex items-center justify-center mb-3">
            <RefreshCw class="w-6 h-6 text-primary-600 dark:text-primary-400 animate-spin" />
          </div>
          <h2 class="text-xl font-bold text-gray-900 dark:text-gray-100">
            {t('update.check')}
          </h2>
        </div>
      {:else}
        <!-- Header -->
        <div class="flex flex-col items-center text-center mb-6">
          <div class="w-12 h-12 rounded-full bg-primary-100 dark:bg-primary-900/30 flex items-center justify-center mb-3">
            <RefreshCw class="w-6 h-6 text-primary-600 dark:text-primary-400" />
          </div>
          <h2 class="text-xl font-bold text-gray-900 dark:text-gray-100">
            {t('update.check')}
          </h2>
        </div>

        <!-- Content -->
        <div class="space-y-4">
          {#if downloadingUpdate}
            <!-- Downloading -->
            <div class="space-y-3">
            <div class="flex items-center justify-between text-sm">
              <span class="text-gray-600 dark:text-gray-400">{t('update.downloading')}</span>
              <span class="font-medium text-gray-900 dark:text-gray-100">{downloadProgress}%</span>
            </div>
            <div class="w-full bg-gray-200 dark:bg-gray-700 rounded-full h-2">
              <div 
                class="bg-primary-600 dark:bg-primary-500 h-2 rounded-full transition-all duration-300"
                style="width: {downloadProgress}%"
              ></div>
            </div>
            </div>
          {:else if showRestartDialog}
          <!-- Restart -->
          <div class="text-center space-y-4">
            <p class="text-sm text-gray-600 dark:text-gray-400">
              {t('update.downloadComplete')}
            </p>
            <div class="flex gap-3">
              <button
                onclick={() => { showRestartDialog = false; showDialog = false; }}
                class="flex-1 px-4 py-2 bg-gray-200 dark:bg-gray-700 hover:bg-gray-300 dark:hover:bg-gray-600 text-gray-900 dark:text-gray-100 rounded-lg transition-colors"
              >
                {t('update.restartLater')}
              </button>
              <button
                onclick={restartApp}
                class="flex-1 px-4 py-2 bg-primary-600 hover:bg-primary-700 text-white rounded-lg transition-colors"
              >
                {t('update.restartToUpdate')}
              </button>
            </div>
            </div>
          {:else if updateInfo}
          <!-- Update Info -->
          <div class="space-y-4">
            <div class="text-center">
              {#if updateInfo.available}
                <p class="text-lg font-semibold text-primary-600 dark:text-primary-400 mb-2">
                  {t('update.available')}
                </p>
              {:else}
                <p class="text-lg font-semibold text-green-600 dark:text-green-400 mb-2">
                  {t('update.latest')}
                </p>
              {/if}
            </div>

            <div class="space-y-2 text-sm">
              <div class="flex justify-between">
                <span class="text-gray-600 dark:text-gray-400">{t('update.currentVersion')}:</span>
                <span class="font-medium text-gray-900 dark:text-gray-100">{updateInfo.currentVersion}</span>
              </div>
              {#if updateInfo.available}
                <div class="flex justify-between">
                  <span class="text-gray-600 dark:text-gray-400">{t('update.latestVersion')}:</span>
                  <span class="font-medium text-gray-900 dark:text-gray-100">{updateInfo.latestVersion}</span>
                </div>
              {/if}
            </div>

            {#if updateInfo.available}
              <div class="flex gap-3">
                {#if updateInfo.releaseUrl}
                  <button
                    onclick={openReleaseUrl}
                    class="flex-1 flex items-center justify-center gap-2 px-4 py-2 bg-gray-200 dark:bg-gray-700 hover:bg-gray-300 dark:hover:bg-gray-600 text-gray-900 dark:text-gray-100 rounded-lg transition-colors"
                  >
                    <span>{t('update.releaseNotes')}</span>
                    <ExternalLink class="w-4 h-4" />
                  </button>
                {/if}
                <button
                  onclick={installUpdate}
                  class="flex-1 flex items-center justify-center gap-2 px-4 py-2 bg-primary-600 hover:bg-primary-700 text-white rounded-lg transition-colors"
                >
                  <Download class="w-4 h-4" />
                  <span>{t('update.download')}</span>
                </button>
              </div>
            {/if}
            </div>
          {/if}
        </div>
      {/if}
    </div>
  </div>
{/if}
