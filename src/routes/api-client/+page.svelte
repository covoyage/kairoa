<script lang="ts">
  import { translationsStore } from '$lib/stores/i18n';
  import { Copy, Check, Send, Trash2, Plus, X, Download, Upload, ChevronDown, Code } from 'lucide-svelte';
  import { browser } from '$app/environment';
  
  // 动态导入 Tauri API（仅在浏览器环境中可用）
  let invokeFn: ((cmd: string, args?: any) => Promise<any>) | null = $state(null);
  let isTauriAvailable = $state(false);
  
  if (browser) {
    // 检查是否在 Tauri 环境中
    if (typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window) {
      isTauriAvailable = true;
      // 异步加载 Tauri API
      import('@tauri-apps/api/core')
        .then((module) => {
          invokeFn = module.invoke;
        })
        .catch((err) => {
          console.error('Failed to load Tauri API:', err);
          isTauriAvailable = false;
        });
    }
  }
  
  type HttpMethod = 'GET' | 'POST' | 'PUT' | 'DELETE' | 'PATCH' | 'HEAD' | 'OPTIONS';
  type BodyType = 'json' | 'text' | 'xml' | 'form-data' | 'url-encoded' | 'none';
  
  interface Header {
    key: string;
    value: string;
    enabled: boolean;
  }
  
  interface FormDataItem {
    key: string;
    value: string;
    enabled: boolean;
    type: 'text' | 'file';
    file: File | null;
  }

  interface TabData {
    id: string;
    name: string;
    method: HttpMethod;
    url: string;
    headers: Header[];
    bodyType: BodyType;
    bodyJson: string;
    bodyText: string;
    bodyXml: string;
    formData: FormDataItem[];
    isSending: boolean;
    responseStatus: number | null;
    responseHeaders: Record<string, string>;
    responseBody: string;
    responseTime: number | null;
    error: string;
    copied: boolean;
  }
  
  let tabs = $state<TabData[]>([
    {
      id: crypto.randomUUID(),
      name: 'New Request',
      method: 'GET',
      url: '',
      headers: [{ key: '', value: '', enabled: true }],
      bodyType: 'none',
      bodyJson: '',
      bodyText: '',
      bodyXml: '',
      formData: [{ key: '', value: '', enabled: true, type: 'text', file: null }],
      isSending: false,
      responseStatus: null,
      responseHeaders: {},
      responseBody: '',
      responseTime: null,
      error: '',
      copied: false
    }
  ]);
  
  let activeTabId = $state('');

  // 初始化 activeTabId
  $effect(() => {
    if (!activeTabId && tabs.length > 0) {
      activeTabId = tabs[0].id;
    }
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

  // 获取当前激活的标签页
  let activeTab = $derived(tabs.find(tab => tab.id === activeTabId) || tabs[0]);

  // 添加新标签页
  function addTab() {
    const newTab: TabData = {
      id: crypto.randomUUID(),
      name: 'New Request',
      method: 'GET',
      url: '',
      headers: [{ key: '', value: '', enabled: true }],
      bodyType: 'none',
      bodyJson: '',
      bodyText: '',
      bodyXml: '',
      formData: [{ key: '', value: '', enabled: true, type: 'text', file: null }],
      isSending: false,
      responseStatus: null,
      responseHeaders: {},
      responseBody: '',
      responseTime: null,
      error: '',
      copied: false
    };
    tabs = [...tabs, newTab];
    activeTabId = newTab.id;
  }

  // 删除标签页
  function removeTab(tabId: string) {
    if (tabs.length === 1) return; // 至少保留一个标签页
    
    tabs = tabs.filter(tab => tab.id !== tabId);
    
    // 如果删除的是当前激活的标签页，切换到第一个标签页
    if (activeTabId === tabId) {
      activeTabId = tabs[0].id;
    }
  }

  // 切换标签页
  function setActiveTab(tabId: string) {
    activeTabId = tabId;
  }

  // 更新标签页名称（基于 URL）
  function updateTabName(tab: TabData) {
    if (tab.url.trim()) {
      try {
        const url = new URL(tab.url);
        tab.name = url.pathname || url.hostname || 'New Request';
      } catch {
        tab.name = tab.url.substring(0, 20) || 'New Request';
      }
    } else {
      tab.name = 'New Request';
    }
  }

  function addHeader(tab: TabData) {
    tab.headers = [...tab.headers, { key: '', value: '', enabled: true }];
  }

  function removeHeader(tab: TabData, index: number) {
    tab.headers = tab.headers.filter((_, i) => i !== index);
    if (tab.headers.length === 0) {
      tab.headers = [{ key: '', value: '', enabled: true }];
    }
  }

  let showBulkHeaderDialog = $state(false);
  let bulkHeaderText = $state('');
  let showImportCurlDialog = $state(false);
  let curlCommandText = $state('');
  let showExportCurlDialog = $state(false);
  let generatedCurlCommand = $state('');
  let curlCopied = $state(false);
  let showDropdown = $state(false);

  function parseBulkHeaders(text: string): Header[] {
    const headers: Header[] = [];
    const lines = text.split('\n');
    
    for (const line of lines) {
      const trimmed = line.trim();
      if (!trimmed) continue;
      
      // 支持 key: value 或 key=value 格式
      let key = '';
      let value = '';
      
      if (trimmed.includes(':')) {
        const colonIndex = trimmed.indexOf(':');
        key = trimmed.substring(0, colonIndex).trim();
        value = trimmed.substring(colonIndex + 1).trim();
      } else if (trimmed.includes('=')) {
        const equalIndex = trimmed.indexOf('=');
        key = trimmed.substring(0, equalIndex).trim();
        value = trimmed.substring(equalIndex + 1).trim();
      } else {
        continue; // 跳过无法解析的行
      }
      
      if (key && value) {
        headers.push({ key, value, enabled: true });
      }
    }
    
    return headers;
  }

  function addBulkHeaders(tab: TabData) {
    const parsedHeaders = parseBulkHeaders(bulkHeaderText);
    if (parsedHeaders.length > 0) {
      // 移除空的 header（如果存在）
      const existingHeaders = tab.headers.filter(h => h.key.trim() || h.value.trim());
      tab.headers = [...existingHeaders, ...parsedHeaders];
      bulkHeaderText = '';
      showBulkHeaderDialog = false;
    }
  }

  function addFormDataItem(tab: TabData) {
    tab.formData = [...tab.formData, { key: '', value: '', enabled: true, type: 'text', file: null }];
  }

  function removeFormDataItem(tab: TabData, index: number) {
    // 清理文件引用（如果存在）
    if (tab.formData[index].file) {
      tab.formData[index].file = null;
    }
    tab.formData = tab.formData.filter((_, i) => i !== index);
    if (tab.formData.length === 0) {
      tab.formData = [{ key: '', value: '', enabled: true, type: 'text', file: null }];
    }
  }

  function handleFileSelect(tab: TabData, index: number, event: Event) {
    const input = event.target as HTMLInputElement;
    const file = input.files?.[0];
    if (file) {
      // 文件大小限制（50MB）
      const maxSize = 50 * 1024 * 1024; // 50MB
      if (file.size > maxSize) {
        tab.error = `File is too large (${(file.size / 1024 / 1024).toFixed(2)}MB). Maximum size is 50MB.`;
        // 重置文件输入
        input.value = '';
        return;
      }
      
      // 清理旧的文件引用
      if (tab.formData[index].file) {
        // 旧文件会被垃圾回收
        tab.formData[index].file = null;
      }
      
      tab.formData[index].file = file;
      tab.formData[index].value = file.name;
      tab.error = ''; // 清除之前的错误
    }
  }

  async function sendRequest(tab: TabData) {
    if (!tab.url.trim()) {
      tab.error = t('apiClient.urlRequired');
      return;
    }

    tab.isSending = true;
    tab.error = '';
    tab.responseStatus = null;
    tab.responseHeaders = {};
    tab.responseBody = '';
    tab.responseTime = null;

    const startTime = Date.now();

    try {
      // 构建请求头
      const requestHeaders: Record<string, string> = {};
      tab.headers.forEach((header) => {
        if (header.enabled && header.key.trim() && header.value.trim()) {
          requestHeaders[header.key.trim()] = header.value.trim();
        }
      });

      // 构建请求体
      let requestBody: string | undefined = undefined;
      
      if (tab.bodyType === 'json' && tab.bodyJson.trim()) {
        try {
          // 验证并压缩 JSON（去除多余空格，保持单行，除非已经是格式化的）
          const parsed = JSON.parse(tab.bodyJson);
          // 如果 JSON 已经是压缩格式（单行），保持原样；否则压缩
          const isCompressed = !tab.bodyJson.includes('\n') && tab.bodyJson.trim() === tab.bodyJson;
          requestBody = isCompressed ? tab.bodyJson : JSON.stringify(parsed);
          // 只有在没有 Content-Type header 时才添加
          if (!requestHeaders['Content-Type'] && !requestHeaders['content-type']) {
            requestHeaders['Content-Type'] = 'application/json';
          }
        } catch (e) {
          tab.error = t('apiClient.invalidJson');
          tab.isSending = false;
          return;
        }
      } else if (tab.bodyType === 'text' && tab.bodyText.trim()) {
        requestBody = tab.bodyText;
        // 只有在没有 Content-Type header 时才添加
        if (!requestHeaders['Content-Type'] && !requestHeaders['content-type']) {
          requestHeaders['Content-Type'] = 'text/plain';
        }
      } else if (tab.bodyType === 'xml' && tab.bodyXml.trim()) {
        requestBody = tab.bodyXml;
        // 只有在没有 Content-Type header 时才添加
        if (!requestHeaders['Content-Type'] && !requestHeaders['content-type']) {
          requestHeaders['Content-Type'] = 'application/xml';
        }
      } else if (tab.bodyType === 'form-data') {
        // 检查是否有文件需要上传
        const hasFiles = tab.formData.some(item => item.enabled && item.type === 'file' && item.file);
        
        // 检查是否在 Tauri 环境中
        const isInTauri = browser && typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;
        
        if (hasFiles) {
          // 如果有文件
          if (!isInTauri) {
            // 浏览器环境：使用 FormData API
            const formData = new FormData();
            
            tab.formData.forEach((item) => {
              if (item.enabled && item.key.trim()) {
                if (item.type === 'file' && item.file) {
                  formData.append(item.key.trim(), item.file);
                } else if (item.type === 'text' && item.value.trim()) {
                  formData.append(item.key.trim(), item.value.trim());
                }
              }
            });
            
            requestBody = formData as any;
            // 不要设置 Content-Type，让浏览器自动设置（包含 boundary）
            delete requestHeaders['Content-Type'];
          } else {
            // Tauri 环境：将 FormData 转换为 base64 格式
            const formDataEntries: Array<{ key: string; value: string; type: 'text' | 'file'; filename?: string }> = [];
            
            for (const item of tab.formData) {
              if (item.enabled && item.key.trim()) {
                if (item.type === 'file' && item.file) {
                  // 读取文件为 base64
                  const arrayBuffer = await item.file.arrayBuffer();
                  const uint8Array = new Uint8Array(arrayBuffer);
                  // 使用块处理避免堆栈溢出
                  let binaryString = '';
                  const chunkSize = 8192; // 8KB chunks
                  for (let i = 0; i < uint8Array.length; i += chunkSize) {
                    const chunk = uint8Array.slice(i, i + chunkSize);
                    // 逐个处理字符以避免堆栈溢出
                    for (let j = 0; j < chunk.length; j++) {
                      binaryString += String.fromCharCode(chunk[j]);
                    }
                  }
                  const base64 = btoa(binaryString);
                  formDataEntries.push({
                    key: item.key.trim(),
                    value: base64,
                    type: 'file',
                    filename: item.file.name
                  });
                } else if (item.type === 'text' && item.value.trim()) {
                  formDataEntries.push({
                    key: item.key.trim(),
                    value: item.value.trim(),
                    type: 'text'
                  });
                }
              }
            }
            
            // 将 formDataEntries 序列化为 JSON，传递给 Rust 后端
            requestBody = JSON.stringify({ type: 'multipart', entries: formDataEntries });
            requestHeaders['Content-Type'] = 'application/json'; // 临时设置，Rust 后端会处理
          }
        } else {
          // 没有文件，使用文本格式的 multipart/form-data
          const boundary = `----WebKitFormBoundary${Date.now()}${Math.random().toString(36).substring(2)}`;
          const parts: string[] = [];
          
          tab.formData.forEach((item) => {
            if (item.enabled && item.key.trim() && item.type === 'text' && item.value.trim()) {
              parts.push(`--${boundary}`);
              parts.push(`Content-Disposition: form-data; name="${item.key.trim()}"`);
              parts.push('');
              parts.push(item.value.trim());
            }
          });
          parts.push(`--${boundary}--`);
          
          requestBody = parts.join('\r\n');
          requestHeaders['Content-Type'] = requestHeaders['Content-Type'] || `multipart/form-data; boundary=${boundary}`;
        }
      } else if (tab.bodyType === 'url-encoded') {
        // FormData 转换为 application/x-www-form-urlencoded 格式
        const formDataPairs: string[] = [];
        tab.formData.forEach((item) => {
          if (item.enabled && item.key.trim() && item.value.trim()) {
            formDataPairs.push(`${encodeURIComponent(item.key.trim())}=${encodeURIComponent(item.value.trim())}`);
          }
        });
        requestBody = formDataPairs.join('&');
        requestHeaders['Content-Type'] = requestHeaders['Content-Type'] || 'application/x-www-form-urlencoded';
      }

      // 检查是否在 Tauri 环境中，如果是则优先使用 Tauri API（绕过 CORS）
      const isInTauri = browser && typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;
      
      // 如果在 Tauri 环境中但 API 还没加载，等待一下
      if (isInTauri && !invokeFn) {
        // 等待最多 1 秒让 Tauri API 加载完成
        for (let i = 0; i < 10; i++) {
          await new Promise(resolve => setTimeout(resolve, 100));
          if (invokeFn) break;
        }
      }

      // 如果可以使用 Tauri API，使用它（绕过 CORS）
      if (isInTauri && invokeFn) {
        // 如果 requestBody 是 FormData（浏览器环境），需要转换为字符串
        let bodyForRequest: string | undefined;
        if (requestBody && typeof requestBody === 'object' && 'append' in requestBody) {
          // 这种情况不应该在 Tauri 环境中发生，因为我们已经转换了
          // 但为了安全起见，我们仍然处理它
          bodyForRequest = undefined;
        } else {
          bodyForRequest = typeof requestBody === 'string' ? requestBody : undefined;
        }
        
        // 使用 Tauri 命令发送请求（绕过 CORS）
        const response = await invokeFn('http_request', {
          request: {
            url: tab.url,
            method: tab.method,
            headers: requestHeaders,
            body: bodyForRequest
          }
        }) as {
          status: number;
          headers: Record<string, string>;
          body: string;
          error: string | null;
        };

        const endTime = Date.now();
        tab.responseTime = endTime - startTime;

        if (response.error) {
          tab.error = response.error;
          tab.responseStatus = null;
        } else {
          tab.responseStatus = response.status;
          tab.responseHeaders = response.headers;
          
          // 尝试格式化 JSON 响应
          if (response.body.trim()) {
            try {
              const json = JSON.parse(response.body);
              tab.responseBody = JSON.stringify(json, null, 2);
            } catch {
              tab.responseBody = response.body;
            }
          } else {
            tab.responseBody = response.body;
          }

          // 更新标签页名称
          updateTabName(tab);
        }
      } else {
        // 回退到使用 fetch（可能在浏览器中运行或 Tauri API 不可用）
        const requestOptions: RequestInit = {
          method: tab.method,
          headers: requestHeaders,
        };

        if (requestBody !== undefined) {
          // 如果是 FormData，不要设置 Content-Type，让浏览器自动设置
          if (requestBody && typeof requestBody === 'object' && 'append' in requestBody) {
            // 删除 Content-Type，让浏览器自动设置 multipart/form-data boundary
            delete (requestOptions.headers as Record<string, string>)['Content-Type'];
          }
          requestOptions.body = requestBody as any;
        }

        const response = await fetch(tab.url, requestOptions);
        const endTime = Date.now();
        tab.responseTime = endTime - startTime;

        // 获取响应状态
        tab.responseStatus = response.status;

        // 获取响应头
        response.headers.forEach((value, key) => {
          tab.responseHeaders[key] = value;
        });

        // 获取响应体
        const contentType = response.headers.get('content-type') || '';
        if (contentType.includes('application/json')) {
          try {
            const json = await response.json();
            tab.responseBody = JSON.stringify(json, null, 2);
          } catch (e) {
            tab.responseBody = await response.text();
          }
        } else {
          tab.responseBody = await response.text();
        }

        // 更新标签页名称
        updateTabName(tab);
      }
    } catch (err) {
      tab.error = err instanceof Error ? err.message : t('apiClient.requestFailed');
      tab.responseTime = Date.now() - startTime;
    } finally {
      tab.isSending = false;
    }
  }

  async function copyResponse(tab: TabData) {
    const text = tab.responseBody || JSON.stringify({ status: tab.responseStatus, headers: tab.responseHeaders, body: tab.responseBody }, null, 2);
    if (!text) return;
    
    try {
      await navigator.clipboard.writeText(text);
      tab.copied = true;
      setTimeout(() => {
        tab.copied = false;
      }, 2000);
    } catch (error) {
      console.error('Failed to copy:', error);
    }
  }

  function clear(tab: TabData) {
    tab.url = '';
    tab.headers = [{ key: '', value: '', enabled: true }];
    tab.bodyType = 'none';
    tab.bodyJson = '';
    tab.bodyText = '';
    tab.bodyXml = '';
    tab.formData = [{ key: '', value: '', enabled: true, type: 'text', file: null }];
    tab.responseStatus = null;
    tab.responseHeaders = {};
    tab.responseBody = '';
    tab.responseTime = null;
    tab.error = '';
    tab.name = 'New Request';
  }

  // 解析 curl 命令
  function parseCurlCommand(curlText: string): Partial<TabData> | null {
    try {
      // 先处理多行命令（移除反斜杠和换行符）
      let curl = curlText.trim();
      if (!curl.startsWith('curl')) {
        return null;
      }
      
      // 合并多行命令（移除行尾的反斜杠和换行符）
      curl = curl.replace(/\\\s*\n\s*/g, ' ').replace(/\n/g, ' ').replace(/\s+/g, ' ').trim();

      const result: Partial<TabData> = {
        method: 'GET',
        url: '',
        headers: [],
        bodyType: 'none' as BodyType,
        bodyJson: '',
        bodyText: '',
        bodyXml: '',
        formData: []
      };

      // 提取方法
      const methodMatch = curl.match(/--request\s+(\w+)|-X\s+(\w+)/i);
      if (methodMatch) {
        const method = (methodMatch[1] || methodMatch[2]).toUpperCase();
        if (['GET', 'POST', 'PUT', 'DELETE', 'PATCH', 'HEAD', 'OPTIONS'].includes(method)) {
          result.method = method as HttpMethod;
        }
      } else {
        // 如果没有明确指定方法，检查是否有 body 参数，如果有则默认为 POST
        // 根据 curl 最佳实践，以下参数会自动将请求方法设置为 POST
        const hasBodyParams = /(?:-d|--data|--data-raw|--data-binary|--data-urlencode|--form|-F)/i.test(curl);
        if (hasBodyParams) {
          result.method = 'POST';
        }
      }

      // 提取 URL（支持单引号、双引号或无引号）
      // 先尝试匹配带协议的 URL（在 curl 之后，所有选项之前）
      // 查找第一个非选项参数（不在 - 或 -- 之后，且不是 -X/-H/-F 等选项的值）
      const parts = curl.split(/\s+/);
      let urlFound = false;
      
      for (let i = 1; i < parts.length; i++) {
        const part = parts[i].replace(/^['"]|['"]$/g, '');
        
        // 跳过选项
        if (part.startsWith('-')) {
          // 某些选项后面有值，需要跳过下一个参数
          if (['-X', '--request', '-H', '--header', '-F', '--form', 
               '--data', '--data-raw', '--data-urlencode', '--data-binary'].includes(part)) {
            i++; // 跳过下一个参数（选项值）
          }
          continue;
        }
        
        // 如果看起来像 URL（包含 :// 或者看起来像域名+路径）
        if (part.includes('://') || (part.includes('.') && (part.includes('/') || part.includes('?')))) {
          result.url = part;
          urlFound = true;
          break;
        }
      }
      
      // 如果没有找到 URL，尝试更宽松的匹配
      if (!urlFound) {
        const urlMatch = curl.match(/curl\s+(?:['"]?)(https?:\/\/[^\s'"]+)(?:['"]?)/i);
        if (urlMatch && urlMatch[1]) {
          result.url = urlMatch[1].replace(/^['"]|['"]$/g, '');
        }
      }

      // 提取 headers
      const headerRegex = /(?:-H|--header)\s+['"]([^'"]+)['"]/gi;
      let headerMatch;
      const headers: Header[] = [];
      while ((headerMatch = headerRegex.exec(curl)) !== null) {
        const headerStr = headerMatch[1];
        const colonIndex = headerStr.indexOf(':');
        if (colonIndex > 0) {
          const key = headerStr.substring(0, colonIndex).trim();
          const value = headerStr.substring(colonIndex + 1).trim();
          if (key && value) {
            headers.push({ key, value, enabled: true });
          }
        }
      }
      if (headers.length > 0) {
        result.headers = headers;
      }

      // 提取 body
      // 使用更精确的正则表达式匹配，支持单引号、双引号或无引号
      const dataPatterns = [
        { pattern: /--data-raw\s+((?:'[^']*'|"[^"]*"|[^\s]+))/gi, type: 'raw' },
        { pattern: /--data-urlencode\s+((?:'[^']*'|"[^"]*"|[^\s]+))/gi, type: 'urlencode' },
        { pattern: /--data-binary\s+((?:'[^']*'|"[^"]*"|[^\s]+))/gi, type: 'binary' },
        { pattern: /--data\s+((?:'[^']*'|"[^"]*"|[^\s]+))/gi, type: 'data' }
      ];
      
      let bodyContent = '';
      let bodyType = '';
      
      for (const { pattern, type } of dataPatterns) {
        const matches = [...curl.matchAll(pattern)];
        if (matches.length > 0) {
          bodyType = type;
          matches.forEach(match => {
            if (match[1]) {
              let content = match[1].trim();
              // 移除引号
              if ((content.startsWith('"') && content.endsWith('"')) || 
                  (content.startsWith("'") && content.endsWith("'"))) {
                content = content.slice(1, -1);
              }
              // 处理转义
              content = content.replace(/\\'/g, "'").replace(/\\"/g, '"');
              bodyContent += content;
            }
          });
          break; // 只处理第一个匹配的类型
        }
      }
      
      if (bodyContent.trim()) {
        if (bodyType === 'urlencode') {
          // --data-urlencode 通常用于 URL encoded 数据
          // 检查是否是 key=value 格式
          if (bodyContent.includes('=') && !bodyContent.includes('{')) {
            result.bodyType = 'url-encoded';
            const pairs = bodyContent.split('&');
            const formData: FormDataItem[] = [];
            pairs.forEach(pair => {
              const equalIndex = pair.indexOf('=');
              if (equalIndex > 0) {
                try {
                  const key = decodeURIComponent(pair.substring(0, equalIndex));
                  const value = decodeURIComponent(pair.substring(equalIndex + 1));
                  formData.push({ key, value, enabled: true, type: 'text', file: null });
                } catch {
                  // 如果解码失败，使用原始值
                  const key = pair.substring(0, equalIndex);
                  const value = pair.substring(equalIndex + 1);
                  formData.push({ key, value, enabled: true, type: 'text', file: null });
                }
              }
            });
            result.formData = formData;
          } else {
            // 否则作为普通文本处理
            try {
              result.bodyType = 'text';
              result.bodyText = decodeURIComponent(bodyContent);
            } catch {
              result.bodyType = 'text';
              result.bodyText = bodyContent;
            }
          }
        } else {
          // 尝试解析为 JSON
          try {
            const json = JSON.parse(bodyContent);
            result.bodyType = 'json';
            result.bodyJson = JSON.stringify(json, null, 2);
          } catch {
            // 检查是否是 XML
            if (bodyContent.trim().startsWith('<')) {
              result.bodyType = 'xml';
              result.bodyXml = bodyContent;
            } else {
              result.bodyType = 'text';
              result.bodyText = bodyContent;
            }
          }
        }
      }

      // 提取 form-data
      const formRegex = /(?:-F|--form)\s+([^\s]+|'[^']*'|"[^"]*")/gi;
      let formMatch;
      const formData: FormDataItem[] = [];
      
      while ((formMatch = formRegex.exec(curl)) !== null) {
        let formValue = formMatch[1];
        // 移除引号
        if ((formValue.startsWith('"') && formValue.endsWith('"')) || 
            (formValue.startsWith("'") && formValue.endsWith("'"))) {
          formValue = formValue.slice(1, -1);
        }
        
        if (formValue.includes('=')) {
          const equalIndex = formValue.indexOf('=');
          const key = formValue.substring(0, equalIndex).trim();
          let value = formValue.substring(equalIndex + 1).trim();
          
          if (key && value) {
            // 检查是否是文件
            if (value.startsWith('@')) {
              formData.push({
                key,
                value: value.substring(1),
                enabled: true,
                type: 'file',
                file: null
              });
            } else {
              formData.push({
                key,
                value,
                enabled: true,
                type: 'text',
                file: null
              });
            }
          }
        }
      }
      
      if (formData.length > 0) {
        result.bodyType = 'form-data';
        result.formData = formData;
      }

      return result;
    } catch (error) {
      console.error('Failed to parse curl command:', error);
      return null;
    }
  }

  // 导入 curl 命令
  function importCurlCommand() {
    const parsed = parseCurlCommand(curlCommandText);
    if (!parsed) {
      activeTab.error = t('apiClient.invalidCurlCommand');
      showImportCurlDialog = false;
      curlCommandText = '';
      return;
    }

    // 应用解析结果
    if (parsed.method) activeTab.method = parsed.method;
    if (parsed.url) activeTab.url = parsed.url;
    if (parsed.headers && parsed.headers.length > 0) {
      activeTab.headers = parsed.headers;
    }
    if (parsed.bodyType) {
      activeTab.bodyType = parsed.bodyType;
      if (parsed.bodyJson) activeTab.bodyJson = parsed.bodyJson;
      if (parsed.bodyText) activeTab.bodyText = parsed.bodyText;
      if (parsed.bodyXml) activeTab.bodyXml = parsed.bodyXml;
    }
    if (parsed.formData && parsed.formData.length > 0) {
      activeTab.formData = parsed.formData;
    }

    showImportCurlDialog = false;
    curlCommandText = '';
    updateTabName(activeTab);
  }

  // 转义 shell 字符串中的特殊字符
  function escapeShellString(str: string, useDoubleQuotes: boolean = false): string {
    if (useDoubleQuotes) {
      // 双引号中需要转义：$ ` " \
      return str.replace(/\\/g, '\\\\')
                .replace(/"/g, '\\"')
                .replace(/\$/g, '\\$')
                .replace(/`/g, '\\`');
    } else {
      // 单引号中需要转义：单引号本身（通过结束引号、转义、开始引号的方式）
      return str.replace(/'/g, "'\\''");
    }
  }

  // 生成 curl 命令（美化格式）
  function generateCurlCommand(tab: TabData): string {
    const parts: string[] = [];
    
    // 基础命令、方法和 URL 放在同一行
    let firstLine = 'curl';
    
    // 添加方法
    if (tab.method !== 'GET') {
      firstLine += ` -X ${tab.method}`;
    }

    // 添加 URL（使用双引号，因为 URL 可能包含特殊字符）
    if (tab.url) {
      const escapedUrl = escapeShellString(tab.url, true);
      firstLine += ` "${escapedUrl}"`;
    }

    // 检查是否有 headers 或 body
    const enabledHeaders = tab.headers.filter(header => 
      header.enabled && header.key.trim() && header.value.trim()
    );
    
    const hasBody = tab.bodyType !== 'none' && (
      (tab.bodyType === 'json' && tab.bodyJson.trim()) ||
      (tab.bodyType === 'text' && tab.bodyText.trim()) ||
      (tab.bodyType === 'xml' && tab.bodyXml.trim()) ||
      (tab.bodyType === 'form-data' && tab.formData.some(item => item.enabled && item.key.trim())) ||
      (tab.bodyType === 'url-encoded' && tab.formData.some(item => item.enabled && item.key.trim() && item.value.trim()))
    );
    
    const hasMoreParams = enabledHeaders.length > 0 || hasBody;
    
    // 如果有后续参数，第一行末尾添加反斜杠
    if (hasMoreParams) {
      firstLine += ' \\';
    }
    
    parts.push(firstLine);

    // 添加 headers
    enabledHeaders.forEach((header, index) => {
      const isLast = index === enabledHeaders.length - 1;
      const hasMore = !isLast || hasBody;
      
      // Header 值中可能包含特殊字符，需要转义
      const escapedKey = escapeShellString(header.key.trim(), true);
      const escapedValue = escapeShellString(header.value.trim(), true);
      parts.push(`  -H "${escapedKey}: ${escapedValue}"${hasMore ? ' \\' : ''}`);
    });

    // 添加 body
    if (tab.bodyType === 'json' && tab.bodyJson.trim()) {
      // JSON 使用单引号包裹，但需要压缩为单行以避免多行问题
      let jsonBody = tab.bodyJson.trim();
      try {
        const parsed = JSON.parse(jsonBody);
        // 压缩为单行，避免多行 JSON 在 shell 中的问题
        jsonBody = JSON.stringify(parsed);
      } catch {
        // 如果解析失败，保持原样
      }
      
      // 转义单引号（通过结束引号、转义、开始引号的方式）
      const escapedJson = escapeShellString(jsonBody, false);
      parts.push(`  --data-raw '${escapedJson}'`);
      
    } else if (tab.bodyType === 'text' && tab.bodyText.trim()) {
      const textBody = tab.bodyText.trim();
      // 对于多行文本，使用单引号，并转义单引号
      const escapedText = escapeShellString(textBody, false);
      parts.push(`  --data-raw '${escapedText}'`);
      
    } else if (tab.bodyType === 'xml' && tab.bodyXml.trim()) {
      const xmlBody = tab.bodyXml.trim();
      // 对于多行 XML，使用单引号，并转义单引号
      const escapedXml = escapeShellString(xmlBody, false);
      parts.push(`  --data-raw '${escapedXml}'`);
      
    } else if (tab.bodyType === 'form-data') {
      const enabledFormData = tab.formData.filter(item => 
        item.enabled && item.key.trim() && (
          (item.type === 'text' && item.value.trim()) ||
          (item.type === 'file' && item.file)
        )
      );
      
      enabledFormData.forEach((item, index) => {
        const isLast = index === enabledFormData.length - 1;
        if (item.type === 'file' && item.file) {
          // 文件名可能需要转义
          const escapedKey = escapeShellString(item.key.trim(), true);
          const escapedFileName = escapeShellString(item.file.name, true);
          parts.push(`  -F "${escapedKey}=@${escapedFileName}"${isLast ? '' : ' \\'}`);
        } else if (item.type === 'text' && item.value.trim()) {
          // Form data 值需要转义
          const escapedKey = escapeShellString(item.key.trim(), true);
          const escapedValue = escapeShellString(item.value.trim(), true);
          parts.push(`  -F "${escapedKey}=${escapedValue}"${isLast ? '' : ' \\'}`);
        }
      });
    } else if (tab.bodyType === 'url-encoded') {
      const pairs: string[] = [];
      tab.formData.forEach(item => {
        if (item.enabled && item.key.trim() && item.value.trim()) {
          pairs.push(`${encodeURIComponent(item.key.trim())}=${encodeURIComponent(item.value.trim())}`);
        }
      });
      if (pairs.length > 0) {
        // URL encoded 数据已经编码过了，直接用双引号包裹
        parts.push(`  --data-urlencode "${pairs.join('&')}"`);
      }
    }

    return parts.join('\n');
  }

  // 导出 curl 命令
  function exportCurlCommand(tab: TabData) {
    generatedCurlCommand = generateCurlCommand(tab);
    showExportCurlDialog = true;
  }

  // 复制 curl 命令
  async function copyCurlCommand() {
    try {
      await navigator.clipboard.writeText(generatedCurlCommand);
      curlCopied = true;
      setTimeout(() => {
        curlCopied = false;
      }, 2000);
    } catch (error) {
      console.error('Failed to copy curl command:', error);
    }
  }

  function getStatusColor(status: number | null): string {
    if (!status) return '';
    if (status >= 200 && status < 300) return 'text-green-600 dark:text-green-400';
    if (status >= 300 && status < 400) return 'text-blue-600 dark:text-blue-400';
    if (status >= 400 && status < 500) return 'text-yellow-600 dark:text-yellow-400';
    if (status >= 500) return 'text-red-600 dark:text-red-400';
    return '';
  }

  // 监听方法变化，自动调整 body 类型
  $effect(() => {
    const currentTab = tabs.find(tab => tab.id === activeTabId);
    if (!currentTab) return;
    
    const method = currentTab.method;
    if (method === 'GET' || method === 'HEAD' || method === 'OPTIONS') {
      if (currentTab.bodyType !== 'none') {
        currentTab.bodyType = 'none';
      }
      // 清理 formData 中的文件引用
      const hasFiles = currentTab.formData.some(item => item.file);
      if (hasFiles) {
        currentTab.formData.forEach(item => {
          if (item.file) {
            item.file = null;
          }
        });
      }
    }
  });

  // 监听 bodyType 变化，清理不相关的数据
  $effect(() => {
    const currentTab = tabs.find(tab => tab.id === activeTabId);
    if (!currentTab) return;
    
    const bodyType = currentTab.bodyType;
    if (bodyType === 'none') {
      if (currentTab.bodyJson) currentTab.bodyJson = '';
      if (currentTab.bodyText) currentTab.bodyText = '';
      if (currentTab.bodyXml) currentTab.bodyXml = '';
      // 清理 formData 中的文件引用
      const hasFiles = currentTab.formData.some(item => item.file);
      if (hasFiles) {
        currentTab.formData.forEach(item => {
          if (item.file) {
            item.file = null;
          }
        });
      }
      // 只在需要时重置 formData
      if (currentTab.formData.length !== 1 || currentTab.formData[0].key !== '' || currentTab.formData[0].value !== '' || currentTab.formData[0].type !== 'text') {
        currentTab.formData = [{ key: '', value: '', enabled: true, type: 'text', file: null }];
      }
    } else if (bodyType !== 'form-data' && bodyType !== 'url-encoded') {
      // 当切换到非 form-data 类型时，清理 formData 中的文件引用
      const hasFiles = currentTab.formData.some(item => item.file);
      if (hasFiles) {
        currentTab.formData.forEach(item => {
          if (item.file) {
            item.file = null;
          }
        });
      }
    }
  });

  // 监听 URL 变化，更新标签页名称
  $effect(() => {
    if (activeTab.url) {
      updateTabName(activeTab);
    }
  });

  // 点击外部关闭下拉菜单和计算位置
  let dropdownRef: HTMLElement | null = $state(null);
  let dropdownButtonRef: HTMLElement | null = $state(null);
  
  if (browser) {
    $effect(() => {
      if (showDropdown && dropdownRef && dropdownButtonRef) {
        // 计算下拉菜单的位置
        const updatePosition = () => {
          const buttonRect = dropdownButtonRef!.getBoundingClientRect();
          dropdownRef!.style.top = `${buttonRect.bottom + 4}px`;
          dropdownRef!.style.right = `${window.innerWidth - buttonRect.right}px`;
        };
        
        updatePosition();
        window.addEventListener('resize', updatePosition);
        window.addEventListener('scroll', updatePosition, true);
        
        const handleClickOutside = (e: MouseEvent) => {
          const target = e.target as HTMLElement;
          if (!target.closest('.relative') && !target.closest('[data-dropdown]')) {
            showDropdown = false;
          }
        };
        document.addEventListener('click', handleClickOutside);
        
        return () => {
          window.removeEventListener('resize', updatePosition);
          window.removeEventListener('scroll', updatePosition, true);
          document.removeEventListener('click', handleClickOutside);
        };
      }
    });
  }
</script>

<div class="w-full ml-0 mr-0 p-2 space-y-6">
  <!-- 标签页导航 -->
  <div class="card p-0">
    <div class="flex items-center border-b border-gray-200 dark:border-gray-700 overflow-x-auto">
      {#each tabs as tab}
        <div class="flex items-center">
          <button
            onclick={() => setActiveTab(tab.id)}
            class="flex items-center gap-2 px-4 py-3 border-b-2 transition-colors whitespace-nowrap {activeTabId === tab.id
              ? 'border-primary-600 dark:border-primary-400 text-primary-600 dark:text-primary-400 bg-gray-50 dark:bg-gray-800'
              : 'border-transparent text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-gray-200 hover:bg-gray-50 dark:hover:bg-gray-800'}"
          >
            <span class="text-sm font-medium">{tab.name}</span>
          </button>
          {#if tabs.length > 1}
            <button
              onclick={() => removeTab(tab.id)}
              class="px-2 py-3 text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-gray-200 hover:bg-gray-50 dark:hover:bg-gray-800 transition-colors"
            >
              <X class="w-3 h-3" />
            </button>
          {/if}
        </div>
      {/each}
      <button
        onclick={addTab}
        class="px-4 py-3 text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-gray-200 hover:bg-gray-50 dark:hover:bg-gray-800 transition-colors"
      >
        <Plus class="w-4 h-4" />
      </button>
    </div>
  </div>

  <!-- 请求配置卡片 -->
  <div class="card">
    <div class="space-y-4">
      <!-- 方法和 URL -->
      <div class="flex gap-2 items-start overflow-visible">
        <select
          bind:value={activeTab.method}
          class="input w-32"
        >
          <option value="GET">GET</option>
          <option value="POST">POST</option>
          <option value="PUT">PUT</option>
          <option value="DELETE">DELETE</option>
          <option value="PATCH">PATCH</option>
          <option value="HEAD">HEAD</option>
          <option value="OPTIONS">OPTIONS</option>
        </select>
        <input
          type="text"
          bind:value={activeTab.url}
          placeholder={t('apiClient.urlPlaceholder')}
          class="input flex-1"
        />
        <div class="flex items-center">
          <button
            onclick={() => sendRequest(activeTab)}
            disabled={activeTab.isSending || !activeTab.url.trim()}
            class="h-10 px-4 text-white transition-colors font-medium hover:opacity-90 disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-2 rounded-l-lg"
            style="background-color: #818089;"
          >
            <Send class="w-4 h-4" />
            {#if activeTab.isSending}
              {t('apiClient.sending')}
            {:else}
              {t('apiClient.send')}
            {/if}
          </button>
          <div class="relative">
            <button
              bind:this={dropdownButtonRef}
              onclick={() => showDropdown = !showDropdown}
              disabled={activeTab.isSending}
              class="h-10 px-3 text-white transition-colors font-medium hover:opacity-90 disabled:opacity-50 disabled:cursor-not-allowed flex items-center justify-center rounded-r-lg border-l border-white/20"
              style="background-color: #818089;"
            >
              <ChevronDown class="w-4 h-4" />
            </button>
            {#if showDropdown}
              <div 
                bind:this={dropdownRef}
                data-dropdown
                class="fixed bg-white dark:bg-gray-800 rounded-lg shadow-lg border border-gray-200 dark:border-gray-700 py-1 min-w-[200px] z-[9999]"
                onclick={(e) => e.stopPropagation()}
              >
                <!-- 三角形指示器 -->
                <div class="absolute -top-1 right-4 w-2 h-2 bg-white dark:bg-gray-800 border-l border-t border-gray-200 dark:border-gray-700 transform rotate-45"></div>
                <button
                  onclick={() => { curlCommandText = ''; showImportCurlDialog = true; showDropdown = false; }}
                  disabled={activeTab.isSending}
                  class="w-full px-4 py-2 text-left text-sm text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 flex items-center gap-3 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
                >
                  <Upload class="w-4 h-4" />
                  <span>{t('apiClient.importCurl')}</span>
                </button>
                <button
                  onclick={() => { exportCurlCommand(activeTab); showDropdown = false; }}
                  disabled={activeTab.isSending || !activeTab.url.trim()}
                  class="w-full px-4 py-2 text-left text-sm text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 flex items-center gap-3 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
                >
                  <Code class="w-4 h-4" />
                  <span>{t('apiClient.showCode')}</span>
                </button>
                <div class="border-t border-gray-200 dark:border-gray-700 my-1"></div>
                <button
                  onclick={() => { clear(activeTab); showDropdown = false; }}
                  disabled={activeTab.isSending}
                  class="w-full px-4 py-2 text-left text-sm text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 flex items-center gap-3 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
                >
                  <Trash2 class="w-4 h-4" />
                  <span>{t('apiClient.clearAll')}</span>
                </button>
              </div>
            {/if}
          </div>
        </div>
      </div>

      <!-- Headers -->
      <div>
        <div class="flex items-center justify-between mb-2">
          <div class="block text-sm font-medium text-gray-700 dark:text-gray-300">
            {t('apiClient.headers')}
          </div>
          <button
            onclick={() => { bulkHeaderText = ''; showBulkHeaderDialog = true; }}
            class="btn-secondary text-sm"
          >
            {t('apiClient.bulkAdd')}
          </button>
        </div>
        <div class="space-y-2">
          {#each activeTab.headers as header, index}
            <div class="flex gap-2 items-center">
              <input
                type="checkbox"
                bind:checked={header.enabled}
                class="w-4 h-4 text-primary-600 bg-gray-100 border-gray-300 rounded focus:ring-primary-500 dark:focus:ring-primary-600 dark:ring-offset-gray-800 focus:ring-2 dark:bg-gray-700 dark:border-gray-600"
              />
              <input
                type="text"
                bind:value={header.key}
                placeholder={t('apiClient.headerKey')}
                class="input flex-1"
                disabled={!header.enabled}
              />
              <input
                type="text"
                bind:value={header.value}
                placeholder={t('apiClient.headerValue')}
                class="input flex-1"
                disabled={!header.enabled}
              />
              <button
                onclick={() => removeHeader(activeTab, index)}
                class="p-2 text-red-600 dark:text-red-400 hover:bg-red-50 dark:hover:bg-red-900/20 rounded"
                disabled={activeTab.headers.length === 1}
              >
                <Trash2 class="w-4 h-4" />
              </button>
            </div>
          {/each}
          <button
            onclick={() => addHeader(activeTab)}
            class="btn-secondary text-sm"
          >
            {t('apiClient.addHeader')}
          </button>
        </div>
      </div>

      <!-- 批量添加 Header 对话框 -->
      {#if showBulkHeaderDialog}
        <div 
          class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50"
          onclick={() => { showBulkHeaderDialog = false; bulkHeaderText = ''; }}
          onkeydown={(e) => { if (e.key === 'Escape') { showBulkHeaderDialog = false; bulkHeaderText = ''; } }}
          role="dialog"
          aria-modal="true"
          tabindex="-1"
        >
          <div 
            class="bg-white dark:bg-gray-800 rounded-lg shadow-xl p-6 w-full max-w-2xl mx-4"
            onclick={(e) => e.stopPropagation()}
            role="none"
          >
            <h3 class="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-4">
              {t('apiClient.bulkAddHeaders')}
            </h3>
            <p class="text-sm text-gray-600 dark:text-gray-400 mb-4">
              {t('apiClient.bulkAddHeadersHint')}
            </p>
            <textarea
              bind:value={bulkHeaderText}
              placeholder={t('apiClient.bulkAddHeadersPlaceholder')}
              class="textarea font-mono text-sm min-h-[200px] mb-4"
            ></textarea>
            <div class="flex gap-2 justify-end">
              <button
                onclick={() => { showBulkHeaderDialog = false; bulkHeaderText = ''; }}
                class="btn-secondary"
              >
                {t('apiClient.cancel')}
              </button>
              <button
                onclick={() => addBulkHeaders(activeTab)}
                class="px-4 py-2 text-white rounded-lg transition-colors font-medium hover:opacity-90"
                style="background-color: #818089;"
              >
                {t('apiClient.add')}
              </button>
            </div>
          </div>
        </div>
      {/if}

      <!-- Body -->
      {#if activeTab.method !== 'GET' && activeTab.method !== 'HEAD' && activeTab.method !== 'OPTIONS'}
        <div>
          <div class="block text-sm font-medium mb-2 text-gray-700 dark:text-gray-300">
            {t('apiClient.body')}
          </div>
          <div class="flex gap-2 mb-2">
            <button
              onclick={() => activeTab.bodyType = 'none'}
              class="px-3 py-1 text-sm rounded transition-colors {activeTab.bodyType === 'none' ? 'bg-gray-300 dark:bg-gray-600 text-gray-900 dark:text-gray-100' : 'bg-gray-100 dark:bg-gray-700 text-gray-700 dark:text-gray-300'}"
            >
              None
            </button>
            <button
              onclick={() => activeTab.bodyType = 'json'}
              class="px-3 py-1 text-sm rounded transition-colors {activeTab.bodyType === 'json' ? 'bg-gray-300 dark:bg-gray-600 text-gray-900 dark:text-gray-100' : 'bg-gray-100 dark:bg-gray-700 text-gray-700 dark:text-gray-300'}"
            >
              JSON
            </button>
            <button
              onclick={() => activeTab.bodyType = 'text'}
              class="px-3 py-1 text-sm rounded transition-colors {activeTab.bodyType === 'text' ? 'bg-gray-300 dark:bg-gray-600 text-gray-900 dark:text-gray-100' : 'bg-gray-100 dark:bg-gray-700 text-gray-700 dark:text-gray-300'}"
            >
              Text
            </button>
            <button
              onclick={() => activeTab.bodyType = 'xml'}
              class="px-3 py-1 text-sm rounded transition-colors {activeTab.bodyType === 'xml' ? 'bg-gray-300 dark:bg-gray-600 text-gray-900 dark:text-gray-100' : 'bg-gray-100 dark:bg-gray-700 text-gray-700 dark:text-gray-300'}"
            >
              XML
            </button>
            <button
              onclick={() => activeTab.bodyType = 'form-data'}
              class="px-3 py-1 text-sm rounded transition-colors {activeTab.bodyType === 'form-data' ? 'bg-gray-300 dark:bg-gray-600 text-gray-900 dark:text-gray-100' : 'bg-gray-100 dark:bg-gray-700 text-gray-700 dark:text-gray-300'}"
            >
              Form Data
            </button>
            <button
              onclick={() => activeTab.bodyType = 'url-encoded'}
              class="px-3 py-1 text-sm rounded transition-colors {activeTab.bodyType === 'url-encoded' ? 'bg-gray-300 dark:bg-gray-600 text-gray-900 dark:text-gray-100' : 'bg-gray-100 dark:bg-gray-700 text-gray-700 dark:text-gray-300'}"
            >
              URL Encoded
            </button>
          </div>

          {#if activeTab.bodyType === 'json'}
            <textarea
              bind:value={activeTab.bodyJson}
              placeholder={t('apiClient.jsonPlaceholder')}
              class="textarea font-mono text-sm min-h-[150px]"
            ></textarea>
          {:else if activeTab.bodyType === 'text'}
            <textarea
              bind:value={activeTab.bodyText}
              placeholder={t('apiClient.textPlaceholder')}
              class="textarea font-mono text-sm min-h-[150px]"
            ></textarea>
          {:else if activeTab.bodyType === 'xml'}
            <textarea
              bind:value={activeTab.bodyXml}
              placeholder={t('apiClient.xmlPlaceholder')}
              class="textarea font-mono text-sm min-h-[150px]"
            ></textarea>
          {:else if activeTab.bodyType === 'form-data' || activeTab.bodyType === 'url-encoded'}
            <div class="space-y-2">
              {#each activeTab.formData as item, index}
                <div class="flex gap-2 items-center">
                  <input
                    type="checkbox"
                    bind:checked={item.enabled}
                    class="w-4 h-4 text-primary-600 bg-gray-100 border-gray-300 rounded focus:ring-primary-500 dark:focus:ring-primary-600 dark:ring-offset-gray-800 focus:ring-2 dark:bg-gray-700 dark:border-gray-600"
                  />
                  <input
                    type="text"
                    bind:value={item.key}
                    placeholder={t('apiClient.formKey')}
                    class="input flex-1"
                    disabled={!item.enabled}
                  />
                  {#if activeTab.bodyType === 'form-data'}
                    <select
                      bind:value={item.type}
                      class="input w-24"
                      disabled={!item.enabled}
                      onchange={() => {
                        if (item.type === 'text') {
                          item.file = null;
                          item.value = '';
                        } else {
                          item.value = '';
                        }
                      }}
                    >
                      <option value="text">Text</option>
                      <option value="file">File</option>
                    </select>
                  {/if}
                  {#if item.type === 'text' || activeTab.bodyType === 'url-encoded'}
                    <input
                      type="text"
                      bind:value={item.value}
                      placeholder={t('apiClient.formValue')}
                      class="input flex-1"
                      disabled={!item.enabled}
                    />
                  {:else if item.type === 'file'}
                    <label class="input flex-1 cursor-pointer flex items-center gap-2 {!item.enabled ? 'opacity-50 cursor-not-allowed' : ''}">
                      <input
                        type="file"
                        class="hidden"
                        disabled={!item.enabled}
                        onchange={(e) => handleFileSelect(activeTab, index, e)}
                      />
                      <span class="flex-1 text-gray-700 dark:text-gray-300">
                        {item.value || t('apiClient.selectFile')}
                      </span>
                      {#if item.file}
                        <span class="text-xs text-gray-500 dark:text-gray-400">
                          ({(item.file.size / 1024).toFixed(2)} KB)
                        </span>
                      {/if}
                    </label>
                  {/if}
                  <button
                    onclick={() => removeFormDataItem(activeTab, index)}
                    class="p-2 text-red-600 dark:text-red-400 hover:bg-red-50 dark:hover:bg-red-900/20 rounded"
                    disabled={activeTab.formData.length === 1}
                  >
                    <Trash2 class="w-4 h-4" />
                  </button>
                </div>
              {/each}
              <button
                onclick={() => addFormDataItem(activeTab)}
                class="btn-secondary text-sm"
              >
                {t('apiClient.addFormData')}
              </button>
            </div>
          {/if}
        </div>
      {/if}

      {#if activeTab.error}
        <div class="p-4 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg">
          <p class="text-sm text-red-800 dark:text-red-200">{activeTab.error}</p>
        </div>
      {/if}
    </div>
  </div>

  <!-- 响应卡片 -->
  {#if activeTab.responseStatus !== null || activeTab.responseBody}
    <div class="card">
      <div class="space-y-4">
        <div class="flex items-center justify-between">
          <h3 class="text-lg font-semibold text-gray-900 dark:text-gray-100">
            {t('apiClient.response')}
          </h3>
          {#if activeTab.responseTime !== null}
            <span class="text-sm text-gray-500 dark:text-gray-400">
              {activeTab.responseTime}ms
            </span>
          {/if}
        </div>

        {#if activeTab.responseStatus !== null}
          <div>
            <span class="text-sm font-medium text-gray-700 dark:text-gray-300">
              {t('apiClient.status')}:
            </span>
            <span class="ml-2 font-semibold {getStatusColor(activeTab.responseStatus)}">
              {activeTab.responseStatus}
            </span>
          </div>
        {/if}

        <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
          <!-- 响应头 -->
          <div class="flex flex-col">
            <div class="flex items-center justify-between mb-2 h-[2.5rem]">
              <div class="block text-sm font-medium text-gray-700 dark:text-gray-300">
                {t('apiClient.responseHeaders')}
              </div>
              <div class="w-0"></div>
            </div>
            <div class="bg-white dark:bg-gray-800 border border-gray-300 dark:border-gray-600 rounded-lg px-4 py-2 h-[calc(100vh-400px)] min-h-[300px] overflow-y-auto {Object.keys(activeTab.responseHeaders).length === 0 ? 'flex items-center justify-center' : ''}">
              {#if Object.keys(activeTab.responseHeaders).length > 0}
                {#each Object.entries(activeTab.responseHeaders) as [key, value]}
                  <div class="text-sm font-mono py-1">
                    <span class="text-gray-600 dark:text-gray-400">{key}:</span>
                    <span class="ml-2 text-gray-900 dark:text-gray-100">{value}</span>
                  </div>
                {/each}
              {:else}
                <span class="text-sm text-gray-400 dark:text-gray-500">{t('apiClient.noHeaders')}</span>
              {/if}
            </div>
          </div>

          <!-- 响应体 -->
          <div class="flex flex-col">
            <div class="flex items-center justify-between mb-2 h-[2.5rem]">
              <div class="block text-sm font-medium text-gray-700 dark:text-gray-300">
                {t('apiClient.responseBody')}
              </div>
              {#if activeTab.responseBody}
                <button
                  onclick={() => copyResponse(activeTab)}
                  class="btn-secondary whitespace-nowrap transition-all duration-200 {activeTab.copied ? 'bg-green-500 hover:bg-green-600 text-white' : ''}"
                >
                  {#if activeTab.copied}
                    <span class="flex items-center gap-1">
                      <Check class="w-4 h-4" />
                      {t('common.copied')}
                    </span>
                  {:else}
                    <span class="flex items-center gap-1">
                      <Copy class="w-4 h-4" />
                      {t('common.copy')}
                    </span>
                  {/if}
                </button>
              {:else}
                <div class="w-0"></div>
              {/if}
            </div>
            {#if activeTab.responseBody}
              <textarea
                value={activeTab.responseBody}
                readonly
                class="textarea font-mono text-sm h-[calc(100vh-400px)] min-h-[300px] resize-none {activeTab.copied ? 'bg-green-50 dark:bg-green-900/20 border-green-300 dark:border-green-700' : ''} transition-colors duration-300"
              ></textarea>
            {:else}
              <div class="bg-white dark:bg-gray-800 border border-gray-300 dark:border-gray-600 rounded-lg px-4 py-2 h-[calc(100vh-400px)] min-h-[300px] overflow-y-auto flex items-center justify-center">
                <span class="text-sm text-gray-400 dark:text-gray-500">{t('apiClient.noResponseBody')}</span>
              </div>
            {/if}
          </div>
        </div>
      </div>
    </div>
  {/if}

  <!-- 导入 curl 对话框 -->
  {#if showImportCurlDialog}
    <div 
      class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50"
      onclick={() => { showImportCurlDialog = false; curlCommandText = ''; }}
      onkeydown={(e) => { if (e.key === 'Escape') { showImportCurlDialog = false; curlCommandText = ''; } }}
      role="dialog"
      aria-modal="true"
      tabindex="-1"
    >
      <div 
        class="bg-white dark:bg-gray-800 rounded-lg shadow-xl p-6 w-full max-w-3xl mx-4"
        onclick={(e) => e.stopPropagation()}
        role="none"
      >
        <h3 class="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-4">
          {t('apiClient.importCurl')}
        </h3>
        <p class="text-sm text-gray-600 dark:text-gray-400 mb-4">
          {t('apiClient.importCurlHint')}
        </p>
        <textarea
          bind:value={curlCommandText}
          placeholder={t('apiClient.importCurlPlaceholder')}
          class="textarea font-mono text-sm min-h-[200px] mb-4"
        ></textarea>
        <div class="flex gap-2 justify-end">
          <button
            onclick={() => { showImportCurlDialog = false; curlCommandText = ''; }}
            class="btn-secondary"
          >
            {t('apiClient.cancel')}
          </button>
          <button
            onclick={() => importCurlCommand()}
            class="px-4 py-2 text-white rounded-lg transition-colors font-medium hover:opacity-90"
            style="background-color: #818089;"
          >
            {t('apiClient.import')}
          </button>
        </div>
      </div>
    </div>
  {/if}

  <!-- 导出 curl 对话框 -->
  {#if showExportCurlDialog}
    <div 
      class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50"
      onclick={() => { showExportCurlDialog = false; generatedCurlCommand = ''; }}
      onkeydown={(e) => { if (e.key === 'Escape') { showExportCurlDialog = false; generatedCurlCommand = ''; } }}
      role="dialog"
      aria-modal="true"
      tabindex="-1"
    >
      <div 
        class="bg-white dark:bg-gray-800 rounded-lg shadow-xl p-6 w-full max-w-3xl mx-4"
        onclick={(e) => e.stopPropagation()}
        role="none"
      >
        <h3 class="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-4">
          {t('apiClient.exportCurl')}
        </h3>
        <p class="text-sm text-gray-600 dark:text-gray-400 mb-4">
          {t('apiClient.exportCurlHint')}
        </p>
        <textarea
          value={generatedCurlCommand}
          readonly
          class="textarea font-mono text-sm min-h-[200px] mb-4"
        ></textarea>
        <div class="flex gap-2 justify-end">
          <button
            onclick={() => { showExportCurlDialog = false; generatedCurlCommand = ''; }}
            class="btn-secondary"
          >
            {t('apiClient.close')}
          </button>
          <button
            onclick={() => copyCurlCommand()}
            class="px-4 py-2 text-white rounded-lg transition-colors font-medium hover:opacity-90 flex items-center gap-2 {curlCopied ? 'bg-green-500 hover:bg-green-600' : ''}"
            style={curlCopied ? '' : 'background-color: #818089;'}
          >
            {#if curlCopied}
              <Check class="w-4 h-4" />
              {t('common.copied')}
            {:else}
              <Copy class="w-4 h-4" />
              {t('common.copy')}
            {/if}
          </button>
        </div>
      </div>
    </div>
  {/if}
</div>

