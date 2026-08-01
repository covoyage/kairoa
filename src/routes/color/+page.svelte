<script lang="ts">
  import * as convert from 'color-convert';
  import { translationsStore } from '$lib/stores/i18n';

  type ColorKey = 'hex' | 'rgb' | 'hsl' | 'hwb' | 'lch' | 'oklch' | 'cmyk' | 'name';

  const defaultHex = '#1ea54c';

  let values = $state<Record<ColorKey, string>>({
    hex: defaultHex,
    rgb: '',
    hsl: '',
    hwb: '',
    lch: '',
    oklch: '',
    cmyk: '',
    name: ''
  });
  let error = $state('');
  let copiedKey = $state<ColorKey | null>(null);

  let translations = $derived($translationsStore);

  const rows: { key: ColorKey; label: () => string }[] = [
    { key: 'hex', label: () => t('color.labels.hex') },
    { key: 'rgb', label: () => t('color.labels.rgb') },
    { key: 'hsl', label: () => t('color.labels.hsl') },
    { key: 'hwb', label: () => t('color.labels.hwb') },
    { key: 'lch', label: () => t('color.labels.lch') },
    { key: 'oklch', label: () => t('color.labels.oklch') },
    { key: 'cmyk', label: () => t('color.labels.cmyk') },
    { key: 'name', label: () => t('color.labels.name') }
  ];

  function t(key: string): string {
    const keys = key.split('.');
    let value: any = translations;
    for (const k of keys) {
      value = value?.[k];
    }
    return value || key;
  }

  function parseNumbers(value: string, expected: number): number[] | null {
    const matches = value.match(/-?\d+(?:\.\d+)?%?/g);
    if (!matches || matches.length < expected) return null;
    return matches.slice(0, expected).map((m) => parseFloat(m.replace('%', '')));
  }

  function parseHex(hex: string): { r: number; g: number; b: number } | null {
    if (!hex) return null;
    let normalized = hex.trim().replace('#', '');
    if (normalized.length === 3) {
      normalized = normalized.split('').map((c) => c + c).join('');
    }
    if (normalized.length !== 6) return null;

    const r = parseInt(normalized.slice(0, 2), 16);
    const g = parseInt(normalized.slice(2, 4), 16);
    const b = parseInt(normalized.slice(4, 6), 16);
    if ([r, g, b].some((n) => Number.isNaN(n))) return null;
    return { r, g, b };
  }

  function parseRgb(input: string) {
    const nums = parseNumbers(input, 3);
    if (!nums) return null;
    const [r, g, b] = nums.map((n) => Math.round(n));
    if ([r, g, b].some((n) => n < 0 || n > 255)) return null;
    return { r, g, b };
  }

  function parseHsl(input: string) {
    const nums = parseNumbers(input, 3);
    if (!nums) return null;
    const [h, s, l] = nums;
    if (h < 0 || h > 360 || s < 0 || s > 100 || l < 0 || l > 100) return null;
    return { h, s, l };
  }

  function parseHwb(input: string) {
    const nums = parseNumbers(input, 3);
    if (!nums) return null;
    const [h, w, b] = nums;
    if (h < 0 || h > 360 || w < 0 || w > 100 || b < 0 || b > 100) return null;
    return { h, w, b };
  }

  function parseLch(input: string) {
    const nums = parseNumbers(input, 3);
    if (!nums) return null;
    const [l, c, h] = nums;
    if (l < 0 || l > 100 || c < 0) return null;
    return { l, c, h };
  }

  function parseOklch(input: string) {
    const nums = parseNumbers(input, 3);
    if (!nums) return null;
    const [l, c, h] = nums;
    if (l < 0 || l > 1 || c < 0 || h < 0 || h > 360) return null;
    return { l, c, h };
  }

  // ── OKLCH ↔ RGB conversion (no external dependency) ──
  // Reference: https://bottosson.github.io/posts/oklab/
  function srgbToLinear(c: number): number {
    return c <= 0.04045 ? c / 12.92 : Math.pow((c + 0.055) / 1.055, 2.4);
  }

  function linearToSrgb(c: number): number {
    return c <= 0.0031308 ? c * 12.92 : 1.055 * Math.pow(c, 1 / 2.4) - 0.055;
  }

  function rgbToOklab(r: number, g: number, b: number): { L: number; a: number; b: number } {
    const lr = srgbToLinear(r / 255);
    const lg = srgbToLinear(g / 255);
    const lb = srgbToLinear(b / 255);

    const l = 0.4122214708 * lr + 0.5363325363 * lg + 0.0514459929 * lb;
    const m = 0.2119034982 * lr + 0.6806993517 * lg + 0.1073969578 * lb;
    const s = 0.0883024619 * lr + 0.2817188376 * lg + 0.6299787005 * lb;

    const l_ = Math.cbrt(l);
    const m_ = Math.cbrt(m);
    const s_ = Math.cbrt(s);

    return {
      L: 0.2104542553 * l_ + 0.793617785 * m_ - 0.0040720468 * s_,
      a: 1.9779984951 * l_ - 2.428592205 * m_ + 0.4505937099 * s_,
      b: 0.0259040375 * l_ + 0.7827717663 * m_ - 0.808675766 * s_
    };
  }

  function oklabToRgb(L: number, a: number, b: number): { r: number; g: number; b: number } {
    const l_ = L + 0.3963377774 * a + 0.2158037573 * b;
    const m_ = L - 0.1055613458 * a - 0.0638541728 * b;
    const s_ = L - 0.0894841775 * a - 1.1913699082 * b;

    const l = l_ * l_ * l_;
    const m = m_ * m_ * m_;
    const s = s_ * s_ * s_;

    const lr = 4.0767416621 * l - 3.3077115913 * m + 0.2309699292 * s;
    const lg = -1.2684380046 * l + 2.6097572851 * m - 0.3413193965 * s;
    const lb = -0.0041960863 * l - 0.7034186147 * m + 1.707614781 * s;

    return {
      r: Math.max(0, Math.min(255, Math.round(linearToSrgb(lr) * 255))),
      g: Math.max(0, Math.min(255, Math.round(linearToSrgb(lg) * 255))),
      b: Math.max(0, Math.min(255, Math.round(linearToSrgb(lb) * 255)))
    };
  }

  function oklabToOklch(L: number, a: number, b: number): { L: number; C: number; H: number } {
    const C = Math.sqrt(a * a + b * b);
    let H = Math.atan2(b, a) * 180 / Math.PI;
    if (H < 0) H += 360;
    return { L, C, H };
  }

  function oklchToOklab(L: number, C: number, H: number): { L: number; a: number; b: number } {
    const rad = H * Math.PI / 180;
    return { L, a: C * Math.cos(rad), b: C * Math.sin(rad) };
  }

  function parseCmyk(input: string) {
    const nums = parseNumbers(input, 4);
    if (!nums) return null;
    const [c, m, y, k] = nums;
    if ([c, m, y, k].some((n) => n < 0 || n > 100)) return null;
    return { c, m, y, k };
  }

  function rgbToHex(r: number, g: number, b: number): string {
    return (
      '#' +
      [r, g, b]
        .map((x) => {
          const h = x.toString(16);
          return h.length === 1 ? `0${h}` : h;
        })
        .join('')
    );
  }

  function toFixed(value: number, digits = 2) {
    return Number.isFinite(value) ? Number(value.toFixed(digits)) : value;
  }

  function setFromRgb(r: number, g: number, b: number) {
    const [h, s, l] = convert.rgb.hsl([r, g, b]);
    const [hwbH, whiteness, blackness] = convert.rgb.hwb([r, g, b]);
    const [lVal, cVal, hVal] = convert.rgb.lch([r, g, b]);
    const oklab = rgbToOklab(r, g, b);
    const oklch = oklabToOklch(oklab.L, oklab.a, oklab.b);
    const [c, m, y, k] = convert.rgb.cmyk([r, g, b]);
    const keyword = convert.rgb.keyword([r, g, b]);

    values = {
      hex: rgbToHex(r, g, b),
      rgb: `rgb(${r}, ${g}, ${b})`,
      hsl: `hsl(${Math.round(h)}, ${Math.round(s)}%, ${Math.round(l)}%)`,
      hwb: `hwb(${Math.round(hwbH)} ${toFixed(whiteness)}% ${toFixed(blackness)}%)`,
      lch: `lch(${toFixed(lVal)}% ${toFixed(cVal)} ${toFixed(hVal)})`,
      oklch: `oklch(${toFixed(oklch.L)} ${toFixed(oklch.C)} ${toFixed(oklch.H)})`,
      cmyk: `device-cmyk(${toFixed(c)}% ${toFixed(m)}% ${toFixed(y)}% ${toFixed(k)}%)`,
      name: keyword ?? t('color.noName')
    };

    error = '';
  }

  function convertFrom(key: ColorKey, value: string) {
    values = { ...values, [key]: value };
    if (!value.trim()) {
      error = '';
      return;
    }

    try {
      switch (key) {
        case 'hex': {
          const rgb = parseHex(value);
          if (!rgb) throw new Error(t('color.invalidHex'));
          setFromRgb(rgb.r, rgb.g, rgb.b);
          break;
        }
        case 'rgb': {
          const rgb = parseRgb(value);
          if (!rgb) throw new Error(t('color.invalidRGB'));
          setFromRgb(rgb.r, rgb.g, rgb.b);
          break;
        }
        case 'hsl': {
          const hsl = parseHsl(value);
          if (!hsl) throw new Error(t('color.invalidHSL'));
          const [r, g, b] = convert.hsl.rgb([hsl.h, hsl.s, hsl.l]);
          setFromRgb(Math.round(r), Math.round(g), Math.round(b));
          break;
        }
        case 'hwb': {
          const hwb = parseHwb(value);
          if (!hwb) throw new Error(t('color.invalidHSL'));
          const [r, g, b] = convert.hwb.rgb([hwb.h, hwb.w, hwb.b]);
          setFromRgb(Math.round(r), Math.round(g), Math.round(b));
          break;
        }
        case 'lch': {
          const lch = parseLch(value);
          if (!lch) throw new Error(t('color.parseError'));
          const [r, g, b] = convert.lch.rgb([lch.l, lch.c, lch.h]);
          setFromRgb(Math.round(r), Math.round(g), Math.round(b));
          break;
        }
        case 'oklch': {
          const oklch = parseOklch(value);
          if (!oklch) throw new Error(t('color.parseError'));
          const oklab = oklchToOklab(oklch.l, oklch.c, oklch.h);
          const rgb = oklabToRgb(oklab.L, oklab.a, oklab.b);
          setFromRgb(rgb.r, rgb.g, rgb.b);
          break;
        }
        case 'cmyk': {
          const cmyk = parseCmyk(value);
          if (!cmyk) throw new Error(t('color.parseError'));
          const [r, g, b] = convert.cmyk.rgb([cmyk.c, cmyk.m, cmyk.y, cmyk.k]);
          setFromRgb(Math.round(r), Math.round(g), Math.round(b));
          break;
        }
        case 'name': {
          const rgb = convert.keyword.rgb(value.trim());
          if (!rgb) throw new Error(t('color.parseError'));
          setFromRgb(rgb[0], rgb[1], rgb[2]);
          break;
        }
      }
    } catch (e) {
      error = e instanceof Error ? e.message : t('color.convertError');
    }
  }

  function onHexChange(value: string) {
    convertFrom('hex', value.startsWith('#') ? value : `#${value}`);
  }

  function onPickerChange(value: string) {
    convertFrom('hex', value);
  }

  async function copyValue(key: ColorKey) {
    if (!values[key]) return;
    try {
      await navigator.clipboard.writeText(values[key]);
      copiedKey = key;
      // keep the value stable during async to avoid race with rapid edits
      const currentKey = key;
      setTimeout(() => {
        if (copiedKey === currentKey) copiedKey = null;
      }, 800);
    } catch (err) {
      console.error('Copy failed', err);
    }
  }

  function clearField(key: ColorKey) {
    values = { ...values, [key]: '' };
  }

  function resetAll() {
    convertFrom('hex', defaultHex);
  }

  // initialize once with default color
  convertFrom('hex', values.hex);
</script>

<div class="flex flex-col h-full w-full p-2">
  <div class="card flex-1 flex flex-col space-y-6">
    <div class="flex items-center gap-3">
      <div class="w-28 text-right text-sm font-medium text-gray-700 dark:text-gray-300">
        {t('color.labels.colorPicker')}:
      </div>
      <div class="flex-1">
        <input
          type="color"
          class="h-10 w-full rounded-md border border-gray-300 dark:border-gray-700 cursor-pointer"
          bind:value={values.hex}
          oninput={(e) => onPickerChange((e.target as HTMLInputElement).value)}
          style={`background: ${values.hex};`}
        />
      </div>
    </div>

    <div class="space-y-2">
      {#each rows as row}
        <div class="flex items-center gap-3">
          <div class="w-28 text-right text-base font-medium text-gray-800 dark:text-gray-100">
            {row.label()}:
          </div>
          <input
            class="input flex-1 font-mono text-sm"
            bind:value={values[row.key]}
            oninput={(e) => convertFrom(row.key, (e.target as HTMLInputElement).value)}
          />
          <div class="flex items-center gap-2">
            <button class="btn-secondary px-3 py-2" onclick={() => copyValue(row.key)}>
              {copiedKey === row.key ? t('common.copied') : t('common.copy')}
            </button>
          </div>
        </div>
      {/each}
    </div>

    {#if error}
      <div class="p-3 bg-red-50 dark:bg-red-900/30 border border-red-200 dark:border-red-800 rounded">
        <p class="text-sm text-red-700 dark:text-red-200">{error}</p>
      </div>
    {/if}
  </div>
</div>
