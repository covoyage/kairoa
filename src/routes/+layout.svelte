<script lang="ts">
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { theme } from '$lib/stores/theme';
  import { locale } from '$lib/stores/i18n';
  import { initDeepLinkListener } from '$lib/stores/deepLink';
  import Sidebar from '$lib/components/Sidebar.svelte';
  import AboutDialog from '$lib/components/AboutDialog.svelte';
  import UpdateDialog from '$lib/components/UpdateDialog.svelte';
  import SettingsDialog from '$lib/components/SettingsDialog.svelte';
  import { browser } from '$app/environment';
  import '../app.css';

  let aboutDialog: any;
  let updateDialog: any;
  let settingsDialog: any;

  onMount(async () => {
    theme.init();
    locale.init();
    
    // Initialize deep link listener
    initDeepLinkListener((route: string) => goto(route));
    
    // 检查是否有文件需要打开（Tauri 环境）
    if (browser && typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window) {
      try {
        const { invoke } = await import('@tauri-apps/api/core');
        const openFilePath = await invoke('get_open_file_path');
        
        if (openFilePath) {
          const filePath = openFilePath as string;
          console.log('Opening file on startup:', filePath);
          
          // 根据文件扩展名确定类型并导航到 previewer
          const ext = filePath.split('.').pop()?.toLowerCase();
          let type = 'svg';
          if (ext === 'md' || ext === 'markdown') {
            type = 'markdown';
          } else if (ext === 'mmd' || ext === 'mermaid') {
            type = 'mermaid';
          }
          
          localStorage.setItem('kairoa-open-file-path', filePath);
          await invoke('clear_open_file_path');
          goto(`/previewer?type=${type}`);
        }
      } catch (err) {
        console.error('Failed to check open file path:', err);
      }
      
      // 监听文件打开事件（应用运行时右键打开文件）
      import('@tauri-apps/api/event').then((eventModule) => {
        eventModule.listen('open-file', async (event: any) => {
          const filePath = event.payload as string;
          if (filePath) {
            console.log('Opening file:', filePath);
            
            // 根据文件扩展名确定类型并导航到 previewer
            const ext = filePath.split('.').pop()?.toLowerCase();
            let type = 'svg';
            if (ext === 'md' || ext === 'markdown') {
              type = 'markdown';
            } else if (ext === 'mmd' || ext === 'mermaid') {
              type = 'mermaid';
            }
            
            localStorage.setItem('kairoa-open-file-path', filePath);
            goto(`/previewer?type=${type}`);
          }
        });
      });
    }
    
    // 添加键盘快捷键 Cmd+I 来显示 About 对话框（用于测试）
    const handleKeyDown = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key === 'i') {
        e.preventDefault();
        aboutDialog?.show();
      }
    };
    
    // 全局链接处理：拦截所有链接点击，使用默认浏览器打开外部链接
    const handleLinkClick = async (e: MouseEvent) => {
      const target = e.target as HTMLElement;
      const link = target.closest('a');
      
      if (!link) return;
      
      const href = link.getAttribute('href');
      if (!href) return;
      
      // 检查是否是外部链接（http/https）
      if (href.startsWith('http://') || href.startsWith('https://')) {
        e.preventDefault();
        
        // 在 Tauri 环境中使用 opener 插件打开链接
        if (browser && typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window) {
          try {
            const { openUrl } = await import('@tauri-apps/plugin-opener');
            await openUrl(href);
          } catch (error) {
            console.error('Failed to open URL:', error);
            // 降级到 window.open
            window.open(href, '_blank', 'noopener,noreferrer');
          }
        } else {
          // 在浏览器环境中使用 window.open
          window.open(href, '_blank', 'noopener,noreferrer');
        }
      }
      // 内部链接（以 / 开头）保持默认行为，由 SvelteKit 路由处理
    };
    
    if (browser) {
      window.addEventListener('keydown', handleKeyDown);
      // 使用捕获阶段（第三个参数为 true）确保在 stopPropagation 之前处理
      window.addEventListener('click', handleLinkClick, true);
      
      // 监听来自 Tauri 的设置事件（已在 SettingsDialog 组件中处理）
      // 不再需要导航，直接显示对话框
      
      return () => {
        window.removeEventListener('keydown', handleKeyDown);
        window.removeEventListener('click', handleLinkClick, true);
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
<UpdateDialog bind:this={updateDialog} />
<SettingsDialog bind:this={settingsDialog} />
