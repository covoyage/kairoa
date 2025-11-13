<script lang="ts">
  import { page } from '$app/stores';
  import { locale, translationsStore } from '$lib/stores/i18n';
  import { browser } from '$app/environment';
  import { goto } from '$app/navigation';
  import { Hash, Clock, Key, FileJson, Code, Calendar, Palette, Binary, FileText, Shield, Globe, Minimize2, Maximize2, Settings, GitCompare, Eye, Lock, Image } from 'lucide-svelte';

  const navItems = [
    { path: '/api-client', icon: Globe, key: 'nav.apiClient' },
    { path: '/hash', icon: Hash, key: 'nav.hash' },
    { path: '/time', icon: Clock, key: 'nav.time' },
    { path: '/uuid', icon: Key, key: 'nav.uuid' },
    { 
      path: '/encode-decode', 
      icon: Code, 
      key: 'nav.encodeDecode',
      subItems: [
        { label: 'Base64', key: 'encodeDecode.base64', type: 'base64' },
        { label: 'Image Base64', key: 'encodeDecode.imageBase64', type: 'image-base64' },
        { label: 'URL', key: 'encodeDecode.urlEncoded', type: 'url' },
        { label: 'ASCII', key: 'encodeDecode.ascii', type: 'ascii' },
        { label: 'JWT', key: 'jwt.title', type: 'jwt' }
      ]
    },
    { 
      path: '/crypto', 
      icon: Lock, 
      key: 'nav.crypto',
      subItems: [
        { label: '密钥生成器', key: 'crypto.keyGenerator', type: 'keygen' },
        { label: '非对称算法', key: 'crypto.asymmetric.title', type: 'asymmetric' },
        { label: '对称算法', key: 'crypto.symmetric.title', type: 'symmetric' },
        { label: '密码哈希', key: 'crypto.hash.title', type: 'hash' },
        { label: 'BIP39', key: 'crypto.bip39.title', type: 'bip39' }
      ]
    },
    { path: '/json', icon: FileJson, key: 'nav.json' },
    { 
      path: '/text-processing', 
      icon: FileText, 
      key: 'nav.textProcessing',
      subItems: [
        { label: '文本统计', key: 'textStats.title', type: 'stats' },
        { label: '文本差异对比', key: 'textDiff.title', type: 'diff' },
        { label: '大小写转换', key: 'textCase.title', type: 'case' }
      ]
    },
    { 
      path: '/previewer', 
      icon: Eye, 
      key: 'nav.previewer',
      subItems: [
        { label: 'SVG', key: 'previewer.svg', type: 'svg' },
        { label: 'Markdown', key: 'previewer.markdown', type: 'markdown' },
        { label: 'Mermaid', key: 'previewer.mermaid', type: 'mermaid' }
      ]
    },
    { 
      path: '/image-tools', 
      icon: Image, 
      key: 'nav.imageTools',
      subItems: [
        { label: '旋转', key: 'imageTools.rotate.title', type: 'rotate' },
        { label: '缩放', key: 'imageTools.scale.title', type: 'scale' },
        { label: '格式转换', key: 'imageTools.convert.title', type: 'convert' }
      ]
    },
    { path: '/crontab', icon: Calendar, key: 'nav.crontab' },
    { path: '/color', icon: Palette, key: 'nav.color' },
    { path: '/base-converter', icon: Binary, key: 'nav.baseConverter' },
  ];

  // 从 localStorage 加载侧边栏状态
  function loadSidebarCollapsed(): boolean {
    if (!browser) return false;
    try {
      const saved = localStorage.getItem('sidebarCollapsed');
      return saved === 'true';
    } catch (e) {
      return false;
    }
  }

  let isCollapsed = $state(loadSidebarCollapsed());
  let logoHovered = $state(false);
  let hoveredMenuItem = $state<string | null>(null);
  let hoveredSettingsButton = $state(false);
  let hoveredToggleButton = $state(false);
  let tooltipPosition = $state<{ x: number; y: number } | null>(null);
  let settingsTooltipPosition = $state<{ x: number; y: number } | null>(null);
  let toggleTooltipPosition = $state<{ x: number; y: number } | null>(null);
  let hideTooltipTimeout: ReturnType<typeof setTimeout> | null = null;

  function toggleSidebar() {
    isCollapsed = !isCollapsed;
    hoveredToggleButton = false;
    toggleTooltipPosition = null;
    if (browser) {
      try {
        localStorage.setItem('sidebarCollapsed', String(isCollapsed));
      } catch (e) {
        console.error('Failed to save sidebar state:', e);
      }
    }
  }

  let currentPath = $derived($page.url.pathname);
  let translations = $derived($translationsStore);
  let currentLocale = $derived($locale);

  function t(key: string): string {
    const keys = key.split('.');
    let value: any = translations;
    for (const k of keys) {
      value = value?.[k];
    }
    return value || key;
  }
</script>

<aside class="bg-white dark:bg-gray-800 border-r border-gray-200 dark:border-gray-700 flex flex-col h-full transition-all duration-300 {isCollapsed ? 'w-20' : 'w-64'} relative z-10">
  <!-- 标题区域 -->
  <div class="px-6 py-3 border-b border-gray-200 dark:border-gray-700 {isCollapsed ? 'px-0 py-2' : ''} relative">
    {#if !isCollapsed}
      <div class="flex items-center gap-2 cursor-pointer" onclick={() => goto('/')}>
        <img src="/icon.png" alt={t('app.title')} class="w-12 h-12 object-contain flex-shrink-0" title={t('app.title')} />
        <h1 class="text-2xl font-bold text-gray-900 dark:text-gray-100 {currentLocale === 'zh' ? 'app-title-zh' : ''}">
          {t('app.title')}
        </h1>
      </div>
    {:else}
      <div class="flex items-center justify-center w-full relative group">
        <img 
          src="/icon.png" 
          alt={t('app.title')} 
          class="w-12 h-12 object-contain flex-shrink-0 cursor-pointer" 
          onclick={() => goto('/')}
          onmouseenter={() => logoHovered = true}
          onmouseleave={() => logoHovered = false}
        />
        {#if logoHovered}
          <div class="absolute left-full ml-2 px-2 py-1 bg-gray-900 dark:bg-gray-700 text-white text-sm rounded whitespace-nowrap z-50 pointer-events-none {currentLocale === 'zh' ? 'app-title-zh' : ''}">
            {t('app.title')}
          </div>
        {/if}
      </div>
    {/if}
    
    <!-- 收起/展开按钮 -->
    <button
      onclick={toggleSidebar}
      class="absolute top-0.5 right-0.5 w-6 h-6 rounded-full bg-transparent border border-white dark:border-gray-800 flex items-center justify-center hover:bg-gray-100 dark:hover:bg-gray-700/50 transition-colors z-10"
      aria-label={isCollapsed ? 'Expand sidebar' : 'Collapse sidebar'}
      onmouseenter={(e) => {
        hoveredToggleButton = true;
        if (e.currentTarget) {
          const rect = e.currentTarget.getBoundingClientRect();
          toggleTooltipPosition = {
            x: rect.right + 8,
            y: rect.top + rect.height / 2
          };
        }
      }}
      onmouseleave={() => {
        hoveredToggleButton = false;
        toggleTooltipPosition = null;
      }}
    >
      {#if isCollapsed}
        <Maximize2 class="w-4 h-4 text-gray-600 dark:text-gray-400" />
      {:else}
        <Minimize2 class="w-4 h-4 text-gray-600 dark:text-gray-400" />
      {/if}
    </button>
    
    <!-- 收起/展开按钮 Tooltip -->
    {#if hoveredToggleButton && toggleTooltipPosition}
      <div 
        class="fixed px-2 py-1 bg-gray-900 dark:bg-gray-700 text-white text-sm rounded whitespace-nowrap z-[9999] pointer-events-none"
        style="left: {toggleTooltipPosition.x}px; top: {toggleTooltipPosition.y}px; transform: translateY(-50%);"
      >
        {isCollapsed ? t('sidebar.expand') : t('sidebar.collapse')}
      </div>
    {/if}
  </div>

  <!-- 工具列表区域 -->
  <div class="flex-1 overflow-y-auto px-4 py-4 {isCollapsed ? 'px-2' : ''}">
    <nav class="space-y-1">
      {#each navItems as item}
        {@const Icon = item.icon}
        <a
          href={item.path}
          class="flex items-center {isCollapsed ? 'justify-center w-full min-w-0' : 'gap-3'} px-3 py-2.5 rounded-lg transition-colors relative {currentPath === item.path
            ? 'bg-gray-100 dark:bg-gray-700 text-gray-900 dark:text-gray-100'
            : 'text-gray-700 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-gray-700/50'}"
          onmouseenter={(e) => {
            // 取消隐藏工具提示的延迟
            if (hideTooltipTimeout) {
              clearTimeout(hideTooltipTimeout);
              hideTooltipTimeout = null;
            }
            hoveredMenuItem = item.path;
            if (isCollapsed && e.currentTarget) {
              const rect = e.currentTarget.getBoundingClientRect();
              // 如果有子菜单，从顶部开始显示；否则垂直居中
              tooltipPosition = {
                x: rect.right + 4, // 减少间隙，从 8px 改为 4px
                y: (item.subItems && item.subItems.length > 0) ? rect.top : rect.top + rect.height / 2
              };
            }
          }}
          onmouseleave={(e) => {
            // 检查鼠标是否移动到子菜单
            const relatedTarget = e.relatedTarget as HTMLElement;
            if (!relatedTarget || !relatedTarget.closest('.submenu-tooltip')) {
              // 添加延迟，给用户时间移动到子菜单
              if (item.subItems && item.subItems.length > 0) {
                if (hideTooltipTimeout) {
                  clearTimeout(hideTooltipTimeout);
                }
                hideTooltipTimeout = setTimeout(() => {
                  // 再次检查鼠标是否在子菜单上
                  const tooltip = document.querySelector('.submenu-tooltip');
                  if (!tooltip || !tooltip.matches(':hover')) {
                    hoveredMenuItem = null;
                    tooltipPosition = null;
                  }
                  hideTooltipTimeout = null;
                }, 200); // 200ms 延迟
              } else {
                hoveredMenuItem = null;
                tooltipPosition = null;
              }
            }
          }}
        >
          <div class="w-5 h-5 shrink-0 flex-none flex items-center justify-center">
            <Icon class="w-5 h-5" />
          </div>
          {#if !isCollapsed}
            <span class="font-medium text-sm whitespace-nowrap">{t(item.key)}</span>
          {/if}
        </a>
      {/each}
    </nav>
  </div>

  <!-- 菜单项 Tooltip（收起状态，固定在右侧） -->
  {#if isCollapsed && hoveredMenuItem && tooltipPosition}
    {@const item = navItems.find(i => i.path === hoveredMenuItem)}
    {#if item}
      {#if item.subItems && item.subItems.length > 0}
        <!-- 显示子菜单列表 -->
        <div 
          role="menu"
          class="submenu-tooltip fixed bg-gray-900 dark:bg-gray-700 text-white text-sm rounded-lg shadow-lg z-[9999] py-2 min-w-[160px]"
          style="left: {tooltipPosition.x}px; top: {tooltipPosition.y}px;"
          onmouseenter={() => {
            // 取消隐藏工具提示的延迟
            if (hideTooltipTimeout) {
              clearTimeout(hideTooltipTimeout);
              hideTooltipTimeout = null;
            }
            // 保持工具提示显示，确保 hoveredMenuItem 仍然有效
            if (!hoveredMenuItem) {
              hoveredMenuItem = item.path;
            }
          }}
          onmouseleave={() => {
            hoveredMenuItem = null;
            tooltipPosition = null;
            if (hideTooltipTimeout) {
              clearTimeout(hideTooltipTimeout);
              hideTooltipTimeout = null;
            }
          }}
        >
          <div class="px-3 py-1.5 font-semibold border-b border-gray-700 dark:border-gray-600">
            {t(item.key)}
          </div>
          <div class="py-1">
            {#each item.subItems as subItem}
              <a
                href="{item.path}{subItem.type ? `?type=${subItem.type}` : ''}"
                class="block px-3 py-1.5 text-gray-300 dark:text-gray-400 hover:bg-gray-800 dark:hover:bg-gray-600 hover:text-white dark:hover:text-gray-100 cursor-pointer transition-colors"
                onclick={(e) => {
                  e.preventDefault();
                  goto(`${item.path}${subItem.type ? `?type=${subItem.type}` : ''}`);
                  hoveredMenuItem = null;
                  tooltipPosition = null;
                }}
              >
                {t(subItem.key)}
              </a>
            {/each}
          </div>
        </div>
      {:else}
        <!-- 显示单个标签 -->
        <div 
          class="fixed px-2 py-1 bg-gray-900 dark:bg-gray-700 text-white text-sm rounded whitespace-nowrap z-[9999] pointer-events-none"
          style="left: {tooltipPosition.x}px; top: {tooltipPosition.y}px; transform: translateY(-50%);"
        >
          {t(item.key)}
        </div>
      {/if}
    {/if}
  {/if}

  <!-- 设置按钮（固定在左下角） -->
  <div class="px-4 py-4 border-t border-gray-200 dark:border-gray-700 {isCollapsed ? 'px-2' : ''}">
    <a
      href="/settings"
      class="flex items-center {isCollapsed ? 'justify-center w-full min-w-0' : 'gap-3'} px-3 py-2.5 rounded-lg transition-colors relative {currentPath === '/settings'
        ? 'bg-gray-100 dark:bg-gray-700 text-gray-900 dark:text-gray-100'
        : 'text-gray-700 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-gray-700/50'}"
      onmouseenter={(e) => {
        hoveredSettingsButton = true;
        if (isCollapsed && e.currentTarget) {
          const rect = e.currentTarget.getBoundingClientRect();
          settingsTooltipPosition = {
            x: rect.right + 8,
            y: rect.top + rect.height / 2
          };
        }
      }}
      onmouseleave={() => {
        hoveredSettingsButton = false;
        settingsTooltipPosition = null;
      }}
    >
      <div class="w-5 h-5 shrink-0 flex-none flex items-center justify-center">
        <Settings class="w-5 h-5" />
      </div>
      {#if !isCollapsed}
        <span class="font-medium text-sm whitespace-nowrap">{t('nav.settings')}</span>
      {/if}
    </a>
    
    <!-- 设置按钮 Tooltip（收起状态） -->
    {#if isCollapsed && hoveredSettingsButton && settingsTooltipPosition}
      <div 
        class="fixed px-2 py-1 bg-gray-900 dark:bg-gray-700 text-white text-sm rounded whitespace-nowrap z-[9999] pointer-events-none"
        style="left: {settingsTooltipPosition.x}px; top: {settingsTooltipPosition.y}px; transform: translateY(-50%);"
      >
        {t('nav.settings')}
      </div>
    {/if}
  </div>
</aside>

