<script lang="ts">
  import { translationsStore } from '$lib/stores/i18n';
  import { X, ExternalLink } from 'lucide-svelte';
  import { browser } from '$app/environment';

  let translations = $derived($translationsStore);
  let showDialog = $state(false);
  let appVersion = $state('0.1.0');

  function t(key: string): string {
    const keys = key.split('.');
    let value: any = translations;
    for (const k of keys) {
      value = value?.[k];
    }
    return value || key;
  }

  // 获取应用版本
  if (browser && typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window) {
    import('@tauri-apps/api/app').then((module) => {
      module.getVersion().then((version) => {
        appVersion = version;
      });
    });
  }

  // 监听 About 菜单事件
  if (browser && typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window) {
    import('@tauri-apps/api/event').then((module) => {
      // 监听来自 Rust 后端的 show-about 事件
      module.listen('show-about', () => {
        console.log('Received show-about event'); // 调试日志
        showDialog = true;
      });
    });
  }

  function closeDialog() {
    showDialog = false;
  }

  async function openGitHub() {
    if (browser && typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window) {
      try {
        const { openUrl } = await import('@tauri-apps/plugin-opener');
        await openUrl('https://github.com/covoyage/kairoa');
      } catch (error) {
        console.error('Failed to open URL:', error);
        window.open('https://github.com/covoyage/kairoa', '_blank');
      }
    } else {
      window.open('https://github.com/covoyage/kairoa', '_blank');
    }
  }

  // 导出函数供外部调用
  export function show() {
    showDialog = true;
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
      <div class="absolute top-4 right-4">
        <button
          onclick={closeDialog}
          class="p-1 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors"
          aria-label="Close"
        >
          <X class="w-5 h-5 text-gray-600 dark:text-gray-400" />
        </button>
      </div>

      <!-- Header - Centered -->
      <div class="flex flex-col items-center text-center mb-6">
        <img src="/icon.png" alt="Kairoa" class="w-20 h-20 object-contain mb-3" />
        <h2 class="text-2xl font-bold text-gray-900 dark:text-gray-100 mb-1">
          {t('app.title')}
        </h2>
        <p class="text-sm text-gray-600 dark:text-gray-400">
          {t('app.subtitle')}
        </p>
      </div>

      <!-- Version - Centered -->
      <div class="mb-6 pb-6 border-b border-gray-200 dark:border-gray-700 text-center">
        <p class="text-sm text-gray-600 dark:text-gray-400">
          Version {appVersion}
        </p>
      </div>

      <!-- GitHub Link - Centered -->
      <button
        onclick={openGitHub}
        class="w-full flex items-center justify-center gap-2 px-4 py-3 bg-gray-900 dark:bg-gray-700 hover:bg-gray-800 dark:hover:bg-gray-600 text-white rounded-lg transition-colors"
      >
        <svg class="w-5 h-5" fill="currentColor" viewBox="0 0 24 24" aria-hidden="true">
          <path fill-rule="evenodd" d="M12 2C6.477 2 2 6.484 2 12.017c0 4.425 2.865 8.18 6.839 9.504.5.092.682-.217.682-.483 0-.237-.008-.868-.013-1.703-2.782.605-3.369-1.343-3.369-1.343-.454-1.158-1.11-1.466-1.11-1.466-.908-.62.069-.608.069-.608 1.003.07 1.531 1.032 1.531 1.032.892 1.53 2.341 1.088 2.91.832.092-.647.35-1.088.636-1.338-2.22-.253-4.555-1.113-4.555-4.951 0-1.093.39-1.988 1.029-2.688-.103-.253-.446-1.272.098-2.65 0 0 .84-.27 2.75 1.026A9.564 9.564 0 0112 6.844c.85.004 1.705.115 2.504.337 1.909-1.296 2.747-1.027 2.747-1.027.546 1.379.202 2.398.1 2.651.64.7 1.028 1.595 1.028 2.688 0 3.848-2.339 4.695-4.566 4.943.359.309.678.92.678 1.855 0 1.338-.012 2.419-.012 2.747 0 .268.18.58.688.482A10.019 10.019 0 0022 12.017C22 6.484 17.522 2 12 2z" clip-rule="evenodd" />
        </svg>
        <span>View on GitHub</span>
        <ExternalLink class="w-4 h-4" />
      </button>

      <!-- License - Centered -->
      <div class="mt-4 text-center">
        <p class="text-xs text-gray-500 dark:text-gray-400">
          Licensed under AGPL-3.0 License
        </p>
      </div>
    </div>
  </div>
{/if}
