<script lang="ts">
  import { page } from '$app/stores';
  import { locale, translationsStore } from '$lib/stores/i18n';
  import { browser } from '$app/environment';
  import { Hash, Clock, Key, FileJson, Code, Calendar, Palette, Binary, FileText, Shield, Globe, Minimize2, Maximize2, Settings, GitCompare, Eye } from 'lucide-svelte';

  const navItems = [
    { path: '/api-client', icon: Globe, key: 'nav.apiClient' },
    { path: '/hash', icon: Hash, key: 'nav.hash' },
    { path: '/time', icon: Clock, key: 'nav.time' },
    { path: '/uuid', icon: Key, key: 'nav.uuid' },
    { path: '/encode-decode', icon: Code, key: 'nav.encodeDecode' },
    { path: '/json', icon: FileJson, key: 'nav.json' },
    { path: '/text-stats', icon: FileText, key: 'nav.textStats' },
    { path: '/text-diff', icon: GitCompare, key: 'nav.textDiff' },
    { path: '/previewer', icon: Eye, key: 'nav.previewer' },
    { path: '/crontab', icon: Calendar, key: 'nav.crontab' },
    { path: '/color', icon: Palette, key: 'nav.color' },
    { path: '/base-converter', icon: Binary, key: 'nav.baseConverter' },
    { path: '/rsa', icon: Shield, key: 'nav.rsa' },
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
      <div class="flex items-center gap-2">
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
            hoveredMenuItem = item.path;
            if (isCollapsed && e.currentTarget) {
              const rect = e.currentTarget.getBoundingClientRect();
              tooltipPosition = {
                x: rect.right + 8,
                y: rect.top + rect.height / 2
              };
            }
          }}
          onmouseleave={() => {
            hoveredMenuItem = null;
            tooltipPosition = null;
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
      <div 
        class="fixed px-2 py-1 bg-gray-900 dark:bg-gray-700 text-white text-sm rounded whitespace-nowrap z-[9999] pointer-events-none"
        style="left: {tooltipPosition.x}px; top: {tooltipPosition.y}px; transform: translateY(-50%);"
      >
        {t(item.key)}
      </div>
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

