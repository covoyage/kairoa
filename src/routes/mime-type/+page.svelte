<script lang="ts">
  import { translationsStore } from '$lib/stores/i18n';
  import { Search, Copy, Check, Trash2, FileType } from 'lucide-svelte';
  import { onMount } from 'svelte';

  interface MimeInfo {
    mime: string;
    extensions: string[];
    description?: string;
  }

  // MIME 类型数据库
  const mimeDatabase: MimeInfo[] = [
    // 文本类型
    { mime: 'text/plain', extensions: ['txt'], description: 'Plain text' },
    { mime: 'text/html', extensions: ['html', 'htm'], description: 'HTML document' },
    { mime: 'text/css', extensions: ['css'], description: 'CSS stylesheet' },
    { mime: 'text/javascript', extensions: ['js', 'mjs'], description: 'JavaScript' },
    { mime: 'text/json', extensions: ['json'], description: 'JSON data' },
    { mime: 'text/xml', extensions: ['xml'], description: 'XML document' },
    { mime: 'text/csv', extensions: ['csv'], description: 'CSV data' },
    { mime: 'text/markdown', extensions: ['md', 'markdown'], description: 'Markdown document' },
    { mime: 'text/yaml', extensions: ['yaml', 'yml'], description: 'YAML document' },
    
    // 图片类型
    { mime: 'image/jpeg', extensions: ['jpg', 'jpeg'], description: 'JPEG image' },
    { mime: 'image/png', extensions: ['png'], description: 'PNG image' },
    { mime: 'image/gif', extensions: ['gif'], description: 'GIF image' },
    { mime: 'image/webp', extensions: ['webp'], description: 'WebP image' },
    { mime: 'image/svg+xml', extensions: ['svg'], description: 'SVG image' },
    { mime: 'image/bmp', extensions: ['bmp'], description: 'BMP image' },
    { mime: 'image/tiff', extensions: ['tiff', 'tif'], description: 'TIFF image' },
    { mime: 'image/x-icon', extensions: ['ico'], description: 'Icon image' },
    { mime: 'image/avif', extensions: ['avif'], description: 'AVIF image' },
    { mime: 'image/apng', extensions: ['apng'], description: 'Animated PNG' },
    { mime: 'image/heic', extensions: ['heic', 'heif'], description: 'HEIC/HEIF image' },
    { mime: 'image/jxl', extensions: ['jxl'], description: 'JPEG XL image' },
    { mime: 'image/qoi', extensions: ['qoi'], description: 'Quite OK Image Format' },
    { mime: 'image/x-bpg', extensions: ['bpg'], description: 'Better Portable Graphics (HEVC-based)' },
    { mime: 'image/vnd.adobe.photoshop', extensions: ['psd'], description: 'Photoshop document' },
    { mime: 'image/x-pcx', extensions: ['pcx'], description: 'PCX image' },
    
    // 音频类型
    { mime: 'audio/mpeg', extensions: ['mp3'], description: 'MP3 audio' },
    { mime: 'audio/ogg', extensions: ['ogg', 'oga'], description: 'OGG audio' },
    { mime: 'audio/wav', extensions: ['wav'], description: 'WAV audio' },
    { mime: 'audio/aac', extensions: ['aac'], description: 'AAC audio' },
    { mime: 'audio/flac', extensions: ['flac'], description: 'FLAC audio' },
    { mime: 'audio/webm', extensions: ['weba'], description: 'WebM audio' },
    { mime: 'audio/midi', extensions: ['mid', 'midi'], description: 'MIDI audio' },
    
    // 视频类型
    { mime: 'video/mp4', extensions: ['mp4'], description: 'MP4 video' },
    { mime: 'video/mpeg', extensions: ['mpeg', 'mpg'], description: 'MPEG video' },
    { mime: 'video/quicktime', extensions: ['mov'], description: 'QuickTime video' },
    { mime: 'video/x-msvideo', extensions: ['avi'], description: 'AVI video' },
    { mime: 'video/webm', extensions: ['webm'], description: 'WebM video' },
    { mime: 'video/x-matroska', extensions: ['mkv'], description: 'Matroska video' },
    { mime: 'video/x-flv', extensions: ['flv'], description: 'Flash video' },
    { mime: 'video/mp4', extensions: ['m4v'], description: 'MPEG-4 video' },
    { mime: 'video/x-ms-wmv', extensions: ['wmv'], description: 'Windows Media Video' },
    { mime: 'video/3gpp', extensions: ['3gp', '3g2'], description: '3GPP video' },
    { mime: 'video/ogg', extensions: ['ogv'], description: 'Ogg video' },
    { mime: 'video/mp2t', extensions: ['ts', 'm2ts'], description: 'MPEG transport stream' },
    { mime: 'video/x-ms-asf', extensions: ['asf'], description: 'ASF video' },
    
    // 应用类型
    { mime: 'application/pdf', extensions: ['pdf'], description: 'PDF document' },
    { mime: 'application/zip', extensions: ['zip'], description: 'ZIP archive' },
    { mime: 'application/x-tar', extensions: ['tar'], description: 'TAR archive' },
    { mime: 'application/gzip', extensions: ['gz'], description: 'GZIP archive' },
    { mime: 'application/x-rar-compressed', extensions: ['rar'], description: 'RAR archive' },
    { mime: 'application/x-7z-compressed', extensions: ['7z'], description: '7z archive' },
    { mime: 'application/x-bzip2', extensions: ['bz2'], description: 'BZIP2 archive' },
    { mime: 'application/x-xz', extensions: ['xz'], description: 'XZ archive' },
    { mime: 'application/x-zstd', extensions: ['zst', 'zstd'], description: 'Zstandard compressed archive' },
    { mime: 'application/x-lz4', extensions: ['lz4'], description: 'LZ4 compressed archive' },
    { mime: 'application/zip', extensions: ['zipx'], description: 'ZIPX archive (extended ZIP)' },
    { mime: 'application/x-cbr', extensions: ['cbr'], description: 'Comic book archive (RAR)' },
    { mime: 'application/x-cbz', extensions: ['cbz'], description: 'Comic book archive (ZIP)' },
    { mime: 'application/x-cbt', extensions: ['cbt'], description: 'Comic book archive (TAR)' },
    { mime: 'application/x-cb7', extensions: ['cb7'], description: 'Comic book archive (7z)' },
    { mime: 'application/json', extensions: ['json'], description: 'JSON data' },
    { mime: 'application/xml', extensions: ['xml'], description: 'XML document' },
    { mime: 'application/javascript', extensions: ['js'], description: 'JavaScript' },
    { mime: 'application/octet-stream', extensions: ['bin'], description: 'Binary data' },
    { mime: 'application/x-sh', extensions: ['sh'], description: 'Shell script' },
    { mime: 'application/x-bat', extensions: ['bat'], description: 'Batch file' },
    { mime: 'application/x-msdownload', extensions: ['exe'], description: 'Windows executable' },
    { mime: 'application/x-apple-diskimage', extensions: ['dmg'], description: 'macOS disk image' },
    { mime: 'application/vnd.ms-excel', extensions: ['xls'], description: 'Excel spreadsheet (old)' },
    { mime: 'application/vnd.openxmlformats-officedocument.spreadsheetml.sheet', extensions: ['xlsx'], description: 'Excel spreadsheet' },
    { mime: 'application/vnd.ms-powerpoint', extensions: ['ppt'], description: 'PowerPoint presentation (old)' },
    { mime: 'application/vnd.openxmlformats-officedocument.presentationml.presentation', extensions: ['pptx'], description: 'PowerPoint presentation' },
    { mime: 'application/vnd.ms-word', extensions: ['doc'], description: 'Word document (old)' },
    { mime: 'application/vnd.openxmlformats-officedocument.wordprocessingml.document', extensions: ['docx'], description: 'Word document' },
    { mime: 'application/vnd.oasis.opendocument.text', extensions: ['odt'], description: 'OpenDocument text' },
    { mime: 'application/vnd.oasis.opendocument.spreadsheet', extensions: ['ods'], description: 'OpenDocument spreadsheet' },
    { mime: 'application/vnd.oasis.opendocument.presentation', extensions: ['odp'], description: 'OpenDocument presentation' },
    { mime: 'application/epub+zip', extensions: ['epub'], description: 'EPUB ebook' },
    { mime: 'application/epub+zip', extensions: ['epub3'], description: 'EPUB3 ebook' },
    { mime: 'application/rtf', extensions: ['rtf'], description: 'Rich Text Format' },
    { mime: 'application/x-font-ttf', extensions: ['ttf'], description: 'TrueType font' },
    { mime: 'application/x-font-otf', extensions: ['otf'], description: 'OpenType font' },
    { mime: 'application/font-woff', extensions: ['woff'], description: 'WOFF font' },
    { mime: 'application/font-woff2', extensions: ['woff2'], description: 'WOFF2 font' },
    { mime: 'application/vnd.ms-fontobject', extensions: ['eot'], description: 'Embedded OpenType font' },
    { mime: 'application/x-iso9660-image', extensions: ['iso'], description: 'ISO disk image' },
    { mime: 'application/vnd.debian.binary-package', extensions: ['deb'], description: 'Debian package' },
    { mime: 'application/x-rpm', extensions: ['rpm'], description: 'RPM package' },
    { mime: 'application/vnd.android.package-archive', extensions: ['apk'], description: 'Android package' },
    { mime: 'application/ipa', extensions: ['ipa'], description: 'iOS app package' },
    { mime: 'application/wasm', extensions: ['wasm'], description: 'WebAssembly' },
    
    // 配置文件
    { mime: 'text/x-ini', extensions: ['ini'], description: 'INI configuration' },
    { mime: 'application/toml', extensions: ['toml'], description: 'TOML configuration' },
    { mime: 'text/x-properties', extensions: ['properties'], description: 'Properties file' },
    { mime: 'text/x-env', extensions: ['env'], description: 'Environment file' },
    { mime: 'text/x-config', extensions: ['conf', 'config'], description: 'Configuration file' },
    
    // 其他
    { mime: 'application/x-www-form-urlencoded', extensions: [], description: 'URL encoded form data' },
    { mime: 'multipart/form-data', extensions: [], description: 'Multipart form data' },
    { mime: 'application/x-httpd-php', extensions: ['php'], description: 'PHP script' },
    { mime: 'application/x-python', extensions: ['py'], description: 'Python script' },
    { mime: 'application/x-ruby', extensions: ['rb'], description: 'Ruby script' },
    { mime: 'application/x-java', extensions: ['java'], description: 'Java source' },
    { mime: 'application/x-c', extensions: ['c'], description: 'C source' },
    { mime: 'application/x-cpp', extensions: ['cpp', 'cxx', 'cc'], description: 'C++ source' },
    { mime: 'application/x-csharp', extensions: ['cs'], description: 'C# source' },
    { mime: 'application/x-go', extensions: ['go'], description: 'Go source' },
    { mime: 'application/x-rust', extensions: ['rs'], description: 'Rust source' },
    { mime: 'application/typescript', extensions: ['ts'], description: 'TypeScript source' },
    { mime: 'text/typescript', extensions: ['ts'], description: 'TypeScript source' },
    { mime: 'text/jsx', extensions: ['jsx'], description: 'JSX source' },
    { mime: 'text/tsx', extensions: ['tsx'], description: 'TSX source' },
    { mime: 'application/x-vue', extensions: ['vue'], description: 'Vue component' },
    { mime: 'application/x-svelte', extensions: ['svelte'], description: 'Svelte component' },
    { mime: 'application/x-swift', extensions: ['swift'], description: 'Swift source' },
    { mime: 'application/x-kotlin', extensions: ['kt', 'kts'], description: 'Kotlin source' },
    { mime: 'application/x-scala', extensions: ['scala', 'sc'], description: 'Scala source' },
    { mime: 'application/x-dart', extensions: ['dart'], description: 'Dart source' },
    { mime: 'application/x-lua', extensions: ['lua'], description: 'Lua script' },
    { mime: 'application/x-perl', extensions: ['pl', 'pm'], description: 'Perl script' },
    { mime: 'application/x-shellscript', extensions: ['zsh', 'fish', 'bash'], description: 'Shell script' },
    { mime: 'application/sql', extensions: ['sql'], description: 'SQL script' },
    { mime: 'application/x-sqlite3', extensions: ['sqlite', 'sqlite3', 'db'], description: 'SQLite database' },
    { mime: 'application/x-msaccess', extensions: ['mdb', 'accdb'], description: 'Microsoft Access database' },
    { mime: 'text/x-log', extensions: ['log'], description: 'Log file' },
    { mime: 'text/x-diff', extensions: ['diff', 'patch'], description: 'Diff/Patch file' },
    { mime: 'application/x-latex', extensions: ['tex', 'latex'], description: 'LaTeX document' },
    { mime: 'application/postscript', extensions: ['ps', 'eps'], description: 'PostScript' },
    { mime: 'application/x-illustrator', extensions: ['ai'], description: 'Adobe Illustrator' },
    { mime: 'application/x-indesign', extensions: ['indd'], description: 'Adobe InDesign' },
    { mime: 'application/x-after-effects', extensions: ['aep'], description: 'Adobe After Effects' },
    { mime: 'application/x-premiere', extensions: ['prproj'], description: 'Adobe Premiere Pro' },
    { mime: 'image/x-xcf', extensions: ['xcf'], description: 'GIMP image' },
    { mime: 'application/x-blender', extensions: ['blend'], description: 'Blender file' },
    { mime: 'model/obj', extensions: ['obj'], description: '3D model (OBJ)' },
    { mime: 'model/stl', extensions: ['stl'], description: '3D model (STL)' },
    { mime: 'model/gltf+json', extensions: ['gltf'], description: '3D model (glTF)' },
    { mime: 'model/gltf-binary', extensions: ['glb'], description: '3D model (glTF Binary)' },
  ];

  let translations = $derived($translationsStore);
  let query = $state('');
  let results = $state<MimeInfo[]>([]);
  let searchMode = $state<'extension' | 'mime'>('extension');
  let copied = $state(false);

  function t(key: string): string {
    const keys = key.split('.');
    let value: any = translations;
    for (const k of keys) {
      value = value?.[k];
    }
    return value || key;
  }

  // 搜索函数
  function search() {
    const trimmedQuery = query.trim().toLowerCase();
    if (!trimmedQuery) {
      results = [];
      return;
    }

    if (searchMode === 'extension') {
      // 移除点号（如果有）
      const ext = trimmedQuery.startsWith('.') ? trimmedQuery.slice(1) : trimmedQuery;
      results = mimeDatabase.filter(item => 
        item.extensions.some(e => e.toLowerCase() === ext)
      );
    } else {
      // MIME 类型搜索
      results = mimeDatabase.filter(item => 
        item.mime.toLowerCase().includes(trimmedQuery)
      );
    }
  }

  // 复制结果
  async function copyResult(mime: string) {
    try {
      await navigator.clipboard.writeText(mime);
      copied = true;
      setTimeout(() => {
        copied = false;
      }, 2000);
    } catch (error) {
      console.error('Failed to copy:', error);
    }
  }

  // 清空
  function clear() {
    query = '';
    results = [];
  }

  // 处理键盘事件
  function handleKeyPress(event: KeyboardEvent) {
    if (event.key === 'Enter') {
      search();
    }
  }

  // 自动搜索
  $effect(() => {
    search();
  });

  // 从 localStorage 加载
  function loadFromStorage() {
    if (typeof window !== 'undefined') {
      const saved = localStorage.getItem('mime-type-query');
      if (saved) {
        try {
          const parsed = JSON.parse(saved);
          query = parsed.query || '';
          searchMode = parsed.searchMode || 'extension';
        } catch (e) {
          console.error('Failed to load query:', e);
        }
      }
    }
  }

  // 保存到 localStorage
  function saveToStorage() {
    if (typeof window !== 'undefined') {
      localStorage.setItem('mime-type-query', JSON.stringify({ query, searchMode }));
    }
  }

  $effect(() => {
    query;
    searchMode;
    saveToStorage();
  });

  onMount(() => {
    loadFromStorage();
  });
</script>

<div class="flex flex-col h-full w-full ml-0 mr-0 p-2">
  <div class="card flex-1 flex flex-col min-h-0">
    <div class="space-y-4 mb-6">
      <!-- 搜索模式切换 -->
      <div class="flex gap-2">
        <button
          class="px-4 py-2 rounded-lg transition-colors {searchMode === 'extension'
            ? 'bg-primary-600 text-white'
            : 'bg-gray-200 dark:bg-gray-700 text-gray-900 dark:text-gray-100'}"
          onclick={() => { searchMode = 'extension'; search(); }}
        >
          {t('mimeType.searchByExtension')}
        </button>
        <button
          class="px-4 py-2 rounded-lg transition-colors {searchMode === 'mime'
            ? 'bg-primary-600 text-white'
            : 'bg-gray-200 dark:bg-gray-700 text-gray-900 dark:text-gray-100'}"
          onclick={() => { searchMode = 'mime'; search(); }}
        >
          {t('mimeType.searchByMime')}
        </button>
      </div>

      <!-- 查询输入 -->
      <div>
        <label for="query-input" class="block text-sm font-medium mb-2 text-gray-700 dark:text-gray-300">
          {searchMode === 'extension' ? t('mimeType.extension') : t('mimeType.mimeType')}
        </label>
        <div class="flex gap-2">
          <input
            id="query-input"
            type="text"
            bind:value={query}
            placeholder={searchMode === 'extension' ? t('mimeType.extensionPlaceholder') : t('mimeType.mimePlaceholder')}
            class="input flex-1"
            onkeydown={handleKeyPress}
          />
          <button
            onclick={search}
            class="btn-secondary"
            disabled={!query.trim()}
          >
            <Search class="w-4 h-4 inline mr-1" />
            {t('mimeType.search')}
          </button>
          {#if query || results.length > 0}
            <button type="button" class="btn-secondary" onclick={clear}>
              <Trash2 class="w-4 h-4 inline mr-1" />
              {t('common.clear')}
            </button>
          {/if}
        </div>
        <p class="text-xs text-gray-500 dark:text-gray-400 mt-1">
          {searchMode === 'extension' ? t('mimeType.extensionHint') : t('mimeType.mimeHint')}
        </p>
      </div>
    </div>

    <!-- 结果列表 -->
    <div class="flex-1 overflow-y-auto">
      {#if results.length === 0 && query.trim()}
        <div class="flex flex-col items-center justify-center py-16 text-center">
          <FileType class="w-12 h-12 text-gray-400 dark:text-gray-500 mb-4" />
          <p class="text-lg font-medium text-gray-900 dark:text-gray-100 mb-2">
            {t('mimeType.noResults')}
          </p>
          <p class="text-sm text-gray-600 dark:text-gray-400">
            {t('mimeType.noResultsDesc')}
          </p>
        </div>
      {:else if results.length > 0}
        <div class="space-y-3">
          {#each results as result (result.mime)}
            <div class="bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-lg p-4 hover:border-primary-500 dark:hover:border-primary-500 transition-colors">
              <div class="flex items-start justify-between gap-4">
                <div class="flex-1 min-w-0">
                  <div class="flex items-center gap-2 mb-2">
                    <span class="font-mono font-semibold text-sm text-gray-900 dark:text-gray-100">
                      {result.mime}
                    </span>
                    <button
                      onclick={() => copyResult(result.mime)}
                      class="p-1 rounded hover:bg-gray-100 dark:hover:bg-gray-700 text-gray-600 dark:text-gray-400 transition-colors"
                      title={t('common.copy')}
                    >
                      {#if copied}
                        <Check class="w-4 h-4" />
                      {:else}
                        <Copy class="w-4 h-4" />
                      {/if}
                    </button>
                  </div>
                  {#if result.extensions.length > 0}
                    <div class="flex flex-wrap items-center gap-2 mb-2">
                      <span class="text-xs text-gray-500 dark:text-gray-400">
                        {t('mimeType.extensions')}:
                      </span>
                      {#each result.extensions as ext}
                        <span class="px-2 py-1 bg-gray-100 dark:bg-gray-700 rounded text-xs font-mono text-gray-900 dark:text-gray-100">
                          .{ext}
                        </span>
                      {/each}
                    </div>
                  {/if}
                  {#if result.description}
                    <p class="text-sm text-gray-600 dark:text-gray-400">
                      {result.description}
                    </p>
                  {/if}
                </div>
              </div>
            </div>
          {/each}
        </div>
      {:else}
        <div class="flex flex-col items-center justify-center py-16 text-center">
          <FileType class="w-12 h-12 text-gray-400 dark:text-gray-500 mb-4" />
          <p class="text-lg font-medium text-gray-900 dark:text-gray-100 mb-2">
            {t('mimeType.welcome')}
          </p>
          <p class="text-sm text-gray-600 dark:text-gray-400">
            {t('mimeType.welcomeDesc')}
          </p>
        </div>
      {/if}
    </div>
  </div>
</div>
