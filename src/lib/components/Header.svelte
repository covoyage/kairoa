<script lang="ts">
  import { theme } from '$lib/stores/theme';
  import { locale, t } from '$lib/stores/i18n';
  import { Moon, Sun, Globe } from 'lucide-svelte';

  let currentTheme = $state<'light' | 'dark'>('light');
  let currentLocale = $state<'en' | 'zh'>('en');

  theme.subscribe((t) => currentTheme = t);
  locale.subscribe((l) => {
    currentLocale = l;
  });

  function toggleTheme() {
    theme.toggle();
  }

  function toggleLocale() {
    locale.toggle();
  }
</script>

<header class="h-16 border-b border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800 flex items-center justify-between px-6">
  <div class="flex items-center gap-4">
    <h1 class="text-xl font-bold text-gray-900 dark:text-gray-100 {currentLocale === 'zh' ? 'app-title-zh' : ''}">{t('app.title')}</h1>
    <span class="text-sm text-gray-500 dark:text-gray-400">{t('app.subtitle')}</span>
  </div>
  
  <div class="flex items-center gap-3">
    <button
      onclick={toggleLocale}
      class="p-2 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors"
      title="Toggle Language"
    >
      <Globe class="w-5 h-5 text-gray-600 dark:text-gray-400" />
      <span class="sr-only">{currentLocale === 'en' ? '中文' : 'English'}</span>
    </button>
    
    <button
      onclick={toggleTheme}
      class="p-2 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors"
      title="Toggle Theme"
    >
      {#if currentTheme === 'light'}
        <Moon class="w-5 h-5 text-gray-600 dark:text-gray-400" />
      {:else}
        <Sun class="w-5 h-5 text-gray-600 dark:text-gray-400" />
      {/if}
      <span class="sr-only">Toggle theme</span>
    </button>
  </div>
</header>

