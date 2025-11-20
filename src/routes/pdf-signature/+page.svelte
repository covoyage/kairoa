<script lang="ts">
  import { translationsStore } from '$lib/stores/i18n';
  import { browser } from '$app/environment';
  import { onMount } from 'svelte';
  import { PDFDocument } from 'pdf-lib';
  import { Upload, AlertTriangle, ShieldCheck, Loader2, FileWarning, XCircle } from 'lucide-svelte';
  import { fromBER } from 'asn1js';
  import { ContentInfo, SignedData, setEngine, CryptoEngine, Certificate } from 'pkijs';
  import { Convert } from 'pvtsutils';

  let translations = $derived($translationsStore);
  let pdfTips = $derived(() => {
    const value = translations?.pdfSignature?.tips;
    return Array.isArray(value) ? value : [];
  });

  function t(key: string): any {
    const keys = key.split('.');
    let value: any = translations;
    for (const k of keys) {
      value = value?.[k];
    }
    return value ?? key;
  }

  const MAX_FILE_SIZE = 20 * 1024 * 1024; // 20 MB

  type SignatureStatus = 'valid' | 'invalid' | 'unknown';

  interface CertificateInfo {
    subject?: string;
    issuer?: string;
    serialNumber?: string;
    validity?: string;
    email?: string | null;
  }

  interface SignatureResult {
    id: number;
    name?: string | null;
    reason?: string | null;
    location?: string | null;
    contactInfo?: string | null;
    signingTime?: string | null;
    byteRange?: number[] | null;
    subFilter?: string | null;
    filter?: string | null;
    contentsLength?: string | null;
    status: SignatureStatus;
    statusMessage: string;
    certificate?: CertificateInfo | null;
  }

  let fileInput: HTMLInputElement | null = null;
  let isDragging = $state(false);
  let isLoading = $state(false);
  let errorMessage = $state('');
  let pdfFileName = $state('');
  let pdfSize = $state(0);
  let pageCount = $state<number | null>(null);
  let signatures = $state<SignatureResult[]>([]);
  let cryptoSupported = $state(false);
  let isTauriApp = $state(false);
  let tauriDialogModule: typeof import('@tauri-apps/plugin-dialog') | null = null;
  let tauriFsModule: typeof import('@tauri-apps/plugin-fs') | null = null;
  const supportsFilePicker = browser && typeof window !== 'undefined' && 'showOpenFilePicker' in window;
  let lastCheckedAt = $state<Date | null>(null);

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
      
      // 监听 Tauri 文件拖放事件
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
      const fileName = path.split('/').pop() || path.split('\\').pop() || 'file.pdf';
      const blob = new Blob([contents], { type: 'application/pdf' });
      const file = new File([blob], fileName, { type: 'application/pdf' });
      await loadPdfFile(file);
    } catch (err) {
      console.error('Failed to read dropped file:', err);
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

  function isPdfFile(file: File | null | undefined): file is File {
    if (!file) return false;
    const nameMatch = file.name?.toLowerCase().endsWith('.pdf');
    const typeMatch = file.type?.toLowerCase() === 'application/pdf';
    return Boolean(nameMatch || typeMatch);
  }

  async function handleFiles(files: FileList | File[] | null) {
    if (!files || files.length === 0) return;
    const list = Array.from(files);
    const file = list.find(isPdfFile);
    if (!file) {
      errorMessage = t('pdfSignature.errors.invalidPdf');
      return;
    }
    if (file.size > MAX_FILE_SIZE) {
      errorMessage = t('pdfSignature.errors.fileTooLarge');
      return;
    }
    const buffer = await file.arrayBuffer();
    await analyzePdf(buffer, file);
  }

  async function ensureTauriModules() {
    if (!tauriDialogModule) {
      tauriDialogModule = await import('@tauri-apps/plugin-dialog');
    }
    if (!tauriFsModule) {
      tauriFsModule = await import('@tauri-apps/plugin-fs');
    }
  }

  function getFileNameFromPath(path: string) {
    const segments = path.split(/[/\\]/);
    const name = segments[segments.length - 1];
    return name && name.trim().length > 0 ? name : 'document.pdf';
  }

  async function openFilePicker() {
    if (isTauriApp) {
      try {
        await ensureTauriModules();
        const { open } = tauriDialogModule ?? {};
        const { readFile } = tauriFsModule ?? {};
        if (typeof open !== 'function' || typeof readFile !== 'function') {
          throw new Error('Failed to load Tauri file picker dependencies');
        }

        const result = await open({
          multiple: false,
          filters: [{ name: 'PDF', extensions: ['pdf'] }]
        });

        if (!result) return;

        const filePath = Array.isArray(result) ? result[0] : result;
        if (!filePath) return;

        const contents = await readFile(filePath);
        const fileName = getFileNameFromPath(filePath);
        const blob = new Blob([contents], { type: 'application/pdf' });
        const file = new File([blob], fileName, { type: 'application/pdf' });
        await handleFiles([file]);
        return;
      } catch (err) {
        console.error('Tauri file picker failed, falling back to browser picker:', err);
      }
    }

    if (supportsFilePicker && typeof window.showOpenFilePicker === 'function') {
      try {
        const [handle] = await window.showOpenFilePicker({
          multiple: false,
          excludeAcceptAllOption: true,
          types: [
            {
              description: 'PDF',
              accept: {
                'application/pdf': ['.pdf']
              }
            }
          ]
        });
        if (handle) {
          const file = await handle.getFile();
          await handleFiles([file]);
        }
      } catch (err) {
        if ((err as DOMException)?.name === 'AbortError') {
          return;
        }
        console.error('File picker failed:', err);
      }
    } else {
      fileInput?.click();
    }
  }

  function clearFile() {
    pdfFileName = '';
    pdfSize = 0;
    pageCount = null;
    signatures = [];
    errorMessage = '';
    lastCheckedAt = null;
  }

  async function analyzePdf(buffer: ArrayBuffer, file: File) {
    try {
      isLoading = true;
      errorMessage = '';
      pdfFileName = file.name;
      pdfSize = file.size;
      const pdfBytes = new Uint8Array(buffer);
      const pdfDoc = await PDFDocument.load(buffer, { ignoreEncryption: true });
      pageCount = pdfDoc.getPageCount();
      signatures = await extractSignatures(pdfBytes);
      lastCheckedAt = new Date();
    } catch (err) {
      console.error(err);
      errorMessage = `${t('pdfSignature.errors.parseFailed')}: ${err instanceof Error ? err.message : err}`;
      signatures = [];
    } finally {
      isLoading = false;
    }
  }

  async function extractSignatures(pdfBytes: Uint8Array): Promise<SignatureResult[]> {
    const pdfText = decodePdf(pdfBytes);
    const objectRegex = /(\d+)\s+(\d+)\s+obj([\s\S]*?)endobj/g;
    const objectMap = new Map<string, string>();
    let objectMatch: RegExpExecArray | null;

    while ((objectMatch = objectRegex.exec(pdfText)) !== null) {
      const key = `${objectMatch[1]} ${objectMatch[2]}`;
      objectMap.set(key, objectMatch[3]);
    }

    const results: SignatureResult[] = [];
    let id = 1;

    for (const content of objectMap.values()) {
      if (!/\/Type\s*\/Sig/.test(content)) continue;

      const byteRange = parseByteRange(content);
      const contentsBytes = extractContents(content, objectMap);
      const signature: SignatureResult = {
        id,
        name: getPdfString(content, 'Name'),
        reason: getPdfString(content, 'Reason'),
        location: getPdfString(content, 'Location'),
        contactInfo: getPdfString(content, 'ContactInfo'),
        signingTime: formatDateTime(parsePdfDate(getPdfString(content, 'M'))),
        byteRange,
        subFilter: getNameValue(content, 'SubFilter'),
        filter: getNameValue(content, 'Filter'),
        contentsLength: contentsBytes ? formatBytes(contentsBytes.length) : null,
        status: 'unknown',
        statusMessage: t('pdfSignature.status.pending'),
        certificate: null
      };

      if (!byteRange || !contentsBytes) {
        signature.statusMessage = t('pdfSignature.status.missingContents');
        results.push(signature);
        id++;
        continue;
      }

      if (cryptoSupported) {
        const verification = await verifySignature(contentsBytes, byteRange, pdfBytes);
        signature.status = verification.status;
        signature.statusMessage = verification.message;
        signature.certificate = verification.certificate ?? null;
      } else {
        signature.statusMessage = t('pdfSignature.status.unavailable');
      }

      results.push(signature);
      id++;
    }

    return results;
  }

  async function verifySignature(signatureBytes: Uint8Array, byteRange: number[], pdfBytes: Uint8Array) {
    try {
      const signedPortion = collectSignedPortion(pdfBytes, byteRange);
      const asn1 = fromBER(signatureBytes.buffer);
      if (asn1.offset === -1) {
        throw new Error('Malformed signature');
      }
      const contentInfo = new ContentInfo({ schema: asn1.result });
      const signedData = new SignedData({ schema: contentInfo.content });
      const verified = await signedData.verify({
        signer: 0,
        data: signedPortion.buffer
      });
      let parsedCertificate: Certificate | null = null;
      if (signedData.certificates) {
        for (const cert of signedData.certificates) {
          if (cert instanceof Certificate) {
            parsedCertificate = cert;
            break;
          }
        }
      }
      const certificate = parsedCertificate ? extractCertificateInfo(parsedCertificate) : null;

      return {
        status: verified ? ('valid' as SignatureStatus) : ('invalid' as SignatureStatus),
        message: verified ? t('pdfSignature.status.valid') : t('pdfSignature.status.invalid'),
        certificate
      };
    } catch (err) {
      return {
        status: 'unknown' as SignatureStatus,
        message: `${t('pdfSignature.status.error')}: ${err instanceof Error ? err.message : 'Unknown error'}`
      };
    }
  }

  function collectSignedPortion(pdfBytes: Uint8Array, byteRange: number[]) {
    const [start1, length1, start2, length2] = byteRange;
    const totalLength = length1 + length2;
    const buffer = new Uint8Array(totalLength);
    buffer.set(pdfBytes.slice(start1, start1 + length1), 0);
    buffer.set(pdfBytes.slice(start2, start2 + length2), length1);
    return buffer;
  }

  function parseByteRange(content: string): number[] | null {
    const match = content.match(/\/ByteRange\s*\[\s*([^\]]+)\]/);
    if (!match) return null;
    const parts = match[1]
      .trim()
      .split(/\s+/)
      .map((value) => Number.parseInt(value, 10))
      .filter((value) => Number.isFinite(value));
    return parts.length === 4 ? parts : null;
  }

  function extractContents(content: string, objectMap: Map<string, string>): Uint8Array | null {
    const directMatch = content.match(/\/Contents\s*<([\s\S]*?)>/);
    if (directMatch) {
      return hexToBytes(directMatch[1]);
    }

    const refMatch = content.match(/\/Contents\s+(\d+)\s+(\d+)\s+R/);
    if (refMatch) {
      const key = `${refMatch[1]} ${refMatch[2]}`;
      const referenced = objectMap.get(key);
      if (referenced) {
        const hexMatch = referenced.match(/<([\s\S]*?)>/);
        if (hexMatch) {
          return hexToBytes(hexMatch[1]);
        }
        const streamMatch = referenced.match(/stream([\s\S]*?)endstream/);
        if (streamMatch) {
          const sanitized = streamMatch[1].trim();
          return hexToBytes(sanitized);
        }
      }
    }
    return null;
  }

  function getPdfString(content: string, key: string): string | null {
    const literal = new RegExp(`/${key}\\s*\\(([^)]*)\\)`).exec(content);
    if (literal) {
      return decodePdfString(literal[1]);
    }
    const hex = new RegExp(`/${key}\\s*<([0-9A-Fa-f]+)>`).exec(content);
    if (hex) {
      return new TextDecoder().decode(hexToBytes(hex[1]));
    }
    return null;
  }

  function getNameValue(content: string, key: string): string | null {
    const match = new RegExp(`/${key}\\s*/([^\\s/>]+)`).exec(content);
    return match ? match[1] : null;
  }

  function decodePdfString(value: string): string {
    return value
      .replace(/\\\(/g, '(')
      .replace(/\\\)/g, ')')
      .replace(/\\\\/g, '\\')
      .replace(/\\n/g, '\n')
      .replace(/\\r/g, '\r')
      .replace(/\\t/g, '\t');
  }

  function hexToBytes(value: string): Uint8Array {
    const sanitized = value.replace(/[^0-9A-Fa-f]/g, '');
    const bytes = new Uint8Array(sanitized.length / 2);
    for (let i = 0; i < sanitized.length; i += 2) {
      bytes[i / 2] = Number.parseInt(sanitized.slice(i, i + 2), 16);
    }
    return bytes;
  }

  function decodePdf(pdfBytes: Uint8Array): string {
    try {
      return new TextDecoder('windows-1252', { fatal: false }).decode(pdfBytes);
    } catch {
      return new TextDecoder().decode(pdfBytes);
    }
  }

  function parsePdfDate(raw: string | null): Date | null {
    if (!raw) return null;
    const clean = raw.replace(/^D:/, '');
    const pad = (value: string | undefined, fallback: string) => (value && value.length === 2 ? value : fallback);
    const year = clean.slice(0, 4);
    if (!year) return null;
    const month = pad(clean.slice(4, 6), '01');
    const day = pad(clean.slice(6, 8), '01');
    const hour = pad(clean.slice(8, 10), '00');
    const minute = pad(clean.slice(10, 12), '00');
    const second = pad(clean.slice(12, 14), '00');

    let timezone = 'Z';
    const tzMatch = clean.match(/([+-Z])(\d{2})'?(\d{2})'?/);
    if (tzMatch) {
      if (tzMatch[1] === 'Z') {
        timezone = 'Z';
      } else {
        timezone = `${tzMatch[1]}${tzMatch[2] ?? '00'}:${tzMatch[3] ?? '00'}`;
      }
    }

    const iso = `${year}-${month}-${day}T${hour}:${minute}:${second}${timezone}`;
    const date = new Date(iso);
    return Number.isNaN(date.getTime()) ? null : date;
  }

  function formatDateTime(date: Date | null): string | null {
    if (!date) return null;
    try {
      return new Intl.DateTimeFormat(undefined, {
        dateStyle: 'medium',
        timeStyle: 'short'
      }).format(date);
    } catch {
      return date.toISOString();
    }
  }

  function extractCertificateInfo(certificate: Certificate): CertificateInfo {
    const subject = formatDistinguishedName(certificate.subject.typesAndValues);
    const issuer = formatDistinguishedName(certificate.issuer.typesAndValues);
    const serialNumber = Convert.ToHex(new Uint8Array(certificate.serialNumber.valueBlock.valueHex));
    const notBefore = certificate.notBefore.value instanceof Date ? certificate.notBefore.value : null;
    const notAfter = certificate.notAfter.value instanceof Date ? certificate.notAfter.value : null;
    const validity =
      notBefore && notAfter
        ? `${formatDateTime(notBefore)} → ${formatDateTime(notAfter)}`
        : null;

    const emailAttr = certificate.subject.typesAndValues.find(
      (attr) => attr.type === '1.2.840.113549.1.9.1'
    );
    const email = emailAttr ? String(emailAttr.value.valueBlock.value) : null;

    return {
      subject: subject || undefined,
      issuer: issuer || undefined,
      serialNumber: serialNumber || undefined,
      validity: validity || undefined,
      email: email || undefined
    };
  }

  function formatDistinguishedName(values: any[]): string {
    const oidMap: Record<string, string> = {
      '2.5.4.6': 'C',
      '2.5.4.10': 'O',
      '2.5.4.11': 'OU',
      '2.5.4.3': 'CN',
      '2.5.4.7': 'L',
      '2.5.4.8': 'ST',
      '1.2.840.113549.1.9.1': 'E'
    };

    return values
      .map((attr) => {
        const label = oidMap[attr.type] ?? attr.type;
        const value = attr.value.valueBlock.value;
        return `${label}=${value}`;
      })
      .join(', ');
  }

  function formatBytes(bytes: number | null | undefined): string | null {
    if (bytes === null || bytes === undefined) return null;
    if (bytes < 1024) return `${bytes} B`;
    const units = ['KB', 'MB', 'GB'];
    let value = bytes / 1024;
    let unitIndex = 0;
    while (value >= 1024 && unitIndex < units.length - 1) {
      value /= 1024;
      unitIndex++;
    }
    return `${value.toFixed(1)} ${units[unitIndex]}`;
  }

  function formatSignatureCount(count: number) {
    return t('pdfSignature.signatureCount').replace('{count}', String(count));
  }

  function signatureTitle(id: number) {
    return t('pdfSignature.signatureCardTitle').replace('{index}', String(id));
  }

  function statusClass(status: SignatureStatus) {
    if (status === 'valid') {
      return 'bg-green-100 text-green-800 dark:bg-green-900/30 dark:text-green-200';
    }
    if (status === 'invalid') {
      return 'bg-red-100 text-red-800 dark:bg-red-900/30 dark:text-red-200';
    }
    return 'bg-amber-100 text-amber-800 dark:bg-amber-900/30 dark:text-amber-200';
  }

  function displayValue(value?: string | null) {
    return value ?? translations.pdfSignature?.notProvided ?? '—';
  }
</script>

<div class="flex flex-col h-full w-full ml-0 mr-0 p-2 space-y-4 overflow-y-auto">
  <div class="card">
    <div class="flex flex-col gap-4">
      <div>
        <h1 class="text-2xl font-semibold text-gray-900 dark:text-gray-100">{t('pdfSignature.title')}</h1>
        <p class="text-sm text-gray-600 dark:text-gray-400 mt-1">{t('pdfSignature.description')}</p>
      </div>
      <div
        class="border-2 border-dashed rounded-xl p-8 text-center transition-colors cursor-pointer {isDragging ? 'border-primary-500 bg-primary-50 dark:bg-primary-900/10' : 'border-gray-300 dark:border-gray-600'}"
        on:dragover|preventDefault={handleDragOver}
        on:dragleave|preventDefault={handleDragLeave}
        on:drop={handleDrop}
        on:click={openFilePicker}
      >
        <Upload class="w-10 h-10 mx-auto text-gray-400 dark:text-gray-500 mb-4" />
        <p class="text-lg font-medium text-gray-900 dark:text-gray-100">{t('pdfSignature.dropTitle')}</p>
        <p class="text-sm text-gray-600 dark:text-gray-400 mt-1">{t('pdfSignature.dropSubtitle')}</p>
        <p class="text-xs text-gray-500 dark:text-gray-500 mt-2">{t('pdfSignature.helperText')}</p>
        <div class="mt-4 flex flex-wrap items-center justify-center gap-3">
          <button type="button" class="btn-primary" on:click|stopPropagation={openFilePicker}>
            {t('pdfSignature.selectFile')}
          </button>
          {#if pdfFileName}
            <button type="button" class="btn-secondary" on:click|stopPropagation={clearFile}>
              {t('pdfSignature.clearFile')}
            </button>
          {/if}
        </div>
        <input
          bind:this={fileInput}
          type="file"
          accept=".pdf,application/pdf"
          class="hidden"
          on:change={onFileChange}
        />
      </div>
      <p class="text-xs text-gray-500 dark:text-gray-400 text-center">{t('pdfSignature.dropHint')}</p>
    </div>
  </div>

  {#if errorMessage}
    <div class="p-4 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg flex items-start gap-3">
      <XCircle class="w-5 h-5 text-red-600 dark:text-red-300 mt-0.5" />
      <div>
        <p class="text-sm text-red-800 dark:text-red-200">{errorMessage}</p>
      </div>
    </div>
  {/if}

  {#if pdfFileName}
    <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
      <div class="card space-y-2">
        <h2 class="text-lg font-semibold text-gray-900 dark:text-gray-100">{t('pdfSignature.fileInfoTitle')}</h2>
        <div class="text-sm text-gray-700 dark:text-gray-300 space-y-1.5">
          <p><span class="font-medium">{t('pdfSignature.fileName')}:</span> {pdfFileName}</p>
          <p><span class="font-medium">{t('pdfSignature.fileSize')}:</span> {formatBytes(pdfSize)}</p>
          {#if pageCount !== null}
            <p><span class="font-medium">{t('pdfSignature.pageCount')}:</span> {pageCount}</p>
          {/if}
          {#if lastCheckedAt}
            <p><span class="font-medium">{t('pdfSignature.checkedAt')}:</span> {formatDateTime(lastCheckedAt)}</p>
          {/if}
        </div>
      </div>

      <div class="card space-y-3">
        <div class="flex items-center justify-between">
          <h2 class="text-lg font-semibold text-gray-900 dark:text-gray-100">{t('pdfSignature.summaryTitle')}</h2>
          <ShieldCheck class="w-5 h-5 text-primary-500" />
        </div>
        {#if isLoading}
          <div class="flex items-center gap-2 text-sm text-gray-600 dark:text-gray-400">
            <Loader2 class="w-4 h-4 animate-spin" />
            {t('pdfSignature.verifying')}
          </div>
        {:else if signatures.length > 0}
          <p class="text-sm text-gray-700 dark:text-gray-300">
            {formatSignatureCount(signatures.length)}
          </p>
        {/if}
        {#if !cryptoSupported}
          <div class="p-3 bg-amber-50 dark:bg-amber-900/20 border border-amber-200 dark:border-amber-800 rounded-lg flex items-start gap-2">
            <AlertTriangle class="w-4 h-4 text-amber-600 dark:text-amber-300 mt-0.5" />
            <div class="text-xs text-amber-800 dark:text-amber-100">
              <p class="font-semibold">{t('pdfSignature.cryptoUnsupportedTitle')}</p>
              <p>{t('pdfSignature.cryptoUnsupportedDesc')}</p>
            </div>
          </div>
        {/if}
      </div>
    </div>
  {/if}

  {#if signatures.length > 0}
    <div class="space-y-4">
      {#each signatures as sig}
        <div class="card space-y-4">
          <div class="flex flex-wrap items-center justify-between gap-3">
            <div>
              <h3 class="text-lg font-semibold text-gray-900 dark:text-gray-100">{signatureTitle(sig.id)}</h3>
              {#if sig.signingTime}
                <p class="text-sm text-gray-600 dark:text-gray-400">{t('pdfSignature.signingTime')}: {sig.signingTime}</p>
              {/if}
            </div>
            <span class="px-3 py-1 text-xs font-semibold rounded-full {statusClass(sig.status)}">
              {t('pdfSignature.statusLabel')}: {sig.statusMessage}
            </span>
          </div>

          <div class="grid grid-cols-1 md:grid-cols-2 gap-3 text-sm text-gray-700 dark:text-gray-300">
            <p><span class="font-medium">{t('pdfSignature.signer')}:</span> {displayValue(sig.name)}</p>
            <p><span class="font-medium">{t('pdfSignature.reason')}:</span> {displayValue(sig.reason)}</p>
            <p><span class="font-medium">{t('pdfSignature.location')}:</span> {displayValue(sig.location)}</p>
            <p><span class="font-medium">{t('pdfSignature.contactInfo')}:</span> {displayValue(sig.contactInfo)}</p>
            <p><span class="font-medium">{t('pdfSignature.filter')}:</span> {displayValue(sig.filter)}</p>
            <p><span class="font-medium">{t('pdfSignature.subFilter')}:</span> {displayValue(sig.subFilter)}</p>
            <p><span class="font-medium">{t('pdfSignature.byteRange')}:</span> {displayValue(sig.byteRange ? sig.byteRange.join(', ') : null)}</p>
            <p><span class="font-medium">{t('pdfSignature.contentsLength')}:</span> {displayValue(sig.contentsLength)}</p>
          </div>

          {#if sig.certificate}
            <div class="border-t border-dashed border-gray-200 dark:border-gray-700 pt-4">
              <h4 class="text-sm font-semibold text-gray-900 dark:text-gray-100 mb-2 flex items-center gap-2">
                <ShieldCheck class="w-4 h-4 text-primary-500" />
                {t('pdfSignature.certificateTitle')}
              </h4>
              <div class="grid grid-cols-1 md:grid-cols-2 gap-3 text-sm text-gray-700 dark:text-gray-300">
                <p><span class="font-medium">{t('pdfSignature.certificateSubject')}:</span> {displayValue(sig.certificate.subject)}</p>
                <p><span class="font-medium">{t('pdfSignature.certificateIssuer')}:</span> {displayValue(sig.certificate.issuer)}</p>
                <p><span class="font-medium">{t('pdfSignature.certificateSerial')}:</span> {displayValue(sig.certificate.serialNumber)}</p>
                <p><span class="font-medium">{t('pdfSignature.certificateValidity')}:</span> {displayValue(sig.certificate.validity)}</p>
                <p><span class="font-medium">{t('pdfSignature.certificateEmail')}:</span> {displayValue(sig.certificate.email)}</p>
              </div>
            </div>
          {/if}
        </div>
      {/each}
    </div>
  {/if}
</div>
