<script lang="ts">
  import { onMount } from 'svelte';
  import { theme } from '$lib/stores/theme';
  import { locale } from '$lib/stores/i18n';
  import Sidebar from '$lib/components/Sidebar.svelte';
  import AboutDialog from '$lib/components/AboutDialog.svelte';
  import { browser } from '$app/environment';
  import '../app.css';

  let aboutDialog: any;

  onMount(() => {
    theme.init();
    locale.init();
    
    // 添加键盘快捷键 Cmd+I 来显示 About 对话框（用于测试）
    const handleKeyDown = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key === 'i') {
        e.preventDefault();
        aboutDialog?.show();
      }
    };
    
    if (browser) {
      window.addEventListener('keydown', handleKeyDown);
      return () => {
        window.removeEventListener('keydown', handleKeyDown);
      };
    }
  });
</script>

<div class="flex h-screen overflow-hidden bg-gray-50 dark:bg-gray-900">
  <Sidebar />
  <main class="flex-1 overflow-y-auto bg-white dark:bg-gray-900">
    <slot />
  </main>
</div>

<AboutDialog bind:this={aboutDialog} />
