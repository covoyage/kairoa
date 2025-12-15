<script lang="ts">
  import { translationsStore } from '$lib/stores/i18n';
  import { browser } from '$app/environment';
  import { onMount } from 'svelte';
  import { Upload, FileText, AlertCircle, CheckCircle2, XCircle } from 'lucide-svelte';
  import { fromBER } from 'asn1js';
  import { Certificate, setEngine, CryptoEngine } from 'pkijs';

  let translations = $derived($translationsStore);

  function t(key: string): string {
    const keys = key.split('.');
    let value: any = translations;
    for (const k of keys) {
      value = value?.[k];
    }
    return value || key;
  }

  const MAX_FILE_SIZE = 5 * 1024 * 1024; // 5 MB

  interface CertificateInfo {
    subject?: string;
    issuer?: string;
    serialNumber?: string;
    validFrom?: string;
    validTo?: string;
    algorithm?: string;
    keySize?: number;
    fingerprint?: string;
    email?: string;
    organization?: string;
    organizationalUnit?: string;
    country?: string;
    state?: string;
    locality?: string;
    commonName?: string;
    issuerCommonName?: string;
    issuerOrganization?: string;
    version?: number;
    extensions?: Array<{ name: string; value: string; critical: boolean }>;
  }

  let fileInput: HTMLInputElement | null = null;
  let isDragging = $state(false);
  let isLoading = $state(false);
  let errorMessage = $state('');
  let fileName = $state('');
  let fileSize = $state(0);
  let certificateInfo = $state<CertificateInfo | null>(null);
  let certificateRaw = $state<string>('');
  let cryptoSupported = $state(false);
  let isTauriApp = $state(false);

  onMount(() => {
    if (browser && typeof window !== 'undefined' && window.crypto?.subtle) {
      setEngine(
        'kairoa-pkijs',
        window.crypto,
        new CryptoEngine({
          name: 'kairoa-pkijs',
          crypto: window.crypto,
          subtle: window.crypto.subtle
        })
      );
      cryptoSupported = true;
    }

    if (browser && typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window) {
      isTauriApp = true;
      import('@tauri-apps/api/event').then((eventModule) => {
        eventModule.listen('tauri://drag-drop', async (event: any) => {
          const paths = event.payload.paths as string[];
          if (paths && paths.length > 0) {
            await handleTauriFileDrop(paths[0]);
          }
        });
      });
    }
  });

  async function handleTauriFileDrop(path: string) {
    try {
      const { readFile } = await import('@tauri-apps/plugin-fs');
      const contents = await readFile(path);
      const fileName = path.split('/').pop() || path.split('\\').pop() || 'certificate.pem';
      const blob = new Blob([contents]);
      const file = new File([blob], fileName);
      await loadCertificateFile(file);
    } catch (err) {
      console.error('Failed to read dropped file:', err);
      errorMessage = t('certificateViewer.errors.readFailed');
    }
  }

  function handleDragOver(event: DragEvent) {
    event.preventDefault();
    isDragging = true;
  }

  function handleDragLeave(event: DragEvent) {
    event.preventDefault();
    isDragging = false;
  }

  function handleDrop(event: DragEvent) {
    event.preventDefault();
    isDragging = false;
    const files = event.dataTransfer?.files;
    if (files && files.length > 0) {
      handleFiles(files);
    }
  }

  function onFileChange(event: Event) {
    const target = event.currentTarget as HTMLInputElement;
    handleFiles(target.files);
    target.value = '';
  }

  function isCertificateFile(file: File | null | undefined): file is File {
    if (!file) return false;
    const name = file.name.toLowerCase();
    const validExtensions = ['.pem', '.crt', '.cer', '.key', '.cert', '.der', '.p12', '.pfx'];
    return validExtensions.some(ext => name.endsWith(ext));
  }

  async function handleFiles(files: FileList | File[] | null) {
    if (!files || files.length === 0) return;
    const list = Array.from(files);
    const file = list.find(isCertificateFile);
    if (!file) {
      errorMessage = t('certificateViewer.errors.invalidFile');
      return;
    }
    if (file.size > MAX_FILE_SIZE) {
      errorMessage = t('certificateViewer.errors.fileTooLarge');
      return;
    }
    await loadCertificateFile(file);
  }

  async function openFilePicker() {
    if (isTauriApp) {
      try {
        const { open } = await import('@tauri-apps/plugin-dialog');
        const selected = await open({
          multiple: false,
          filters: [
            {
              name: 'Certificate Files',
              extensions: ['pem', 'crt', 'cer', 'key', 'cert', 'der', 'p12', 'pfx']
            }
          ]
        });
        if (selected && typeof selected === 'string') {
          const { readFile } = await import('@tauri-apps/plugin-fs');
          const contents = await readFile(selected);
          const fileName = selected.split('/').pop() || selected.split('\\').pop() || 'certificate.pem';
          const blob = new Blob([contents]);
          const file = new File([blob], fileName);
          await loadCertificateFile(file);
        }
      } catch (err) {
        console.error('Failed to open file picker:', err);
      }
    } else {
      fileInput?.click();
    }
  }

  function parsePEM(content: string): ArrayBuffer | null {
    // Remove PEM headers/footers and whitespace
    const base64Content = content
      .replace(/-----BEGIN[\s\S]*?-----/g, '')
      .replace(/-----END[\s\S]*?-----/g, '')
      .replace(/\s/g, '');
    
    if (!base64Content) return null;

    try {
      const binaryString = atob(base64Content);
      const bytes = new Uint8Array(binaryString.length);
      for (let i = 0; i < binaryString.length; i++) {
        bytes[i] = binaryString.charCodeAt(i);
      }
      return bytes.buffer;
    } catch {
      return null;
    }
  }

  function formatDN(dn: string): Record<string, string> {
    const result: Record<string, string> = {};
    const parts = dn.split(',').map(p => p.trim());
    for (const part of parts) {
      const match = part.match(/^([A-Z]+)=(.+)$/i);
      if (match) {
        const key = match[1].toLowerCase();
        const value = match[2];
        result[key] = value;
      }
    }
    return result;
  }

  function getDNValue(dn: Record<string, string>, key: string): string {
    return dn[key] || dn[key.toUpperCase()] || '';
  }

  async function loadCertificateFile(file: File) {
    isLoading = true;
    errorMessage = '';
    certificateInfo = null;
    certificateRaw = '';
    fileName = file.name;
    fileSize = file.size;

    try {
      const content = await file.text();
      certificateRaw = content;

      let arrayBuffer: ArrayBuffer;

      // Try PEM format first
      if (content.includes('-----BEGIN')) {
        const pemBuffer = parsePEM(content);
        if (!pemBuffer) {
          throw new Error(t('certificateViewer.errors.invalidPem'));
        }
        arrayBuffer = pemBuffer;
      } else {
        // Try DER format (binary)
        arrayBuffer = await file.arrayBuffer();
      }

      const asn1 = fromBER(arrayBuffer);
      if (asn1.offset === -1) {
        throw new Error(t('certificateViewer.errors.parseFailed'));
      }

      const cert = new Certificate({ schema: asn1.result });
      
      // Parse certificate information
      const subjectDN = formatDN(cert.subject.toString());
      const issuerDN = formatDN(cert.issuer.toString());

      const validFrom = cert.notBefore.value;
      const validTo = cert.notAfter.value;
      const now = new Date();
      const isValid = now >= validFrom && now <= validTo;

      // Get public key info
      const publicKey = cert.subjectPublicKeyInfo;
      let algorithm = 'Unknown';
      try {
        // Try to get algorithm from the algorithm object
        const algObj = (publicKey.algorithm as any);
        if (algObj.algorithmId) {
          algorithm = algObj.algorithmId.toString();
        } else if (algObj.toString) {
          algorithm = algObj.toString();
        }
      } catch {
        algorithm = 'Unknown';
      }
      let keySize = 0;
      
      try {
        const keyData = publicKey.subjectPublicKey.valueBlock.valueHexView;
        if (algorithm.includes('rsa')) {
          // RSA key size estimation
          keySize = keyData.length * 8;
        } else if (algorithm.includes('ec')) {
          // ECDSA key size
          keySize = keyData.length * 4;
        }
      } catch {
        // Ignore key size calculation errors
      }

      // Calculate fingerprint (SHA-256)
      let fingerprint = '';
      try {
        const certBuffer = cert.toSchema().toBER(false);
        const hashBuffer = await crypto.subtle.digest('SHA-256', certBuffer);
        const hashArray = Array.from(new Uint8Array(hashBuffer));
        fingerprint = hashArray.map(b => b.toString(16).padStart(2, '0')).join(':').toUpperCase();
      } catch {
        // Ignore fingerprint calculation errors
      }

      // Parse extensions
      const extensions: Array<{ name: string; value: string; critical: boolean }> = [];
      if (cert.extensions) {
        for (const ext of cert.extensions) {
          try {
            const extId = ext.extnID.toString();
            const extValue = ext.extnValue.valueBlock.valueHexView;
            extensions.push({
              name: extId,
              value: Array.from(extValue).map(b => b.toString(16).padStart(2, '0')).join(':'),
              critical: ext.critical || false
            });
          } catch {
            // Ignore extension parsing errors
          }
        }
      }

      certificateInfo = {
        subject: cert.subject.toString(),
        issuer: cert.issuer.toString(),
        serialNumber: cert.serialNumber.valueBlock.toString(),
        validFrom: validFrom.toISOString(),
        validTo: validTo.toISOString(),
        algorithm: algorithm,
        keySize: keySize,
        fingerprint: fingerprint,
        email: getDNValue(subjectDN, 'emailaddress') || getDNValue(subjectDN, 'e'),
        organization: getDNValue(subjectDN, 'o'),
        organizationalUnit: getDNValue(subjectDN, 'ou'),
        country: getDNValue(subjectDN, 'c'),
        state: getDNValue(subjectDN, 'st'),
        locality: getDNValue(subjectDN, 'l'),
        commonName: getDNValue(subjectDN, 'cn'),
        issuerCommonName: getDNValue(issuerDN, 'cn'),
        issuerOrganization: getDNValue(issuerDN, 'o'),
        version: cert.version,
        extensions: extensions
      };
    } catch (err) {
      console.error('Failed to parse certificate:', err);
      if (err instanceof Error) {
        const errorMsg = err.message.toLowerCase();
        // Check for specific pkijs schema verification errors
        if (errorMsg.includes('schema was not verified') || errorMsg.includes('not verified against input')) {
          errorMessage = t('certificateViewer.errors.invalidFormat');
        } else if (errorMsg.includes('invalid pem') || errorMsg.includes('pem')) {
          errorMessage = t('certificateViewer.errors.invalidPem');
        } else {
          errorMessage = t('certificateViewer.errors.parseFailed');
        }
      } else {
        errorMessage = t('certificateViewer.errors.parseFailed');
      }
    } finally {
      isLoading = false;
    }
  }

  function clearFile() {
    fileName = '';
    fileSize = 0;
    certificateInfo = null;
    certificateRaw = '';
    errorMessage = '';
    if (fileInput) {
      fileInput.value = '';
    }
  }

  function formatDate(dateString: string): string {
    try {
      const date = new Date(dateString);
      return date.toLocaleString();
    } catch {
      return dateString;
    }
  }

  function isExpired(): boolean {
    if (!certificateInfo?.validTo) return false;
    return new Date() > new Date(certificateInfo.validTo);
  }

  function isNotYetValid(): boolean {
    if (!certificateInfo?.validFrom) return false;
    return new Date() < new Date(certificateInfo.validFrom);
  }
</script>

<div class="flex flex-col h-full w-full p-2">
  <div class="card flex-1 flex flex-col space-y-6">
    <div class="flex items-center justify-between">
      <div>
        <h1 class="text-xl font-semibold text-gray-800 dark:text-gray-100">{t('certificateViewer.title')}</h1>
        <p class="text-sm text-gray-500 dark:text-gray-400 mt-1">{t('certificateViewer.description')}</p>
      </div>
      {#if fileName}
        <button type="button" class="btn-secondary" onclick={clearFile}>
          {t('certificateViewer.clear')}
        </button>
      {/if}
    </div>

    {#if !fileName}
      <div
        class="flex-1 flex flex-col items-center justify-center border-2 border-dashed border-gray-300 dark:border-gray-600 rounded-lg hover:border-primary-500 dark:hover:border-primary-400 transition-colors bg-gray-50 dark:bg-gray-800/50 {isDragging ? 'border-primary-500 dark:border-primary-400 bg-primary-50 dark:bg-primary-900/30' : ''}"
        ondrop={handleDrop}
        ondragover={handleDragOver}
        ondragenter={handleDragOver}
        ondragleave={handleDragLeave}
        role="region"
        aria-label={t('certificateViewer.dropTitle')}
      >
        <div class="text-center cursor-pointer" onclick={openFilePicker} onkeydown={(e) => e.key === 'Enter' && openFilePicker()} role="button" tabindex="0">
          <Upload class="w-12 h-12 text-gray-400 dark:text-gray-500 mb-4 mx-auto" />
          <p class="text-gray-600 dark:text-gray-400 mb-2 font-medium">
            {t('certificateViewer.dropTitle')}
          </p>
          <p class="text-sm text-gray-500 dark:text-gray-500 mb-4">
            {t('certificateViewer.dropSubtitle')}
          </p>
          <button class="btn-primary">
            {t('certificateViewer.selectFile')}
          </button>
        </div>
        <p class="text-xs text-gray-400 dark:text-gray-500 mt-4">
          {t('certificateViewer.helperText')}
        </p>
      </div>
    {/if}

    <input
      type="file"
      bind:this={fileInput}
      accept=".pem,.crt,.cer,.key,.cert,.der,.p12,.pfx"
      onchange={onFileChange}
      class="hidden"
    />

    {#if isLoading}
      <div class="flex items-center justify-center py-8">
        <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-primary-500"></div>
        <span class="ml-3 text-gray-600 dark:text-gray-400">{t('certificateViewer.loading')}</span>
      </div>
    {/if}

    {#if errorMessage}
      <div class="p-4 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg">
        <div class="flex items-start">
          <AlertCircle class="w-5 h-5 text-red-600 dark:text-red-400 mr-2 flex-shrink-0 mt-0.5" />
          <p class="text-sm text-red-800 dark:text-red-200">{errorMessage}</p>
        </div>
      </div>
    {/if}

    {#if certificateInfo && !isLoading}
      <div class="space-y-6">
        <!-- File Info -->
        <div class="bg-gray-50 dark:bg-gray-800/50 rounded-lg p-4">
          <h2 class="text-sm font-semibold text-gray-700 dark:text-gray-300 mb-3">{t('certificateViewer.fileInfo')}</h2>
          <div class="grid grid-cols-2 gap-4 text-sm">
            <div>
              <span class="text-gray-500 dark:text-gray-400">{t('certificateViewer.fileName')}:</span>
              <span class="ml-2 text-gray-900 dark:text-gray-100">{fileName}</span>
            </div>
            <div>
              <span class="text-gray-500 dark:text-gray-400">{t('certificateViewer.fileSize')}:</span>
              <span class="ml-2 text-gray-900 dark:text-gray-100">{(fileSize / 1024).toFixed(2)} KB</span>
            </div>
          </div>
        </div>

        <!-- Validity Status -->
        <div class="bg-gray-50 dark:bg-gray-800/50 rounded-lg p-4">
          <h2 class="text-sm font-semibold text-gray-700 dark:text-gray-300 mb-3">{t('certificateViewer.validity')}</h2>
          <div class="flex items-center gap-2 mb-3">
            {#if isExpired()}
              <XCircle class="w-5 h-5 text-red-500" />
              <span class="text-red-600 dark:text-red-400 font-medium">{t('certificateViewer.status.expired')}</span>
            {:else if isNotYetValid()}
              <AlertCircle class="w-5 h-5 text-yellow-500" />
              <span class="text-yellow-600 dark:text-yellow-400 font-medium">{t('certificateViewer.status.notYetValid')}</span>
            {:else}
              <CheckCircle2 class="w-5 h-5 text-green-500" />
              <span class="text-green-600 dark:text-green-400 font-medium">{t('certificateViewer.status.valid')}</span>
            {/if}
          </div>
          <div class="grid grid-cols-2 gap-4 text-sm">
            <div>
              <span class="text-gray-500 dark:text-gray-400">{t('certificateViewer.validFrom')}:</span>
              <span class="ml-2 text-gray-900 dark:text-gray-100">{formatDate(certificateInfo.validFrom || '')}</span>
            </div>
            <div>
              <span class="text-gray-500 dark:text-gray-400">{t('certificateViewer.validTo')}:</span>
              <span class="ml-2 text-gray-900 dark:text-gray-100">{formatDate(certificateInfo.validTo || '')}</span>
            </div>
          </div>
        </div>

        <!-- Subject Information -->
        <div class="bg-gray-50 dark:bg-gray-800/50 rounded-lg p-4">
          <h2 class="text-sm font-semibold text-gray-700 dark:text-gray-300 mb-3">{t('certificateViewer.subject')}</h2>
          <div class="space-y-2 text-sm">
            {#if certificateInfo.commonName}
              <div>
                <span class="text-gray-500 dark:text-gray-400">{t('certificateViewer.commonName')}:</span>
                <span class="ml-2 text-gray-900 dark:text-gray-100 font-mono">{certificateInfo.commonName}</span>
              </div>
            {/if}
            {#if certificateInfo.organization}
              <div>
                <span class="text-gray-500 dark:text-gray-400">{t('certificateViewer.organization')}:</span>
                <span class="ml-2 text-gray-900 dark:text-gray-100">{certificateInfo.organization}</span>
              </div>
            {/if}
            {#if certificateInfo.organizationalUnit}
              <div>
                <span class="text-gray-500 dark:text-gray-400">{t('certificateViewer.organizationalUnit')}:</span>
                <span class="ml-2 text-gray-900 dark:text-gray-100">{certificateInfo.organizationalUnit}</span>
              </div>
            {/if}
            {#if certificateInfo.email}
              <div>
                <span class="text-gray-500 dark:text-gray-400">{t('certificateViewer.email')}:</span>
                <span class="ml-2 text-gray-900 dark:text-gray-100">{certificateInfo.email}</span>
              </div>
            {/if}
            {#if certificateInfo.locality}
              <div>
                <span class="text-gray-500 dark:text-gray-400">{t('certificateViewer.locality')}:</span>
                <span class="ml-2 text-gray-900 dark:text-gray-100">{certificateInfo.locality}</span>
              </div>
            {/if}
            {#if certificateInfo.state}
              <div>
                <span class="text-gray-500 dark:text-gray-400">{t('certificateViewer.state')}:</span>
                <span class="ml-2 text-gray-900 dark:text-gray-100">{certificateInfo.state}</span>
              </div>
            {/if}
            {#if certificateInfo.country}
              <div>
                <span class="text-gray-500 dark:text-gray-400">{t('certificateViewer.country')}:</span>
                <span class="ml-2 text-gray-900 dark:text-gray-100">{certificateInfo.country}</span>
              </div>
            {/if}
            <div class="pt-2 border-t border-gray-200 dark:border-gray-700">
              <span class="text-gray-500 dark:text-gray-400">{t('certificateViewer.fullDN')}:</span>
              <div class="mt-1 text-gray-900 dark:text-gray-100 font-mono text-xs break-all">{certificateInfo.subject}</div>
            </div>
          </div>
        </div>

        <!-- Issuer Information -->
        <div class="bg-gray-50 dark:bg-gray-800/50 rounded-lg p-4">
          <h2 class="text-sm font-semibold text-gray-700 dark:text-gray-300 mb-3">{t('certificateViewer.issuer')}</h2>
          <div class="space-y-2 text-sm">
            {#if certificateInfo.issuerCommonName}
              <div>
                <span class="text-gray-500 dark:text-gray-400">{t('certificateViewer.commonName')}:</span>
                <span class="ml-2 text-gray-900 dark:text-gray-100 font-mono">{certificateInfo.issuerCommonName}</span>
              </div>
            {/if}
            {#if certificateInfo.issuerOrganization}
              <div>
                <span class="text-gray-500 dark:text-gray-400">{t('certificateViewer.organization')}:</span>
                <span class="ml-2 text-gray-900 dark:text-gray-100">{certificateInfo.issuerOrganization}</span>
              </div>
            {/if}
            <div class="pt-2 border-t border-gray-200 dark:border-gray-700">
              <span class="text-gray-500 dark:text-gray-400">{t('certificateViewer.fullDN')}:</span>
              <div class="mt-1 text-gray-900 dark:text-gray-100 font-mono text-xs break-all">{certificateInfo.issuer}</div>
            </div>
          </div>
        </div>

        <!-- Certificate Details -->
        <div class="bg-gray-50 dark:bg-gray-800/50 rounded-lg p-4">
          <h2 class="text-sm font-semibold text-gray-700 dark:text-gray-300 mb-3">{t('certificateViewer.details')}</h2>
          <div class="grid grid-cols-2 gap-4 text-sm">
            <div>
              <span class="text-gray-500 dark:text-gray-400">{t('certificateViewer.serialNumber')}:</span>
              <span class="ml-2 text-gray-900 dark:text-gray-100 font-mono break-all">{certificateInfo.serialNumber}</span>
            </div>
            <div>
              <span class="text-gray-500 dark:text-gray-400">{t('certificateViewer.version')}:</span>
              <span class="ml-2 text-gray-900 dark:text-gray-100">v{certificateInfo.version}</span>
            </div>
            <div>
              <span class="text-gray-500 dark:text-gray-400">{t('certificateViewer.algorithm')}:</span>
              <span class="ml-2 text-gray-900 dark:text-gray-100 font-mono">{certificateInfo.algorithm}</span>
            </div>
            {#if certificateInfo.keySize}
              <div>
                <span class="text-gray-500 dark:text-gray-400">{t('certificateViewer.keySize')}:</span>
                <span class="ml-2 text-gray-900 dark:text-gray-100">{certificateInfo.keySize} bits</span>
              </div>
            {/if}
            {#if certificateInfo.fingerprint}
              <div class="col-span-2">
                <span class="text-gray-500 dark:text-gray-400">{t('certificateViewer.fingerprint')}:</span>
                <div class="mt-1 text-gray-900 dark:text-gray-100 font-mono text-xs break-all">{certificateInfo.fingerprint}</div>
              </div>
            {/if}
          </div>
        </div>

        <!-- Extensions -->
        {#if certificateInfo.extensions && certificateInfo.extensions.length > 0}
          <div class="bg-gray-50 dark:bg-gray-800/50 rounded-lg p-4">
            <h2 class="text-sm font-semibold text-gray-700 dark:text-gray-300 mb-3">{t('certificateViewer.extensions')}</h2>
            <div class="space-y-2">
              {#each certificateInfo.extensions as ext}
                <div class="text-sm">
                  <div class="flex items-center gap-2">
                    <span class="text-gray-900 dark:text-gray-100 font-mono">{ext.name}</span>
                    {#if ext.critical}
                      <span class="px-2 py-0.5 bg-yellow-100 dark:bg-yellow-900/30 text-yellow-800 dark:text-yellow-200 text-xs rounded">
                        {t('certificateViewer.critical')}
                      </span>
                    {/if}
                  </div>
                  <div class="text-gray-600 dark:text-gray-400 font-mono text-xs mt-1 break-all">{ext.value}</div>
                </div>
              {/each}
            </div>
          </div>
        {/if}

        <!-- Raw Certificate -->
        <div class="bg-gray-50 dark:bg-gray-800/50 rounded-lg p-4">
          <h2 class="text-sm font-semibold text-gray-700 dark:text-gray-300 mb-3">{t('certificateViewer.raw')}</h2>
          <pre class="text-xs font-mono bg-white dark:bg-gray-900 p-3 rounded border border-gray-200 dark:border-gray-700 overflow-auto max-h-64">{certificateRaw}</pre>
        </div>
      </div>
    {/if}
  </div>
</div>
