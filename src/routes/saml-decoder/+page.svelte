<script lang="ts">
  import { translationsStore } from '$lib/stores/i18n';
  import { fromBER } from 'asn1js';
  import { Certificate, setEngine, CryptoEngine } from 'pkijs';
  import { Copy, Check, Trash2, FileCode2, ShieldCheck, User, Clock, Building2, BadgeCheck, AlertTriangle, XCircle } from 'lucide-svelte';
  import { browser } from '$app/environment';
  import { onMount } from 'svelte';

  let translations = $derived($translationsStore);

  function t(key: string): string {
    const keys = key.split('.');
    let value: any = translations;
    for (const k of keys) {
      value = value?.[k];
    }
    return value || key;
  }

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
    }
  });

  let input = $state('');
  let error = $state('');
  let decoded = $state(false);
  let decoding = $state(false);

  // ── Parsed fields ──
  let rawXml = $state('');
  let formattedXml = $state('');
  let messageType = $state('');
  let issuer = $state('');
  let destination = $state('');
  let id = $state('');
  let issueInstant = $state('');
  let version = $state('');

  // Subject
  let subjectNameId = $state('');
  let subjectNameIdFormat = $state('');

  // Conditions
  let conditionsNotBefore = $state('');
  let conditionsNotOnOrAfter = $state('');
  let audience = $state('');

  // Authn statement
  let authnInstant = $state('');
  let sessionIndex = $state('');
  let authnContextClassRef = $state('');

  // Attributes
  let attributes: { name: string; value: string }[] = $state([]);

  // ── Signature & validation ──
  let hasSignature = $state(false);
  let signatureValue = $state('');
  let signatureAlgorithm = $state('');
  let certSubject = $state('');
  let certIssuer = $state('');
  let certSerial = $state('');
  let certValidFrom = $state('');
  let certValidTo = $state('');
  let signatureVerified = $state<'verified' | 'failed' | 'not-checked' | 'error'>('not-checked');
  let signatureError = $state('');

  // Time validity
  let timeValid = $state<'valid' | 'expired' | 'not-yet-valid' | 'unknown'>('unknown');
  let timeStatusDetail = $state('');

  let copiedXml = $state(false);

  // ── Base64 decode ──
  function base64Decode(str: string): Uint8Array {
    let s = str.replace(/-/g, '+').replace(/_/g, '/').replace(/\s/g, '');
    while (s.length % 4) s += '=';
    const binary = atob(s);
    const bytes = new Uint8Array(binary.length);
    for (let i = 0; i < binary.length; i++) bytes[i] = binary.charCodeAt(i);
    return bytes;
  }

  // ── Inflate (DEFLATE-raw) ──
  async function inflateRawAsync(data: Uint8Array): Promise<string> {
    if (typeof DecompressionStream !== 'undefined') {
      const ds = new DecompressionStream('deflate-raw');
      const writer = ds.writable.getWriter();
      writer.write(data);
      writer.close();
      const reader = ds.readable.getReader();
      const chunks: Uint8Array[] = [];
      let totalLen = 0;
      // eslint-disable-next-line no-constant-condition
      while (true) {
        const { done, value } = await reader.read();
        if (done) break;
        chunks.push(value);
        totalLen += value.length;
      }
      const result = new Uint8Array(totalLen);
      let offset = 0;
      for (const c of chunks) { result.set(c, offset); offset += c.length; }
      return new TextDecoder().decode(result);
    }
    throw new Error(t('samlDecoder.errorDecompress'));
  }

  // ── XML helpers ──
  function getElementText(xml: string, tagName: string): string {
    const regex = new RegExp(`<(?:[\\w]+:)?${tagName}[^>]*>([^<]*)<\\/(?:[\\w]+:)?${tagName}>`, 'i');
    const match = xml.match(regex);
    return match ? match[1].trim() : '';
  }

  function getAttributeValue(xml: string, tagName: string, attr: string): string {
    const regex = new RegExp(`<(?:[\\w]+:)?${tagName}[^>]*\\s${attr}=["']([^"']*)["']`, 'i');
    const match = xml.match(regex);
    return match ? match[1] : '';
  }

  // ── Format XML ──
  function formatXml(xml: string): string {
    let formatted = '';
    let indent = 0;
    const pad = '  ';
    const reg = /(<\/?[\w:.-]+[^>]*>|<!--[\s\S]*?-->|<[^>]+\/?>)/g;
    let last = 0;
    let match;
    while ((match = reg.exec(xml)) !== null) {
      const text = xml.slice(last, match.index).trim();
      if (text) formatted += pad.repeat(indent) + text + '\n';
      const tag = match[0];
      if (tag.startsWith('</')) {
        indent = Math.max(0, indent - 1);
        formatted = formatted.trimEnd() + tag + '\n';
      } else if (tag.endsWith('/>') || tag.startsWith('<?') || tag.startsWith('<!--')) {
        formatted += pad.repeat(indent) + tag + '\n';
      } else {
        formatted += pad.repeat(indent) + tag + '\n';
        indent++;
      }
      last = match.index + tag.length;
    }
    return formatted || xml;
  }

  // ── Extract SAML attributes ──
  function extractAttributes(xml: string): { name: string; value: string }[] {
    const attrs: { name: string; value: string }[] = [];
    const attrRegex = /<(?:[\w]+:)?Attribute\s+Name=["']([^"']*)["'][^>]*>([\s\S]*?)<\/(?:[\w]+:)?Attribute>/gi;
    let match;
    while ((match = attrRegex.exec(xml)) !== null) {
      const name = match[1];
      const valueRegex = /<(?:[\w]+:)?AttributeValue[^>]*>([\s\S]*?)<\/(?:[\w]+:)?AttributeValue>/gi;
      let valMatch;
      const values: string[] = [];
      while ((valMatch = valueRegex.exec(match[2])) !== null) {
        const v = valMatch[1].trim();
        if (v) values.push(v);
      }
      attrs.push({ name, value: values.join(', ') });
    }
    return attrs;
  }

  // ── Extract signature info ──
  function extractSignatureInfo(xml: string) {
    // Check for Signature element
    const sigRegex = /<(?:[\w]+:)?Signature[\s>]/i;
    hasSignature = sigRegex.test(xml);

    if (!hasSignature) return;

    // Extract SignatureValue
    signatureValue = getElementText(xml, 'SignatureValue');

    // Extract SignatureMethod Algorithm
    const sigMethodMatch = xml.match(/<(?:[\w]+:)?SignatureMethod\s+Algorithm=["']([^"']*)["']/i);
    signatureAlgorithm = sigMethodMatch ? sigMethodMatch[1] : '';

    // Extract X509Certificate
    const certB64 = getElementText(xml, 'X509Certificate');
    if (certB64) {
      try {
        const certBytes = base64Decode(certB64);
        const asn1 = fromBER(certBytes.buffer);
        if (asn1.offset !== -1) {
          const cert = new Certificate({ schema: asn1.result });
          certSubject = cert.subject.toString();
          certIssuer = cert.issuer.toString();
          certSerial = (cert.serialNumber as any).value?.toString() || cert.serialNumber.toString();
          certValidFrom = cert.notBefore.value instanceof Date
            ? cert.notBefore.value.toISOString()
            : new Date(cert.notBefore.value as any).toISOString();
          certValidTo = cert.notAfter.value instanceof Date
            ? cert.notAfter.value.toISOString()
            : new Date(cert.notAfter.value as any).toISOString();
        }
      } catch {
        // ignore certificate parse errors
      }
    }
  }

  // ── Check time validity ──
  function checkTimeValidity() {
    const now = new Date();

    if (conditionsNotBefore && conditionsNotOnOrAfter) {
      const notBefore = new Date(conditionsNotBefore);
      const notOnOrAfter = new Date(conditionsNotOnOrAfter);

      if (now < notBefore) {
        timeValid = 'not-yet-valid';
        timeStatusDetail = t('samlDecoder.notYetValidDetail');
      } else if (now >= notOnOrAfter) {
        timeValid = 'expired';
        timeStatusDetail = t('samlDecoder.expiredDetail');
      } else {
        timeValid = 'valid';
        timeStatusDetail = t('samlDecoder.timeValidDetail');
      }
    } else if (conditionsNotOnOrAfter) {
      const notOnOrAfter = new Date(conditionsNotOnOrAfter);
      if (now >= notOnOrAfter) {
        timeValid = 'expired';
        timeStatusDetail = t('samlDecoder.expiredDetail');
      } else {
        timeValid = 'valid';
        timeStatusDetail = t('samlDecoder.timeValidDetail');
      }
    } else if (conditionsNotBefore) {
      const notBefore = new Date(conditionsNotBefore);
      if (now < notBefore) {
        timeValid = 'not-yet-valid';
        timeStatusDetail = t('samlDecoder.notYetValidDetail');
      } else {
        timeValid = 'valid';
        timeStatusDetail = t('samlDecoder.timeValidDetail');
      }
    } else {
      timeValid = 'unknown';
      timeStatusDetail = t('samlDecoder.noConditions');
    }
  }

  // ── Verify signature using pkijs ──
  async function verifySignature() {
    if (!hasSignature || !rawXml || !browser || !window.crypto?.subtle) {
      signatureVerified = 'not-checked';
      return;
    }

    try {
      // Use DOMParser for proper XML parsing
      const parser = new DOMParser();
      const doc = parser.parseFromString(rawXml, 'application/xml');

      // Find the Signature element (with or without namespace prefix)
      let sigElement = doc.getElementsByTagName('Signature')[0]
        || doc.getElementsByTagName('ds:Signature')[0]
        || doc.getElementsByTagName('ns0:Signature')[0];

      // Also try by local name
      if (!sigElement) {
        const allElements = doc.getElementsByTagName('*');
        for (let i = 0; i < allElements.length; i++) {
          if (allElements[i].localName === 'Signature') {
            sigElement = allElements[i];
            break;
          }
        }
      }

      if (!sigElement) {
        signatureVerified = 'error';
        signatureError = t('samlDecoder.sigNotFound');
        return;
      }

      // Extract the signed content (the parent element minus the Signature child)
      // For SAML, the signature usually wraps the Response or Assertion
      const signedElement = sigElement.parentNode;
      if (!signedElement) {
        signatureVerified = 'error';
        signatureError = t('samlDecoder.sigNoParent');
        return;
      }

      // Reconstruct the signed XML by removing the Signature element
      const signedClone = signedElement.cloneNode(true) as Element;
      const sigsToRemove = signedClone.getElementsByTagName('Signature');
      for (let i = sigsToRemove.length - 1; i >= 0; i--) {
        const sig = sigsToRemove[i];
        if (sig.parentNode) {
          // Remove all namespace declaration attributes that might have been on the original
          sig.parentNode.removeChild(sig);
        }
      }
      // Also handle prefixed signatures
      const prefixedSigs = signedClone.getElementsByTagName('*');
      for (let i = prefixedSigs.length - 1; i >= 0; i--) {
        if (prefixedSigs[i].localName === 'Signature' && prefixedSigs[i].parentNode) {
          prefixedSigs[i].parentNode!.removeChild(prefixedSigs[i] as Node);
        }
      }

      // Canonicalize the signed content (simple C14N - serialize XML)
      // Note: This is a simplified canonicalization. True XML C14N is complex.
      const serializer = new XMLSerializer();
      const canonicalXml = serializer.serializeToString(signedClone);

      // Extract signature algorithm and value
      const sigValueEl = sigElement.getElementsByTagName('SignatureValue')[0]
        || Array.from(sigElement.getElementsByTagName('*')).find(e => e.localName === 'SignatureValue');
      const sigValueB64 = sigValueEl?.textContent?.trim() || '';

      if (!sigValueB64) {
        signatureVerified = 'error';
        signatureError = t('samlDecoder.sigNoValue');
        return;
      }

      // Extract digest algorithm from Reference
      const digestMethodEl = sigElement.getElementsByTagName('DigestMethod')[0]
        || Array.from(sigElement.getElementsByTagName('*')).find(e => e.localName === 'DigestMethod');
      const digestAlg = digestMethodEl?.getAttribute('Algorithm') || '';

      // Extract SignatureMethod
      const sigMethodEl = sigElement.getElementsByTagName('SignatureMethod')[0]
        || Array.from(sigElement.getElementsByTagName('*')).find(e => e.localName === 'SignatureMethod');
      const sigAlg = sigMethodEl?.getAttribute('Algorithm') || '';

      // Compute digest of signed content
      const encoder = new TextEncoder();
      const signedData = encoder.encode(canonicalXml);
      let digestAlgo: string = 'SHA-256';
      if (digestAlg.includes('sha1')) digestAlgo = 'SHA-1';
      else if (digestAlg.includes('sha256')) digestAlgo = 'SHA-256';
      else if (digestAlg.includes('sha512')) digestAlgo = 'SHA-512';

      const computedDigest = await crypto.subtle.digest(digestAlgo, signedData);

      // Extract the digest from DigestValue
      const digestValueEl = sigElement.getElementsByTagName('DigestValue')[0]
        || Array.from(sigElement.getElementsByTagName('*')).find(e => e.localName === 'DigestValue');
      const expectedDigestB64 = digestValueEl?.textContent?.trim() || '';

      if (!expectedDigestB64) {
        signatureVerified = 'error';
        signatureError = t('samlDecoder.sigNoDigest');
        return;
      }

      const expectedDigest = base64Decode(expectedDigestB64);

      // Compare digests
      let digestMatch = true;
      const computedDigestArr = new Uint8Array(computedDigest);
      if (computedDigestArr.length !== expectedDigest.length) {
        digestMatch = false;
      } else {
        for (let i = 0; i < computedDigestArr.length; i++) {
          if (computedDigestArr[i] !== expectedDigest[i]) {
            digestMatch = false;
            break;
          }
        }
      }

      if (!digestMatch) {
        signatureVerified = 'failed';
        signatureError = t('samlDecoder.digestMismatch');
        return;
      }

      // Extract X509Certificate and verify the actual signature
      const certB64 = getElementText(rawXml, 'X509Certificate');
      if (certB64) {
        const certBytes = base64Decode(certB64);
        const asn1 = fromBER(certBytes.buffer);
        if (asn1.offset !== -1) {
          const cert = new Certificate({ schema: asn1.result });

          // Get public key from certificate
          const pubKeyInfo = cert.subjectPublicKeyInfo;
          const pubKeyBuffer = pubKeyInfo.subjectPublicKey.valueBlock.valueHexView;

          // Determine key algorithm for verification
          let verifyAlgo: { name: string; hash: string };
          if (sigAlg.includes('rsa')) {
            verifyAlgo = {
              name: 'RSASSA-PKCS1-v1_5',
              hash: digestAlgo
            };
          } else if (sigAlg.includes('ecdsa')) {
            verifyAlgo = {
              name: 'ECDSA',
              hash: digestAlgo
            };
          } else {
            // Default to RSA
            verifyAlgo = {
              name: 'RSASSA-PKCS1-v1_5',
              hash: digestAlgo
            };
          }

          // Import public key
          let cryptoKey: CryptoKey;
          try {
            // For RSA keys, we need to construct the SPKI
            const spkiBuffer = pubKeyInfo.toSchema().toBER(false);
            cryptoKey = await crypto.subtle.importKey('spki', spkiBuffer, verifyAlgo, false, ['verify']);
          } catch {
            // Fallback: try raw public key
            cryptoKey = await crypto.subtle.importKey('spki', pubKeyBuffer, verifyAlgo, false, ['verify']);
          }

          // Decode signature value
          const sigValueBytes = base64Decode(sigValueB64);

          // The signed content for RSA-PKCS1 is typically the SignedInfo element
          const signedInfoEl = sigElement.getElementsByTagName('SignedInfo')[0]
            || Array.from(sigElement.getElementsByTagName('*')).find(e => e.localName === 'SignedInfo');
          if (!signedInfoEl) {
            signatureVerified = 'error';
            signatureError = t('samlDecoder.sigNoSignedInfo');
            return;
          }

          const signedInfoXml = serializer.serializeToString(signedInfoEl);
          const signedInfoData = encoder.encode(signedInfoXml);

          let verified: boolean;
          if (verifyAlgo.name === 'ECDSA') {
            // ECDSA signature might need ASN.1 encoding/decoding conversion
            verified = await crypto.subtle.verify(verifyAlgo, cryptoKey, sigValueBytes, signedInfoData);
          } else {
            verified = await crypto.subtle.verify(verifyAlgo, cryptoKey, sigValueBytes, signedInfoData);
          }

          signatureVerified = verified ? 'verified' : 'failed';
          if (!verified) {
            signatureError = t('samlDecoder.sigVerifyFailed');
          }
        } else {
          signatureVerified = 'error';
          signatureError = t('samlDecoder.certParseFailed');
        }
      } else {
        // No certificate found, but digest matched
        signatureVerified = 'not-checked';
        signatureError = t('samlDecoder.digestMatchedNoCert');
      }
    } catch (err) {
      signatureVerified = 'error';
      signatureError = err instanceof Error ? err.message : t('samlDecoder.sigVerifyError');
    }
  }

  // ── Decode SAML ──
  async function decode() {
    error = '';
    decoded = false;
    decoding = true;
    rawXml = '';
    formattedXml = '';
    messageType = '';
    issuer = '';
    destination = '';
    id = '';
    issueInstant = '';
    version = '';
    subjectNameId = '';
    subjectNameIdFormat = '';
    conditionsNotBefore = '';
    conditionsNotOnOrAfter = '';
    audience = '';
    authnInstant = '';
    sessionIndex = '';
    authnContextClassRef = '';
    attributes = [];
    hasSignature = false;
    signatureValue = '';
    signatureAlgorithm = '';
    certSubject = '';
    certIssuer = '';
    certSerial = '';
    certValidFrom = '';
    certValidTo = '';
    signatureVerified = 'not-checked';
    signatureError = '';
    timeValid = 'unknown';
    timeStatusDetail = '';

    const trimmed = input.trim();
    if (!trimmed) { decoding = false; return; }

    try {
      let xml = '';

      if (trimmed.startsWith('<')) {
        xml = trimmed;
      } else {
        let bytes: Uint8Array;
        try { bytes = base64Decode(trimmed); }
        catch { throw new Error(t('samlDecoder.errorBase64')); }

        const asText = new TextDecoder().decode(bytes);
        if (asText.trim().startsWith('<')) {
          xml = asText;
        } else {
          try {
            xml = await inflateRawAsync(bytes);
            if (!xml.trim().startsWith('<')) throw new Error(t('samlDecoder.errorNotXml'));
          } catch { throw new Error(t('samlDecoder.errorDecode')); }
        }
      }

      rawXml = xml;

      // Message type
      if (xml.includes('samlp:Response') || xml.includes('<Response')) messageType = 'SAML Response';
      else if (xml.includes('samlp:AuthnRequest') || xml.includes('<AuthnRequest')) messageType = 'SAML AuthnRequest';
      else if (xml.includes('samlp:LogoutRequest') || xml.includes('<LogoutRequest')) messageType = 'SAML LogoutRequest';
      else messageType = 'Unknown SAML Message';

      // Common fields
      issuer = getElementText(xml, 'Issuer');
      destination = getAttributeValue(xml, 'Response', 'Destination') || getAttributeValue(xml, 'AuthnRequest', 'Destination') || getAttributeValue(xml, 'LogoutRequest', 'Destination');
      id = getAttributeValue(xml, 'Response', 'ID') || getAttributeValue(xml, 'AuthnRequest', 'ID') || getAttributeValue(xml, 'LogoutRequest', 'ID');
      issueInstant = getAttributeValue(xml, 'Response', 'IssueInstant') || getAttributeValue(xml, 'AuthnRequest', 'IssueInstant') || getAttributeValue(xml, 'LogoutRequest', 'IssueInstant');
      version = getAttributeValue(xml, 'Response', 'Version') || getAttributeValue(xml, 'AuthnRequest', 'Version') || getAttributeValue(xml, 'LogoutRequest', 'Version');

      // Assertion fields
      subjectNameId = getElementText(xml, 'NameID');
      subjectNameIdFormat = getAttributeValue(xml, 'NameID', 'Format');
      conditionsNotBefore = getAttributeValue(xml, 'Conditions', 'NotBefore');
      conditionsNotOnOrAfter = getAttributeValue(xml, 'Conditions', 'NotOnOrAfter');
      audience = getElementText(xml, 'Audience');
      authnInstant = getAttributeValue(xml, 'AuthnStatement', 'AuthnInstant');
      sessionIndex = getAttributeValue(xml, 'AuthnStatement', 'SessionIndex');
      authnContextClassRef = getElementText(xml, 'AuthnContextClassRef');

      // Attributes
      attributes = extractAttributes(xml);

      // Signature info
      extractSignatureInfo(xml);

      // Time validity
      checkTimeValidity();

      // Format XML
      formattedXml = formatXml(xml);

      decoded = true;

      // Verify signature asynchronously
      if (hasSignature) {
        await verifySignature();
      }
    } catch (err) {
      error = err instanceof Error ? err.message : t('samlDecoder.errorDecode');
    } finally {
      decoding = false;
    }
  }

  async function copyXml() {
    if (!formattedXml) return;
    try {
      await navigator.clipboard.writeText(formattedXml);
      copiedXml = true;
      setTimeout(() => { copiedXml = false; }, 2000);
    } catch { /* ignore */ }
  }

  function clear() {
    input = ''; error = ''; decoded = false; decoding = false;
    rawXml = ''; formattedXml = ''; messageType = ''; issuer = ''; destination = '';
    id = ''; issueInstant = ''; version = ''; subjectNameId = ''; subjectNameIdFormat = '';
    conditionsNotBefore = ''; conditionsNotOnOrAfter = ''; audience = '';
    authnInstant = ''; sessionIndex = ''; authnContextClassRef = '';
    attributes = []; hasSignature = false; signatureValue = ''; signatureAlgorithm = '';
    certSubject = ''; certIssuer = ''; certSerial = ''; certValidFrom = ''; certValidTo = '';
    signatureVerified = 'not-checked'; signatureError = '';
    timeValid = 'unknown'; timeStatusDetail = ''; copiedXml = false;
  }

  let decodeTimer: ReturnType<typeof setTimeout> | null = null;
  $effect(() => {
    if (input.trim()) {
      if (decodeTimer) clearTimeout(decodeTimer);
      decodeTimer = setTimeout(() => decode(), 500);
    } else {
      decoded = false; error = ''; decoding = false;
    }
  });
</script>

<div class="w-full ml-0 mr-0 p-2 space-y-4">
  <!-- Input card -->
  <div class="card">
    <div class="space-y-4">
      <div class="flex items-center gap-2">
        <ShieldCheck class="w-5 h-5 text-primary-600 dark:text-primary-400" />
        <h2 class="text-lg font-bold text-gray-900 dark:text-gray-100">{t('samlDecoder.title')}</h2>
      </div>

      <div>
        <span class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
          {t('samlDecoder.input')}
        </span>
        <textarea
          bind:value={input}
          placeholder={t('samlDecoder.placeholder')}
          class="textarea font-mono text-sm min-h-[100px]"
        ></textarea>
        <p class="text-xs text-gray-500 dark:text-gray-400 mt-1.5">
          {t('samlDecoder.inputHint')}
        </p>
      </div>

      <div class="flex gap-2">
        <button onclick={decode} class="btn-primary" disabled={decoding}>
          {decoding ? t('samlDecoder.decoding') : t('samlDecoder.decode')}
        </button>
        <button onclick={clear} class="btn-secondary">
          <Trash2 class="w-4 h-4 inline mr-1" />
          {t('common.clear')}
        </button>
      </div>

      {#if error}
        <div class="p-4 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg">
          <p class="text-sm text-red-800 dark:text-red-200">{error}</p>
        </div>
      {/if}
    </div>
  </div>

  {#if decoded}
    <!-- ═══ Validation Status ═══ -->
    <div class="card">
      <h3 class="text-base font-semibold text-gray-900 dark:text-gray-100 mb-3 flex items-center gap-2">
        <BadgeCheck class="w-4 h-4 text-primary-600 dark:text-primary-400" />
        {t('samlDecoder.validationStatus')}
      </h3>

      <div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
        <!-- Time validity -->
        <div class="rounded-lg p-3 border flex items-center gap-3
          {timeValid === 'valid' ? 'bg-green-50 dark:bg-green-900/20 border-green-200 dark:border-green-800'
          : timeValid === 'expired' ? 'bg-red-50 dark:bg-red-900/20 border-red-200 dark:border-red-800'
          : timeValid === 'not-yet-valid' ? 'bg-amber-50 dark:bg-amber-900/20 border-amber-200 dark:border-amber-800'
          : 'bg-gray-50 dark:bg-gray-800 border-gray-200 dark:border-gray-700'}">
          {#if timeValid === 'valid'}
            <Check class="w-5 h-5 text-green-600 dark:text-green-400 flex-shrink-0" />
          {:else if timeValid === 'expired'}
            <XCircle class="w-5 h-5 text-red-600 dark:text-red-400 flex-shrink-0" />
          {:else if timeValid === 'not-yet-valid'}
            <AlertTriangle class="w-5 h-5 text-amber-600 dark:text-amber-400 flex-shrink-0" />
          {:else}
            <Clock class="w-5 h-5 text-gray-400 flex-shrink-0" />
          {/if}
          <div class="flex-1 min-w-0">
            <span class="text-xs font-medium text-gray-500 dark:text-gray-400">{t('samlDecoder.timeValidity')}</span>
            <p class="text-sm font-semibold text-gray-900 dark:text-gray-100">
              {timeValid === 'valid' ? t('samlDecoder.timeValid') : timeValid === 'expired' ? t('samlDecoder.timeExpired') : timeValid === 'not-yet-valid' ? t('samlDecoder.timeNotYetValid') : t('samlDecoder.timeUnknown')}
            </p>
            {#if timeStatusDetail}
              <p class="text-xs text-gray-500 dark:text-gray-400 mt-0.5">{timeStatusDetail}</p>
            {/if}
          </div>
        </div>

        <!-- Signature status -->
        <div class="rounded-lg p-3 border flex items-center gap-3
          {signatureVerified === 'verified' ? 'bg-green-50 dark:bg-green-900/20 border-green-200 dark:border-green-800'
          : signatureVerified === 'failed' ? 'bg-red-50 dark:bg-red-900/20 border-red-200 dark:border-red-800'
          : signatureVerified === 'error' ? 'bg-red-50 dark:bg-red-900/20 border-red-200 dark:border-red-800'
          : hasSignature ? 'bg-amber-50 dark:bg-amber-900/20 border-amber-200 dark:border-amber-800'
          : 'bg-gray-50 dark:bg-gray-800 border-gray-200 dark:border-gray-700'}">
          {#if signatureVerified === 'verified'}
            <Check class="w-5 h-5 text-green-600 dark:text-green-400 flex-shrink-0" />
          {:else if signatureVerified === 'failed' || signatureVerified === 'error'}
            <XCircle class="w-5 h-5 text-red-600 dark:text-red-400 flex-shrink-0" />
          {:else if hasSignature}
            <AlertTriangle class="w-5 h-5 text-amber-600 dark:text-amber-400 flex-shrink-0" />
          {:else}
            <XCircle class="w-5 h-5 text-gray-400 flex-shrink-0" />
          {/if}
          <div class="flex-1 min-w-0">
            <span class="text-xs font-medium text-gray-500 dark:text-gray-400">{t('samlDecoder.signatureStatus')}</span>
            <p class="text-sm font-semibold text-gray-900 dark:text-gray-100">
              {signatureVerified === 'verified' ? t('samlDecoder.sigVerified')
              : signatureVerified === 'failed' ? t('samlDecoder.sigFailed')
              : signatureVerified === 'error' ? t('samlDecoder.sigError')
              : hasSignature ? t('samlDecoder.sigChecking')
              : t('samlDecoder.sigNotPresent')}
            </p>
            {#if signatureError}
              <p class="text-xs text-gray-500 dark:text-gray-400 mt-0.5 break-all">{signatureError}</p>
            {/if}
          </div>
        </div>
      </div>
    </div>

    <!-- ═══ Summary ═══ -->
    <div class="card">
      <div class="space-y-4">
        <h3 class="text-base font-semibold text-gray-900 dark:text-gray-100 flex items-center gap-2">
          <FileCode2 class="w-4 h-4 text-primary-600 dark:text-primary-400" />
          {t('samlDecoder.summary')}
        </h3>
        <div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
          {#if messageType}
            <div class="bg-gray-50 dark:bg-gray-800 rounded-lg p-3 border border-gray-200 dark:border-gray-700">
              <span class="text-xs font-medium text-gray-500 dark:text-gray-400">{t('samlDecoder.messageType')}</span>
              <p class="text-sm font-semibold text-gray-900 dark:text-gray-100">{messageType}</p>
            </div>
          {/if}
          {#if version}
            <div class="bg-gray-50 dark:bg-gray-800 rounded-lg p-3 border border-gray-200 dark:border-gray-700">
              <span class="text-xs font-medium text-gray-500 dark:text-gray-400">{t('samlDecoder.version')}</span>
              <p class="text-sm font-semibold text-gray-900 dark:text-gray-100">{version}</p>
            </div>
          {/if}
          {#if id}
            <div class="bg-gray-50 dark:bg-gray-800 rounded-lg p-3 border border-gray-200 dark:border-gray-700">
              <span class="text-xs font-medium text-gray-500 dark:text-gray-400">ID</span>
              <p class="text-sm font-mono text-gray-900 dark:text-gray-100 break-all">{id}</p>
            </div>
          {/if}
          {#if issueInstant}
            <div class="bg-gray-50 dark:bg-gray-800 rounded-lg p-3 border border-gray-200 dark:border-gray-700">
              <span class="text-xs font-medium text-gray-500 dark:text-gray-400">{t('samlDecoder.issueInstant')}</span>
              <p class="text-sm font-mono text-gray-900 dark:text-gray-100">{issueInstant}</p>
            </div>
          {/if}
        </div>
      </div>
    </div>

    <!-- ═══ Issuer & Destination ═══ -->
    {#if issuer || destination}
      <div class="card">
        <h3 class="text-base font-semibold text-gray-900 dark:text-gray-100 mb-3 flex items-center gap-2">
          <Building2 class="w-4 h-4 text-primary-600 dark:text-primary-400" />
          {t('samlDecoder.issuerDest')}
        </h3>
        <div class="space-y-3">
          {#if issuer}
            <div>
              <span class="text-xs font-medium text-gray-500 dark:text-gray-400">{t('samlDecoder.issuer')}</span>
              <p class="text-sm font-mono text-gray-900 dark:text-gray-100 break-all">{issuer}</p>
            </div>
          {/if}
          {#if destination}
            <div>
              <span class="text-xs font-medium text-gray-500 dark:text-gray-400">{t('samlDecoder.destination')}</span>
              <p class="text-sm font-mono text-gray-900 dark:text-gray-100 break-all">{destination}</p>
            </div>
          {/if}
        </div>
      </div>
    {/if}

    <!-- ═══ Subject ═══ -->
    {#if subjectNameId || subjectNameIdFormat}
      <div class="card">
        <h3 class="text-base font-semibold text-gray-900 dark:text-gray-100 mb-3 flex items-center gap-2">
          <User class="w-4 h-4 text-primary-600 dark:text-primary-400" />
          {t('samlDecoder.subject')}
        </h3>
        <div class="space-y-3">
          {#if subjectNameId}
            <div>
              <span class="text-xs font-medium text-gray-500 dark:text-gray-400">{t('samlDecoder.nameId')}</span>
              <p class="text-sm font-mono text-gray-900 dark:text-gray-100 break-all">{subjectNameId}</p>
            </div>
          {/if}
          {#if subjectNameIdFormat}
            <div>
              <span class="text-xs font-medium text-gray-500 dark:text-gray-400">{t('samlDecoder.nameIdFormat')}</span>
              <p class="text-sm text-gray-900 dark:text-gray-100 break-all">{subjectNameIdFormat}</p>
            </div>
          {/if}
        </div>
      </div>
    {/if}

    <!-- ═══ Conditions ═══ -->
    {#if conditionsNotBefore || conditionsNotOnOrAfter || audience}
      <div class="card">
        <h3 class="text-base font-semibold text-gray-900 dark:text-gray-100 mb-3 flex items-center gap-2">
          <Clock class="w-4 h-4 text-primary-600 dark:text-primary-400" />
          {t('samlDecoder.conditions')}
        </h3>
        <div class="space-y-3">
          {#if conditionsNotBefore}
            <div class="flex items-center justify-between">
              <span class="text-xs font-medium text-gray-500 dark:text-gray-400">{t('samlDecoder.notBefore')}</span>
              <span class="text-sm font-mono text-gray-900 dark:text-gray-100">{conditionsNotBefore}</span>
            </div>
          {/if}
          {#if conditionsNotOnOrAfter}
            <div class="flex items-center justify-between">
              <span class="text-xs font-medium text-gray-500 dark:text-gray-400">{t('samlDecoder.notOnOrAfter')}</span>
              <span class="text-sm font-mono text-gray-900 dark:text-gray-100">{conditionsNotOnOrAfter}</span>
            </div>
          {/if}
          {#if audience}
            <div>
              <span class="text-xs font-medium text-gray-500 dark:text-gray-400">{t('samlDecoder.audience')}</span>
              <p class="text-sm font-mono text-gray-900 dark:text-gray-100 break-all">{audience}</p>
            </div>
          {/if}
        </div>
      </div>
    {/if}

    <!-- ═══ Authn Statement ═══ -->
    {#if authnInstant || sessionIndex || authnContextClassRef}
      <div class="card">
        <h3 class="text-base font-semibold text-gray-900 dark:text-gray-100 mb-3">
          {t('samlDecoder.authnStatement')}
        </h3>
        <div class="space-y-3">
          {#if authnInstant}
            <div class="flex items-center justify-between">
              <span class="text-xs font-medium text-gray-500 dark:text-gray-400">{t('samlDecoder.authnInstant')}</span>
              <span class="text-sm font-mono text-gray-900 dark:text-gray-100">{authnInstant}</span>
            </div>
          {/if}
          {#if sessionIndex}
            <div class="flex items-center justify-between">
              <span class="text-xs font-medium text-gray-500 dark:text-gray-400">{t('samlDecoder.sessionIndex')}</span>
              <span class="text-sm font-mono text-gray-900 dark:text-gray-100 break-all">{sessionIndex}</span>
            </div>
          {/if}
          {#if authnContextClassRef}
            <div>
              <span class="text-xs font-medium text-gray-500 dark:text-gray-400">{t('samlDecoder.authnContext')}</span>
              <p class="text-sm font-mono text-gray-900 dark:text-gray-100 break-all">{authnContextClassRef}</p>
            </div>
          {/if}
        </div>
      </div>
    {/if}

    <!-- ═══ Signature details ═══ -->
    {#if hasSignature}
      <div class="card">
        <h3 class="text-base font-semibold text-gray-900 dark:text-gray-100 mb-3 flex items-center gap-2">
          <ShieldCheck class="w-4 h-4 text-primary-600 dark:text-primary-400" />
          {t('samlDecoder.signatureDetails')}
        </h3>
        <div class="space-y-3">
          {#if signatureAlgorithm}
            <div class="flex items-center justify-between">
              <span class="text-xs font-medium text-gray-500 dark:text-gray-400">{t('samlDecoder.sigAlgorithm')}</span>
              <span class="text-sm font-mono text-gray-900 dark:text-gray-100 break-all">{signatureAlgorithm}</span>
            </div>
          {/if}
          {#if signatureValue}
            <div>
              <span class="text-xs font-medium text-gray-500 dark:text-gray-400">{t('samlDecoder.sigValue')}</span>
              <p class="text-sm font-mono text-gray-900 dark:text-gray-100 break-all">{signatureValue}</p>
            </div>
          {/if}
          {#if certSubject}
            <div>
              <span class="text-xs font-medium text-gray-500 dark:text-gray-400">{t('samlDecoder.certSubject')}</span>
              <p class="text-sm text-gray-900 dark:text-gray-100 break-all">{certSubject}</p>
            </div>
          {/if}
          {#if certIssuer}
            <div>
              <span class="text-xs font-medium text-gray-500 dark:text-gray-400">{t('samlDecoder.certIssuer')}</span>
              <p class="text-sm text-gray-900 dark:text-gray-100 break-all">{certIssuer}</p>
            </div>
          {/if}
          {#if certSerial}
            <div class="flex items-center justify-between">
              <span class="text-xs font-medium text-gray-500 dark:text-gray-400">{t('samlDecoder.certSerial')}</span>
              <span class="text-sm font-mono text-gray-900 dark:text-gray-100 break-all">{certSerial}</span>
            </div>
          {/if}
          {#if certValidFrom}
            <div class="flex items-center justify-between">
              <span class="text-xs font-medium text-gray-500 dark:text-gray-400">{t('samlDecoder.certValidFrom')}</span>
              <span class="text-sm font-mono text-gray-900 dark:text-gray-100">{certValidFrom}</span>
            </div>
          {/if}
          {#if certValidTo}
            <div class="flex items-center justify-between">
              <span class="text-xs font-medium text-gray-500 dark:text-gray-400">{t('samlDecoder.certValidTo')}</span>
              <span class="text-sm font-mono text-gray-900 dark:text-gray-100">{certValidTo}</span>
            </div>
          {/if}
        </div>
      </div>
    {/if}

    <!-- ═══ Attributes (Claims) ═══ -->
    {#if attributes.length > 0}
      <div class="card">
        <h3 class="text-base font-semibold text-gray-900 dark:text-gray-100 mb-3">
          {t('samlDecoder.attributes')}
        </h3>
        <div class="overflow-x-auto">
          <table class="w-full text-sm">
            <thead>
              <tr class="border-b border-gray-200 dark:border-gray-700">
                <th class="text-left py-2 px-3 font-medium text-gray-500 dark:text-gray-400">{t('samlDecoder.attributeName')}</th>
                <th class="text-left py-2 px-3 font-medium text-gray-500 dark:text-gray-400">{t('samlDecoder.attributeValue')}</th>
              </tr>
            </thead>
            <tbody>
              {#each attributes as attr}
                <tr class="border-b border-gray-100 dark:border-gray-700/50">
                  <td class="py-2 px-3 font-mono text-gray-700 dark:text-gray-300 break-all">{attr.name}</td>
                  <td class="py-2 px-3 text-gray-900 dark:text-gray-100 break-all">{attr.value}</td>
                </tr>
              {/each}
            </tbody>
          </table>
        </div>
      </div>
    {/if}

    <!-- ═══ Formatted XML ═══ -->
    <div class="card">
      <div class="flex items-center justify-between mb-3">
        <h3 class="text-base font-semibold text-gray-900 dark:text-gray-100">
          {t('samlDecoder.formattedXml')}
        </h3>
        <button
          onclick={copyXml}
          class="btn-secondary text-sm {copiedXml ? 'bg-green-500 hover:bg-green-600 text-white' : ''}"
        >
          {#if copiedXml}
            <Check class="w-4 h-4 inline mr-1" />
            {t('common.copied')}
          {:else}
            <Copy class="w-4 h-4 inline mr-1" />
            {t('common.copy')}
          {/if}
        </button>
      </div>
      <textarea
        value={formattedXml}
        readonly
        class="textarea font-mono text-sm min-h-[200px] max-h-[500px] overflow-y-auto"
      ></textarea>
    </div>
  {/if}
</div>
