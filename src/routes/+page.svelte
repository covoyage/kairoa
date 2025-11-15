<script lang="ts">
  import { translationsStore } from '$lib/stores/i18n';
  import { goto } from '$app/navigation';
  import { Hash, Clock, Key, FileJson, Code, Calendar, Palette, Binary, FileText, Globe, Eye, Lock, Image, Search, X, QrCode, Terminal, Keyboard, ShieldCheck, Timer, Sparkles } from 'lucide-svelte';

  let translations = $derived($translationsStore);
  let searchQuery = $state('');

  function t(key: string): string {
    const keys = key.split('.');
    let value: any = translations;
    for (const k of keys) {
      value = value?.[k];
    }
    return value || key;
  }

  interface ToolCard {
    path: string;
    icon: any;
    titleKey: string;
    parentTitleKey?: string; // 父菜单的 titleKey
    descriptionKey?: string;
    category?: string;
  }

  // 定义所有导航项（包括子菜单项）
  const navItems = [
    { path: '/api-client', icon: Globe, key: 'nav.apiClient', subItems: [] },
    { path: '/hash', icon: Hash, key: 'nav.hash', subItems: [] },
    { path: '/time', icon: Clock, key: 'nav.time', subItems: [] },
    { 
      path: '/generator', 
      icon: Sparkles, 
      key: 'nav.generator',
      subItems: [
        { label: 'UUID/ULID', key: 'nav.uuid', type: null, directPath: '/uuid' },
        { label: 'OTP', key: 'nav.otp', type: null, directPath: '/otp' },
        { label: 'Basic Auth', key: 'nav.basicAuth', type: null, directPath: '/basic-auth' }
      ]
    },
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
    { path: '/json', icon: FileJson, key: 'nav.json', subItems: [] },
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
        { label: '格式转换', key: 'imageTools.convert.title', type: 'convert' },
        { label: '压缩', key: 'imageTools.compress.title', type: 'compress' },
        { label: '背景透明', key: 'imageTools.transparent.title', type: 'transparent' },
        { label: 'PDF 转换', key: 'imageTools.pdfConvert.title', type: 'pdf-convert' }
      ]
    },
    { path: '/crontab', icon: Calendar, key: 'nav.crontab', subItems: [] },
    { path: '/color', icon: Palette, key: 'nav.color', subItems: [] },
    { path: '/base-converter', icon: Binary, key: 'nav.baseConverter', subItems: [] },
    { path: '/qr-code', icon: QrCode, key: 'nav.qrCode', subItems: [] },
    { path: '/chmod', icon: Terminal, key: 'nav.chmod', subItems: [] },
    { path: '/keycode', icon: Keyboard, key: 'nav.keycode', subItems: [] },
  ];

  // 展开所有菜单项和子菜单项为卡片
  const toolCards: ToolCard[] = [];
  
  navItems.forEach(item => {
    if (item.subItems && item.subItems.length > 0) {
      // 如果有子菜单，为每个子菜单项创建一个卡片
      item.subItems.forEach(subItem => {
        const subItemPath = (subItem as any).directPath || `${item.path}${subItem.type ? `?type=${subItem.type}` : ''}`;
        toolCards.push({
          path: subItemPath,
          icon: item.icon,
          titleKey: subItem.key,
          parentTitleKey: item.key, // 保存父菜单的 titleKey
          category: item.path.replace('/', '')
        });
      });
    } else {
      // 如果没有子菜单，直接添加主菜单项
      const pathKey = item.path.replace('/', '').replace(/-/g, '');
      toolCards.push({
        path: item.path,
        icon: item.icon,
        titleKey: item.key,
        descriptionKey: `home.${pathKey}Desc`,
        category: item.path.replace('/', '')
      });
    }
  });

  function handleCardClick(path: string) {
    goto(path);
  }

  // 获取卡片显示标题（包含父菜单名称）
  function getCardTitle(card: ToolCard): string {
    if (card.parentTitleKey) {
      return `${t(card.parentTitleKey)} / ${t(card.titleKey)}`;
    }
    return t(card.titleKey);
  }

  // 过滤卡片
  const filteredCards = $derived.by(() => {
    if (!searchQuery.trim()) {
      return toolCards;
    }
    
    const query = searchQuery.toLowerCase().trim();
    return toolCards.filter(card => {
      const title = t(card.titleKey).toLowerCase();
      const parentTitle = card.parentTitleKey ? t(card.parentTitleKey).toLowerCase() : '';
      const fullTitle = getCardTitle(card).toLowerCase();
      const description = card.descriptionKey ? t(card.descriptionKey).toLowerCase() : '';
      return fullTitle.includes(query) || title.includes(query) || parentTitle.includes(query) || description.includes(query);
    });
  });

  function clearSearch() {
    searchQuery = '';
  }
</script>

<div class="flex flex-col h-full w-full p-6 min-h-0">
  <!-- 搜索区域 -->
  <div class="mb-6 flex-shrink-0">
    <div class="relative">
      <div class="absolute inset-y-0 left-0 pl-4 flex items-center pointer-events-none">
        <Search class="w-5 h-5 text-gray-400 dark:text-gray-500" />
      </div>
      <input
        type="text"
        bind:value={searchQuery}
        placeholder={t('home.searchPlaceholder')}
        class="input pl-12 pr-10 w-full"
        autofocus
      />
      {#if searchQuery}
        <button
          onclick={clearSearch}
          class="absolute inset-y-0 right-0 pr-4 flex items-center text-gray-400 dark:text-gray-500 hover:text-gray-600 dark:hover:text-gray-300 transition-colors"
        >
          <X class="w-5 h-5" />
        </button>
      {/if}
    </div>
    {#if searchQuery}
      <p class="mt-2 text-sm text-gray-600 dark:text-gray-400">
        {filteredCards.length} {t('home.resultsFound')}
      </p>
    {/if}
  </div>

  <!-- 工具卡片网格 -->
  <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4 overflow-y-auto flex-1 pb-4">
    {#if filteredCards.length === 0}
      <div class="col-span-full flex flex-col items-center justify-center py-16 text-center">
        <Search class="w-12 h-12 text-gray-400 dark:text-gray-500 mb-4" />
        <p class="text-lg font-medium text-gray-900 dark:text-gray-100 mb-2">
          {t('home.noResults')}
        </p>
        <p class="text-sm text-gray-600 dark:text-gray-400">
          {t('home.noResultsDesc')}
        </p>
      </div>
    {:else}
      {#each filteredCards as card}
      {@const Icon = card.icon}
      <button
        onclick={() => handleCardClick(card.path)}
        class="group bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 p-5 hover:border-primary-500 dark:hover:border-primary-500 hover:shadow-md transition-all duration-200 cursor-pointer"
      >
        <div class="flex items-start gap-4">
          <div class="flex-shrink-0 p-2.5 rounded-lg bg-gray-100 dark:bg-gray-700 text-gray-700 dark:text-gray-300 group-hover:bg-primary-100 dark:group-hover:bg-primary-900/30 group-hover:text-primary-600 dark:group-hover:text-primary-400 transition-colors">
            <Icon class="w-5 h-5" />
          </div>
          <div class="flex-1 min-w-0">
            <h3 class="text-base font-semibold text-gray-900 dark:text-gray-100 mb-1.5 group-hover:text-primary-600 dark:group-hover:text-primary-400 transition-colors">
              {getCardTitle(card)}
            </h3>
            {#if card.descriptionKey}
              <p class="text-sm text-gray-600 dark:text-gray-400 line-clamp-2">
                {t(card.descriptionKey)}
              </p>
            {/if}
          </div>
        </div>
      </button>
      {/each}
    {/if}
  </div>
</div>

<style>
  .line-clamp-2 {
    display: -webkit-box;
    -webkit-line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }
</style>
