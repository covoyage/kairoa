<script lang="ts">
  import { translationsStore } from '$lib/stores/i18n';
  import { Copy, Check, Send, Trash2, Plus, X, Download, Upload, ChevronDown, ChevronRight, Code, Braces, FolderPlus, Folder, MoreVertical, Pencil, ArrowRight, Inbox, Search, Copy as CopyIcon, Clock, Key, GripVertical, Eye, FileCode, ExternalLink } from 'lucide-svelte';
  import { browser } from '$app/environment';
  import hljs from 'highlight.js';
  
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

  type RequestView = 'headers' | 'body' | 'auth' | 'settings' | 'preRequest' | 'tests' | 'params';

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
    responseSize: number | null;
    error: string;
    copied: boolean;
    collectionId: string | null;
    order: number;
    authType: AuthType;
    bearerToken: string;
    basicUsername: string;
    basicPassword: string;
    preRequestScript: string;
    testScript: string;
    testResults: { name: string; passed: boolean; error?: string }[];
    requestView: RequestView;
    cancelled: boolean;
  }

  function defaultTabFields(): Pick<TabData, 'authType' | 'bearerToken' | 'basicUsername' | 'basicPassword' | 'preRequestScript' | 'testScript' | 'testResults' | 'requestView' | 'responseSize' | 'cancelled'> {
    return {
      authType: 'none',
      bearerToken: '',
      basicUsername: '',
      basicPassword: '',
      preRequestScript: '',
      testScript: '',
      testResults: [],
      requestView: 'headers',
      responseSize: null,
      cancelled: false
    };
  }

  interface Collection {
    id: string;
    name: string;
    collapsed: boolean;
  }

  interface HistoryEntry {
    id: string;
    name: string;
    method: HttpMethod;
    url: string;
    status: number | null;
    time: number | null;
    timestamp: number;
    snapshot?: {
      method: HttpMethod;
      url: string;
      headers: Header[];
      bodyType: BodyType;
      bodyJson: string;
      bodyText: string;
      bodyXml: string;
      formData: { key: string; value: string; enabled: boolean; type: 'text' | 'file' }[];
      authType: AuthType;
      bearerToken: string;
      basicUsername: string;
      basicPassword: string;
      collectionId: string | null;
    };
  }

  type AuthType = 'none' | 'bearer' | 'basic';
  
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
      copied: false,
      collectionId: null,
      order: 0,
      ...defaultTabFields()
    }
  ]);

  let collections = $state<Collection[]>([]);
  
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

  // 添加新标签页
  function addTab(collectionId: string | null = null) {
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
      copied: false,
      collectionId,
      order: tabs.length,
      ...defaultTabFields()
    };
    tabs = [...tabs, newTab];
    activeTabId = newTab.id;
    if (collectionId) {
      const col = collections.find(c => c.id === collectionId);
      if (col) col.collapsed = false;
    }
    // 新请求：聚焦 URL 输入框并全选，方便直接输入
    requestAnimationFrame(() => {
      urlInputRef?.focus();
      urlInputRef?.select();
    });
  }

  // 删除标签页
  function removeTab(tabId: string) {
    if (tabs.length === 1) return;
    tabs = tabs.filter(tab => tab.id !== tabId);
    if (activeTabId === tabId) {
      activeTabId = tabs[0].id;
    }
  }

  // ── Collection management ──
  function nextCollectionName(): string {
    const base = t('apiClient.newCollection');
    if (!collections.some(c => c.name === base)) return base;
    let n = 2;
    while (collections.some(c => c.name === `${base} ${n}`)) n++;
    return `${base} ${n}`;
  }

  function addCollection() {
    const newCol: Collection = {
      id: crypto.randomUUID(),
      name: nextCollectionName(),
      collapsed: false
    };
    collections = [...collections, newCol];
    editingCollectionId = newCol.id;
    editingCollectionName = newCol.name;
  }

  function renameCollection(colId: string, name: string) {
    const col = collections.find(c => c.id === colId);
    if (col) col.name = name;
  }

  function deleteCollection(colId: string) {
    const col = collections.find(c => c.id === colId);
    if (!col) return;
    const count = tabs.filter(tab => tab.collectionId === colId).length;
    const msg = t('apiClient.deleteCollectionConfirm')
      .replace('{name}', col.name ?? '')
      .replace('{count}', String(count));
    if (!confirm(msg)) return;
    tabs.forEach(tab => {
      if (tab.collectionId === colId) tab.collectionId = null;
    });
    collections = collections.filter(c => c.id !== colId);
    if (editingCollectionId === colId) {
      editingCollectionId = null;
      editingCollectionName = '';
    }
  }

  function duplicateCollection(colId: string) {
    const col = collections.find(c => c.id === colId);
    if (!col) return;
    const newColId = crypto.randomUUID();
    let baseName = `${col.name} (copy)`;
    let newName = baseName;
    let n = 2;
    while (collections.some(c => c.name === newName)) {
      newName = `${baseName} ${n++}`;
    }
    collections = [...collections, { id: newColId, name: newName, collapsed: false }];
    const copies = tabs
      .filter(tab => tab.collectionId === colId)
      .map(tab => ({
        ...tab,
        id: crypto.randomUUID(),
        collectionId: newColId,
        isSending: false,
        responseStatus: null,
        responseHeaders: {},
        responseBody: '',
        responseTime: null,
        responseSize: null,
        error: '',
        copied: false,
        cancelled: false,
        testResults: [],
        headers: tab.headers.map(h => ({ ...h })),
        formData: (tab.formData || []).map(item => ({ ...item, file: null })),
        order: tabs.length
      }));
    if (copies.length > 0) tabs = [...tabs, ...copies];
  }

  function toggleCollapse(colId: string) {
    const col = collections.find(c => c.id === colId);
    if (col) col.collapsed = !col.collapsed;
  }

  function moveTabToCollection(tabId: string, collectionId: string | null) {
    const tab = tabs.find(t => t.id === tabId);
    if (tab) tab.collectionId = collectionId;
  }

  function tabsInCollection(colId: string) {
    return tabs.filter(t => t.collectionId === colId);
  }

  let ungroupedTabs = $derived(tabs.filter(t => t.collectionId === null));

  // ── Search ──
  let searchText = $state('');

  let filteredUngroupedTabs = $derived(
    searchText.trim()
      ? ungroupedTabs.filter(t =>
          t.name.toLowerCase().includes(searchText.toLowerCase()) ||
          t.url.toLowerCase().includes(searchText.toLowerCase()))
      : ungroupedTabs
  );

  function filteredTabsInCollection(colId: string) {
    const colTabs = tabs.filter(t => t.collectionId === colId);
    if (!searchText.trim()) return colTabs;
    const q = searchText.toLowerCase();
    return colTabs.filter(t =>
      t.name.toLowerCase().includes(q) || t.url.toLowerCase().includes(q));
  }

  let showCollectionMenu = $state<string | null>(null);
  let showMoveMenu = $state<string | null>(null);
  let editingCollectionId = $state<string | null>(null);
  let editingCollectionName = $state('');

  function startRenameCollection(colId: string) {
    const col = collections.find(c => c.id === colId);
    if (col) {
      editingCollectionId = colId;
      editingCollectionName = col.name;
    }
  }

  function confirmRenameCollection() {
    if (editingCollectionId && editingCollectionName.trim()) {
      renameCollection(editingCollectionId, editingCollectionName.trim());
    }
    editingCollectionId = null;
    editingCollectionName = '';
  }

  function focusAndSelect(node: HTMLInputElement) {
    requestAnimationFrame(() => {
      node.focus();
      node.select();
    });
  }

  function addHeader(tab: TabData) {
    tab.headers = [...tab.headers, { key: '', value: '', enabled: true }];
  }

  async function copyHeaders(tab: TabData) {
    const lines = tab.headers
      .filter((h) => h.enabled && (h.key.trim() || h.value.trim()))
      .map((h) => `${h.key.trim()}: ${h.value.trim()}`);

    const text = lines.join('\n');
    if (!text) return;

    try {
      await navigator.clipboard.writeText(text);
      headersCopied = true;
      setTimeout(() => {
        headersCopied = false;
      }, 1200);
    } catch (error) {
      console.error('Failed to copy headers:', error);
    }
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
  let headersCopied = $state(false);
  let showDropdown = $state(false);
  const STORAGE_SIDEBAR_WIDTH_KEY = 'apiClient.sidebarWidth.v1';
  let sidebarWidth = $state(256);
  let isResizing = $state(false);

  if (browser) {
    try {
      const savedWidth = localStorage.getItem(STORAGE_SIDEBAR_WIDTH_KEY);
      if (savedWidth) {
        const w = parseInt(savedWidth, 10);
        if (!isNaN(w)) sidebarWidth = Math.max(180, Math.min(500, w));
      }
    } catch {}
  }

  function startResize(e: MouseEvent) {
    e.preventDefault();
    isResizing = true;
    const startX = e.clientX;
    const startWidth = sidebarWidth;

    const onMouseMove = (moveEvent: MouseEvent) => {
      const delta = moveEvent.clientX - startX;
      const newWidth = Math.max(180, Math.min(500, startWidth + delta));
      sidebarWidth = newWidth;
    };

    const onMouseUp = () => {
      isResizing = false;
      document.removeEventListener('mousemove', onMouseMove);
      document.removeEventListener('mouseup', onMouseUp);
      document.body.style.cursor = '';
      document.body.style.userSelect = '';
      try {
        if (browser) localStorage.setItem(STORAGE_SIDEBAR_WIDTH_KEY, String(sidebarWidth));
      } catch {}
    };

    document.body.style.cursor = 'col-resize';
    document.body.style.userSelect = 'none';
    document.addEventListener('mousemove', onMouseMove);
    document.addEventListener('mouseup', onMouseUp);
  }

  // ── Duplicate ──
  function duplicateTab(tabId: string) {
    const tab = tabs.find(t => t.id === tabId);
    if (!tab) return;
    const newTab: TabData = {
      ...tab,
      id: crypto.randomUUID(),
      name: tab.name + ' (copy)',
      isSending: false,
      responseStatus: null,
      responseHeaders: {},
      responseBody: '',
      responseTime: null,
      responseSize: null,
      error: '',
      copied: false,
      cancelled: false,
      testResults: [],
      headers: tab.headers.map(h => ({ ...h })),
      formData: (tab.formData || []).map(item => ({ ...item, file: null })),
      order: tabs.length
    };
    tabs = [...tabs, newTab];
    activeTabId = newTab.id;
  }

  // ── Drag & drop reordering ──
  let draggedTabId = $state<string | null>(null);

  function onDragStart(e: DragEvent, tabId: string) {
    draggedTabId = tabId;
    e.dataTransfer?.setData('text/plain', tabId);
  }

  function onDrop(e: DragEvent, targetTabId: string, targetColId: string | null) {
    e.preventDefault();
    e.stopPropagation();
    if (!draggedTabId || draggedTabId === targetTabId) return;
    const draggedTab = tabs.find(t => t.id === draggedTabId);
    if (!draggedTab) return;
    // Move to target collection
    draggedTab.collectionId = targetColId;
    // Reorder: remove from array, insert before target
    const filtered = tabs.filter(t => t.id !== draggedTabId);
    const targetIdx = filtered.findIndex(t => t.id === targetTabId);
    if (targetIdx >= 0) {
      filtered.splice(targetIdx, 0, draggedTab);
    } else {
      filtered.push(draggedTab);
    }
    // Recalculate order
    filtered.forEach((t, i) => { t.order = i; });
    tabs = filtered;
    draggedTabId = null;
  }

  function onDropOnCollection(e: DragEvent, colId: string | null) {
    e.preventDefault();
    e.stopPropagation();
    if (!draggedTabId) return;
    const draggedTab = tabs.find(t => t.id === draggedTabId);
    if (!draggedTab) return;
    draggedTab.collectionId = colId;
    draggedTabId = null;
  }

  // ── Query params (synced with URL ?k=v) ──
  interface QueryParam { key: string; value: string; enabled: boolean; }
  let showSidebarMenu = $state(false);

  function getQueryParams(url: string): QueryParam[] {
    try {
      const qIndex = url.indexOf('?');
      if (qIndex < 0) return [];
      const hashSplit = url.slice(qIndex + 1).split('#')[0];
      if (!hashSplit) return [];
      return hashSplit.split('&').filter(Boolean).map(pair => {
        const eq = pair.indexOf('=');
        const rawK = eq < 0 ? pair : pair.slice(0, eq);
        const rawV = eq < 0 ? '' : pair.slice(eq + 1);
        let key = rawK, value = rawV, enabled = true;
        if (key.startsWith('!')) { enabled = false; key = key.slice(1); }
        try { key = decodeURIComponent(key.replace(/\+/g, ' ')); } catch {}
        try { value = decodeURIComponent(value.replace(/\+/g, ' ')); } catch {}
        if (/^\{\{.*\}\}$/.test(key.trim()) || /^\{\{.*\}\}$/.test(value.trim())) {
          return { key, value, enabled };
        }
        return { key, value, enabled };
      });
    } catch {
      return [];
    }
  }

  function setQueryParams(url: string, params: QueryParam[]): string {
    const qIndex = url.indexOf('?');
    const base = qIndex < 0 ? url.split('#')[0] : url.slice(0, qIndex);
    const hash = url.includes('#') ? url.slice(url.indexOf('#')) : '';
    const parts: string[] = [];
    for (const p of params) {
      if (!p.enabled) continue;
      if (!p.key.trim() && !p.value.trim()) continue;
      const k = /^\{\{.*\}\}$/.test(p.key.trim()) ? p.key.trim() : encodeURIComponent(p.key.trim());
      const v = /^\{\{.*\}\}$/.test(p.value.trim()) ? p.value.trim() : encodeURIComponent(p.value.trim());
      parts.push(`${k}=${v}`);
    }
    const disabled = params.filter(p => !p.enabled && (p.key.trim() || p.value.trim()));
    for (const p of disabled) {
      parts.push(`!${encodeURIComponent(p.key.trim())}=${encodeURIComponent(p.value.trim())}`);
    }
    return parts.length > 0 ? `${base}?${parts.join('&')}${hash}` : `${base}${hash}`;
  }

  let queryParams = $derived(getQueryParams(activeTab?.url ?? ''));

  function updateQueryParam(index: number, field: 'key' | 'value' | 'enabled', val: string | boolean) {
    if (!activeTab) return;
    const params = getQueryParams(activeTab.url);
    while (params.length <= index) params.push({ key: '', value: '', enabled: true });
    (params[index] as any)[field] = val;
    activeTab.url = setQueryParams(activeTab.url, params);
  }

  function addQueryParam() {
    if (!activeTab) return;
    const params = getQueryParams(activeTab.url);
    params.push({ key: '', value: '', enabled: true });
    activeTab.url = setQueryParams(activeTab.url, params);
  }

  function removeQueryParam(index: number) {
    if (!activeTab) return;
    const params = getQueryParams(activeTab.url);
    params.splice(index, 1);
    activeTab.url = setQueryParams(activeTab.url, params);
  }

  // ── JSON inline validation ──
  let jsonInlineError = $derived.by(() => {
    const tab = activeTab;
    if (!tab || tab.bodyType !== 'json' || !tab.bodyJson.trim()) return '';
    try {
      JSON.parse(tab.bodyJson);
      return '';
    } catch (e) {
      return e instanceof Error ? e.message : String(e);
    }
  });

  // ── History ──
  let history = $state<HistoryEntry[]>([]);
  let showHistory = $state(false);
  const STORAGE_HISTORY_KEY = 'apiClient.history.v1';
  const MAX_HISTORY = 50;

  function loadHistory() {
    if (!browser) return;
    try {
      const saved = localStorage.getItem(STORAGE_HISTORY_KEY);
      if (saved) history = JSON.parse(saved);
    } catch {}
  }

  function saveHistory() {
    if (!browser) return;
    localStorage.setItem(STORAGE_HISTORY_KEY, JSON.stringify(history));
  }

  loadHistory();

  function addHistoryEntry(tab: TabData) {
    const entry: HistoryEntry = {
      id: crypto.randomUUID(),
      name: tab.name,
      method: tab.method,
      url: tab.url,
      status: tab.responseStatus,
      time: tab.responseTime,
      timestamp: Date.now(),
      snapshot: {
        method: tab.method,
        url: tab.url,
        headers: tab.headers.map(h => ({ ...h })),
        bodyType: tab.bodyType,
        bodyJson: tab.bodyJson,
        bodyText: tab.bodyText,
        bodyXml: tab.bodyXml,
        formData: (tab.formData || []).map(({ key, value, enabled, type }) => ({ key, value, enabled, type })),
        authType: tab.authType,
        bearerToken: tab.bearerToken,
        basicUsername: tab.basicUsername,
        basicPassword: tab.basicPassword,
        collectionId: tab.collectionId
      }
    };
    history = [entry, ...history].slice(0, MAX_HISTORY);
    saveHistory();
  }

  function restoreHistoryEntry(entry: HistoryEntry) {
    const snap = entry.snapshot;
    if (!snap) {
      addTab(null);
      if (activeTab) {
        activeTab.method = entry.method;
        activeTab.url = entry.url;
      }
      showHistory = false;
      return;
    }
    addTab(snap.collectionId && collections.some(c => c.id === snap.collectionId) ? snap.collectionId : null);
    const tab = activeTab;
    if (tab) {
      tab.method = snap.method;
      tab.url = snap.url;
      tab.headers = snap.headers.length > 0 ? snap.headers : [{ key: '', value: '', enabled: true }];
      tab.bodyType = snap.bodyType;
      tab.bodyJson = snap.bodyJson;
      tab.bodyText = snap.bodyText;
      tab.bodyXml = snap.bodyXml;
      tab.formData = snap.formData.length > 0 ? snap.formData.map(item => ({ ...item, file: null })) : [{ key: '', value: '', enabled: true, type: 'text' as const, file: null }];
      tab.authType = snap.authType;
      tab.bearerToken = snap.bearerToken;
      tab.basicUsername = snap.basicUsername;
      tab.basicPassword = snap.basicPassword;
      tab.requestView = 'headers';
      tab.testResults = [];
      updateTabName(tab);
    }
    showHistory = false;
  }

  function clearHistory() {
    history = [];
    saveHistory();
  }

  // ── Auth (per-tab, stored on TabData) ──
  function getAuthHeaders(tab?: TabData): Record<string, string> {
    const t = tab ?? activeTab;
    if (!t) return {};
    const headers: Record<string, string> = {};
    if (t.authType === 'bearer' && t.bearerToken.trim()) {
      headers['Authorization'] = `Bearer ${t.bearerToken.trim()}`;
    } else if (t.authType === 'basic' && t.basicUsername.trim()) {
      headers['Authorization'] = `Basic ${btoa(`${t.basicUsername}:${t.basicPassword}`)}`;
    }
    return headers;
  }

  // ── Timeout & redirect ──
  let requestTimeout = $state(0);
  let followRedirects = $state(true);

  // ── Pre-request Script & Tests (per-tab, stored on TabData) ──

  // Build Postman-compatible `pm` object
  function buildPm(envVars: Record<string, string>, scriptVars: Record<string, string>, response?: { status: number; body: string; headers: Record<string, string>; time: number }, results?: { name: string; passed: boolean; error?: string }[]) {
    const pm = {
      environment: {
        get: (key: string) => envVars[key],
        set: (key: string, value: string) => { envVars[key] = value; },
        unset: (key: string) => { delete envVars[key]; },
        has: (key: string) => key in envVars,
      },
      variables: {
        get: (key: string) => scriptVars[key] ?? envVars[key],
        set: (key: string, value: string) => { scriptVars[key] = value; },
        unset: (key: string) => { delete scriptVars[key]; },
        has: (key: string) => key in scriptVars || key in envVars,
      },
      response: response ? {
        code: response.status,
        status: response.status,
        headers: {
          get: (key: string) => response.headers[key] ?? response.headers[key.toLowerCase()],
        },
        json: () => { try { return JSON.parse(response.body); } catch { return null; } },
        text: () => response.body,
        responseTime: response.time,
      } : undefined,
      test: (name: string, fn: () => void) => {
        if (!results) return;
        try {
          fn();
          results.push({ name, passed: true });
        } catch (e) {
          results.push({ name, passed: false, error: e instanceof Error ? e.message : String(e) });
        }
      },
      expect: (actual: unknown) => {
        const r = results;
        const makeResult = (ok: boolean, msg: string) => {
          if (r) r.push({ name: msg, passed: ok, error: ok ? undefined : msg });
          return ok;
        };
        return {
          to: {
            equal: (expected: unknown) => makeResult(JSON.stringify(actual) === JSON.stringify(expected), `expect(${JSON.stringify(actual)}).to.equal(${JSON.stringify(expected)}`),
            eql: (expected: unknown) => makeResult(JSON.stringify(actual) === JSON.stringify(expected), `expect(${JSON.stringify(actual)}).to.eql(${JSON.stringify(expected)}`),
            be: (expected: unknown) => makeResult(actual === expected, `expect(${actual}).to.be(${expected})`),
            include: (expected: string) => makeResult(typeof actual === 'string' && actual.includes(expected), `expect().to.include("${expected}")`),
            match: (regex: RegExp) => makeResult(typeof actual === 'string' && regex.test(actual), `expect().to.match(${regex})`),
            have: {
              property: (key: string) => {
                const ok = actual != null && typeof actual === 'object' && key in (actual as Record<string, unknown>);
                return makeResult(ok, `expect().to.have.property("${key}")`);
              },
              status: (expected: number) => makeResult((actual as any)?.code === expected || (actual as any)?.status === expected, `expect().to.have.status(${expected})`),
            },
          },
          toEqual: (expected: unknown) => makeResult(JSON.stringify(actual) === JSON.stringify(expected), `expect(${JSON.stringify(actual)}).toEqual(${JSON.stringify(expected)}`),
          toBe: (expected: unknown) => makeResult(actual === expected, `expect(${actual}).toBe(${expected})`),
          toEqual1: (expected: unknown) => makeResult(JSON.stringify(actual) === JSON.stringify(expected), `expect.toEqual`),
          toBeGreaterThan: (expected: number) => makeResult((actual as number) > expected, `expect(${actual}).toBeGreaterThan(${expected})`),
          toBeLessThan: (expected: number) => makeResult((actual as number) < expected, `expect(${actual}).toBeLessThan(${expected})`),
          toContain: (expected: string) => makeResult(typeof actual === 'string' && actual.includes(expected), `expect().toContain("${expected}")`),
          toMatch: (regex: RegExp) => makeResult(typeof actual === 'string' && regex.test(actual), `expect().toMatch(${regex})`),
        };
      },
    };
    return pm;
  }

  function runPreRequestScript(tab?: TabData): Record<string, string> {
    const scriptVars: Record<string, string> = {};
    const script = (tab ?? activeTab)?.preRequestScript ?? '';
    if (!script.trim()) return scriptVars;

    try {
      // No persistent environments: pm.environment works as an ephemeral
      // in-memory store for the duration of this script run.
      const envVars: Record<string, string> = {};

      const pm = buildPm(envVars, scriptVars);
      const fn = new Function('pm', `
        try { ${script} } catch(e) { throw e; }
      `);
      fn(pm);
    } catch (e) {
      console.error('Pre-request script error:', e);
    }
    return scriptVars;
  }

  function runTestScript(tab: TabData) {
    tab.testResults = [];
    const script = tab.testScript ?? '';
    if (!script.trim()) return;

    try {
      const envVars: Record<string, string> = {};

      const scriptVars: Record<string, string> = {};
      const results: { name: string; passed: boolean; error?: string }[] = [];
      const response = {
        status: tab.responseStatus ?? 0,
        body: tab.responseBody,
        headers: tab.responseHeaders,
        time: tab.responseTime ?? 0,
      };

      const pm = buildPm(envVars, scriptVars, response, results);
      const script = tab.testScript ?? '';
      const fn = new Function('pm', `
        try { ${script} } catch(e) { results.push({ name: 'Script Error', passed: false, error: e.message }); }
      `);
      // Pass results as closure variable
      (fn as Function)(pm, results);
      tab.testResults = results;
    } catch (e) {
      tab.testResults = [{ name: 'Script Error', passed: false, error: e instanceof Error ? e.message : String(e) }];
    }
  }

  let showResponseDialog = $state(false);
  let responsePopped = $state(false);
  let bodyTextareaRef = $state<HTMLTextAreaElement | null>(null);
  let jsonOverlayRef = $state<HTMLElement | null>(null);
  let xmlOverlayRef = $state<HTMLElement | null>(null);
  let responseOverlayRef = $state<HTMLElement | null>(null);
  let responseTextareaRef = $state<HTMLTextAreaElement | null>(null);
  let inlineResponseOverlayRef = $state<HTMLElement | null>(null);
  let inlineResponseTextareaRef = $state<HTMLTextAreaElement | null>(null);

  function syncInlineResponseScroll() {
    if (inlineResponseOverlayRef && inlineResponseTextareaRef) {
      inlineResponseOverlayRef.scrollTop = inlineResponseTextareaRef.scrollTop;
      inlineResponseOverlayRef.scrollLeft = inlineResponseTextareaRef.scrollLeft;
    }
  }
  let responseViewMode = $state<'pretty' | 'raw' | 'preview' | 'headers'>('pretty');
  let abortController = $state<AbortController | null>(null);
  let jsonError = $state('');

  function cancelRequest(tab: TabData) {
    tab.cancelled = true;
    try {
      abortController?.abort();
    } catch {}
    tab.isSending = false;
    if (!tab.error) tab.error = t('apiClient.cancelled') !== 'apiClient.cancelled' ? t('apiClient.cancelled') : 'Request cancelled';
  }

  let highlightedResponse = $derived.by(() => {
    const body = activeTab?.responseBody ?? '';
    if (!body.trim()) return '';
    if (body.length > 100000) return body;
    try {
      JSON.parse(body);
      return hljs.highlight(body, { language: 'json' }).value;
    } catch {
      return body;
    }
  });

  function syncResponseScroll() {
    if (responseOverlayRef && responseTextareaRef) {
      responseOverlayRef.scrollTop = responseTextareaRef.scrollTop;
      responseOverlayRef.scrollLeft = responseTextareaRef.scrollLeft;
    }
  }
  const STORAGE_KEY = 'apiClient.tabs.v1';
  const STORAGE_ACTIVE_KEY = 'apiClient.activeTabId.v1';
  const STORAGE_COLLECTIONS_KEY = 'apiClient.collections.v1';
  let hasLoadedFromStorage = false;

  // 自动调整 textarea 高度，保持最小高度 150px，且不超过视口底部
  function autoResizeTextarea(el: HTMLTextAreaElement | null) {
    if (!el) return;
    // 先重置高度以获取正确的 scrollHeight
    el.style.height = 'auto';

    const minHeight = 150;
    const contentHeight = el.scrollHeight;

    // 计算可用空间：窗口高度 - 当前元素距顶部的位置 - 适当的底部留白（24px）
    const rect = el.getBoundingClientRect();
    const available = window.innerHeight - rect.top - 24;
    const maxHeight = Math.max(minHeight, available);

    el.style.height = `${Math.min(Math.max(contentHeight, minHeight), maxHeight)}px`;
  }

  // 从本地存储读取已保存的请求
  function loadSavedTabs() {
    if (!browser || hasLoadedFromStorage) return;
    hasLoadedFromStorage = true;

    try {
      const savedTabs = localStorage.getItem(STORAGE_KEY);
      const savedActive = localStorage.getItem(STORAGE_ACTIVE_KEY);
      const savedCollections = localStorage.getItem(STORAGE_COLLECTIONS_KEY);

      if (savedCollections) {
        collections = JSON.parse(savedCollections);
      }

      if (savedTabs) {
        const parsed = JSON.parse(savedTabs) as TabData[];
        const tabDefaults = defaultTabFields();
        tabs = parsed.map((tab) => ({
          ...tabDefaults,
          ...tab,
          authType: tab.authType ?? tabDefaults.authType,
          bearerToken: tab.bearerToken ?? '',
          basicUsername: tab.basicUsername ?? '',
          basicPassword: tab.basicPassword ?? '',
          preRequestScript: tab.preRequestScript ?? '',
          testScript: tab.testScript ?? '',
          testResults: tab.testResults ?? [],
          requestView: tab.requestView ?? tabDefaults.requestView,
          responseSize: tab.responseSize ?? null,
          cancelled: false,
          formData: (tab.formData || []).map((item) => ({
            ...item,
            file: null
          })),
          isSending: false,
          collectionId: tab.collectionId ?? null
        }));
      }

      if (savedActive) {
        activeTabId = savedActive;
      }
    } catch (error) {
      console.error('Failed to load saved API client tabs:', error);
    }
  }

  // 保存当前请求到本地存储
  function saveTabs() {
    if (!browser) return;
    try {
      const sanitized = tabs.map((tab) => ({
        ...tab,
        formData: (tab.formData || []).map(({ key, value, enabled, type }) => ({
          key,
          value,
          enabled,
          type
        })),
        file: undefined,
        isSending: false
      }));
      localStorage.setItem(STORAGE_KEY, JSON.stringify(sanitized));
      localStorage.setItem(STORAGE_ACTIVE_KEY, activeTabId);
      localStorage.setItem(STORAGE_COLLECTIONS_KEY, JSON.stringify(collections));
    } catch (error) {
      console.error('Failed to save API client tabs:', error);
    }
  }

  // 首次加载时从本地存储恢复
  loadSavedTabs();

  // tabs、activeTabId 或 collections 变化时保存
  $effect(() => {
    tabs;
    activeTabId;
    collections;
    saveTabs();
  });

  // ── Body syntax highlighting ──
  let highlightedJson = $derived.by(() => {
    const json = activeTab?.bodyJson ?? '';
    if (!json.trim()) return '';
    if (json.length > 100000) return json;
    try {
      return hljs.highlight(json, { language: 'json' }).value;
    } catch {
      return json;
    }
  });

  let highlightedXml = $derived.by(() => {
    const xml = activeTab?.bodyXml ?? '';
    if (!xml.trim()) return '';
    if (xml.length > 100000) return xml;
    try {
      return hljs.highlight(xml, { language: 'xml' }).value;
    } catch {
      return xml;
    }
  });

  let highlightedPreRequest = $derived.by(() => {
    const script = activeTab?.preRequestScript ?? '';
    if (!script.trim()) return '';
    if (script.length > 100000) return script;
    try {
      return hljs.highlight(script, { language: 'javascript' }).value;
    } catch {
      return script;
    }
  });

  let highlightedTestScript = $derived.by(() => {
    const script = activeTab?.testScript ?? '';
    if (!script.trim()) return '';
    if (script.length > 100000) return script;
    try {
      return hljs.highlight(script, { language: 'javascript' }).value;
    } catch {
      return script;
    }
  });

  function syncJsonScroll() {
    if (jsonOverlayRef && bodyTextareaRef) {
      jsonOverlayRef.scrollTop = bodyTextareaRef.scrollTop;
      jsonOverlayRef.scrollLeft = bodyTextareaRef.scrollLeft;
    }
  }

  function syncXmlScroll() {
    if (xmlOverlayRef && bodyTextareaRef) {
      xmlOverlayRef.scrollTop = bodyTextareaRef.scrollTop;
      xmlOverlayRef.scrollLeft = bodyTextareaRef.scrollLeft;
    }
  }

  function formatJson() {
    if (!activeTab?.bodyJson?.trim()) return;
    try {
      const parsed = JSON.parse(activeTab.bodyJson);
      activeTab.bodyJson = JSON.stringify(parsed, null, 2);
      setTimeout(() => { if (bodyTextareaRef) autoResizeTextarea(bodyTextareaRef); }, 0);
    } catch {
      // ignore invalid JSON
    }
  }

  function minifyJson() {
    if (!activeTab?.bodyJson?.trim()) return;
    try {
      const parsed = JSON.parse(activeTab.bodyJson);
      activeTab.bodyJson = JSON.stringify(parsed);
      setTimeout(() => { if (bodyTextareaRef) autoResizeTextarea(bodyTextareaRef); }, 0);
    } catch {
      // ignore invalid JSON
    }
  }

  function formatXml() {
    if (!activeTab?.bodyXml?.trim()) return;
    const xml = activeTab.bodyXml.trim();
    let formatted = '';
    let indent = 0;
    const pad = '  ';
    // Simple XML formatter: tokenize tags and text
    const reg = /(<\/?[\w:.-]+[^>]*>|<!--[^]*?-->|>[^<]*<)/g;
    let last = 0;
    let match;
    while ((match = reg.exec(xml)) !== null) {
      const between = xml.slice(last, match.index).trim();
      if (between) formatted += pad.repeat(indent) + between + '\n';
      const tok = match[0];
      if (tok.startsWith('</')) {
        indent = Math.max(0, indent - 1);
        formatted += pad.repeat(indent) + tok + '\n';
      } else if (tok.startsWith('<!') || tok.startsWith('<?')) {
        formatted += pad.repeat(indent) + tok + '\n';
      } else if (tok.endsWith('/>')) {
        formatted += pad.repeat(indent) + tok + '\n';
      } else if (tok.startsWith('>')) {
        // text content + opening of next tag
        formatted += pad.repeat(indent) + tok + '\n';
      } else {
        formatted += pad.repeat(indent) + tok + '\n';
        indent++;
      }
      last = match.index + tok.length;
    }
    const trailing = xml.slice(last).trim();
    if (trailing) formatted += trailing;
    activeTab.bodyXml = formatted || xml;
    setTimeout(() => { if (bodyTextareaRef) autoResizeTextarea(bodyTextareaRef); }, 0);
  }

  // 切换 Body 类型或内容变化时自适应高度
  $effect(() => {
    bodyTextareaRef;
    setTimeout(() => autoResizeTextarea(bodyTextareaRef), 0);
  });

  function parseBulkHeaders(text: string): Header[] {
    const headers: Header[] = [];
    const lines = text.split('\n');
    
    const stripWrappingQuotes = (input: string): string => {
      const trimmed = input.trim();
      if (
        (trimmed.startsWith('"') && trimmed.endsWith('"')) ||
        (trimmed.startsWith("'") && trimmed.endsWith("'"))
      ) {
        return trimmed.slice(1, -1);
      }
      return trimmed;
    };

    for (const line of lines) {
      const trimmed = line.trim();
      if (!trimmed) continue;
      
      // 支持 key: value 或 key=value 格式
      let key = '';
      let value = '';
      
      if (trimmed.includes(':')) {
        const colonIndex = trimmed.indexOf(':');
        key = trimmed.substring(0, colonIndex).trim();
        value = stripWrappingQuotes(trimmed.substring(colonIndex + 1));
      } else if (trimmed.includes('=')) {
        const equalIndex = trimmed.indexOf('=');
        key = trimmed.substring(0, equalIndex).trim();
        value = stripWrappingQuotes(trimmed.substring(equalIndex + 1));
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
    tab.responseSize = null;
    tab.cancelled = false;

    const startTime = Date.now();
    let timeoutId: ReturnType<typeof setTimeout> | undefined;

    try {
      // 执行 Pre-request Script
      const scriptVars = runPreRequestScript(tab);

      // 解析脚本变量（pm.variables.set），无环境变量机制
      const applyScriptVars = (text: string) => text.replaceAll(/\{\{(\w+)\}\}/g, (_, key) => scriptVars[key] ?? `{{${key}}}`);
      const resolvedUrl = applyScriptVars(tab.url);

      // 构建请求头
      const requestHeaders: Record<string, string> = {};
      // 添加 Auth headers
      const authHeaders = getAuthHeaders(tab);
      Object.assign(requestHeaders, authHeaders);

      tab.headers.forEach((header) => {
        if (header.enabled && header.key.trim() && header.value.trim()) {
          requestHeaders[applyScriptVars(header.key.trim())] = applyScriptVars(header.value.trim());
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
          if (item.enabled && item.key.trim()) {
            // 对于 url-encoded，即使 value 为空也要包含（key= 的形式）
            const key = encodeURIComponent(item.key.trim());
            const value = item.value.trim() ? encodeURIComponent(item.value.trim()) : '';
            formDataPairs.push(`${key}=${value}`);
          }
        });
        if (formDataPairs.length > 0) {
          requestBody = formDataPairs.join('&');
          // 只有在没有 Content-Type header 时才添加
          if (!requestHeaders['Content-Type'] && !requestHeaders['content-type']) {
            requestHeaders['Content-Type'] = 'application/x-www-form-urlencoded';
          }
        }
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
            url: resolvedUrl,
            method: tab.method,
            headers: requestHeaders,
            body: bodyForRequest,
            follow_redirects: followRedirects,
            timeout_secs: requestTimeout > 0 ? requestTimeout : undefined
          }
        }) as {
          status: number;
          headers: Record<string, string>;
          body: string;
          error: string | null;
        };

        const endTime = Date.now();
        tab.responseTime = endTime - startTime;

        // 已取消：丢弃回包，不写 response
        if (tab.cancelled) {
          // discard
        } else if (response.error) {
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
          tab.responseSize = new TextEncoder().encode(tab.responseBody).length;

          // 更新标签页名称
          updateTabName(tab);
        }
      } else {
        // 回退到使用 fetch（可能在浏览器中运行或 Tauri API 不可用）
        abortController = new AbortController();
        const requestOptions: RequestInit = {
          method: tab.method,
          headers: requestHeaders,
          redirect: followRedirects ? 'follow' : 'manual',
          signal: abortController.signal,
        };

        // Timeout
        if (requestTimeout > 0) {
          timeoutId = setTimeout(() => {
            try {
              abortController?.abort();
            } catch {}
          }, requestTimeout * 1000);
        }

        if (requestBody !== undefined) {
          // 如果是 FormData，不要设置 Content-Type，让浏览器自动设置
          if (requestBody && typeof requestBody === 'object' && 'append' in requestBody) {
            // 删除 Content-Type，让浏览器自动设置 multipart/form-data boundary
            delete (requestOptions.headers as Record<string, string>)['Content-Type'];
          }
          requestOptions.body = requestBody as any;
        }

        const response = await fetch(resolvedUrl, requestOptions);
        const endTime = Date.now();
        tab.responseTime = endTime - startTime;

        // 已取消：丢弃回包，不写 response
        if (!tab.cancelled) {
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
          tab.responseSize = new TextEncoder().encode(tab.responseBody).length;

          // 更新标签页名称
          updateTabName(tab);
        }
      }
    } catch (err) {
      tab.responseTime = Date.now() - startTime;
      if (!tab.cancelled) {
        tab.error = err instanceof Error ? err.message : t('apiClient.requestFailed');
      }
    } finally {
        tab.isSending = false;
        abortController = null;
        if (timeoutId) clearTimeout(timeoutId);
        addHistoryEntry(tab);
        if (!tab.cancelled) {
          runTestScript(tab);
        }
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

  function formatSize(bytes: number | null): string {
    if (bytes === null || bytes === undefined) return '';
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / 1024 / 1024).toFixed(2)} MB`;
  }

  function saveResponse(tab: TabData) {
    if (!tab.responseBody) return;
    const ct = tab.responseHeaders['content-type'] ?? tab.responseHeaders['Content-Type'] ?? '';
    const ext = ct.includes('json') ? 'json' : 'txt';
    browserDownload(tab.responseBody, `response-${tab.responseStatus ?? 'none'}-${Date.now()}.${ext}`);
  }

  // 快捷键：Cmd/Ctrl + Enter 发送当前请求
  function handleGlobalKeydown(e: KeyboardEvent) {
    if ((e.metaKey || e.ctrlKey) && e.key === 'Enter') {
      if (activeTab && !activeTab.isSending) {
        e.preventDefault();
        sendRequest(activeTab);
      }
    }
  }

  function clear(tab: TabData) {
    const keepView = tab.requestView;
    const defaults = defaultTabFields();
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
    tab.responseSize = null;
    tab.cancelled = false;
    tab.error = '';
    tab.authType = defaults.authType;
    tab.bearerToken = '';
    tab.basicUsername = '';
    tab.basicPassword = '';
    tab.preRequestScript = '';
    tab.testScript = '';
    tab.testResults = [];
    tab.requestView = keepView;
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
      // 注意：-d 和 --data 的优先级应该低于 --data-raw、--data-urlencode 等
      const dataPatterns = [
        { pattern: /--data-raw\s+((?:'[^']*'|"[^"]*"|[^\s]+))/gi, type: 'raw' },
        { pattern: /--data-urlencode\s+((?:'[^']*'|"[^"]*"|[^\s]+))/gi, type: 'urlencode' },
        { pattern: /--data-binary\s+((?:'[^']*'|"[^"]*"|[^\s]+))/gi, type: 'binary' },
        { pattern: /(?:^|\s)-d\s+((?:'[^']*'|"[^"]*"|[^\s]+))/gi, type: 'data' },
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
      
      // 检查是否有 Content-Type: application/x-www-form-urlencoded header
      const hasUrlEncodedHeader = result.headers?.some(h => 
        h.key.toLowerCase() === 'content-type' && 
        h.value.toLowerCase().includes('application/x-www-form-urlencoded')
      ) || false;
      
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
              } else if (equalIndex === 0) {
                // 处理 key= 的情况（空值）
                try {
                  const key = decodeURIComponent(pair.substring(1));
                  formData.push({ key, value: '', enabled: true, type: 'text', file: null });
                } catch {
                  formData.push({ key: pair.substring(1), value: '', enabled: true, type: 'text', file: null });
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
        } else if (bodyType === 'data' || bodyType === 'raw') {
          // 对于 -d 或 --data，检查是否是 URL encoded 格式
          // 1. 如果有 Content-Type: application/x-www-form-urlencoded header
          // 2. 或者 body 内容是 key=value&key=value 格式（不包含 JSON 或 XML 特征）
          const isUrlEncodedFormat = bodyContent.includes('=') && 
                                     bodyContent.includes('&') && 
                                     !bodyContent.includes('{') && 
                                     !bodyContent.includes('[') &&
                                     !bodyContent.trim().startsWith('<');
          
          if (hasUrlEncodedHeader || isUrlEncodedFormat) {
            // 解析为 url-encoded
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
              } else if (equalIndex === 0) {
                // 处理 key= 的情况（空值）
                try {
                  const key = decodeURIComponent(pair.substring(1));
                  formData.push({ key, value: '', enabled: true, type: 'text', file: null });
                } catch {
                  formData.push({ key: pair.substring(1), value: '', enabled: true, type: 'text', file: null });
                }
              }
            });
            result.formData = formData;
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

  // ── Bulk export / import ──
  function sanitizeTabForExport(tab: TabData) {
    return {
      name: tab.name,
      method: tab.method,
      url: tab.url,
      headers: tab.headers.filter(h => h.key.trim()),
      bodyType: tab.bodyType,
      bodyJson: tab.bodyJson,
      bodyText: tab.bodyText,
      bodyXml: tab.bodyXml,
      formData: (tab.formData || []).map(({ key, value, enabled, type }) => ({ key, value, enabled, type })),
      collectionId: tab.collectionId,
      authType: tab.authType,
      bearerToken: tab.bearerToken,
      basicUsername: tab.basicUsername,
      basicPassword: tab.basicPassword,
      preRequestScript: tab.preRequestScript,
      testScript: tab.testScript,
      requestView: tab.requestView
    };
  }

  function exportAllRequests() {
    const data = {
      collections: collections.map(c => ({ id: c.id, name: c.name })),
      requests: tabs.map(sanitizeTabForExport)
    };
    downloadJson(data, `Kairoa-api-all-${Date.now()}.json`);
  }

  function exportCollection(colId: string) {
    const col = collections.find(c => c.id === colId);
    if (!col) return;
    const data = {
      collections: [{ id: col.id, name: col.name }],
      requests: tabs.filter(t => t.collectionId === colId).map(sanitizeTabForExport)
    };
    downloadJson(data, `Kairoa-api-${col.name}-${Date.now()}.json`);
  }

  function downloadJson(data: any, filename: string) {
    const json = JSON.stringify(data, null, 2);
    const isTauri = typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;
    if (isTauri) {
      // Tauri: use dialog save
      import('@tauri-apps/plugin-dialog').then(async (dialogMod) => {
        const { save } = dialogMod;
        const filePath = await save({
          defaultPath: filename,
          filters: [{ name: 'JSON', extensions: ['json'] }]
        });
        if (!filePath) return;
        const encoder = new TextEncoder();
        const fsMod = await import('@tauri-apps/plugin-fs');
        await fsMod.writeFile(filePath, encoder.encode(json));
      }).catch((err) => {
        console.error('Tauri export failed, falling back to browser download:', err);
        browserDownload(json, filename);
      });
    } else {
      browserDownload(json, filename);
    }
  }

  function browserDownload(content: string, filename: string) {
    const blob = new Blob([content], { type: 'application/json;charset=utf-8' });
    const url = URL.createObjectURL(blob);
    const link = document.createElement('a');
    link.href = url;
    link.download = filename;
    document.body.appendChild(link);
    link.click();
    document.body.removeChild(link);
    URL.revokeObjectURL(url);
  }

  let showImportDialog = $state(false);
  let importFileInput = $state<HTMLInputElement | null>(null);

  function triggerImport() {
    if (importFileInput) importFileInput.click();
  }

  async function handleImportFile(event: Event) {
    const target = event.target as HTMLInputElement;
    const file = target.files?.[0];
    if (!file) return;
    try {
      const text = await file.text();
      const data = JSON.parse(text);
      if (!data.requests || !Array.isArray(data.requests)) return;

      // Import collections
      if (data.collections && Array.isArray(data.collections)) {
        for (const impCol of data.collections) {
          if (!collections.find(c => c.id === impCol.id)) {
            collections = [...collections, { id: impCol.id, name: impCol.name, collapsed: false }];
          }
        }
      }

      // Import requests
      const knownColIds = new Set(collections.map(c => c.id));
      for (const req of data.requests) {
        const rawColId = req.collectionId || null;
        const safeColId = rawColId && knownColIds.has(rawColId) ? rawColId : null;
        const newTab: TabData = {
          id: crypto.randomUUID(),
          name: req.name || 'Imported',
          method: req.method || 'GET',
          url: req.url || '',
          headers: req.headers || [{ key: '', value: '', enabled: true }],
          bodyType: req.bodyType || 'none',
          bodyJson: req.bodyJson || '',
          bodyText: req.bodyText || '',
          bodyXml: req.bodyXml || '',
          formData: (req.formData || []).map((item: any) => ({ ...item, file: null })),
          isSending: false,
          responseStatus: null,
          responseHeaders: {},
          responseBody: '',
          responseTime: null,
          error: '',
          copied: false,
          collectionId: safeColId,
          order: tabs.length,
          ...defaultTabFields(),
          authType: req.authType || 'none',
          bearerToken: req.bearerToken || '',
          basicUsername: req.basicUsername || '',
          basicPassword: req.basicPassword || '',
          preRequestScript: req.preRequestScript || '',
          testScript: req.testScript || '',
          requestView: req.requestView || 'headers'
        };
        tabs = [...tabs, newTab];
      }
    } catch (err) {
      console.error('Import failed:', err);
    }
    target.value = '';
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

  // GET/HEAD/OPTIONS 不再强制清空 Body：Body Tab 仅禁用并提示，数据保留，requestView 保持不变

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
  let urlInputRef: HTMLInputElement | null = $state(null);
  
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

<svelte:window onkeydown={handleGlobalKeydown} />

<div class="w-full ml-0 mr-0 p-2 flex gap-4 h-[calc(100vh-2rem)]">
  <input type="file" accept=".json" bind:this={importFileInput} onchange={handleImportFile} class="hidden" />
  <!-- Collections Sidebar -->
  <div class="card p-0 flex-shrink-0 flex flex-col overflow-visible relative" style="width: {sidebarWidth}px; {isResizing ? 'pointer-events: none;' : ''}">
    <!-- Resize handle on right edge (outside border) -->
    <div
      class="absolute top-1/2 -right-[9px] -translate-y-1/2 w-3 h-12 cursor-col-resize z-50 flex items-center justify-center group"
      onmousedown={startResize}
      role="separator"
      tabindex="-1"
    >
      <div class="w-0.5 h-6 rounded-full transition-colors {isResizing ? 'bg-primary-500' : 'bg-gray-300 dark:bg-gray-600 group-hover:bg-primary-400 dark:group-hover:bg-primary-500'}"></div>
    </div>
    <div class="flex items-center justify-between px-3 py-2.5 border-b border-gray-200 dark:border-gray-700 flex-shrink-0">
      <span class="text-sm font-semibold text-gray-700 dark:text-gray-300">{t('apiClient.collections')}</span>
      <div class="relative">
        <button
          onclick={() => showSidebarMenu = !showSidebarMenu}
          class="p-1 text-gray-500 hover:text-gray-900 dark:text-gray-400 dark:hover:text-gray-200 rounded hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors"
          title="Menu"
        >
          <MoreVertical class="w-4 h-4" />
        </button>
        {#if showSidebarMenu}
          <!-- svelte-ignore a11y_click_events_have_key_events -->
          <!-- svelte-ignore a11y_no_static_element_interactions -->
          <div class="fixed inset-0 z-10" onclick={() => showSidebarMenu = false}></div>
          <div class="absolute right-0 mt-1 w-44 bg-white dark:bg-gray-800 rounded-lg shadow-lg border border-gray-200 dark:border-gray-700 z-20 py-1">
            <button
              onclick={() => { addCollection(); showSidebarMenu = false; }}
              class="w-full text-left px-3 py-1.5 text-sm text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 flex items-center gap-2"
            >
              <FolderPlus class="w-3.5 h-3.5" /> {t('apiClient.newCollection')}
            </button>
            <button
              onclick={() => { exportAllRequests(); showSidebarMenu = false; }}
              class="w-full text-left px-3 py-1.5 text-sm text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 flex items-center gap-2"
            >
              <Download class="w-3.5 h-3.5" /> {t('apiClient.exportAll')}
            </button>
            <button
              onclick={() => { triggerImport(); showSidebarMenu = false; }}
              class="w-full text-left px-3 py-1.5 text-sm text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 flex items-center gap-2"
            >
              <Upload class="w-3.5 h-3.5" /> {t('apiClient.importRequests')}
            </button>
            <div class="border-t border-gray-200 dark:border-gray-700 my-1"></div>
            <button
              onclick={() => { showHistory = true; showSidebarMenu = false; }}
              class="w-full text-left px-3 py-1.5 text-sm text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 flex items-center gap-2"
            >
              <Clock class="w-3.5 h-3.5" /> {t('apiClient.history')}
            </button>
          </div>
        {/if}
      </div>
    </div>
    <div class="px-2 py-2 border-b border-gray-200 dark:border-gray-700 flex items-center gap-1.5 flex-shrink-0">
      <div class="relative flex-1">
        <Search class="w-3.5 h-3.5 absolute left-2 top-1/2 -translate-y-1/2 text-gray-400 dark:text-gray-500" />
        <input
          type="text"
          bind:value={searchText}
          placeholder={t('apiClient.search')}
          class="w-full pl-7 pr-2 py-1 text-xs bg-gray-100 dark:bg-gray-700/50 border border-gray-200 dark:border-gray-600 rounded text-gray-900 dark:text-gray-100 placeholder:text-gray-400 dark:placeholder:text-gray-500 focus:outline-none focus:ring-1 focus:ring-primary-500"
        />
      </div>
    </div>
    <div class="flex-1 overflow-y-auto py-1">
      {#each collections as col}
        {@const colFiltered = filteredTabsInCollection(col.id)}
        {#if !searchText.trim() || colFiltered.length > 0}
        <!-- svelte-ignore a11y_no_static_element_interactions -->
        <div
          class="group/col"
          ondragover={(e) => e.preventDefault()}
          ondrop={(e) => onDropOnCollection(e, col.id)}
        >
          <!-- svelte-ignore a11y_no_static_element_interactions -->
          <div
            class="flex items-center px-2 py-1 hover:bg-gray-50 dark:hover:bg-gray-700/50 transition-colors"
            ondragover={(e) => e.preventDefault()}
            ondrop={(e) => onDropOnCollection(e, col.id)}
          >
            <button
              onclick={() => toggleCollapse(col.id)}
              class="p-0.5 text-gray-400 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200 flex-shrink-0"
            >
              {#if col.collapsed}
                <ChevronRight class="w-3.5 h-3.5" />
              {:else}
                <ChevronDown class="w-3.5 h-3.5" />
              {/if}
            </button>
            <Folder class="w-3.5 h-3.5 text-gray-400 dark:text-gray-500 flex-shrink-0 ml-0.5" />
            {#if editingCollectionId === col.id}
              <input
                type="text"
                value={editingCollectionName}
                use:focusAndSelect
                oninput={(e) => editingCollectionName = (e.target as HTMLInputElement).value}
                onkeydown={(e) => { if (e.key === 'Enter') confirmRenameCollection(); if (e.key === 'Escape') editingCollectionId = null; }}
                onblur={confirmRenameCollection}
                class="flex-1 ml-1.5 text-sm bg-transparent border-b border-primary-500 outline-none text-gray-900 dark:text-gray-100"
                id={`col-edit-${col.id}`}
              />
            {:else}
              <button
                onclick={() => toggleCollapse(col.id)}
                ondblclick={() => startRenameCollection(col.id)}
                class="flex-1 text-left text-sm font-medium text-gray-700 dark:text-gray-300 ml-1.5 truncate"
                title={t('apiClient.rename')}
              >
                {col.name}
              </button>
            {/if}
            <span class="ml-1 text-[11px] px-1.5 py-0.5 rounded-full bg-gray-100 dark:bg-gray-700 text-gray-500 dark:text-gray-400 flex-shrink-0 tabular-nums">{tabsInCollection(col.id).length}</span>
            <button
              onclick={() => addTab(col.id)}
              class="p-0.5 text-gray-400 hover:text-gray-700 dark:hover:text-gray-200 opacity-0 group-hover/col:opacity-100 focus:opacity-100 transition-opacity flex-shrink-0"
              title={t('apiClient.newRequest')}
            >
              <Plus class="w-3.5 h-3.5" />
            </button>
            <button
              onclick={() => showCollectionMenu = showCollectionMenu === col.id ? null : col.id}
              class="p-0.5 text-gray-400 hover:text-gray-700 dark:hover:text-gray-200 opacity-0 group-hover/col:opacity-100 transition-opacity flex-shrink-0"
            >
              <MoreVertical class="w-3.5 h-3.5" />
            </button>
          </div>
          {#if showCollectionMenu === col.id}
            <!-- svelte-ignore a11y_click_events_have_key_events -->
            <!-- svelte-ignore a11y_no_static_element_interactions -->
            <div class="fixed inset-0 z-10" onclick={() => showCollectionMenu = null}></div>
            <div class="absolute ml-7 mt-1 w-40 bg-white dark:bg-gray-800 rounded-lg shadow-lg border border-gray-200 dark:border-gray-700 z-20 py-1">
              <button
                onclick={() => { addTab(col.id); showCollectionMenu = null; }}
                class="w-full text-left px-3 py-1.5 text-sm text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 flex items-center gap-2"
              >
                <Plus class="w-3.5 h-3.5" /> {t('apiClient.newRequest')}
              </button>
              <button
                onclick={() => { startRenameCollection(col.id); showCollectionMenu = null; }}
                class="w-full text-left px-3 py-1.5 text-sm text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 flex items-center gap-2"
              >
                <Pencil class="w-3.5 h-3.5" /> {t('apiClient.rename')}
              </button>
              <button
                onclick={() => { duplicateCollection(col.id); showCollectionMenu = null; }}
                class="w-full text-left px-3 py-1.5 text-sm text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 flex items-center gap-2"
              >
                <CopyIcon class="w-3.5 h-3.5" /> {t('apiClient.duplicate')}
              </button>
              <button
                onclick={() => { exportCollection(col.id); showCollectionMenu = null; }}
                class="w-full text-left px-3 py-1.5 text-sm text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 flex items-center gap-2"
              >
                <Download class="w-3.5 h-3.5" /> {t('apiClient.exportCollection')}
              </button>
              <div class="border-t border-gray-200 dark:border-gray-700 my-1"></div>
              <button
                onclick={() => { deleteCollection(col.id); showCollectionMenu = null; }}
                class="w-full text-left px-3 py-1.5 text-sm text-red-600 dark:text-red-400 hover:bg-red-50 dark:hover:bg-red-900/20 flex items-center gap-2"
              >
                <Trash2 class="w-3.5 h-3.5" /> {t('apiClient.delete')}
              </button>
            </div>
          {/if}
          {#if !col.collapsed || searchText.trim()}
            {#each colFiltered as tab (tab.id)}
              <div
                class="group/req flex items-center pl-8 pr-2 py-1 cursor-pointer {activeTabId === tab.id ? 'bg-primary-50 dark:bg-primary-900/20 text-primary-700 dark:text-primary-300' : 'hover:bg-gray-50 dark:hover:bg-gray-700/50 text-gray-600 dark:text-gray-400'} transition-colors relative"
                draggable="true"
                ondragstart={(e) => onDragStart(e, tab.id)}
                ondragover={(e) => e.preventDefault()}
                ondrop={(e) => onDrop(e, tab.id, col.id)}
                onclick={() => setActiveTab(tab.id)}
              >
                <span class="text-xs font-mono font-bold flex-shrink-0 w-14 truncate {tab.method === 'GET' ? 'text-green-600 dark:text-green-400' : tab.method === 'POST' ? 'text-blue-600 dark:text-blue-400' : tab.method === 'DELETE' ? 'text-red-600 dark:text-red-400' : tab.method === 'PUT' || tab.method === 'PATCH' ? 'text-yellow-600 dark:text-yellow-400' : 'text-gray-500 dark:text-gray-400'}">{tab.method}</span>
                <span class="text-sm truncate flex-1">{tab.name}</span>
                <button
                  onclick={(e) => { e.stopPropagation(); duplicateTab(tab.id); }}
                  class="p-0.5 text-gray-400 hover:text-gray-700 dark:hover:text-gray-200 opacity-0 group-hover/req:opacity-100 transition-opacity flex-shrink-0"
                  title={t('apiClient.duplicate')}
                >
                  <CopyIcon class="w-3 h-3" />
                </button>
                <button
                  onclick={(e) => { e.stopPropagation(); removeTab(tab.id); }}
                  class="p-0.5 text-gray-400 hover:text-red-600 dark:hover:text-red-400 opacity-0 group-hover/req:opacity-100 transition-opacity flex-shrink-0"
                  title={t('apiClient.delete')}
                >
                  <Trash2 class="w-3 h-3" />
                </button>
                <button
                  onclick={(e) => { e.stopPropagation(); showMoveMenu = showMoveMenu === tab.id ? null : tab.id; }}
                  class="p-0.5 text-gray-400 hover:text-gray-700 dark:hover:text-gray-200 opacity-0 group-hover/req:opacity-100 transition-opacity flex-shrink-0"
                >
                  <ArrowRight class="w-3 h-3" />
                </button>
                {#if showMoveMenu === tab.id}
                  <!-- svelte-ignore a11y_click_events_have_key_events -->
                  <!-- svelte-ignore a11y_no_static_element_interactions -->
                  <div class="fixed inset-0 z-10" onclick={() => showMoveMenu = null}></div>
                  <div class="absolute right-2 mt-1 w-48 bg-white dark:bg-gray-800 rounded-lg shadow-lg border border-gray-200 dark:border-gray-700 z-20 py-1 max-h-60 overflow-y-auto">
                    <button
                      onclick={(e) => { e.stopPropagation(); moveTabToCollection(tab.id, null); showMoveMenu = null; }}
                      class="w-full text-left px-3 py-1.5 text-sm text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 flex items-center gap-2"
                    >
                      <Inbox class="w-3.5 h-3.5" /> {t('apiClient.ungrouped')}
                    </button>
                    {#each collections as c}
                      {#if c.id !== col.id}
                        <button
                          onclick={(e) => { e.stopPropagation(); moveTabToCollection(tab.id, c.id); showMoveMenu = null; }}
                          class="w-full text-left px-3 py-1.5 text-sm text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 flex items-center gap-2"
                        >
                          <Folder class="w-3.5 h-3.5" /> {c.name}
                        </button>
                      {/if}
                    {/each}
                  </div>
                {/if}
              </div>
            {/each}
            {#if colFiltered.length === 0 && !searchText.trim()}
              <!-- svelte-ignore a11y_no_static_element_interactions -->
              <div
                class="pl-8 pr-2 py-1 text-xs text-gray-400 dark:text-gray-500 italic"
                ondragover={(e) => e.preventDefault()}
                ondrop={(e) => onDropOnCollection(e, col.id)}
              >{t('apiClient.emptyCollection')}</div>
            {/if}
          {/if}
        </div>
        {/if}
      {/each}

      <!-- Ungrouped requests -->
      {#if collections.length > 0 && ungroupedTabs.length > 0}
        <div class="border-t border-gray-200 dark:border-gray-700 mt-1 pt-1"></div>
      {/if}
      {#if ungroupedTabs.length > 0 || collections.length === 0}
        <div class="px-2 py-1 text-xs font-medium text-gray-400 dark:text-gray-500 uppercase tracking-wide">{t('apiClient.ungrouped')}</div>
      {/if}
      {#each filteredUngroupedTabs as tab (tab.id)}
        <div
          class="group/req flex items-center pl-3 pr-2 py-1 cursor-pointer {activeTabId === tab.id ? 'bg-primary-50 dark:bg-primary-900/20 text-primary-700 dark:text-primary-300' : 'hover:bg-gray-50 dark:hover:bg-gray-700/50 text-gray-600 dark:text-gray-400'} transition-colors relative"
          draggable="true"
          ondragstart={(e) => onDragStart(e, tab.id)}
          ondragover={(e) => e.preventDefault()}
          ondrop={(e) => onDrop(e, tab.id, null)}
          onclick={() => setActiveTab(tab.id)}
        >
          <span class="text-xs font-mono font-bold flex-shrink-0 w-14 truncate {tab.method === 'GET' ? 'text-green-600 dark:text-green-400' : tab.method === 'POST' ? 'text-blue-600 dark:text-blue-400' : tab.method === 'DELETE' ? 'text-red-600 dark:text-red-400' : tab.method === 'PUT' || tab.method === 'PATCH' ? 'text-yellow-600 dark:text-yellow-400' : 'text-gray-500 dark:text-gray-400'}">{tab.method}</span>
          <span class="text-sm truncate flex-1">{tab.name}</span>
          <button
            onclick={(e) => { e.stopPropagation(); duplicateTab(tab.id); }}
            class="p-0.5 text-gray-400 hover:text-gray-700 dark:hover:text-gray-200 opacity-0 group-hover/req:opacity-100 transition-opacity flex-shrink-0"
            title={t('apiClient.duplicate')}
          >
            <CopyIcon class="w-3 h-3" />
          </button>
          <button
            onclick={(e) => { e.stopPropagation(); removeTab(tab.id); }}
            class="p-0.5 text-gray-400 hover:text-red-600 dark:hover:text-red-400 opacity-0 group-hover/req:opacity-100 transition-opacity flex-shrink-0"
            title={t('apiClient.delete')}
          >
            <Trash2 class="w-3 h-3" />
          </button>
          {#if collections.length > 0}
            <button
              onclick={(e) => { e.stopPropagation(); showMoveMenu = showMoveMenu === tab.id ? null : tab.id; }}
              class="p-0.5 text-gray-400 hover:text-gray-700 dark:hover:text-gray-200 opacity-0 group-hover/req:opacity-100 transition-opacity flex-shrink-0"
            >
              <ArrowRight class="w-3 h-3" />
            </button>
            {#if showMoveMenu === tab.id}
              <!-- svelte-ignore a11y_click_events_have_key_events -->
              <!-- svelte-ignore a11y_no_static_element_interactions -->
              <div class="fixed inset-0 z-10" onclick={() => showMoveMenu = null}></div>
              <div class="absolute right-2 mt-1 w-48 bg-white dark:bg-gray-800 rounded-lg shadow-lg border border-gray-200 dark:border-gray-700 z-20 py-1 max-h-60 overflow-y-auto">
                {#each collections as c}
                  <button
                    onclick={(e) => { e.stopPropagation(); moveTabToCollection(tab.id, c.id); showMoveMenu = null; }}
                    class="w-full text-left px-3 py-1.5 text-sm text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 flex items-center gap-2"
                  >
                    <Folder class="w-3.5 h-3.5" /> {c.name}
                  </button>
                {/each}
              </div>
            {/if}
          {/if}
        </div>
      {/each}
    </div>
    <div class="border-t border-gray-200 dark:border-gray-700 p-2 flex-shrink-0">
      <button
        onclick={() => addTab(null)}
        class="w-full btn-secondary text-sm flex items-center justify-center gap-1.5"
      >
        <Plus class="w-4 h-4" /> {t('apiClient.newRequest')}
      </button>
    </div>
  </div>

  <!-- Main Content -->
  <div class="flex-1 flex flex-col gap-4 min-w-0 overflow-hidden">

  <!-- 请求配置卡片 -->
  <div class="card">
    <div class="space-y-4">
      <!-- 方法和 URL -->
      <div class="flex gap-2 items-start overflow-visible">
        <select
          bind:value={activeTab.method}
          class="input w-32 h-10"
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
          bind:this={urlInputRef}
          bind:value={activeTab.url}
          placeholder={t('apiClient.urlPlaceholder')}
          class="input flex-1 font-mono text-sm h-10"
          onkeydown={(e) => { if (e.key === 'Enter' && activeTab && !activeTab.isSending) sendRequest(activeTab); }}
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
          {#if activeTab.isSending}
            <button
              onclick={() => cancelRequest(activeTab)}
              class="h-10 px-4 text-white transition-colors font-medium hover:opacity-90 flex items-center gap-2 rounded-lg bg-red-500 hover:bg-red-600"
              title={t('apiClient.cancelRequest')}
            >
              <X class="w-4 h-4" />
              {t('apiClient.cancelRequest')}
            </button>
          {/if}
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
                role="menu"
                tabindex="-1"
                onclick={(e) => e.stopPropagation()}
                onkeydown={(e) => e.stopPropagation()}
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
                <div class="border-t border-gray-200 dark:border-gray-700 my-1"></div>
                <button
                  onclick={() => { exportAllRequests(); showDropdown = false; }}
                  class="w-full px-4 py-2 text-left text-sm text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 flex items-center gap-3 transition-colors"
                >
                  <Download class="w-4 h-4" />
                  <span>{t('apiClient.exportAll')}</span>
                </button>
                <button
                  onclick={() => { triggerImport(); showDropdown = false; }}
                  class="w-full px-4 py-2 text-left text-sm text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 flex items-center gap-3 transition-colors"
                >
                  <Upload class="w-4 h-4" />
                  <span>{t('apiClient.importRequests')}</span>
                </button>
                <div class="border-t border-gray-200 dark:border-gray-700 my-1"></div>
                <button
                  onclick={() => { exportCurlCommand(activeTab); showDropdown = false; }}
                  disabled={activeTab.isSending || !activeTab.url.trim()}
                  class="w-full px-4 py-2 text-left text-sm text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 flex items-center gap-3 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
                >
                  <Code class="w-4 h-4" />
                  <span>{t('apiClient.showCode')}</span>
                </button>
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

      <!-- Headers and Body Tabs -->
      <div>
        <!-- Tab Navigation -->
        <div class="flex items-center border-b border-gray-200 dark:border-gray-700 mb-4">
          <button
            onclick={() => { if (activeTab) activeTab.requestView = 'headers'; }}
            class="px-4 py-2 border-b-2 transition-colors {activeTab?.requestView === 'headers'
              ? 'border-primary-600 dark:border-primary-400 text-primary-600 dark:text-primary-400'
              : 'border-transparent text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-gray-200'}"
          >
            <span class="text-sm font-medium">{t('apiClient.headers')}</span>
          </button>
          <button
            onclick={() => { if (activeTab) activeTab.requestView = 'params'; }}
            class="px-4 py-2 border-b-2 transition-colors {activeTab?.requestView === 'params'
              ? 'border-primary-600 dark:border-primary-400 text-primary-600 dark:text-primary-400'
              : 'border-transparent text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-gray-200'}"
          >
            <span class="text-sm font-medium">Params{#if queryParams.length > 0}<span class="ml-1 text-[11px] px-1.5 py-0.5 rounded-full bg-gray-100 dark:bg-gray-700 text-gray-500 dark:text-gray-400">{queryParams.length}</span>{/if}</span>
          </button>
          <button
            onclick={() => { if (activeTab) activeTab.requestView = 'body'; }}
            disabled={activeTab?.method === 'GET' || activeTab?.method === 'HEAD' || activeTab?.method === 'OPTIONS'}
            title={t('apiClient.bodyNotAvailable')}
            class="px-4 py-2 border-b-2 transition-colors disabled:opacity-40 disabled:cursor-not-allowed {activeTab?.requestView === 'body'
              ? 'border-primary-600 dark:border-primary-400 text-primary-600 dark:text-primary-400'
              : 'border-transparent text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-gray-200'}"
          >
            <span class="text-sm font-medium">{t('apiClient.body')}</span>
          </button>
          <button
            onclick={() => { if (activeTab) activeTab.requestView = 'auth'; }}
            class="px-4 py-2 border-b-2 transition-colors {activeTab?.requestView === 'auth'
              ? 'border-primary-600 dark:border-primary-400 text-primary-600 dark:text-primary-400'
              : 'border-transparent text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-gray-200'}"
          >
            <span class="text-sm font-medium flex items-center gap-1">
              <Key class="w-3.5 h-3.5" />
              {t('apiClient.auth')}
            </span>
          </button>
          <button
            onclick={() => { if (activeTab) activeTab.requestView = 'settings'; }}
            class="px-4 py-2 border-b-2 transition-colors {activeTab?.requestView === 'settings'
              ? 'border-primary-600 dark:border-primary-400 text-primary-600 dark:text-primary-400'
              : 'border-transparent text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-gray-200'}"
          >
            <span class="text-sm font-medium">{t('apiClient.settings')}</span>
          </button>
          <button
            onclick={() => { if (activeTab) activeTab.requestView = 'preRequest'; }}
            class="px-4 py-2 border-b-2 transition-colors {activeTab?.requestView === 'preRequest'
              ? 'border-primary-600 dark:border-primary-400 text-primary-600 dark:text-primary-400'
              : 'border-transparent text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-gray-200'}"
          >
            <span class="text-sm font-medium flex items-center gap-1">
              <Code class="w-3.5 h-3.5" />
              {t('apiClient.preRequestScript')}
            </span>
          </button>
          <button
            onclick={() => { if (activeTab) activeTab.requestView = 'tests'; }}
            class="px-4 py-2 border-b-2 transition-colors {activeTab?.requestView === 'tests'
              ? 'border-primary-600 dark:border-primary-400 text-primary-600 dark:text-primary-400'
              : 'border-transparent text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-gray-200'}"
          >
            <span class="text-sm font-medium flex items-center gap-1">
              <FileCode class="w-3.5 h-3.5" />
              {t('apiClient.tests')}
              {#if activeTab && activeTab.testResults.length > 0}
                <span class="ml-1 text-xs px-1.5 py-0.5 rounded-full {activeTab.testResults.every(r => r.passed) ? 'bg-green-100 text-green-700 dark:bg-green-900/30 dark:text-green-400' : 'bg-red-100 text-red-700 dark:bg-red-900/30 dark:text-red-400'}">
                  {activeTab.testResults.filter(r => r.passed).length}/{activeTab.testResults.length}
                </span>
              {/if}
            </span>
          </button>
        </div>

        <!-- Tab Content -->
        {#if activeTab?.requestView === 'headers'}
          <!-- Headers Content -->
          <div>
            <div class="flex items-center justify-between mb-3 gap-2 flex-wrap">
              <div class="flex items-center gap-2">
                <button
                  onclick={() => { bulkHeaderText = ''; showBulkHeaderDialog = true; }}
                  class="btn-secondary text-sm"
                >
                  {t('apiClient.bulkAdd')}
                </button>
                <button
                  onclick={() => addHeader(activeTab)}
                  class="btn-secondary text-sm"
                >
                  {t('apiClient.addHeader')}
                </button>
              </div>
              <button
                onclick={() => copyHeaders(activeTab)}
                class="btn-secondary text-sm transition-all duration-200 {headersCopied ? 'bg-green-500 hover:bg-green-600 text-white' : ''}"
                title="Copy headers"
              >
                {#if headersCopied}
                  <Check class="w-4 h-4 inline mr-1" />
                  {t('common.copied')}
                {:else}
                  <Copy class="w-4 h-4 inline mr-1" />
                  Copy Headers
                {/if}
              </button>
            </div>
            <div class="border border-gray-200 dark:border-gray-700 rounded-lg overflow-hidden bg-white dark:bg-gray-800">
              <div class="divide-y divide-gray-200 dark:divide-gray-700">
                {#each activeTab.headers as header, index}
                  <div class="group relative flex items-center hover:bg-gray-50 dark:hover:bg-gray-700/50 transition-colors">
                    <!-- Checkbox (minimal, left side) -->
                    <div class="px-2 py-1 flex items-center opacity-60 group-hover:opacity-100 transition-opacity">
                      <input
                        type="checkbox"
                        bind:checked={header.enabled}
                        class="w-3.5 h-3.5 text-primary-600 bg-gray-100 border-gray-300 rounded focus:ring-primary-500 dark:focus:ring-primary-600 dark:ring-offset-gray-800 focus:ring-1 dark:bg-gray-700 dark:border-gray-600 cursor-pointer"
                        title={header.enabled ? '禁用' : '启用'}
                      />
                    </div>
                    <!-- Key Column -->
                    <div class="flex-1 px-3 py-1 border-r border-gray-200 dark:border-gray-700 min-w-[200px]">
                      <input
                        type="text"
                        bind:value={header.key}
                        placeholder={t('apiClient.headerKey')}
                        class="w-full px-0 py-0 text-sm bg-transparent border-none outline-none text-gray-900 dark:text-gray-100 placeholder:text-gray-400 dark:placeholder:text-gray-500 focus:ring-0 disabled:opacity-50 disabled:cursor-not-allowed"
                        disabled={!header.enabled}
                        style="ime-mode: disabled;"
                        inputmode="text"
                        autocomplete="off"
                        autocorrect="off"
                        autocapitalize="off"
                        spellcheck="false"
                      />
                    </div>
                    <!-- Value Column -->
                    <div class="flex-1 px-3 py-1 flex items-center gap-2">
                      <input
                        type="text"
                        bind:value={header.value}
                        placeholder={t('apiClient.headerValue')}
                        class="flex-1 px-0 py-0 text-sm bg-transparent border-none outline-none text-gray-900 dark:text-gray-100 placeholder:text-gray-400 dark:placeholder:text-gray-500 focus:ring-0 disabled:opacity-50 disabled:cursor-not-allowed"
                        disabled={!header.enabled}
                        style="ime-mode: disabled;"
                        inputmode="text"
                        autocomplete="off"
                        autocorrect="off"
                        autocapitalize="off"
                        spellcheck="false"
                      />
                      <!-- Delete button (show on hover) -->
                      <button
                        onclick={() => removeHeader(activeTab, index)}
                        class="opacity-0 group-hover:opacity-100 group-focus-within:opacity-100 focus-within:opacity-100 focus:opacity-100 p-0.5 text-red-600 dark:text-red-400 hover:bg-red-50 dark:hover:bg-red-900/20 rounded transition-all disabled:opacity-30 disabled:cursor-not-allowed"
                        disabled={activeTab.headers.length === 1}
                        title="删除"
                      >
                        <Trash2 class="w-3.5 h-3.5" />
                      </button>
                    </div>
                  </div>
                {/each}
              </div>
            </div>
          </div>
        {:else if activeTab?.requestView === 'params'}
          <div>
            <div class="flex items-center justify-between mb-3 gap-2 flex-wrap">
              <p class="text-xs text-gray-500 dark:text-gray-400">{t('apiClient.queryParamsHint')}</p>
              <button onclick={addQueryParam} class="btn-secondary text-sm">{t('apiClient.addParam')}</button>
            </div>
            <div class="border border-gray-200 dark:border-gray-700 rounded-lg overflow-hidden bg-white dark:bg-gray-800">
              <div class="divide-y divide-gray-200 dark:divide-gray-700">
                {#each queryParams as param, index}
                  <div class="group relative flex items-center hover:bg-gray-50 dark:hover:bg-gray-700/50 transition-colors">
                    <div class="px-2 py-1 flex items-center opacity-60 group-hover:opacity-100 transition-opacity">
                      <input
                        type="checkbox"
                        checked={param.enabled}
                        onchange={(e) => updateQueryParam(index, 'enabled', (e.target as HTMLInputElement).checked)}
                        class="w-3.5 h-3.5 text-primary-600 bg-gray-100 border-gray-300 rounded focus:ring-primary-500 dark:bg-gray-700 dark:border-gray-600 cursor-pointer"
                      />
                    </div>
                    <div class="flex-1 px-3 py-1 border-r border-gray-200 dark:border-gray-700 min-w-[160px]">
                      <input
                        type="text"
                        value={param.key}
                        oninput={(e) => updateQueryParam(index, 'key', (e.target as HTMLInputElement).value)}
                        placeholder="key"
                        class="w-full px-0 py-0 text-sm bg-transparent border-none outline-none text-gray-900 dark:text-gray-100 placeholder:text-gray-400 focus:ring-0 disabled:opacity-50 font-mono"
                        disabled={!param.enabled}
                        autocomplete="off" autocorrect="off" autocapitalize="off" spellcheck="false"
                      />
                    </div>
                    <div class="flex-1 px-3 py-1 flex items-center gap-2">
                      <input
                        type="text"
                        value={param.value}
                        oninput={(e) => updateQueryParam(index, 'value', (e.target as HTMLInputElement).value)}
                        placeholder="value"
                        class="flex-1 px-0 py-0 text-sm bg-transparent border-none outline-none text-gray-900 dark:text-gray-100 placeholder:text-gray-400 focus:ring-0 disabled:opacity-50 font-mono"
                        disabled={!param.enabled}
                        autocomplete="off" autocorrect="off" autocapitalize="off" spellcheck="false"
                      />
                      <button
                        onclick={() => removeQueryParam(index)}
                        class="opacity-0 group-hover:opacity-100 group-focus-within:opacity-100 focus:opacity-100 p-0.5 text-red-600 dark:text-red-400 hover:bg-red-50 dark:hover:bg-red-900/20 rounded transition-all"
                        title={t('apiClient.delete')}
                      >
                        <Trash2 class="w-3.5 h-3.5" />
                      </button>
                    </div>
                  </div>
                {/each}
                {#if queryParams.length === 0}
                  <div class="px-3 py-4 text-xs text-gray-400 dark:text-gray-500 text-center">{t('apiClient.noParams')}</div>
                {/if}
              </div>
            </div>
          </div>
        {:else if activeTab?.requestView === 'body'}
          <!-- Body Content -->
          <div>
            {#if activeTab && (activeTab.method === 'GET' || activeTab.method === 'HEAD' || activeTab.method === 'OPTIONS')}
              <div class="mb-3 p-3 text-xs text-amber-800 dark:text-amber-200 bg-amber-50 dark:bg-amber-900/20 border border-amber-200 dark:border-amber-800 rounded-lg">
                {t('apiClient.bodyNotAvailable')}
              </div>
            {/if}
            <div class="flex gap-2 mb-4">
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
              <div class="flex items-center justify-between mb-2">
                <span class="text-xs text-gray-500 dark:text-gray-400">JSON{#if jsonInlineError}<span class="ml-2 text-red-600 dark:text-red-400 font-mono">— {jsonInlineError}</span>{/if}</span>
                <div class="flex gap-2">
                  <button onclick={formatJson} class="btn-secondary text-xs px-2 py-1" title="Format JSON">{t('apiClient.format')}</button>
                  <button onclick={minifyJson} class="btn-secondary text-xs px-2 py-1" title="Minify JSON">{t('apiClient.minify')}</button>
                </div>
              </div>
              <div class="json-editor-container">
                <pre class="json-editor-overlay textarea font-mono text-sm" bind:this={jsonOverlayRef} aria-hidden="true"><code class="hljs language-json">{@html highlightedJson}</code></pre>
                <textarea
                  bind:this={bodyTextareaRef}
                  bind:value={activeTab.bodyJson}
                  placeholder={t('apiClient.jsonPlaceholder')}
                  class="json-editor-textarea textarea font-mono text-sm min-h-[150px]"
                  wrap={"off" as any}
                  style="ime-mode: disabled;"
                  inputmode="text"
                  autocomplete="off"
                  autocapitalize="off"
                  spellcheck="false"
                  oninput={(e) => autoResizeTextarea(e.currentTarget as HTMLTextAreaElement)}
                  onfocus={(e) => autoResizeTextarea(e.currentTarget as HTMLTextAreaElement)}
                  onscroll={syncJsonScroll}
                ></textarea>
              </div>
            {:else if activeTab.bodyType === 'text'}
              <textarea
                bind:this={bodyTextareaRef}
                bind:value={activeTab.bodyText}
                placeholder={t('apiClient.textPlaceholder')}
                class="textarea font-mono text-sm min-h-[150px]"
                style="ime-mode: disabled;"
                inputmode="text"
                autocomplete="off"
                autocapitalize="off"
                spellcheck="false"
                oninput={(e) => autoResizeTextarea(e.currentTarget as HTMLTextAreaElement)}
                onfocus={(e) => autoResizeTextarea(e.currentTarget as HTMLTextAreaElement)}
              ></textarea>
            {:else if activeTab.bodyType === 'xml'}
              <div class="flex items-center justify-between mb-2">
                <span class="text-xs text-gray-500 dark:text-gray-400">XML</span>
                <button onclick={formatXml} class="btn-secondary text-xs px-2 py-1" title="Format XML">{t('apiClient.format')}</button>
              </div>
              <div class="json-editor-container">
                <pre class="json-editor-overlay textarea font-mono text-sm" bind:this={xmlOverlayRef} aria-hidden="true"><code class="hljs language-xml">{@html highlightedXml}</code></pre>
                <textarea
                  bind:this={bodyTextareaRef}
                  bind:value={activeTab.bodyXml}
                  placeholder={t('apiClient.xmlPlaceholder')}
                  class="json-editor-textarea textarea font-mono text-sm min-h-[150px]"
                  wrap={"off" as any}
                  style="ime-mode: disabled;"
                  inputmode="text"
                  autocomplete="off"
                  autocapitalize="off"
                  spellcheck="false"
                  oninput={(e) => autoResizeTextarea(e.currentTarget as HTMLTextAreaElement)}
                  onfocus={(e) => autoResizeTextarea(e.currentTarget as HTMLTextAreaElement)}
                  onscroll={syncXmlScroll}
                ></textarea>
              </div>
            {:else if activeTab.bodyType === 'form-data' || activeTab.bodyType === 'url-encoded'}
              <div>
                <div class="flex items-center justify-end mb-3">
                  <button
                    onclick={() => addFormDataItem(activeTab)}
                    class="btn-secondary text-sm"
                  >
                    {t('apiClient.addFormData')}
                  </button>
                </div>
                <div class="border border-gray-200 dark:border-gray-700 rounded-lg overflow-hidden bg-white dark:bg-gray-800">
                  <div class="divide-y divide-gray-200 dark:divide-gray-700">
                    {#each activeTab.formData as item, index}
                      <div class="group relative flex items-center hover:bg-gray-50 dark:hover:bg-gray-700/50 transition-colors">
                        <!-- Checkbox -->
                        <div class="px-2 py-1 flex items-center opacity-60 group-hover:opacity-100 transition-opacity">
                          <input
                            type="checkbox"
                            bind:checked={item.enabled}
                            class="w-3.5 h-3.5 text-primary-600 bg-gray-100 border-gray-300 rounded focus:ring-primary-500 dark:focus:ring-primary-600 dark:ring-offset-gray-800 focus:ring-1 dark:bg-gray-700 dark:border-gray-600 cursor-pointer"
                            title={item.enabled ? '禁用' : '启用'}
                          />
                        </div>
                        <!-- Key Column -->
                        <div class="flex-1 px-3 py-1 border-r border-gray-200 dark:border-gray-700 min-w-[200px]">
                          <input
                            type="text"
                            bind:value={item.key}
                            placeholder={t('apiClient.formKey')}
                            class="w-full px-0 py-0 text-sm bg-transparent border-none outline-none text-gray-900 dark:text-gray-100 placeholder:text-gray-400 dark:placeholder:text-gray-500 focus:ring-0 disabled:opacity-50 disabled:cursor-not-allowed"
                            disabled={!item.enabled}
                            style="ime-mode: disabled;"
                            inputmode="text"
                            autocomplete="off"
                            autocorrect="off"
                            autocapitalize="off"
                            spellcheck="false"
                          />
                        </div>
                        <!-- Type Column (only for form-data) -->
                        {#if activeTab.bodyType === 'form-data'}
                          <div class="px-2 py-1 border-r border-gray-200 dark:border-gray-700 w-24">
                            <div class="relative">
                              <select
                                bind:value={item.type}
                                class="w-full px-2 py-1 text-xs bg-gray-100 dark:bg-gray-700 border border-gray-300 dark:border-gray-600 rounded text-gray-900 dark:text-gray-100 focus:outline-none focus:ring-2 focus:ring-primary-500 dark:focus:ring-primary-400 focus:border-transparent disabled:opacity-50 disabled:cursor-not-allowed cursor-pointer appearance-none pr-6 hover:bg-gray-200 dark:hover:bg-gray-600 transition-colors"
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
                              <!-- Custom dropdown arrow -->
                              <div class="absolute inset-y-0 right-1 flex items-center pointer-events-none">
                                <svg class="w-3 h-3 text-gray-500 dark:text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7"></path>
                                </svg>
                              </div>
                            </div>
                          </div>
                        {/if}
                        <!-- Value Column -->
                        <div class="flex-1 px-3 py-1 flex items-center gap-2">
                          {#if item.type === 'text' || activeTab.bodyType === 'url-encoded'}
                            <input
                              type="text"
                              bind:value={item.value}
                              placeholder={t('apiClient.formValue')}
                              class="flex-1 px-0 py-0 text-sm bg-transparent border-none outline-none text-gray-900 dark:text-gray-100 placeholder:text-gray-400 dark:placeholder:text-gray-500 focus:ring-0 disabled:opacity-50 disabled:cursor-not-allowed"
                              disabled={!item.enabled}
                              style="ime-mode: disabled;"
                              inputmode="text"
                              autocomplete="off"
                              autocorrect="off"
                              autocapitalize="off"
                              spellcheck="false"
                            />
                          {:else if item.type === 'file'}
                            <label class="flex-1 cursor-pointer flex items-center gap-2 {!item.enabled ? 'opacity-50 cursor-not-allowed' : ''}">
                              <input
                                type="file"
                                class="hidden"
                                disabled={!item.enabled}
                                onchange={(e) => handleFileSelect(activeTab, index, e)}
                              />
                              <span class="flex-1 text-sm text-gray-700 dark:text-gray-300 px-0 py-0">
                                {item.value || t('apiClient.selectFile')}
                              </span>
                              {#if item.file}
                                <span class="text-xs text-gray-500 dark:text-gray-400">
                                  ({(item.file.size / 1024).toFixed(2)} KB)
                                </span>
                              {/if}
                            </label>
                          {/if}
                          <!-- Delete button (show on hover) -->
                          <button
                            onclick={() => removeFormDataItem(activeTab, index)}
                            class="opacity-0 group-hover:opacity-100 group-focus-within:opacity-100 focus-within:opacity-100 focus:opacity-100 p-0.5 text-red-600 dark:text-red-400 hover:bg-red-50 dark:hover:bg-red-900/20 rounded transition-all disabled:opacity-30 disabled:cursor-not-allowed"
                            disabled={activeTab.formData.length === 1}
                            title="删除"
                          >
                            <Trash2 class="w-3.5 h-3.5" />
                          </button>
                        </div>
                      </div>
                    {/each}
                  </div>
                </div>
              </div>
            {/if}
          </div>
        {/if}
      </div>

      <!-- 批量添加 Header 对话框 -->
      {#if showBulkHeaderDialog}
        <div 
          class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50"
          role="dialog"
          aria-modal="true"
          tabindex="-1"
        >
          <div 
            class="bg-white dark:bg-gray-800 rounded-lg shadow-xl p-6 w-full max-w-2xl mx-4"
            role="none"
          >
            <div class="flex items-center justify-between mb-4">
              <h3 class="text-lg font-semibold text-gray-900 dark:text-gray-100">
                {t('apiClient.bulkAddHeaders')}
              </h3>
              <button
                onclick={() => { showBulkHeaderDialog = false; bulkHeaderText = ''; }}
                class="p-1 rounded hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors"
                aria-label={t('apiClient.close')}
              >
                <X class="w-5 h-5 text-gray-500 dark:text-gray-400" />
              </button>
            </div>
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

      {#if activeTab?.requestView === 'auth'}
        <div class="space-y-3">
          <div class="flex items-center gap-3">
            <label class="text-sm font-medium text-gray-700 dark:text-gray-300 w-20">{t('apiClient.authType')}</label>
            <select bind:value={activeTab.authType} class="input flex-1">
              <option value="none">No Auth</option>
              <option value="bearer">Bearer Token</option>
              <option value="basic">Basic Auth</option>
            </select>
          </div>
          {#if activeTab?.authType === 'bearer'}
            <div class="flex items-center gap-3">
              <label class="text-sm font-medium text-gray-700 dark:text-gray-300 w-20">Token</label>
              <input type="text" bind:value={activeTab.bearerToken} placeholder="Enter token..." class="input flex-1 font-mono text-sm" />
            </div>
          {/if}
          {#if activeTab?.authType === 'basic'}
            <div class="flex items-center gap-3">
              <label class="text-sm font-medium text-gray-700 dark:text-gray-300 w-20">Username</label>
              <input type="text" bind:value={activeTab.basicUsername} placeholder="Username" class="input flex-1 text-sm" />
            </div>
            <div class="flex items-center gap-3">
              <label class="text-sm font-medium text-gray-700 dark:text-gray-300 w-20">Password</label>
              <input type="password" bind:value={activeTab.basicPassword} placeholder="Password" class="input flex-1 text-sm" />
            </div>
          {/if}
          {#if activeTab?.authType === 'none'}
            <p class="text-sm text-gray-400 dark:text-gray-500 py-4">{t('apiClient.authNoneDesc')}</p>
          {/if}
        </div>
      {/if}

      {#if activeTab?.requestView === 'settings'}
        <div class="space-y-4">
          <div class="flex items-center gap-3">
            <label class="text-sm font-medium text-gray-700 dark:text-gray-300 w-40">{t('apiClient.timeout')}</label>
            <input type="number" bind:value={requestTimeout} min="0" step="1" class="input w-32 text-sm" />
            <span class="text-sm text-gray-500 dark:text-gray-400">{t('apiClient.seconds')} (0 = {t('apiClient.noTimeout')})</span>
          </div>
          <div class="flex items-center gap-3">
            <label class="text-sm font-medium text-gray-700 dark:text-gray-300 w-40">{t('apiClient.followRedirects')}</label>
            <input type="checkbox" bind:checked={followRedirects} class="w-4 h-4 rounded border-gray-300 text-primary-600 focus:ring-primary-500 dark:border-gray-600 dark:bg-gray-700" />
          </div>
        </div>
      {/if}

      {#if activeTab?.requestView === 'preRequest'}
        <div class="space-y-2">
          <details class="text-xs">
            <summary class="cursor-pointer text-gray-600 dark:text-gray-400 font-medium py-1">{t('apiClient.preRequestDesc')}</summary>
            <div class="mt-2 p-3 bg-gray-50 dark:bg-gray-800/50 rounded-lg space-y-1 text-gray-600 dark:text-gray-400 font-mono">
              <div>// {t('apiClient.preRequestSyntaxTitle')}</div>
              <div>pm.variables.<b>get</b>(key)             <span class="text-gray-400">// {t('apiClient.preRequestVarGet')}</span></div>
              <div>pm.variables.<b>set</b>(key, value)      <span class="text-gray-400">// {t('apiClient.preRequestVarSet')}</span></div>
              <div class="mt-1 text-gray-400">// {t('apiClient.preRequestExampleTitle')}</div>
              <div>pm.variables.set('token', btoa('user:pass'))</div>
              <div>pm.variables.set('ts', Date.now().toString())</div>
            </div>
          </details>
          <div class="json-editor-container bordered h-[300px]">
            <pre class="json-editor-overlay textarea font-mono text-sm" aria-hidden="true"><code class="hljs language-javascript">{@html highlightedPreRequest}</code></pre>
            <textarea
              bind:value={activeTab.preRequestScript}
              placeholder="// pm.variables.set('timestamp', Date.now().toString())&#10;// pm.variables.set('token', btoa('user:pass'))"
              class="json-editor-textarea textarea font-mono text-sm h-full"
              wrap={"off" as any}
              spellcheck="false"
            ></textarea>
          </div>
        </div>
      {/if}

      {#if activeTab?.requestView === 'tests'}
        <div class="space-y-3">
          <div class="flex items-center gap-2">
            <p class="text-xs text-gray-500 dark:text-gray-400 flex-1">{t('apiClient.testsDesc')}</p>
            {#if activeTab && activeTab.testResults.length > 0}
              <span class="text-xs px-2 py-0.5 rounded-full {activeTab.testResults.every(r => r.passed) ? 'bg-green-100 text-green-700 dark:bg-green-900/30 dark:text-green-400' : 'bg-red-100 text-red-700 dark:bg-red-900/30 dark:text-red-400'}">
                {activeTab.testResults.filter(r => r.passed).length}/{activeTab.testResults.length} {t('apiClient.passed')}
              </span>
            {/if}
          </div>
          <details class="text-xs">
            <summary class="cursor-pointer text-gray-600 dark:text-gray-400 font-medium py-1">{t('apiClient.testsSyntaxTitle')}</summary>
            <div class="mt-2 p-3 bg-gray-50 dark:bg-gray-800/50 rounded-lg space-y-1 text-gray-600 dark:text-gray-400 font-mono">
              <div>// {t('apiClient.testsApiTitle')}</div>
              <div>pm.<b>test</b>(name, fn)                    <span class="text-gray-400">// {t('apiClient.testsTestDesc')}</span></div>
              <div>pm.<b>expect</b>(actual)                    <span class="text-gray-400">// {t('apiClient.testsExpectDesc')}</span></div>
              <div class="ml-4">.to.equal(expected)             <span class="text-gray-400">// {t('apiClient.testsToEqual')}</span></div>
              <div class="ml-4">.to.eql(expected)              <span class="text-gray-400">// {t('apiClient.testsToEql')}</span></div>
              <div class="ml-4">.to.be(expected)               <span class="text-gray-400">// {t('apiClient.testsToBe')}</span></div>
              <div class="ml-4">.to.include(substring)        <span class="text-gray-400">// {t('apiClient.testsToInclude')}</span></div>
              <div class="ml-4">.to.match(regex)               <span class="text-gray-400">// {t('apiClient.testsToMatch')}</span></div>
              <div class="ml-4">.to.have.property(key)        <span class="text-gray-400">// {t('apiClient.testsToHave')}</span></div>
              <div class="ml-4">.to.have.status(code)         <span class="text-gray-400">// {t('apiClient.testsToHaveStatus')}</span></div>
              <div class="mt-2">// pm.response</div>
              <div>pm.response.<b>code</b>                       <span class="text-gray-400">// {t('apiClient.testsRespCode')}</span></div>
              <div>pm.response.<b>text</b>()                    <span class="text-gray-400">// {t('apiClient.testsRespText')}</span></div>
              <div>pm.response.<b>json</b>()                    <span class="text-gray-400">// {t('apiClient.testsRespJson')}</span></div>
              <div>pm.response.<b>responseTime</b>              <span class="text-gray-400">// {t('apiClient.testsRespTime')}</span></div>
              <div>pm.response.headers.<b>get</b>(key)          <span class="text-gray-400">// {t('apiClient.testsRespHeader')}</span></div>
              <div class="mt-2 text-gray-400">// {t('apiClient.testsExampleTitle')}</div>
              <div>pm.test('Status is 200', () => {'{'}</div>
              <div class="ml-2">pm.expect(pm.response.code).to.equal(200)</div>
              <div>{'}'})</div>
              <div>pm.test('Body has id', () => {'{'}</div>
              <div class="ml-2">pm.expect(pm.response.json()).to.have.property('id')</div>
              <div>{'}'})</div>
            </div>
          </details>
          <div class="json-editor-container bordered h-[200px]">
            <pre class="json-editor-overlay textarea font-mono text-sm" aria-hidden="true"><code class="hljs language-javascript">{@html highlightedTestScript}</code></pre>
            <textarea
              bind:value={activeTab.testScript}
              placeholder={`// pm.test("Status is 200", () => {
//   pm.expect(pm.response.code).to.equal(200)
// })
// pm.test("Body contains success", () => {
//   pm.expect(pm.response.text()).to.include("success")
// })`}
              class="json-editor-textarea textarea font-mono text-sm h-full"
              wrap={"off" as any}
              spellcheck="false"
            ></textarea>
          </div>
          {#if activeTab && activeTab.testResults.length > 0}
            <div class="space-y-1">
              {#each activeTab.testResults as result}
                <div class="flex items-center gap-2 px-3 py-1.5 rounded text-sm {result.passed ? 'bg-green-50 dark:bg-green-900/20 text-green-700 dark:text-green-400' : 'bg-red-50 dark:bg-red-900/20 text-red-700 dark:text-red-400'}">
                  {#if result.passed}
                    <Check class="w-4 h-4 flex-shrink-0" />
                  {:else}
                    <X class="w-4 h-4 flex-shrink-0" />
                  {/if}
                  <span class="font-mono text-xs">{result.name}</span>
                  {#if result.error}
                    <span class="text-xs opacity-70">- {result.error}</span>
                  {/if}
                </div>
              {/each}
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

  <!-- 内联 Response Panel（常驻） -->
  {#if activeTab && (activeTab.responseStatus !== null || activeTab.responseBody || activeTab.error || activeTab.responseTime !== null)}
    <div class="card">
      <div class="flex items-center justify-between mb-3 flex-wrap gap-2">
        <div class="flex items-center gap-3">
          <h3 class="text-sm font-semibold text-gray-900 dark:text-gray-100">{t('apiClient.inlineResponse')}</h3>
          {#if activeTab.responseStatus !== null}
            <span class="px-2 py-0.5 text-sm font-semibold rounded {getStatusColor(activeTab.responseStatus)}">
              {activeTab.responseStatus}
            </span>
          {/if}
          {#if activeTab.responseTime !== null}
            <span class="text-xs text-gray-500 dark:text-gray-400">{activeTab.responseTime}ms</span>
          {/if}
          {#if activeTab.responseSize !== null}
            <span class="text-xs text-gray-500 dark:text-gray-400">{formatSize(activeTab.responseSize)}</span>
          {/if}
          {#if activeTab.cancelled}
            <span class="text-xs px-2 py-0.5 rounded-full bg-gray-100 text-gray-600 dark:bg-gray-700 dark:text-gray-300">{t('apiClient.cancelled')}</span>
          {/if}
        </div>
        <div class="flex items-center gap-2 flex-wrap">
          <div class="flex items-center gap-0.5 bg-gray-100 dark:bg-gray-700 rounded p-0.5">
            <button
              onclick={() => responseViewMode = 'pretty'}
              class="px-2 py-0.5 text-xs rounded transition-colors {responseViewMode === 'pretty' ? 'bg-white dark:bg-gray-600 text-primary-600 dark:text-primary-400 font-medium shadow-sm' : 'text-gray-500 dark:text-gray-400 hover:text-gray-700 dark:hover:text-gray-200'}"
            >Pretty</button>
            <button
              onclick={() => responseViewMode = 'raw'}
              class="px-2 py-0.5 text-xs rounded transition-colors {responseViewMode === 'raw' ? 'bg-white dark:bg-gray-600 text-primary-600 dark:text-primary-400 font-medium shadow-sm' : 'text-gray-500 dark:text-gray-400 hover:text-gray-700 dark:hover:text-gray-200'}"
            >Raw</button>
            <button
              onclick={() => responseViewMode = 'preview'}
              class="px-2 py-0.5 text-xs rounded transition-colors {responseViewMode === 'preview' ? 'bg-white dark:bg-gray-600 text-primary-600 dark:text-primary-400 font-medium shadow-sm' : 'text-gray-500 dark:text-gray-400 hover:text-gray-700 dark:hover:text-gray-200'}"
            >Preview</button>
            <button
              onclick={() => responseViewMode = 'headers'}
              class="px-2 py-0.5 text-xs rounded transition-colors {responseViewMode === 'headers' ? 'bg-white dark:bg-gray-600 text-primary-600 dark:text-primary-400 font-medium shadow-sm' : 'text-gray-500 dark:text-gray-400 hover:text-gray-700 dark:hover:text-gray-200'}"
            >Headers</button>
          </div>
          <button
            onclick={() => copyResponse(activeTab)}
            class="btn-secondary text-xs whitespace-nowrap transition-all duration-200 {activeTab.copied ? 'bg-green-500 hover:bg-green-600 text-white' : ''}"
            title={t('common.copy')}
          >
            {#if activeTab.copied}
              <span class="flex items-center gap-1">
                <Check class="w-3.5 h-3.5" />
                {t('common.copied')}
              </span>
            {:else}
              <span class="flex items-center gap-1">
                <Copy class="w-3.5 h-3.5" />
                {t('common.copy')}
              </span>
            {/if}
          </button>
          <button
            onclick={() => saveResponse(activeTab)}
            disabled={!activeTab.responseBody}
            class="btn-secondary text-xs whitespace-nowrap disabled:opacity-50 disabled:cursor-not-allowed"
            title={t('apiClient.saveResponse')}
          >
            <span class="flex items-center gap-1">
              <Download class="w-3.5 h-3.5" />
              {t('apiClient.saveResponse')}
            </span>
          </button>
          <button
            onclick={() => { responsePopped = true; showResponseDialog = true; }}
            disabled={activeTab.responseStatus === null && !activeTab.responseBody && !activeTab.error}
            class="btn-secondary text-xs whitespace-nowrap disabled:opacity-50 disabled:cursor-not-allowed"
            title={t('apiClient.popout')}
          >
            <span class="flex items-center gap-1">
              <ExternalLink class="w-3.5 h-3.5" />
              {t('apiClient.popout')}
            </span>
          </button>
        </div>
      </div>
      {#if activeTab.error && !activeTab.responseBody && activeTab.responseStatus === null}
        <div class="p-4 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg">
          <p class="text-sm text-red-800 dark:text-red-200">{activeTab.error}</p>
        </div>
      {:else if responseViewMode === 'headers'}
        <div class="bg-gray-50 dark:bg-gray-900 border border-gray-300 dark:border-gray-600 rounded-lg px-4 py-3 max-h-[400px] overflow-y-auto {Object.keys(activeTab.responseHeaders).length === 0 ? 'flex items-center justify-center' : ''}">
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
      {:else if responseViewMode === 'preview'}
        <div class="bg-white dark:bg-gray-900 border border-gray-300 dark:border-gray-600 rounded-lg h-[400px] overflow-y-auto">
          {#if activeTab.responseHeaders['content-type']?.includes('image/') || activeTab.responseHeaders['Content-Type']?.includes('image/')}
            <div class="flex items-center justify-center p-4 h-full">
              <img src={activeTab.url} alt="Response preview" class="max-w-full max-h-full object-contain" />
            </div>
          {:else}
            <iframe
              title="Response Preview"
              class="w-full h-full border-0"
              srcdoc={activeTab.responseBody}
              sandbox="allow-same-origin"
            ></iframe>
          {/if}
        </div>
      {:else if responseViewMode === 'raw'}
        <textarea
          value={activeTab.responseBody}
          readonly
          class="textarea font-mono text-sm h-[400px] resize-none overflow-y-auto"
        ></textarea>
      {:else if activeTab.responseBody}
        <div class="json-editor-container bordered h-[400px]">
          <pre class="json-editor-overlay textarea font-mono text-sm" bind:this={inlineResponseOverlayRef} aria-hidden="true"><code class="hljs language-json">{@html highlightedResponse}</code></pre>
          <textarea
            bind:this={inlineResponseTextareaRef}
            value={activeTab.responseBody}
            readonly
            wrap={"off" as any}
            class="json-editor-textarea textarea font-mono text-sm h-full"
            onscroll={syncInlineResponseScroll}
          ></textarea>
        </div>
      {:else}
        <div class="bg-gray-50 dark:bg-gray-900 border border-gray-300 dark:border-gray-600 rounded-lg px-4 py-3 h-[200px] overflow-y-auto flex items-center justify-center">
          <span class="text-sm text-gray-400 dark:text-gray-500">{t('apiClient.noResponseBody')}</span>
        </div>
      {/if}
    </div>
  {/if}

  <!-- 响应对话框 -->
  {#if showResponseDialog && (activeTab.responseStatus !== null || activeTab.responseBody || activeTab.error)}
    <div 
      class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50"
      role="dialog"
      aria-modal="true"
      tabindex="-1"
    >
      <div 
        class="bg-white dark:bg-gray-800 rounded-lg shadow-xl w-full max-w-6xl mx-4 max-h-[90vh] flex flex-col"
        onclick={(e) => e.stopPropagation()}
        role="none"
      >
        <!-- 对话框头部 -->
        <div class="flex items-center justify-between p-6 border-b border-gray-200 dark:border-gray-700">
          <div class="flex items-center gap-4">
            <h3 class="text-lg font-semibold text-gray-900 dark:text-gray-100">
              {t('apiClient.response')}
            </h3>
            {#if activeTab.responseStatus !== null}
              <span class="px-2 py-1 text-sm font-semibold rounded {getStatusColor(activeTab.responseStatus)}">
                {activeTab.responseStatus}
              </span>
            {/if}
            {#if activeTab.responseTime !== null}
              <span class="text-sm text-gray-500 dark:text-gray-400">
                {activeTab.responseTime}ms
              </span>
            {/if}
          </div>
          <button
            onclick={() => { showResponseDialog = false; responsePopped = false; }}
            class="p-2 text-gray-400 hover:text-gray-600 dark:hover:text-gray-200 rounded transition-colors"
            title="关闭"
          >
            <X class="w-5 h-5" />
          </button>
        </div>

        <!-- 对话框内容 -->
        <div class="flex-1 overflow-y-auto p-6">
          {#if activeTab.error}
            <div class="mb-4 p-4 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg">
              <p class="text-sm text-red-800 dark:text-red-200">{activeTab.error}</p>
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
              <div class="bg-gray-50 dark:bg-gray-900 border border-gray-300 dark:border-gray-600 rounded-lg px-4 py-3 h-[400px] overflow-y-auto {Object.keys(activeTab.responseHeaders).length === 0 ? 'flex items-center justify-center' : ''}">
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
                <div class="flex items-center gap-2">
                  <span class="text-sm font-medium text-gray-700 dark:text-gray-300">{t('apiClient.responseBody')}</span>
                  {#if activeTab.responseBody}
                    <div class="flex items-center gap-0.5 bg-gray-100 dark:bg-gray-700 rounded p-0.5">
                      <button
                        onclick={() => responseViewMode = 'pretty'}
                        class="px-2 py-0.5 text-xs rounded transition-colors {responseViewMode === 'pretty' ? 'bg-white dark:bg-gray-600 text-primary-600 dark:text-primary-400 font-medium shadow-sm' : 'text-gray-500 dark:text-gray-400 hover:text-gray-700 dark:hover:text-gray-200'}"
                      >Pretty</button>
                      <button
                        onclick={() => responseViewMode = 'raw'}
                        class="px-2 py-0.5 text-xs rounded transition-colors {responseViewMode === 'raw' ? 'bg-white dark:bg-gray-600 text-primary-600 dark:text-primary-400 font-medium shadow-sm' : 'text-gray-500 dark:text-gray-400 hover:text-gray-700 dark:hover:text-gray-200'}"
                      >Raw</button>
                      <button
                        onclick={() => responseViewMode = 'preview'}
                        class="px-2 py-0.5 text-xs rounded transition-colors {responseViewMode === 'preview' ? 'bg-white dark:bg-gray-600 text-primary-600 dark:text-primary-400 font-medium shadow-sm' : 'text-gray-500 dark:text-gray-400 hover:text-gray-700 dark:hover:text-gray-200'}"
                      >Preview</button>
                    </div>
                  {/if}
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
                {#if responseViewMode === 'preview'}
                  <!-- Preview mode: render HTML/images -->
                  <div class="bg-white dark:bg-gray-900 border border-gray-300 dark:border-gray-600 rounded-lg h-[400px] overflow-y-auto">
                    {#if activeTab.responseHeaders['content-type']?.includes('image/') || activeTab.responseHeaders['Content-Type']?.includes('image/')}
                      <div class="flex items-center justify-center p-4 h-full">
                        <img src="{activeTab.url}" alt="Response preview" class="max-w-full max-h-full object-contain" />
                      </div>
                    {:else}
                      <iframe
                        title="Response Preview"
                        class="w-full h-full border-0"
                        srcdoc={activeTab.responseBody}
                        sandbox="allow-same-origin"
                      ></iframe>
                    {/if}
                  </div>
                {:else if responseViewMode === 'raw'}
                  <!-- Raw mode: plain text -->
                  <textarea
                    value={activeTab.responseBody}
                    readonly
                    class="textarea font-mono text-sm h-[400px] resize-none overflow-y-auto {activeTab.copied ? 'bg-green-50 dark:bg-green-900/20 border-green-300 dark:border-green-700' : ''} transition-colors duration-300"
                  ></textarea>
                {:else}
                  <!-- Pretty mode: syntax highlighted -->
                  <div class="json-editor-container bordered h-[400px] {activeTab.copied ? 'border-green-300 dark:border-green-700' : ''}">
                    <pre class="json-editor-overlay textarea font-mono text-sm" bind:this={responseOverlayRef} aria-hidden="true"><code class="hljs language-json">{@html highlightedResponse}</code></pre>
                    <textarea
                      bind:this={responseTextareaRef}
                      value={activeTab.responseBody}
                      readonly
                      wrap={"off" as any}
                      class="json-editor-textarea textarea font-mono text-sm h-full {activeTab.copied ? 'bg-green-50 dark:bg-green-900/20 border-green-300 dark:border-green-700' : ''} transition-colors duration-300"
                      onscroll={syncResponseScroll}
                    ></textarea>
                  </div>
                {/if}
              {:else}
                <div class="bg-gray-50 dark:bg-gray-900 border border-gray-300 dark:border-gray-600 rounded-lg px-4 py-3 h-[400px] overflow-y-auto flex items-center justify-center">
                  <span class="text-sm text-gray-400 dark:text-gray-500">{t('apiClient.noResponseBody')}</span>
                </div>
              {/if}
            </div>
          </div>
        </div>

        <!-- 对话框底部 -->
        <div class="flex items-center justify-end gap-2 p-6 border-t border-gray-200 dark:border-gray-700">
          <!-- Footer intentionally left without close to enforce only top-right close -->
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

  <!-- History Dialog -->
  {#if showHistory}
    <div
      class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50"
      onclick={() => showHistory = false}
      onkeydown={(e) => { if (e.key === 'Escape') showHistory = false; }}
      role="dialog"
      aria-modal="true"
      tabindex="-1"
    >
      <div class="bg-white dark:bg-gray-800 rounded-lg shadow-xl p-6 w-full max-w-2xl mx-4 max-h-[80vh] flex flex-col" onclick={(e) => e.stopPropagation()}>
        <div class="flex items-center justify-between mb-4">
          <h3 class="text-lg font-semibold text-gray-900 dark:text-gray-100">{t('apiClient.history')}</h3>
          <div class="flex items-center gap-2">
            <button onclick={clearHistory} class="btn-secondary text-xs flex items-center gap-1">
              <Trash2 class="w-3.5 h-3.5" /> {t('apiClient.clearHistory')}
            </button>
            <button onclick={() => showHistory = false} class="p-1 rounded hover:bg-gray-100 dark:hover:bg-gray-700">
              <X class="w-5 h-5 text-gray-500 dark:text-gray-400" />
            </button>
          </div>
        </div>
        <div class="flex-1 overflow-y-auto space-y-1">
          {#each history as entry}
            <button onclick={() => restoreHistoryEntry(entry)} title={t('apiClient.restore')} class="w-full flex items-center gap-3 px-3 py-2 rounded hover:bg-gray-50 dark:hover:bg-gray-700/50 transition-colors text-left">
              <span class="text-xs font-mono font-bold w-14 flex-shrink-0 truncate {entry.method === 'GET' ? 'text-green-600 dark:text-green-400' : entry.method === 'POST' ? 'text-blue-600 dark:text-blue-400' : entry.method === 'DELETE' ? 'text-red-600 dark:text-red-400' : 'text-gray-500'}">{entry.method}</span>
              <span class="text-sm truncate flex-1 text-gray-700 dark:text-gray-300">{entry.url}</span>
              {#if entry.status !== null}
                <span class="text-xs font-mono px-1.5 py-0.5 rounded {entry.status < 300 ? 'bg-green-100 text-green-700 dark:bg-green-900/30 dark:text-green-400' : 'bg-red-100 text-red-700 dark:bg-red-900/30 dark:text-red-400'}">{entry.status}</span>
              {/if}
              {#if entry.time !== null}
                <span class="text-xs text-gray-400">{entry.time}ms</span>
              {/if}
              <span class="text-xs text-gray-400">{new Date(entry.timestamp).toLocaleTimeString()}</span>
            </button>
          {/each}
          {#if history.length === 0}
            <p class="text-sm text-gray-400 dark:text-gray-500 text-center py-8">{t('apiClient.noHistory')}</p>
          {/if}
        </div>
      </div>
    </div>
  {/if}

  </div>
</div>

