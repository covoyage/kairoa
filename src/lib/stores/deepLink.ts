import { writable, get } from 'svelte/store';
import { goto } from '$app/navigation';
import { page } from '$app/stores';

// Deep link action interface
export interface DeepLinkAction {
  tool: string;
  data: Record<string, string>;
}

// Current deep link action store
export const deepLinkAction = writable<DeepLinkAction | null>(null);

// Tool data store - for passing data to tool pages
export const toolDataStore = writable<Record<string, Record<string, string>>>({});

// All supported tools with their routes and parameters
export const TOOL_CONFIGS: Record<string, {
  route: string;
  params: string[];
}> = {
  // Text & Encoding
  'text-processing': { route: '/text-processing', params: ['text'] },
  'text-stats': { route: '/text-stats', params: ['text'] },
  'text-diff': { route: '/text-diff', params: ['text1', 'text2'] },
  'encode-decode': { route: '/encode-decode', params: ['text', 'type'] },
  'base64': { route: '/base64', params: ['text', 'action'] },
  'url-encode': { route: '/encode-decode', params: ['text', 'type'] },
  'html-encode': { route: '/encode-decode', params: ['text', 'type'] },
  'jwt': { route: '/jwt', params: ['token'] },
  
  // Formatting
  'json': { route: '/json', params: ['text'] },
  'sql-formatter': { route: '/sql-formatter', params: ['text'] },
  'config-converter': { route: '/config-converter', params: ['text', 'from', 'to'] },
  
  // Crypto & Security
  'hash': { route: '/hash', params: ['text'] },
  'crypto': { route: '/crypto', params: ['text', 'algorithm', 'action'] },
  'hmac': { route: '/hmac', params: ['text', 'key', 'algorithm'] },
  'rsa': { route: '/rsa', params: ['action'] },
  'certificate-viewer': { route: '/certificate-viewer', params: [] },
  'password-strength': { route: '/password-strength', params: ['password'] },
  'password-vault': { route: '/password-vault', params: [] },
  'basic-auth': { route: '/basic-auth', params: ['username', 'password'] },
  
  // Code Tools
  'uuid': { route: '/uuid', params: ['count', 'version'] },
  'ulid': { route: '/uuid', params: ['count'] },
  'qr-code': { route: '/qr-code', params: ['text', 'size'] },
  'ascii-art': { route: '/ascii-art', params: ['text', 'font'] },
  'mock-generator': { route: '/mock-generator', params: ['type', 'count'] },
  
  // Data & Color
  'data-converter': { route: '/data-converter', params: ['text', 'from', 'to'] },
  'color': { route: '/color', params: ['color', 'type'] },
  'base-converter': { route: '/base-converter', params: ['number', 'from', 'to'] },
  'roman-numeral': { route: '/roman-numeral', params: ['text', 'direction'] },
  'coordinate-converter': { route: '/coordinate-converter', params: ['lat', 'lng', 'from'] },
  'iban': { route: '/iban', params: ['iban'] },
  
  // Time & Regex
  'time': { route: '/time', params: ['timestamp', 'format'] },
  'crontab': { route: '/crontab', params: ['expression'] },
  'regex-tester': { route: '/regex-tester', params: ['pattern', 'text'] },
  
  // Network Tools
  'api-client': { route: '/api-client', params: ['url', 'method'] },
  'dns-lookup': { route: '/dns-lookup', params: ['host', 'type'] },
  'port-scanner': { route: '/port-scanner', params: ['host', 'start', 'end'] },
  'traceroute': { route: '/traceroute', params: ['host'] },
  'tls-checker': { route: '/tls-checker', params: ['host', 'port'] },
  'websocket': { route: '/websocket', params: ['url'] },
  'url-parser': { route: '/url', params: ['url'] },
  'user-agent': { route: '/user-agent', params: ['ua'] },
  
  // Other Tools
  'chmod': { route: '/chmod', params: ['mode'] },
  'docker-commands': { route: '/docker-commands', params: [] },
  'pdf-signature': { route: '/pdf-signature', params: [] },
  'previewer': { route: '/previewer', params: [] },
  'ai-chat': { route: '/ai-chat', params: ['message'] },
  
  // Additional tools
  'generator': { route: '/generator', params: ['type'] },
  'image-tools': { route: '/image-tools', params: [] },
  'ip-lookup': { route: '/ip-lookup', params: ['ip'] },
  'keycode': { route: '/keycode', params: [] },
  'mime-type': { route: '/mime-type', params: ['extension'] },
  'otp': { route: '/otp', params: ['secret'] },
  'http-status': { route: '/http-status', params: ['code'] },
  'env-manager': { route: '/env-manager', params: [] },
  'git-commands': { route: '/git-commands', params: [] },
  'file-to-markdown': { route: '/file-to-markdown', params: [] },
};

/**
 * Parse a deep link URL into a DeepLinkAction
 * @param url - The deep link URL (e.g., "kairoa://hash?text=hello")
 * @returns DeepLinkAction or null if invalid
 */
export function parseDeepLink(url: string): DeepLinkAction | null {
  try {
    // Handle both "kairoa://tool?params" and just "//tool?params" formats
    let urlToParse = url;
    if (url.startsWith('kairoa://')) {
      urlToParse = 'https:' + url.substring(8); // Convert to standard URL format
    } else if (url.startsWith('//')) {
      urlToParse = 'https:' + url;
    }
    
    const parsed = new URL(urlToParse);
    
    // Extract tool name from hostname (kairoa://hash -> hash)
    const tool = parsed.hostname;
    
    if (!tool || !TOOL_CONFIGS[tool]) {
      console.warn(`Unknown tool in deep link: ${tool}`);
      return null;
    }
    
    // Extract parameters
    const data: Record<string, string> = {};
    parsed.searchParams.forEach((value, key) => {
      data[key] = decodeURIComponent(value);
    });
    
    return { tool, data };
  } catch (error) {
    console.error('Failed to parse deep link:', error);
    return null;
  }
}

/**
 * Handle a deep link action by navigating to the tool and setting data
 * @param action - The deep link action to handle
 * @param navigate - Function to navigate to a route
 */
export function handleDeepLinkAction(
  action: DeepLinkAction,
  navigate: (route: string) => void
): void {
  const config = TOOL_CONFIGS[action.tool];
  if (!config) {
    console.warn(`No config found for tool: ${action.tool}`);
    return;
  }
  
  // Store data for the tool page to use
  toolDataStore.update(store => ({
    ...store,
    [action.tool]: action.data
  }));
  
  // Navigate to the tool route
  navigate(config.route);
  
  // Set the current action
  deepLinkAction.set(action);
}

/**
 * Initialize deep link listener
 * Call this in the root layout
 */
export async function initDeepLinkListener(
  navigate: (route: string) => void
): Promise<() => void> {
  // Check if running in Tauri
  if (typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window) {
    try {
      const { onOpenUrl } = await import('@tauri-apps/plugin-deep-link');
      
      const unlisten = await onOpenUrl((urls) => {
        for (const url of urls) {
          console.log('Deep link received:', url);
          const action = parseDeepLink(url);
          if (action) {
            handleDeepLinkAction(action, navigate);
          }
        }
      });
      
      console.log('Deep link listener initialized');
      return unlisten;
    } catch (error) {
      console.error('Failed to initialize deep link listener:', error);
      return () => {};
    }
  }
  
  return () => {};
}

/**
 * Get data for a specific tool and optionally clear it
 * @param tool - Tool name
 * @param clearAfterRead - Whether to clear the data after reading
 * @returns Tool data or undefined
 */
export function getToolData(tool: string, clearAfterRead: boolean = true): Record<string, string> | undefined {
  const store = get(toolDataStore);
  const data = store[tool];
  
  if (data && clearAfterRead) {
    toolDataStore.update(s => {
      const newStore = { ...s };
      delete newStore[tool];
      return newStore;
    });
  }
  
  return data;
}

/**
 * Generate a deep link URL for a tool
 * @param tool - Tool name
 * @param params - Parameters
 * @returns Deep link URL
 */
export function generateDeepLink(tool: string, params: Record<string, string> = {}): string {
  const searchParams = new URLSearchParams();
  Object.entries(params).forEach(([key, value]) => {
    searchParams.set(key, value);
  });
  
  const queryString = searchParams.toString();
  return `kairoa://${tool}${queryString ? '?' + queryString : ''}`;
}
