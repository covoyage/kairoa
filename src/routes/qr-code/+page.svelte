<script lang="ts">
  import { translationsStore } from '$lib/stores/i18n';
  import { Download, Trash2 } from 'lucide-svelte';
  import QRCode from 'qrcode';
  import { browser } from '$app/environment';
  
  let translations = $derived($translationsStore);
  
  function t(key: string): string {
    const keys = key.split('.');
    let value: any = translations;
    for (const k of keys) {
      value = value?.[k];
    }
    return value || key;
  }

  let input = $state('');
  let qrCodeDataUrl = $state<string>('');
  let error = $state('');
  let isGenerating = $state(false);
  let qrSize = $state(256);
  let margin = $state(4);
  let colorDark = $state('#000000');
  let colorLight = $state('#FFFFFF');
  let errorCorrectionLevel = $state<'L' | 'M' | 'Q' | 'H'>('M');

  // Tauri API
  let saveFn: ((options: any) => Promise<string | null>) | null = $state(null);
  let writeFileFn: ((path: string, contents: Uint8Array) => Promise<void>) | null = $state(null);
  let isTauriAvailable = $state(false);

  if (browser) {
    if (typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window) {
      isTauriAvailable = true;
      Promise.all([
        import('@tauri-apps/plugin-dialog'),
        import('@tauri-apps/plugin-fs')
      ]).then(([dialogModule, fsModule]) => {
        saveFn = dialogModule.save;
        writeFileFn = fsModule.writeFile;
      }).catch((err) => {
        console.error('Failed to load Tauri APIs:', err);
        isTauriAvailable = false;
      });
    }
  }

  async function generateQRCode() {
    if (!input.trim()) {
      error = t('qrCode.inputRequired');
      return;
    }

    isGenerating = true;
    error = '';
    qrCodeDataUrl = '';

    try {
      const dataUrl = await QRCode.toDataURL(input, {
        width: qrSize,
        margin: margin,
        color: {
          dark: colorDark,
          light: colorLight
        },
        errorCorrectionLevel: errorCorrectionLevel
      });
      
      qrCodeDataUrl = dataUrl;
    } catch (err) {
      error = `Error: ${err instanceof Error ? err.message : 'Unknown error'}`;
    } finally {
      isGenerating = false;
    }
  }

  // Auto-regenerate when options change (if QR code already exists)
  let lastOptions = $state('');
  $effect(() => {
    const currentOptions = `${qrSize}-${margin}-${colorDark}-${colorLight}-${errorCorrectionLevel}`;
    if (input.trim() && qrCodeDataUrl && !isGenerating && currentOptions !== lastOptions) {
      lastOptions = currentOptions;
      generateQRCode();
    }
  });

  async function downloadQRCode() {
    if (!qrCodeDataUrl) return;
    
    successMessage = '';
    error = '';
    
    // Use Tauri save dialog if available
    if (isTauriAvailable && saveFn && writeFileFn) {
      try {
        const defaultName = `kairoa_qrcode.png`;
        
        // Show save dialog
        const filePath = await saveFn({
          defaultPath: defaultName,
          filters: [{
            name: 'Image',
            extensions: ['png']
          }]
        });
        
        if (filePath) {
          const response = await fetch(qrCodeDataUrl);
          const arrayBuffer = await response.arrayBuffer();
          const uint8Array = new Uint8Array(arrayBuffer);
          
          // Write file using Tauri
          await writeFileFn(filePath, uint8Array);
          
          // Show success message
          successMessage = t('qrCode.saveSuccess');
          setTimeout(() => {
            successMessage = '';
          }, 3000);
        }
      } catch (err) {
        error = `${t('qrCode.saveFailed')}: ${err instanceof Error ? err.message : 'Unknown error'}`;
      }
    } else {
      // Fallback to browser download
      const a = document.createElement('a');
      a.href = qrCodeDataUrl;
      a.download = 'kairoa_qrcode.png';
      a.click();
      
      successMessage = t('qrCode.downloadStarted');
      setTimeout(() => {
        successMessage = '';
      }, 3000);
    }
  }

  let successMessage = $state('');

  function clear() {
    input = '';
    qrCodeDataUrl = '';
    error = '';
    successMessage = '';
  }
</script>

<div class="flex flex-col h-full w-full ml-0 mr-0 p-2">
  <div class="flex-1 flex flex-col space-y-6 min-h-0">
    <!-- 输入区域卡片 -->
    <div class="card flex-shrink-0">
      <div class="space-y-3">
        <div>
          <textarea
            bind:value={input}
            placeholder={t('qrCode.inputPlaceholder')}
            class="textarea font-mono text-sm min-h-16 resize-none"
            rows="3"
          ></textarea>
        </div>

        <!-- 高级选项 -->
        <div class="flex items-end gap-4">
          <div class="flex-1">
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
              {t('qrCode.size')}
            </label>
            <input
              type="number"
              bind:value={qrSize}
              min="100"
              max="1000"
              step="10"
              class="input w-full"
            />
          </div>

          <div class="flex-1">
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
              {t('qrCode.margin')}
            </label>
            <input
              type="number"
              bind:value={margin}
              min="0"
              max="10"
              class="input w-full"
            />
          </div>

          <div class="flex-1">
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
              {t('qrCode.colorDark')}
            </label>
            <div class="flex gap-2">
              <input
                type="color"
                bind:value={colorDark}
                class="w-12 h-10 rounded border border-gray-300 dark:border-gray-600 cursor-pointer flex-shrink-0"
              />
              <input
                type="text"
                bind:value={colorDark}
                class="input flex-1 font-mono"
                placeholder="#000000"
              />
            </div>
          </div>

          <div class="flex-1">
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
              {t('qrCode.colorLight')}
            </label>
            <div class="flex gap-2">
              <input
                type="color"
                bind:value={colorLight}
                class="w-12 h-10 rounded border border-gray-300 dark:border-gray-600 cursor-pointer flex-shrink-0"
              />
              <input
                type="text"
                bind:value={colorLight}
                class="input flex-1 font-mono"
                placeholder="#FFFFFF"
              />
            </div>
          </div>

          <div class="flex-1">
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
              {t('qrCode.errorCorrectionLevel')}
            </label>
            <select bind:value={errorCorrectionLevel} class="input w-full">
              <option value="L">{t('qrCode.errorCorrectionL')} (~7%)</option>
              <option value="M">{t('qrCode.errorCorrectionM')} (~15%)</option>
              <option value="Q">{t('qrCode.errorCorrectionQ')} (~25%)</option>
              <option value="H">{t('qrCode.errorCorrectionH')} (~30%)</option>
            </select>
          </div>
        </div>

        <div class="flex gap-2">
          <button onclick={generateQRCode} class="btn-primary" disabled={isGenerating || !input.trim()}>
            {#if isGenerating}
              {t('qrCode.generating')}
            {:else}
              {t('qrCode.generate')}
            {/if}
          </button>
          <button onclick={clear} class="btn-secondary">
            <Trash2 class="w-4 h-4 inline mr-1" />
            {t('qrCode.clear')}
          </button>
        </div>
      </div>
    </div>

    <!-- 二维码预览卡片 -->
    {#if qrCodeDataUrl || error}
      <div class="card flex-1 min-h-0">
        <div class="space-y-4">
          {#if error}
            <div class="p-4 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg">
              <p class="text-sm text-red-800 dark:text-red-200">{error}</p>
            </div>
          {/if}

          {#if qrCodeDataUrl}
            <div class="flex flex-col items-center justify-center space-y-4 h-full overflow-auto">
              <div class="p-4 bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 flex-shrink-0 max-w-full max-h-full overflow-hidden">
                <img
                  src={qrCodeDataUrl}
                  alt="QR Code"
                  class="max-w-full max-h-[calc(100vh-300px)] w-auto h-auto object-contain"
                />
              </div>

              <div class="flex gap-2">
                <button
                  onclick={downloadQRCode}
                  class="btn-secondary flex items-center gap-2"
                >
                  <Download class="w-4 h-4" />
                  {t('qrCode.download')}
                </button>
              </div>
            </div>
          {/if}
        </div>
      </div>
    {/if}

    {#if successMessage}
      <div class="p-4 bg-green-50 dark:bg-green-900/20 border border-green-200 dark:border-green-800 rounded-lg">
        <p class="text-sm text-green-800 dark:text-green-200">{successMessage}</p>
      </div>
    {/if}
  </div>
</div>

