<script lang="ts">
  import { locale, translationsStore } from '$lib/stores/i18n';
  import { theme } from '$lib/stores/theme';
  import { X, Sun, Moon, Globe, Check, Palette } from 'lucide-svelte';
  import { browser } from '$app/environment';

  let translations = $derived($translationsStore);
  let showDialog = $state(false);
  let currentTheme = $state<'kairoa-light' | 'kairoa-dark' | 'solarized-light' | 'solarized-dark'>('kairoa-light');
  let currentLocale = $state<'en' | 'zh'>('en');

  function t(key: string): string {
    const keys = key.split('.');
    let value: any = translations;
    for (const k of keys) {
      value = value?.[k];
    }
    return value || key;
  }

  // 监听主题变化
  $effect(() => {
    const unsubscribe = theme.subscribe((t) => {
      currentTheme = t;
    });
    return unsubscribe;
  });

  // 监听语言变化
  $effect(() => {
    const unsubscribe = locale.subscribe((l) => {
      currentLocale = l;
    });
    return unsubscribe;
  });

  // 监听设置事件
  if (browser && typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window) {
    import('@tauri-apps/api/event').then((module) => {
      module.listen('show-settings', () => {
        console.log('Received show-settings event');
        showDialog = true;
      });
    });
  }

  function closeDialog() {
    showDialog = false;
  }

  function setTheme(themeValue: 'kairoa-light' | 'kairoa-dark' | 'solarized-light' | 'solarized-dark') {
    theme.set(themeValue);
  }

  function setLocale(localeValue: 'en' | 'zh') {
    locale.set(localeValue);
  }

  function getThemeLabel(themeValue: string): string {
    switch (themeValue) {
      case 'kairoa-light':
        return t('settings.kairoaLight');
      case 'kairoa-dark':
        return t('settings.kairoaDark');
      case 'solarized-light':
        return t('settings.solarizedLight');
      case 'solarized-dark':
        return t('settings.solarizedDark');
      default:
        return themeValue;
    }
  }

  function getThemeIcon(themeValue: string) {
    return themeValue.includes('dark') ? Moon : Sun;
  }

  const themes: Array<'kairoa-light' | 'kairoa-dark' | 'solarized-light' | 'solarized-dark'> = [
    'kairoa-light',
    'kairoa-dark',
    'solarized-light',
    'solarized-dark'
  ];

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

      <!-- Header -->
      <div class="mb-6">
        <h2 class="text-2xl font-bold text-gray-900 dark:text-gray-100">
          {t('settings.title')}
        </h2>
      </div>

      <!-- Content -->
      <div class="space-y-6">
        <!-- Language Section -->
        <div>
          <h3 class="text-sm font-semibold text-gray-900 dark:text-gray-100 mb-3 flex items-center gap-2">
            <Globe class="w-4 h-4" />
            {t('settings.language')}
          </h3>
          <div class="space-y-2">
            <button
              onclick={() => setLocale('en')}
              class="w-full flex items-center justify-between px-4 py-3 rounded-lg border transition-colors {currentLocale === 'en'
                ? 'border-primary-500 bg-primary-50 dark:bg-primary-900/20'
                : 'border-gray-200 dark:border-gray-700 hover:bg-gray-50 dark:hover:bg-gray-700/50'}"
            >
              <span class="text-sm font-medium text-gray-900 dark:text-gray-100">English</span>
              {#if currentLocale === 'en'}
                <Check class="w-5 h-5 text-primary-600 dark:text-primary-400" />
              {/if}
            </button>
            <button
              onclick={() => setLocale('zh')}
              class="w-full flex items-center justify-between px-4 py-3 rounded-lg border transition-colors {currentLocale === 'zh'
                ? 'border-primary-500 bg-primary-50 dark:bg-primary-900/20'
                : 'border-gray-200 dark:border-gray-700 hover:bg-gray-50 dark:hover:bg-gray-700/50'}"
            >
              <span class="text-sm font-medium text-gray-900 dark:text-gray-100">简体中文</span>
              {#if currentLocale === 'zh'}
                <Check class="w-5 h-5 text-primary-600 dark:text-primary-400" />
              {/if}
            </button>
          </div>
        </div>

        <!-- Theme Section -->
        <div>
          <h3 class="text-sm font-semibold text-gray-900 dark:text-gray-100 mb-3 flex items-center gap-2">
            <Palette class="w-4 h-4" />
            {t('settings.theme')}
          </h3>
          <div class="space-y-2">
            {#each themes as themeOption}
              {@const ThemeIcon = getThemeIcon(themeOption)}
              <button
                onclick={() => setTheme(themeOption)}
                class="w-full flex items-center justify-between px-4 py-3 rounded-lg border transition-colors {currentTheme === themeOption
                  ? 'border-primary-500 bg-primary-50 dark:bg-primary-900/20'
                  : 'border-gray-200 dark:border-gray-700 hover:bg-gray-50 dark:hover:bg-gray-700/50'}"
              >
                <div class="flex items-center gap-3">
                  <ThemeIcon class="w-5 h-5 text-gray-600 dark:text-gray-400" />
                  <span class="text-sm font-medium text-gray-900 dark:text-gray-100">
                    {getThemeLabel(themeOption)}
                  </span>
                </div>
                {#if currentTheme === themeOption}
                  <Check class="w-5 h-5 text-primary-600 dark:text-primary-400" />
                {/if}
              </button>
            {/each}
          </div>
        </div>
      </div>
    </div>
  </div>
{/if}
