<script lang="ts">
  import { translationsStore } from '$lib/stores/i18n';
  import { Search, Copy, Check, Trash2, Server } from 'lucide-svelte';
  import { onMount } from 'svelte';

  interface StatusCodeInfo {
    code: number;
    name: string;
    description: string;
    category: 'informational' | 'success' | 'redirection' | 'client-error' | 'server-error';
    rfc?: string;
  }

  // HTTP 状态码数据库
  const statusCodeDatabase: StatusCodeInfo[] = [
    // 1xx Informational
    { code: 100, name: 'Continue', description: 'The server has received the request headers and the client should proceed to send the request body', category: 'informational', rfc: 'RFC 7231' },
    { code: 101, name: 'Switching Protocols', description: 'The requester has asked the server to switch protocols and the server has agreed', category: 'informational', rfc: 'RFC 7231' },
    { code: 102, name: 'Processing', description: 'The server has received and is processing the request, but no response is available yet', category: 'informational', rfc: 'RFC 2518' },
    { code: 103, name: 'Early Hints', description: 'Used to return some response headers before final HTTP message', category: 'informational', rfc: 'RFC 8297' },
    { code: 104, name: 'Upload Resumption Supported', description: 'Indicates that the server supports resumable uploads', category: 'informational', rfc: 'RFC (temporary)' },
    
    // 2xx Success
    { code: 200, name: 'OK', description: 'The request has succeeded', category: 'success', rfc: 'RFC 7231' },
    { code: 201, name: 'Created', description: 'The request has been fulfilled and resulted in a new resource being created', category: 'success', rfc: 'RFC 7231' },
    { code: 202, name: 'Accepted', description: 'The request has been accepted for processing, but the processing has not been completed', category: 'success', rfc: 'RFC 7231' },
    { code: 203, name: 'Non-Authoritative Information', description: 'The server successfully processed the request, but is returning information from another source', category: 'success', rfc: 'RFC 7231' },
    { code: 204, name: 'No Content', description: 'The server successfully processed the request but is not returning any content', category: 'success', rfc: 'RFC 7231' },
    { code: 205, name: 'Reset Content', description: 'The server successfully processed the request, asks that the requester reset its document view', category: 'success', rfc: 'RFC 7231' },
    { code: 206, name: 'Partial Content', description: 'The server is delivering only part of the resource due to a range header sent by the client', category: 'success', rfc: 'RFC 7233' },
    { code: 207, name: 'Multi-Status', description: 'The message body that follows is an XML message and can contain a number of separate response codes', category: 'success', rfc: 'RFC 4918' },
    { code: 208, name: 'Already Reported', description: 'The members of a DAV binding have already been enumerated in a previous reply', category: 'success', rfc: 'RFC 5842' },
    { code: 226, name: 'IM Used', description: 'The server has fulfilled a request for the resource, and the response is a representation of the result of one or more instance-manipulations', category: 'success', rfc: 'RFC 3229' },
    
    // 3xx Redirection
    { code: 300, name: 'Multiple Choices', description: 'Indicates multiple options for the resource from which the client may choose', category: 'redirection', rfc: 'RFC 7231' },
    { code: 301, name: 'Moved Permanently', description: 'This and all future requests should be directed to the given URI', category: 'redirection', rfc: 'RFC 7231' },
    { code: 302, name: 'Found', description: 'The resource was found, but at a different URI', category: 'redirection', rfc: 'RFC 7231' },
    { code: 303, name: 'See Other', description: 'The response to the request can be found under another URI using a GET method', category: 'redirection', rfc: 'RFC 7231' },
    { code: 304, name: 'Not Modified', description: 'Indicates that the resource has not been modified since the version specified by the request headers', category: 'redirection', rfc: 'RFC 7232' },
    { code: 305, name: 'Use Proxy', description: 'The requested resource is available only through a proxy', category: 'redirection', rfc: 'RFC 7231' },
    { code: 306, name: 'Switch Proxy', description: 'No longer used', category: 'redirection', rfc: 'RFC 7231' },
    { code: 307, name: 'Temporary Redirect', description: 'The request should be repeated with another URI', category: 'redirection', rfc: 'RFC 7231' },
    { code: 308, name: 'Permanent Redirect', description: 'The request and all future requests should be repeated using another URI', category: 'redirection', rfc: 'RFC 7238' },
    
    // 4xx Client Error
    { code: 400, name: 'Bad Request', description: 'The request cannot be fulfilled due to bad syntax', category: 'client-error', rfc: 'RFC 7231' },
    { code: 401, name: 'Unauthorized', description: 'Authentication is required and has failed or has not yet been provided', category: 'client-error', rfc: 'RFC 7235' },
    { code: 402, name: 'Payment Required', description: 'Reserved for future use', category: 'client-error', rfc: 'RFC 7231' },
    { code: 403, name: 'Forbidden', description: 'The request was valid, but the server is refusing action', category: 'client-error', rfc: 'RFC 7231' },
    { code: 404, name: 'Not Found', description: 'The requested resource could not be found', category: 'client-error', rfc: 'RFC 7231' },
    { code: 405, name: 'Method Not Allowed', description: 'A request method is not supported for the requested resource', category: 'client-error', rfc: 'RFC 7231' },
    { code: 406, name: 'Not Acceptable', description: 'The requested resource is only capable of generating content not acceptable according to the Accept headers sent in the request', category: 'client-error', rfc: 'RFC 7231' },
    { code: 407, name: 'Proxy Authentication Required', description: 'Authentication is required with the proxy', category: 'client-error', rfc: 'RFC 7235' },
    { code: 408, name: 'Request Timeout', description: 'The server timed out waiting for the request', category: 'client-error', rfc: 'RFC 7231' },
    { code: 409, name: 'Conflict', description: 'The request could not be processed because of conflict in the request', category: 'client-error', rfc: 'RFC 7231' },
    { code: 410, name: 'Gone', description: 'The resource is no longer available and will not be available again', category: 'client-error', rfc: 'RFC 7231' },
    { code: 411, name: 'Length Required', description: 'The request did not specify the length of its content, which is required by the requested resource', category: 'client-error', rfc: 'RFC 7231' },
    { code: 412, name: 'Precondition Failed', description: 'The server does not meet one of the preconditions that the requester put on the request', category: 'client-error', rfc: 'RFC 7232' },
    { code: 413, name: 'Payload Too Large', description: 'The request is larger than the server is willing or able to process', category: 'client-error', rfc: 'RFC 7231' },
    { code: 414, name: 'URI Too Long', description: 'The URI provided was too long for the server to process', category: 'client-error', rfc: 'RFC 7231' },
    { code: 415, name: 'Unsupported Media Type', description: 'The request entity has a media type which the server or resource does not support', category: 'client-error', rfc: 'RFC 7231' },
    { code: 416, name: 'Range Not Satisfiable', description: 'The client has asked for a portion of the file, but the server cannot supply that portion', category: 'client-error', rfc: 'RFC 7233' },
    { code: 417, name: 'Expectation Failed', description: 'The server cannot meet the requirements of the Expect request-header field', category: 'client-error', rfc: 'RFC 7231' },
    { code: 418, name: "I'm a teapot", description: 'This code was defined in 1998 as an April Fools\' joke', category: 'client-error', rfc: 'RFC 2324' },
    { code: 421, name: 'Misdirected Request', description: 'The request was directed at a server that is not able to produce a response', category: 'client-error', rfc: 'RFC 7540' },
    { code: 422, name: 'Unprocessable Entity', description: 'The request was well-formed but was unable to be followed due to semantic errors', category: 'client-error', rfc: 'RFC 4918' },
    { code: 423, name: 'Locked', description: 'The resource that is being accessed is locked', category: 'client-error', rfc: 'RFC 4918' },
    { code: 424, name: 'Failed Dependency', description: 'The request failed because it depended on another request and that request failed', category: 'client-error', rfc: 'RFC 4918' },
    { code: 425, name: 'Too Early', description: 'Indicates that the server is unwilling to risk processing a request that might be replayed', category: 'client-error', rfc: 'RFC 8470' },
    { code: 426, name: 'Upgrade Required', description: 'The server refuses to perform the request using the current protocol', category: 'client-error', rfc: 'RFC 7231' },
    { code: 427, name: 'Unassigned', description: 'Reserved for future use', category: 'client-error' },
    { code: 428, name: 'Precondition Required', description: 'The origin server requires the request to be conditional', category: 'client-error', rfc: 'RFC 6585' },
    { code: 429, name: 'Too Many Requests', description: 'The user has sent too many requests in a given amount of time', category: 'client-error', rfc: 'RFC 6585' },
    { code: 430, name: 'Unassigned', description: 'Reserved for future use', category: 'client-error' },
    { code: 431, name: 'Request Header Fields Too Large', description: 'The server is unwilling to process the request because its header fields are too large', category: 'client-error', rfc: 'RFC 6585' },
    { code: 451, name: 'Unavailable For Legal Reasons', description: 'A server operator has received a legal demand to deny access to a resource', category: 'client-error', rfc: 'RFC 7725' },
    
    // 5xx Server Error
    { code: 500, name: 'Internal Server Error', description: 'A generic error message, given when an unexpected condition was encountered', category: 'server-error', rfc: 'RFC 7231' },
    { code: 501, name: 'Not Implemented', description: 'The server either does not recognize the request method, or it lacks the ability to fulfill the request', category: 'server-error', rfc: 'RFC 7231' },
    { code: 502, name: 'Bad Gateway', description: 'The server was acting as a gateway or proxy and received an invalid response from the upstream server', category: 'server-error', rfc: 'RFC 7231' },
    { code: 503, name: 'Service Unavailable', description: 'The server is currently unavailable (because it is overloaded or down for maintenance)', category: 'server-error', rfc: 'RFC 7231' },
    { code: 504, name: 'Gateway Timeout', description: 'The server was acting as a gateway or proxy and did not receive a timely response from the upstream server', category: 'server-error', rfc: 'RFC 7231' },
    { code: 505, name: 'HTTP Version Not Supported', description: 'The server does not support the HTTP protocol version used in the request', category: 'server-error', rfc: 'RFC 7231' },
    { code: 506, name: 'Variant Also Negotiates', description: 'Transparent content negotiation for the request results in a circular reference', category: 'server-error', rfc: 'RFC 2295' },
    { code: 507, name: 'Insufficient Storage', description: 'The method could not be performed on the resource because the server is unable to store the representation', category: 'server-error', rfc: 'RFC 4918' },
    { code: 508, name: 'Loop Detected', description: 'The server detected an infinite loop while processing the request', category: 'server-error', rfc: 'RFC 5842' },
    { code: 509, name: 'Unassigned', description: 'Reserved for future use', category: 'server-error' },
    { code: 510, name: 'Not Extended', description: 'Further extensions to the request are required for the server to fulfill it', category: 'server-error', rfc: 'RFC 2774' },
    { code: 511, name: 'Network Authentication Required', description: 'The client needs to authenticate to gain network access', category: 'server-error', rfc: 'RFC 6585' },
  ];

  // 常用状态码列表
  const commonStatusCodes = [200, 201, 301, 302, 304, 400, 401, 403, 404, 405, 408, 409, 429, 500, 502, 503, 504];

  let translations = $derived($translationsStore);
  let query = $state('');
  let results = $state<StatusCodeInfo[]>([]);
  let searchMode = $state<'code' | 'name'>('code');
  let copied = $state(false);

  // 获取常用状态码
  const commonResults = $derived(
    commonStatusCodes
      .map(code => statusCodeDatabase.find(item => item.code === code))
      .filter((item): item is StatusCodeInfo => item !== undefined)
  );

  function t(key: string): string {
    const keys = key.split('.');
    let value: any = translations;
    for (const k of keys) {
      value = value?.[k];
    }
    return value || key;
  }

  // 获取分类名称
  function getCategoryName(category: string): string {
    const categoryMap: Record<string, string> = {
      'informational': t('httpStatus.informational'),
      'success': t('httpStatus.success'),
      'redirection': t('httpStatus.redirection'),
      'client-error': t('httpStatus.clientError'),
      'server-error': t('httpStatus.serverError')
    };
    return categoryMap[category] || category;
  }

  // 获取分类颜色
  function getCategoryColor(category: string): string {
    const colorMap: Record<string, string> = {
      'informational': 'bg-blue-100 text-blue-800 dark:bg-blue-900/30 dark:text-blue-400',
      'success': 'bg-green-100 text-green-800 dark:bg-green-900/30 dark:text-green-400',
      'redirection': 'bg-yellow-100 text-yellow-800 dark:bg-yellow-900/30 dark:text-yellow-400',
      'client-error': 'bg-red-100 text-red-800 dark:bg-red-900/30 dark:text-red-400',
      'server-error': 'bg-orange-100 text-orange-800 dark:bg-orange-900/30 dark:text-orange-400'
    };
    return colorMap[category] || 'bg-gray-100 text-gray-800 dark:bg-gray-700 dark:text-gray-300';
  }

  // 搜索函数
  function search() {
    const trimmedQuery = query.trim();
    if (!trimmedQuery) {
      results = [];
      return;
    }

    if (searchMode === 'code') {
      // 按状态码搜索
      const code = parseInt(trimmedQuery);
      if (!isNaN(code)) {
        results = statusCodeDatabase.filter(item => item.code === code);
      } else {
        results = [];
      }
    } else {
      // 按名称或描述搜索
      const queryLower = trimmedQuery.toLowerCase();
      results = statusCodeDatabase.filter(item => 
        item.name.toLowerCase().includes(queryLower) ||
        item.description.toLowerCase().includes(queryLower) ||
        item.code.toString().includes(queryLower)
      );
    }
  }

  // 复制结果
  async function copyResult(text: string) {
    try {
      await navigator.clipboard.writeText(text);
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
      const saved = localStorage.getItem('http-status-query');
      if (saved) {
        try {
          const parsed = JSON.parse(saved);
          query = parsed.query || '';
          searchMode = parsed.searchMode || 'code';
        } catch (e) {
          console.error('Failed to load query:', e);
        }
      }
    }
  }

  // 保存到 localStorage
  function saveToStorage() {
    if (typeof window !== 'undefined') {
      localStorage.setItem('http-status-query', JSON.stringify({ query, searchMode }));
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
          class="px-4 py-2 rounded-lg transition-colors {searchMode === 'code'
            ? 'bg-primary-600 text-white'
            : 'bg-gray-200 dark:bg-gray-700 text-gray-900 dark:text-gray-100'}"
          onclick={() => { searchMode = 'code'; search(); }}
        >
          {t('httpStatus.searchByCode')}
        </button>
        <button
          class="px-4 py-2 rounded-lg transition-colors {searchMode === 'name'
            ? 'bg-primary-600 text-white'
            : 'bg-gray-200 dark:bg-gray-700 text-gray-900 dark:text-gray-100'}"
          onclick={() => { searchMode = 'name'; search(); }}
        >
          {t('httpStatus.searchByName')}
        </button>
      </div>

      <!-- 查询输入 -->
      <div>
        <label for="query-input" class="block text-sm font-medium mb-2 text-gray-700 dark:text-gray-300">
          {searchMode === 'code' ? t('httpStatus.statusCode') : t('httpStatus.nameOrDescription')}
        </label>
        <div class="flex gap-2">
          <input
            id="query-input"
            type="text"
            bind:value={query}
            placeholder={searchMode === 'code' ? t('httpStatus.codePlaceholder') : t('httpStatus.namePlaceholder')}
            class="input flex-1"
            onkeydown={handleKeyPress}
          />
          <button
            onclick={search}
            class="btn-secondary"
            disabled={!query.trim()}
          >
            <Search class="w-4 h-4 inline mr-1" />
            {t('httpStatus.search')}
          </button>
          {#if query || results.length > 0}
            <button type="button" class="btn-secondary" onclick={clear}>
              <Trash2 class="w-4 h-4 inline mr-1" />
              {t('common.clear')}
            </button>
          {/if}
        </div>
        <p class="text-xs text-gray-500 dark:text-gray-400 mt-1">
          {searchMode === 'code' ? t('httpStatus.codeHint') : t('httpStatus.nameHint')}
        </p>
      </div>
    </div>

    <!-- 结果列表 -->
    <div class="flex-1 overflow-y-auto">
      {#if results.length === 0 && query.trim()}
        <div class="flex flex-col items-center justify-center py-16 text-center">
          <Server class="w-12 h-12 text-gray-400 dark:text-gray-500 mb-4" />
          <p class="text-lg font-medium text-gray-900 dark:text-gray-100 mb-2">
            {t('httpStatus.noResults')}
          </p>
          <p class="text-sm text-gray-600 dark:text-gray-400">
            {t('httpStatus.noResultsDesc')}
          </p>
        </div>
      {:else if results.length > 0}
        <div class="space-y-3">
          {#each results as result (result.code)}
            <div class="bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-lg p-4 hover:border-primary-500 dark:hover:border-primary-500 transition-colors">
              <div class="flex items-start justify-between gap-4">
                <div class="flex-1 min-w-0">
                  <div class="flex items-center gap-3 mb-2">
                    <span class="text-2xl font-bold text-gray-900 dark:text-gray-100">
                      {result.code}
                    </span>
                    <span class="text-lg font-semibold text-gray-900 dark:text-gray-100">
                      {result.name}
                    </span>
                    <span class="px-2 py-1 rounded text-xs font-medium {getCategoryColor(result.category)}">
                      {getCategoryName(result.category)}
                    </span>
                    {#if result.rfc}
                      <span class="text-xs text-gray-500 dark:text-gray-400 font-mono">
                        {result.rfc}
                      </span>
                    {/if}
                  </div>
                  <p class="text-sm text-gray-700 dark:text-gray-300">
                    {result.description}
                  </p>
                </div>
                <button
                  onclick={() => copyResult(result.code.toString())}
                  class="p-1.5 rounded hover:bg-gray-100 dark:hover:bg-gray-700 text-gray-600 dark:text-gray-400 transition-colors flex-shrink-0"
                  title={t('common.copy')}
                >
                  {#if copied}
                    <Check class="w-4 h-4" />
                  {:else}
                    <Copy class="w-4 h-4" />
                  {/if}
                </button>
              </div>
            </div>
          {/each}
        </div>
      {:else}
        <div class="space-y-3">
          <div class="mb-3">
            <h3 class="text-base font-semibold text-gray-900 dark:text-gray-100 mb-1">
              {t('httpStatus.commonStatusCodes')}
            </h3>
            <p class="text-xs text-gray-600 dark:text-gray-400">
              {t('httpStatus.commonStatusCodesDesc')}
            </p>
          </div>
          <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-2">
            {#each commonResults as result (result.code)}
              <div class="bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-lg p-2.5 hover:border-primary-500 dark:hover:border-primary-500 transition-colors cursor-pointer" onclick={() => { query = result.code.toString(); search(); }}>
                <div class="flex items-start justify-between gap-2">
                  <div class="flex-1 min-w-0">
                    <div class="flex items-center gap-2 mb-1 flex-wrap">
                      <span class="text-xl font-bold text-gray-900 dark:text-gray-100">
                        {result.code}
                      </span>
                      <span class="text-sm font-semibold text-gray-900 dark:text-gray-100">
                        {result.name}
                      </span>
                      <span class="px-1.5 py-0.5 rounded text-xs font-medium {getCategoryColor(result.category)}">
                        {getCategoryName(result.category)}
                      </span>
                    </div>
                    <p class="text-xs text-gray-600 dark:text-gray-400 line-clamp-2">
                      {result.description}
                    </p>
                  </div>
                  <button
                    onclick={(e) => { e.stopPropagation(); copyResult(result.code.toString()); }}
                    class="p-1 rounded hover:bg-gray-100 dark:hover:bg-gray-700 text-gray-600 dark:text-gray-400 transition-colors flex-shrink-0"
                    title={t('common.copy')}
                  >
                    {#if copied}
                      <Check class="w-3.5 h-3.5" />
                    {:else}
                      <Copy class="w-3.5 h-3.5" />
                    {/if}
                  </button>
                </div>
              </div>
            {/each}
          </div>
        </div>
      {/if}
    </div>
  </div>
</div>
