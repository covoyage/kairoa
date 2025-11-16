<script lang="ts">
  import { translationsStore } from '$lib/stores/i18n';
  
  type OutputFormat = 'arabic' | 'roman' | 'chineseLower' | 'chineseUpper' | 'chineseAmount' | 'english';
  
  let inputValue = $state('');
  let error = $state('');
  let copiedFormats = $state<Set<OutputFormat>>(new Set());
  
  let results = $state<Record<OutputFormat, string>>({
    arabic: '',
    roman: '',
    chineseLower: '',
    chineseUpper: '',
    chineseAmount: '',
    english: ''
  });

  let translations = $derived($translationsStore);

  function t(key: string): string {
    const keys = key.split('.');
    let value: any = translations;
    for (const k of keys) {
      value = value?.[k];
    }
    return value || key;
  }

  // 罗马数字到阿拉伯数字的映射
  const romanToArabicMap: Record<string, number> = {
    'M': 1000,
    'CM': 900,
    'D': 500,
    'CD': 400,
    'C': 100,
    'XC': 90,
    'L': 50,
    'XL': 40,
    'X': 10,
    'IX': 9,
    'V': 5,
    'IV': 4,
    'I': 1
  };

  // 中文数字（小写）
  const chineseLowerDigits = ['零', '一', '二', '三', '四', '五', '六', '七', '八', '九'];

  // 中文数字（大写）
  const chineseUpperDigits = ['零', '壹', '贰', '叁', '肆', '伍', '陆', '柒', '捌', '玖'];

  // 英文数字
  const englishOnes = ['', 'One', 'Two', 'Three', 'Four', 'Five', 'Six', 'Seven', 'Eight', 'Nine'];
  const englishTeens = ['Ten', 'Eleven', 'Twelve', 'Thirteen', 'Fourteen', 'Fifteen', 'Sixteen', 'Seventeen', 'Eighteen', 'Nineteen'];
  const englishTens = ['', '', 'Twenty', 'Thirty', 'Forty', 'Fifty', 'Sixty', 'Seventy', 'Eighty', 'Ninety'];

  // 检测输入类型
  function detectInputType(input: string): 'arabic' | 'roman' | null {
    const trimmed = input.trim();
    if (!trimmed) return null;
    
    // 检查是否是阿拉伯数字（限制最大15位）
    if (/^\d+$/.test(trimmed)) {
      // 限制最大15位数字
      if (trimmed.length <= 15) {
        const num = parseInt(trimmed, 10);
        if (num > 0) {
          return 'arabic';
        }
      }
    }
    
    // 检查是否是罗马数字
    const upperTrimmed = trimmed.toUpperCase();
    if (/^[IVXLCDM]+$/.test(upperTrimmed)) {
      return 'roman';
    }
    
    return null;
  }

  // 验证罗马数字格式
  function isValidRoman(roman: string): boolean {
    if (!roman.trim()) return true;
    
    const upperRoman = roman.trim().toUpperCase();
    if (!/^[IVXLCDM]+$/.test(upperRoman)) return false;
    
    const invalidPatterns = [
      /IIII/, /XXXX/, /CCCC/, /MMMM/,
      /VV/, /LL/, /DD/,
      /IL/, /IC/, /ID/, /IM/,
      /VX/, /VL/, /VC/, /VD/, /VM/,
      /XD/, /XM/,
      /LC/, /LD/, /LM/,
      /DM/
    ];
    
    for (const pattern of invalidPatterns) {
      if (pattern.test(upperRoman)) return false;
    }
    
    return true;
  }

  // 罗马数字转阿拉伯数字
  function romanToArabic(roman: string): number {
    const upperRoman = roman.trim().toUpperCase();
    let result = 0;
    let i = 0;

    while (i < upperRoman.length) {
      if (i < upperRoman.length - 1) {
        const twoChar = upperRoman.substring(i, i + 2);
        if (romanToArabicMap[twoChar]) {
          result += romanToArabicMap[twoChar];
          i += 2;
          continue;
        }
      }
      
      const oneChar = upperRoman[i];
      if (romanToArabicMap[oneChar]) {
        result += romanToArabicMap[oneChar];
        i += 1;
      } else {
        throw new Error(t('romanNumeral.invalidRoman'));
      }
    }

    return result;
  }

  // 阿拉伯数字转罗马数字
  function arabicToRoman(num: number): string {
    if (num <= 0 || num > 3999) {
      throw new Error(t('romanNumeral.invalidRange'));
    }

    let result = '';
    let remaining = num;
    const entries = Object.entries(romanToArabicMap);
    
    for (const [roman, value] of entries) {
      const count = Math.floor(remaining / value);
      if (count > 0) {
        result += roman.repeat(count);
        remaining -= value * count;
      }
    }

    return result;
  }

  // 转换四位数字（0-9999）为中文小写
  function convertFourDigitsLower(n: number, needZero: boolean): string {
    if (n === 0) return '';
    
    const numStr = n.toString().padStart(4, '0');
    const digits = numStr.split('').map(d => parseInt(d, 10));
    let result = '';
    let hasPreviousDigit = needZero;
    
    // 千位
    if (digits[0] > 0) {
      result += chineseLowerDigits[digits[0]] + '千';
      hasPreviousDigit = true;
    }
    
    // 百位
    if (digits[1] > 0) {
      if (hasPreviousDigit && digits[0] === 0) result += '零';
      result += chineseLowerDigits[digits[1]] + '百';
      hasPreviousDigit = true;
    } else if (digits[0] > 0 && (digits[2] > 0 || digits[3] > 0)) {
      hasPreviousDigit = true;
    }
    
    // 十位
    if (digits[2] > 0) {
      if (hasPreviousDigit && digits[1] === 0) result += '零';
      if (digits[2] === 1 && digits[0] === 0 && digits[1] === 0) {
        result += '十';
      } else {
        result += chineseLowerDigits[digits[2]] + '十';
      }
      hasPreviousDigit = true;
    } else if ((digits[0] > 0 || digits[1] > 0) && digits[3] > 0) {
      result += '零';
      hasPreviousDigit = true;
    }
    
    // 个位
    if (digits[3] > 0) {
      if (hasPreviousDigit && digits[2] === 0) result += '零';
      result += chineseLowerDigits[digits[3]];
    }
    
    return result;
  }

  // 阿拉伯数字转中文小写
  function arabicToChineseLower(num: number): string {
    if (num === 0) return '零';
    if (num > Number.MAX_SAFE_INTEGER) return 'Number too large';

    const parts: string[] = [];
    let remaining = num;
    
    // 兆（万亿）
    if (remaining >= 1000000000000) {
      const zhao = Math.floor(remaining / 1000000000000);
      const zhaoText = convertFourDigitsLower(zhao, false);
      if (zhaoText) {
        parts.push(zhaoText + '兆');
      }
      remaining = remaining % 1000000000000;
    }
    
    // 亿
    if (remaining >= 100000000) {
      const yi = Math.floor(remaining / 100000000);
      const yiText = convertFourDigitsLower(yi, parts.length > 0);
      if (yiText) {
        parts.push(yiText + '亿');
      }
      remaining = remaining % 100000000;
    }
    
    // 万
    if (remaining >= 10000) {
      const wan = Math.floor(remaining / 10000);
      const wanText = convertFourDigitsLower(wan, parts.length > 0);
      if (wanText) {
        parts.push(wanText + '万');
      }
      remaining = remaining % 10000;
    }
    
    // 个位到千位
    if (remaining > 0) {
      const remainderText = convertFourDigitsLower(remaining, parts.length > 0);
      if (remainderText) {
        parts.push(remainderText);
      }
    }
    
    return parts.join('');
  }

  // 转换四位数字（0-9999）为中文大写
  function convertFourDigitsUpper(n: number, needZero: boolean): string {
    if (n === 0) return '';
    
    const numStr = n.toString().padStart(4, '0');
    const digits = numStr.split('').map(d => parseInt(d, 10));
    let result = '';
    let hasPreviousDigit = needZero;
    
    // 千位
    if (digits[0] > 0) {
      result += chineseUpperDigits[digits[0]] + '仟';
      hasPreviousDigit = true;
    }
    
    // 百位
    if (digits[1] > 0) {
      if (hasPreviousDigit && digits[0] === 0) result += '零';
      result += chineseUpperDigits[digits[1]] + '佰';
      hasPreviousDigit = true;
    } else if (digits[0] > 0 && (digits[2] > 0 || digits[3] > 0)) {
      hasPreviousDigit = true;
    }
    
    // 十位
    if (digits[2] > 0) {
      if (hasPreviousDigit && digits[1] === 0) result += '零';
      if (digits[2] === 1 && digits[0] === 0 && digits[1] === 0) {
        result += '拾';
      } else {
        result += chineseUpperDigits[digits[2]] + '拾';
      }
      hasPreviousDigit = true;
    } else if ((digits[0] > 0 || digits[1] > 0) && digits[3] > 0) {
      result += '零';
      hasPreviousDigit = true;
    }
    
    // 个位
    if (digits[3] > 0) {
      if (hasPreviousDigit && digits[2] === 0) result += '零';
      result += chineseUpperDigits[digits[3]];
    }
    
    return result;
  }

  // 阿拉伯数字转中文大写
  function arabicToChineseUpper(num: number): string {
    if (num === 0) return '零';
    if (num > Number.MAX_SAFE_INTEGER) return 'Number too large';

    const parts: string[] = [];
    let remaining = num;
    
    // 兆（万亿）
    if (remaining >= 1000000000000) {
      const zhao = Math.floor(remaining / 1000000000000);
      const zhaoText = convertFourDigitsUpper(zhao, false);
      if (zhaoText) {
        parts.push(zhaoText + '兆');
      }
      remaining = remaining % 1000000000000;
    }
    
    // 亿
    if (remaining >= 100000000) {
      const yi = Math.floor(remaining / 100000000);
      const yiText = convertFourDigitsUpper(yi, parts.length > 0);
      if (yiText) {
        parts.push(yiText + '亿');
      }
      remaining = remaining % 100000000;
    }
    
    // 万
    if (remaining >= 10000) {
      const wan = Math.floor(remaining / 10000);
      const wanText = convertFourDigitsUpper(wan, parts.length > 0);
      if (wanText) {
        parts.push(wanText + '万');
      }
      remaining = remaining % 10000;
    }
    
    // 个位到千位
    if (remaining > 0) {
      const remainderText = convertFourDigitsUpper(remaining, parts.length > 0);
      if (remainderText) {
        parts.push(remainderText);
      }
    }
    
    return parts.join('');
  }

  // 阿拉伯数字转中文金额
  function arabicToChineseAmount(num: number): string {
    const upper = arabicToChineseUpper(num);
    return upper + '元整';
  }

  // 转换三位数（0-999）为英文
  function convertThreeDigits(n: number): string {
    if (n === 0) return '';
    
    const parts: string[] = [];
    const hundreds = Math.floor(n / 100);
    const remainder = n % 100;
    
    if (hundreds > 0) {
      parts.push(englishOnes[hundreds] + ' Hundred');
    }
    
    if (remainder > 0) {
      if (hundreds > 0) {
        parts.push('and');
      }
      
      if (remainder < 20) {
        if (remainder < 10) {
          parts.push(englishOnes[remainder]);
        } else {
          parts.push(englishTeens[remainder - 10]);
        }
      } else {
        const tens = Math.floor(remainder / 10);
        const ones = remainder % 10;
        parts.push(englishTens[tens]);
        if (ones > 0) {
          parts.push(englishOnes[ones]);
        }
      }
    }
    
    return parts.join(' ');
  }

  // 阿拉伯数字转英文
  function arabicToEnglish(num: number): string {
    if (num === 0) return 'Zero';
    if (num > Number.MAX_SAFE_INTEGER) return 'Number too large';

    const parts: string[] = [];
    let remaining = num;
    
    // 处理万亿位（Trillion）
    if (remaining >= 1000000000000) {
      const trillions = Math.floor(remaining / 1000000000000);
      const trillionText = convertThreeDigits(trillions);
      if (trillionText) {
        parts.push(trillionText + ' Trillion');
      }
      remaining = remaining % 1000000000000;
    }
    
    // 处理十亿位（Billion）
    if (remaining >= 1000000000) {
      const billions = Math.floor(remaining / 1000000000);
      const billionText = convertThreeDigits(billions);
      if (billionText) {
        parts.push(billionText + ' Billion');
      }
      remaining = remaining % 1000000000;
    }
    
    // 处理百万位（Million）
    if (remaining >= 1000000) {
      const millions = Math.floor(remaining / 1000000);
      const millionText = convertThreeDigits(millions);
      if (millionText) {
        parts.push(millionText + ' Million');
      }
      remaining = remaining % 1000000;
    }
    
    // 处理千位（Thousand）
    if (remaining >= 1000) {
      const thousands = Math.floor(remaining / 1000);
      const thousandText = convertThreeDigits(thousands);
      if (thousandText) {
        parts.push(thousandText + ' Thousand');
      }
      remaining = remaining % 1000;
    }
    
    // 处理百位及以下
    if (remaining > 0) {
      const remainderText = convertThreeDigits(remaining);
      if (remainderText) {
        // 如果前面有部分，且百位部分不包含"and"，则需要加"and"
        if (parts.length > 0 && !remainderText.includes('and')) {
          parts.push('and');
        }
        parts.push(remainderText);
      }
    }
    
    return parts.join(' ');
  }

  // 获取阿拉伯数字
  function getArabicNumber(input: string): number | null {
    const inputType = detectInputType(input);
    
    if (inputType === 'arabic') {
      return parseInt(input.trim(), 10);
    } else if (inputType === 'roman') {
      if (!isValidRoman(input)) {
        return null;
      }
      try {
        return romanToArabic(input);
      } catch {
        return null;
      }
    }
    
    return null;
  }

  // 执行转换
  function convert() {
    error = '';
    results = {
      arabic: '',
      roman: '',
      chineseLower: '',
      chineseUpper: '',
      chineseAmount: '',
      english: ''
    };

    if (!inputValue.trim()) {
      return;
    }

    // 检查输入长度
    const trimmedInput = inputValue.trim();
    if (trimmedInput && /^\d+$/.test(trimmedInput) && trimmedInput.length > 15) {
      error = t('romanNumeral.maxDigitsExceeded');
      return;
    }

    const num = getArabicNumber(inputValue);
    if (num === null) {
      const inputType = detectInputType(inputValue);
      if (inputType === null) {
        error = t('romanNumeral.invalidInput');
      } else if (inputType === 'arabic') {
        error = t('romanNumeral.invalidArabic');
      } else {
        error = t('romanNumeral.invalidRoman');
      }
      return;
    }

    try {
      // 罗马数字转换有3999的限制
      let romanResult = '';
      if (num <= 3999) {
        romanResult = arabicToRoman(num);
      } else {
        romanResult = t('romanNumeral.romanLimitExceeded');
      }

      results = {
        arabic: num.toString(),
        roman: romanResult,
        chineseLower: arabicToChineseLower(num),
        chineseUpper: arabicToChineseUpper(num),
        chineseAmount: arabicToChineseAmount(num),
        english: arabicToEnglish(num)
      };
    } catch (e) {
      error = e instanceof Error ? e.message : t('romanNumeral.convertError');
    }
  }

  // 监听输入变化
  $effect(() => {
    convert();
  });

  async function copyToClipboard(format: OutputFormat, value: string) {
    if (!value) return;
    
    try {
      await navigator.clipboard.writeText(value);
      copiedFormats.add(format);
      copiedFormats = new Set(copiedFormats);
      
      setTimeout(() => {
        copiedFormats.delete(format);
        copiedFormats = new Set(copiedFormats);
      }, 2000);
    } catch (error) {
      console.error('Failed to copy:', error);
    }
  }

  function clear() {
    inputValue = '';
    error = '';
    results = {
      arabic: '',
      roman: '',
      chineseLower: '',
      chineseUpper: '',
      chineseAmount: '',
      english: ''
    };
  }

  const formats: OutputFormat[] = ['arabic', 'roman', 'chineseLower', 'chineseUpper', 'chineseAmount', 'english'];
</script>

<div class="w-full ml-0 mr-0 p-2 space-y-6">
  <!-- 输入区域卡片 -->
  <div class="card">
    <div class="space-y-4">
      <div>
        <textarea
          bind:value={inputValue}
          placeholder={t('romanNumeral.inputPlaceholder')}
          class="textarea h-16 font-mono"
        ></textarea>
        <p class="text-xs text-gray-500 dark:text-gray-400 mt-2">
          {t('romanNumeral.inputHint')}
        </p>
      </div>

      {#if error}
        <div class="p-4 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg">
          <p class="text-sm text-red-800 dark:text-red-200">{error}</p>
        </div>
      {/if}

      <div class="flex gap-2">
        <button onclick={clear} class="btn-secondary">
          {t('romanNumeral.clear')}
        </button>
      </div>
    </div>
  </div>

  <!-- 转换结果卡片网格 -->
  <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
    {#each formats as format}
      {@const value = results[format]}
      {@const hasValue = value && value.length > 0}
      {@const isCopied = copiedFormats.has(format)}
      <div class="card relative p-4 {isCopied ? 'ring-2 ring-green-500 ring-offset-2' : ''} transition-all duration-300">
        <div class="space-y-3">
          <div class="flex items-start justify-between">
            <div class="flex-1">
              <h3 class="text-lg font-bold text-gray-900 dark:text-gray-100">
                {#if format === 'arabic'}
                  {t('romanNumeral.formatArabic')}
                {:else if format === 'roman'}
                  {t('romanNumeral.formatRoman')}
                {:else if format === 'chineseLower'}
                  {t('romanNumeral.formatChineseLower')}
                {:else if format === 'chineseUpper'}
                  {t('romanNumeral.formatChineseUpper')}
                {:else if format === 'chineseAmount'}
                  {t('romanNumeral.formatChineseAmount')}
                {:else if format === 'english'}
                  {t('romanNumeral.formatEnglish')}
                {/if}
              </h3>
            </div>
            {#if hasValue}
              <button
                onclick={() => copyToClipboard(format, value)}
                class="btn-secondary text-xs px-3 py-1.5 whitespace-nowrap ml-2 transition-all duration-200 {isCopied ? 'bg-green-500 hover:bg-green-600 text-white' : ''}"
              >
                {#if isCopied}
                  <span class="flex items-center gap-1">
                    <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7"></path>
                    </svg>
                    {t('common.copied')}
                  </span>
                {:else}
                  {t('common.copy')}
                {/if}
              </button>
            {/if}
          </div>
          
          {#if hasValue}
            <div class="min-h-16 p-3 bg-gray-50 dark:bg-gray-800/50 rounded-lg border border-gray-200 dark:border-gray-700 {isCopied ? 'bg-green-50 dark:bg-green-900/20 border-green-300 dark:border-green-700' : ''} transition-colors duration-300">
              <p class="text-sm font-mono break-words text-gray-900 dark:text-gray-100">
                {value}
              </p>
            </div>
          {:else}
            <div class="min-h-16 flex items-center justify-center text-gray-400 dark:text-gray-500 text-sm">
              {t('romanNumeral.inputPlaceholder')}
            </div>
          {/if}
        </div>
      </div>
    {/each}
  </div>
</div>
