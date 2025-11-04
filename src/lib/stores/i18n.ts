import { writable, derived } from 'svelte/store';
import { browser } from '$app/environment';

export type Locale = 'en' | 'zh';

const translations = {
  en: {
    app: {
      title: 'Kairoa',
      subtitle: 'Desktop Utility Tools'
    },
    nav: {
      hash: 'Hash Calculator',
      time: 'Time Converter',
      uuid: 'UUID/ULID Generator',
      json: 'JSON Formatter',
      encodeDecode: 'Encode/Decode',
      textStats: 'Text Statistics',
      crontab: 'Crontab Calculator',
      color: 'Color Converter',
      baseConverter: 'Base Converter',
      rsa: 'RSA Key Pair Generator',
      apiClient: 'REST API Client',
      settings: 'Settings'
    },
    hash: {
      title: 'Hash Calculator',
      input: 'Input Text',
      file: 'File',
      subtitle: 'Enter text content to calculate Hash',
      placeholder: 'Please enter text...',
      filePlaceholder: 'Click to select a file or drag and drop',
      calculate: 'Calculate Hash',
      calculating: 'Calculating...',
      clear: 'Clear',
      algorithms: {
        md5: 'MD5',
        sha1: 'SHA-1',
        sha256: 'SHA-256',
        sha384: 'SHA-384',
        sha512: 'SHA-512'
      },
      algorithmDescriptions: {
        md5: '128-bit hash',
        sha1: '160-bit hash',
        sha256: '256-bit hash',
        sha384: '384-bit hash',
        sha512: '512-bit hash'
      },
      result: 'Result'
    },
    time: {
      title: 'Time Converter',
      timestamp: 'Timestamp',
      date: 'Date',
      timezone: 'Timezone',
      convert: 'Convert',
      clear: 'Clear',
      now: 'Now',
      timestampToDate: 'Timestamp → Date',
      dateToTimestamp: 'Date → Timestamp',
      searchTimezone: 'Search timezone...',
      noTimezoneFound: 'No matching timezone found'
    },
    uuid: {
      title: 'UUID/ULID Generator',
      generate: 'Generate',
      copy: 'Copy',
      generated: 'Generated UUIDs',
      generatedULID: 'Generated ULIDs',
      copyAll: 'Copy All',
      clear: 'Clear',
      count: 'Count',
      includeHyphens: 'Include "-"'
    },
    common: {
      copy: 'Copy',
      copied: 'Copied!',
      clear: 'Clear',
      close: 'Close'
    },
    sidebar: {
      expand: 'Expand sidebar',
      collapse: 'Collapse sidebar'
    },
    update: {
      check: 'Check for Updates',
      checking: 'Checking...',
      available: 'Update Available',
      latest: 'You are using the latest version',
      currentVersion: 'Current Version',
      latestVersion: 'Latest Version',
      download: 'Download',
      error: 'Failed to check for updates',
      releaseNotes: 'Release Notes'
    },
    settings: {
      title: 'Settings',
      appearance: 'Appearance',
      theme: 'Theme',
      language: 'Language',
      languageDescription: 'Choose your preferred language',
      lightMode: 'Light Mode',
      darkMode: 'Dark Mode',
      updates: 'Updates',
      checkForUpdates: 'Check for Updates',
      checkingUpdates: 'Checking for updates...',
      currentVersion: 'Current Version',
      latestVersion: 'Latest Version'
    },
    encodeDecode: {
      encode: 'Encode',
      decode: 'Decode',
      encodeBase64Placeholder: 'Enter text to encode...',
      decodeBase64Placeholder: 'Enter Base64 to decode...',
      encodeURLPlaceholder: 'Enter text to encode...',
      decodeURLPlaceholder: 'Enter URL encoded text...',
      encodeASCIIPlaceholder: 'Enter text to encode...',
      decodeASCIIPlaceholder: 'Enter Unicode escape sequence (e.g., \\u4f60\\u597d)...',
      encodeImageBase64Placeholder: 'Enter Base64 image data...',
      decodeImageBase64Placeholder: 'Enter Base64 image data...',
      selectImage: 'Click to select an image or drag and drop',
      dragDropImage: 'Supports PNG, JPG, GIF, etc.',
      clickToSelectImage: 'Click to select another image',
      download: 'Download',
      downloadImage: 'Download image',
      clear: 'Clear',
      result: 'Result',
      plaintext: 'Plaintext',
      base64: 'Base64',
      urlEncoded: 'URL Encoded',
      ascii: 'ASCII',
      image: 'Image',
      imageBase64: 'Image/Base64',
      base64Encode: 'BASE64 Encode',
      base64Decode: 'BASE64 Decode',
      urlEncode: 'URL Encode',
      urlDecode: 'URL Decode'
    },
    crontab: {
      title: 'Crontab Calculator',
      expression: 'Crontab Expression',
      placeholder: 'e.g., 0 9 * * 1-5',
      format: 'Format: minute hour day month weekday (e.g., 0 9 * * 1-5)',
      description: 'Description',
      nextRuns: 'Future Execution Times',
      clear: 'Clear',
      invalidExpression: 'Invalid crontab expression',
      parseError: 'Failed to parse expression',
      everyMinute: 'Every minute',
      atMinute: 'At minute {minute}',
      atMinutes: 'At minutes {minutes}',
      everyHour: 'Every hour',
      atHour: 'At hour {hour}',
      atHours: 'At hours {hours}',
      everyDay: 'Every day',
      onDay: 'On day {day}',
      onDays: 'On days {days}',
      everyMonth: 'Every month',
      inMonth: 'In {month}',
      inMonths: 'In specified months',
      onWeekday: 'On {weekday}',
      onWeekdays: 'On specified weekdays'
    },
    color: {
      title: 'Color Converter',
      inputFormat: 'Input Format',
      outputFormat: 'Output Format',
      input: 'Input',
      output: 'Output',
      clear: 'Clear',
      placeholder: {
        hex: 'e.g., #FF5733 or FF5733',
        rgb: 'e.g., rgb(255, 87, 51)',
        rgba: 'e.g., rgba(255, 87, 51, 0.8)',
        hsl: 'e.g., hsl(9, 100%, 60%)',
        hsla: 'e.g., hsla(9, 100%, 60%, 0.8)'
      },
      invalidHex: 'Invalid HEX color',
      invalidRGB: 'Invalid RGB/RGBA color',
      invalidHSL: 'Invalid HSL/HSLA color',
      parseError: 'Failed to parse color',
      convertError: 'Failed to convert color'
    },
    baseConverter: {
      title: 'Base Converter',
      inputBase: 'Input Base',
      outputBase: 'Output Base',
      input: 'Input',
      output: 'Output',
      clear: 'Clear',
      binary: 'Binary',
      octal: 'Octal',
      decimal: 'Decimal',
      hexadecimal: 'Hexadecimal',
      inputPlaceholder: 'Enter number...',
      invalidInput: 'Invalid {base} number',
      parseError: 'Failed to parse number',
      convertError: 'Failed to convert number'
    },
    textStats: {
      title: 'Text Statistics',
      placeholder: 'Enter text to analyze...',
      clear: 'Clear',
      statistics: 'Statistics',
      charactersWithSpaces: 'Characters (with spaces)',
      charactersWithoutSpaces: 'Characters (without spaces)',
      words: 'Words',
      lines: 'Lines',
      paragraphs: 'Paragraphs',
      chineseCharacters: 'Chinese Characters',
      englishCharacters: 'English Characters',
      numbers: 'Numbers',
      punctuation: 'Punctuation'
    },
    rsa: {
      title: 'RSA Key Pair Generator',
      keySize: 'Key Size',
      keySizeDescription: 'Larger key sizes provide better security but take longer to generate',
      keyFormat: 'Key Format',
      keyFormatDescription: 'PEM format is human-readable, DER format is binary (shown as hex)',
      generate: 'Generate Key Pair',
      generating: 'Generating...',
      clear: 'Clear',
      publicKey: 'Public Key',
      privateKey: 'Private Key',
      warning: 'Warning:',
      warningDescription: 'Keep your private key secure and never share it with anyone. The private key is sensitive information that should be protected.'
    },
    apiClient: {
      urlPlaceholder: 'Enter API URL...',
      urlRequired: 'URL is required',
      send: 'Send',
      sending: 'Sending...',
      clear: 'Clear',
      headers: 'Headers',
      headerKey: 'Key',
      headerValue: 'Value',
      addHeader: 'Add Header',
      bulkAdd: 'Bulk Add',
      bulkAddHeaders: 'Bulk Add Headers',
      bulkAddHeadersHint: 'Enter headers in the format "key: value" or "key=value", one per line',
      bulkAddHeadersPlaceholder: 'Content-Type: application/json\nAuthorization: Bearer token123\nUser-Agent: MyApp/1.0',
      cancel: 'Cancel',
      add: 'Add',
      body: 'Body',
      jsonPlaceholder: 'Enter JSON body...',
      textPlaceholder: 'Enter text body...',
      xmlPlaceholder: 'Enter XML body...',
      formKey: 'Key',
      formValue: 'Value',
      addFormData: 'Add Form Data',
      selectFile: 'Select File',
      invalidJson: 'Invalid JSON format',
      requestFailed: 'Request failed',
      response: 'Response',
      status: 'Status',
      responseHeaders: 'Response Headers',
      responseBody: 'Response Body',
      noHeaders: 'No headers',
      noResponseBody: 'No response body',
      importCurl: 'Import cURL',
      exportCurl: 'Export cURL',
      importCurlHint: 'Paste your cURL command below and click Import to load the request configuration.',
      importCurlPlaceholder: 'curl -X POST "https://api.example.com/endpoint" -H "Content-Type: application/json" -d \'{"key":"value"}\'',
      exportCurlHint: 'Copy the generated cURL command below.',
      invalidCurlCommand: 'Invalid cURL command format',
      import: 'Import',
      close: 'Close',
      showCode: 'Show code',
      clearAll: 'Clear all'
    }
  },
  zh: {
    app: {
      title: 'Kairoa',
      subtitle: '桌面工具集'
    },
    nav: {
      hash: 'Hash 计算器',
      time: '时间转换',
      uuid: 'UUID/ULID 生成器',
      json: 'JSON 格式化',
      encodeDecode: '编解码',
      textStats: '文本统计',
      crontab: 'Crontab 计算器',
      color: '颜色转换',
      baseConverter: '进制转换',
      rsa: 'RSA 密钥对生成器',
      apiClient: 'REST API 客户端',
      settings: '设置'
    },
    hash: {
      title: '哈希计算器',
      input: '输入文本',
      file: '文件',
      subtitle: '输入需要计算 Hash 的文本内容',
      placeholder: '请输入文本...',
      filePlaceholder: '点击选择文件或拖拽文件到此处',
      calculate: '计算 Hash',
      calculating: '计算中...',
      clear: '清空',
      algorithms: {
        md5: 'MD5',
        sha1: 'SHA-1',
        sha256: 'SHA-256',
        sha384: 'SHA-384',
        sha512: 'SHA-512'
      },
      algorithmDescriptions: {
        md5: '128-bit hash',
        sha1: '160-bit hash',
        sha256: '256-bit hash',
        sha384: '384-bit hash',
        sha512: '512-bit hash'
      },
      result: '结果'
    },
    time: {
      title: '时间转换器',
      timestamp: '时间戳',
      date: '日期',
      timezone: '时区',
      convert: '转换',
      clear: '清空',
      now: '当前时间',
      timestampToDate: '时间戳 → 日期',
      dateToTimestamp: '日期 → 时间戳',
      searchTimezone: '搜索时区...',
      noTimezoneFound: '未找到匹配的时区'
    },
    uuid: {
      title: 'UUID/ULID 生成器',
      generate: '生成',
      copy: '复制',
      generated: '生成的 UUID',
      generatedULID: '生成的 ULID',
      copyAll: '复制全部',
      clear: '清空',
      count: '数量',
      includeHyphens: '包含"-"'
    },
    common: {
      copy: '复制',
      copied: '已复制!',
      clear: '清除',
      close: '关闭'
    },
    sidebar: {
      expand: '展开侧边栏',
      collapse: '收起侧边栏'
    },
    update: {
      check: '检查更新',
      checking: '检查中...',
      available: '有新版本可用',
      latest: '您使用的是最新版本',
      currentVersion: '当前版本',
      latestVersion: '最新版本',
      download: '下载',
      error: '检查更新失败',
      releaseNotes: '更新说明'
    },
    settings: {
      title: '设置',
      appearance: '外观',
      theme: '主题',
      language: '语言',
      languageDescription: '选择您偏好的语言',
      lightMode: '浅色模式',
      darkMode: '深色模式',
      updates: '更新',
      checkForUpdates: '检查更新',
      checkingUpdates: '正在检查更新...',
      currentVersion: '当前版本',
      latestVersion: '最新版本'
    },
    encodeDecode: {
      encode: '编码',
      decode: '解码',
      encodeBase64Placeholder: '输入要编码的文本...',
      decodeBase64Placeholder: '输入要解码的 Base64...',
      encodeURLPlaceholder: '输入要编码的文本...',
      decodeURLPlaceholder: '输入要解码的 URL 编码文本...',
      encodeASCIIPlaceholder: '输入要编码的文本...',
      decodeASCIIPlaceholder: '输入 Unicode 转义序列（例如：\\u4f60\\u597d）...',
      encodeImageBase64Placeholder: '输入 Base64 图片数据...',
      decodeImageBase64Placeholder: '输入 Base64 图片数据...',
      selectImage: '点击选择图片或拖拽图片到此处',
      dragDropImage: '支持 PNG、JPG、GIF 等格式',
      clickToSelectImage: '点击选择其他图片',
      download: '下载',
      downloadImage: '下载图片',
      clear: '清空',
      result: '结果',
      plaintext: '明文',
      base64: 'BASE64',
      urlEncoded: 'URL编码',
      ascii: 'ASCII',
      image: '图片',
      imageBase64: '图片/Base64',
      base64Encode: 'BASE64编码',
      base64Decode: 'BASE64解码',
      urlEncode: 'URL编码',
      urlDecode: 'URL解码'
    },
    crontab: {
      title: 'Crontab 计算器',
      expression: 'Crontab 表达式',
      placeholder: '例如：0 9 * * 1-5',
      format: '格式：分钟 小时 日期 月份 星期（例如：0 9 * * 1-5）',
      description: '说明',
      nextRuns: '未来执行时间点',
      clear: '清空',
      invalidExpression: '无效的 crontab 表达式',
      parseError: '解析表达式失败',
      everyMinute: '每分钟',
      atMinute: '在 {minute} 分',
      atMinutes: '在 {minutes} 分',
      everyHour: '每小时',
      atHour: '在 {hour} 时',
      atHours: '在 {hours} 时',
      everyDay: '每天',
      onDay: '在第 {day} 天',
      onDays: '在第 {days} 天',
      everyMonth: '每月',
      inMonth: '在 {month} 月',
      inMonths: '在指定月份',
      onWeekday: '在 {weekday}',
      onWeekdays: '在指定星期'
    },
    color: {
      title: '颜色转换器',
      inputFormat: '输入格式',
      outputFormat: '输出格式',
      input: '输入',
      output: '输出',
      clear: '清空',
      placeholder: {
        hex: '例如：#FF5733 或 FF5733',
        rgb: '例如：rgb(255, 87, 51)',
        rgba: '例如：rgba(255, 87, 51, 0.8)',
        hsl: '例如：hsl(9, 100%, 60%)',
        hsla: '例如：hsla(9, 100%, 60%, 0.8)'
      },
      invalidHex: '无效的 HEX 颜色',
      invalidRGB: '无效的 RGB/RGBA 颜色',
      invalidHSL: '无效的 HSL/HSLA 颜色',
      parseError: '解析颜色失败',
      convertError: '转换颜色失败'
    },
    baseConverter: {
      title: '进制转换器',
      inputBase: '输入进制',
      outputBase: '输出进制',
      input: '输入',
      output: '输出',
      clear: '清空',
      binary: '二进制',
      octal: '八进制',
      decimal: '十进制',
      hexadecimal: '十六进制',
      inputPlaceholder: '输入数字...',
      invalidInput: '无效的 {base} 进制数字',
      parseError: '解析数字失败',
      convertError: '转换数字失败'
    },
    textStats: {
      title: '文本统计',
      placeholder: '输入文本进行分析...',
      clear: '清空',
      statistics: '统计结果',
      charactersWithSpaces: '字符数（含空格）',
      charactersWithoutSpaces: '字符数（不含空格）',
      words: '单词数',
      lines: '行数',
      paragraphs: '段落数',
      chineseCharacters: '中文字符数',
      englishCharacters: '英文字符数',
      numbers: '数字',
      punctuation: '标点符号'
    },
    rsa: {
      title: 'RSA 密钥对生成器',
      keySize: '密钥长度',
      keySizeDescription: '更大的密钥长度提供更好的安全性，但生成时间更长',
      keyFormat: '密钥格式',
      keyFormatDescription: 'PEM 格式可读性强，DER 格式为二进制（显示为十六进制）',
      generate: '生成密钥对',
      generating: '生成中...',
      clear: '清空',
      publicKey: '公钥',
      privateKey: '私钥',
      warning: '警告：',
      warningDescription: '请妥善保管您的私钥，不要与任何人分享。私钥是敏感信息，应该受到保护。'
    },
    apiClient: {
      urlPlaceholder: '输入 API URL...',
      urlRequired: 'URL 是必填项',
      send: '发送',
      sending: '发送中...',
      clear: '清空',
      headers: '请求头',
      headerKey: '键',
      headerValue: '值',
      addHeader: '添加请求头',
      bulkAdd: '批量添加',
      bulkAddHeaders: '批量添加请求头',
      bulkAddHeadersHint: '每行一个请求头，格式："key: value" 或 "key=value"',
      bulkAddHeadersPlaceholder: 'Content-Type: application/json\nAuthorization: Bearer token123\nUser-Agent: MyApp/1.0',
      cancel: '取消',
      add: '添加',
      body: '请求体',
      jsonPlaceholder: '输入 JSON 请求体...',
      textPlaceholder: '输入文本请求体...',
      xmlPlaceholder: '输入 XML 请求体...',
      formKey: '键',
      formValue: '值',
      addFormData: '添加表单数据',
      selectFile: '选择文件',
      invalidJson: 'JSON 格式无效',
      requestFailed: '请求失败',
      response: '响应',
      status: '状态码',
      responseHeaders: '响应头',
      responseBody: '响应体',
      noHeaders: '无响应头',
      noResponseBody: '无响应体',
      importCurl: '导入 cURL',
      exportCurl: '导出 cURL',
      importCurlHint: '在下方粘贴您的 cURL 命令，点击导入以加载请求配置。',
      importCurlPlaceholder: 'curl -X POST "https://api.example.com/endpoint" -H "Content-Type: application/json" -d \'{"key":"value"}\'',
      exportCurlHint: '复制下方生成的 cURL 命令。',
      invalidCurlCommand: 'cURL 命令格式无效',
      import: '导入',
      close: '关闭',
      showCode: '显示代码',
      clearAll: '清除全部'
    }
  }
};

export type Translations = typeof translations.en;

function createLocaleStore() {
  const { subscribe, set, update } = writable<Locale>('en');

  return {
    subscribe,
    set: (locale: Locale) => {
      if (browser) {
        localStorage.setItem('locale', locale);
      }
      set(locale);
    },
    init: () => {
      if (browser) {
        const stored = localStorage.getItem('locale') as Locale | null;
        const locale = stored || 'en';
        set(locale);
      }
    },
    toggle: () => {
      update((current) => {
        const newLocale = current === 'en' ? 'zh' : 'en';
        if (browser) {
          localStorage.setItem('locale', newLocale);
        }
        return newLocale;
      });
    }
  };
}

export const locale = createLocaleStore();

// 创建响应式的翻译 store
export const translationsStore = derived(locale, ($locale) => {
  return translations[$locale];
});

// 响应式的翻译函数，在组件中使用时需要订阅
export function t(key: string): string {
  let translations: Translations | null = null;
  const unsubscribe = translationsStore.subscribe(t => {
    translations = t;
  });
  unsubscribe();
  
  if (!translations) return key;
  
  const keys = key.split('.');
  let value: any = translations;
  for (const k of keys) {
    value = value?.[k];
  }
  return value || key;
}

// 推荐使用这个函数，它返回一个响应式的翻译函数
export function useTranslations() {
  let $translations: Translations;
  const unsubscribe = translationsStore.subscribe(t => $translations = t);
  
  return {
    t: (key: string): string => {
      const keys = key.split('.');
      let value: any = $translations;
      for (const k of keys) {
        value = value?.[k];
      }
      return value || key;
    },
    locale,
    unsubscribe
  };
}

