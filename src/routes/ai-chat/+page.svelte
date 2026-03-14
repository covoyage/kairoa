<script lang="ts">
  import { translationsStore } from '$lib/stores/i18n';
  import { MessageSquare, Settings, Trash2, Copy, Check, Eye, EyeOff, FileCode } from 'lucide-svelte';
  import { onMount } from 'svelte';
  import { browser } from '$app/environment';
  import 'highlight.js/styles/github.css';
  import { renderMarkdown } from '$lib/utils/ai-chat/markdown';
  import { createThinkStreamParser } from '$lib/utils/ai-chat/streamParser';
  import { loadFromStorage, saveToStorage, loadChatSessions, saveCurrentSession, deleteChatSession } from '$lib/utils/ai-chat/storage';
  import { PROVIDER_PRESETS, type ChatMessage, type ChatSettings, type ProviderPresetId, type ChatSession } from '$lib/utils/ai-chat/types';

  let translations = $derived($translationsStore);
  let messages = $state<ChatMessage[]>([]);
  let input = $state('');
  let isSending = $state(false);
  let error = $state('');
  let copied = $state(false);
  let copyMode = $state<'text' | 'markdown'>('text');
  let showApiKey = $state(false);
  let thinkingExpanded = $state<Record<string, boolean>>({});
  let abortController = $state<AbortController | null>(null);
  let messageUpdateQueue = $state<Map<string, Partial<ChatMessage>>>(new Map());
  let updateTimeout = $state<ReturnType<typeof setTimeout> | null>(null);
  let showModelDropdown = $state(false);
  let showSettingsPanel = $state(false);
  let showAddConfigModal = $state(false);
  let showManageConfigsModal = $state(false);
  let showEditConfigModal = $state(false);
  let showChatHistoryModal = $state(false);
  let editingConfigId = $state<string>('');
  let modelSearchQuery = $state('');
  let currentSessionId = $state<string>('');
  let chatSessions = $state<ChatSession[]>([]);

  // API Configurations management
  interface ModelConfig {
    id: string;
    name: string;
  }

  interface ApiConfig {
    id: string;
    name: string;
    provider: ProviderPresetId;
    apiBaseUrl: string;
    apiKey: string;
    models: ModelConfig[];
  }

  let apiConfigs = $state<ApiConfig[]>([]);
  let activeConfigId = $state<string>('');
  let activeModelId = $state<string>(''); // Currently selected model within the active config

  // New config form
  let newConfig = $state<{
    name: string;
    provider: ProviderPresetId;
    apiBaseUrl: string;
    apiKey: string;
    models: ModelConfig[];
    currentModel: string; // For adding models one by one
  }>({
    name: '',
    provider: 'openai',
    apiBaseUrl: '',
    apiKey: '',
    models: [],
    currentModel: ''
  });

  // File attachment state
  let attachedFiles = $state<Array<{ name: string; content: string; type: string }>>([]);
  let fileInputRef = $state<HTMLInputElement | null>(null);

  // API Key visibility state for modals
  let showApiKeyInAddModal = $state(false);
  let showApiKeyInEditModal = $state(false);

  // Handle file selection
  async function handleFileSelect(event: Event) {
    const target = event.target as HTMLInputElement;
    const files = target.files;
    if (!files || files.length === 0) return;

    for (const file of files) {
      try {
        const content = await readFileContent(file);
        attachedFiles = [...attachedFiles, {
          name: file.name,
          content,
          type: file.type
        }];
      } catch (e) {
        console.error('Failed to read file:', e);
        error = t('aiChat.fileReadError');
      }
    }

    // Reset input
    if (fileInputRef) {
      fileInputRef.value = '';
    }
  }

  // Read file content
  function readFileContent(file: File): Promise<string> {
    return new Promise((resolve, reject) => {
      const reader = new FileReader();
      reader.onload = (e) => {
        resolve(e.target?.result as string);
      };
      reader.onerror = (e) => {
        reject(e);
      };

      // For text files, read as text; for images, read as data URL
      if (file.type.startsWith('image/')) {
        reader.readAsDataURL(file);
      } else {
        reader.readAsText(file);
      }
    });
  }

  // Remove attached file
  function removeAttachedFile(index: number) {
    attachedFiles = attachedFiles.filter((_, i) => i !== index);
  }

  // Auto-fill apiBaseUrl when provider changes
  $effect(() => {
    const provider = newConfig.provider;
    if (provider && provider !== 'custom' && provider in PROVIDER_PRESETS) {
      const preset = PROVIDER_PRESETS[provider as Exclude<ProviderPresetId, 'custom'>];
      newConfig.apiBaseUrl = preset.apiBaseUrl;
      // Note: Model is not auto-filled, user can input custom model name
    }
  });

  // Get recommended model placeholder based on selected provider
  function getModelPlaceholder(): string {
    const provider = newConfig.provider;
    if (provider && provider !== 'custom' && provider in PROVIDER_PRESETS) {
      const preset = PROVIDER_PRESETS[provider as Exclude<ProviderPresetId, 'custom'>];
      return preset.model;
    }
    return t('aiChat.modelPlaceholder');
  }

  // Load API configs from storage
  function loadApiConfigs() {
    if (!browser) return;
    try {
      const stored = localStorage.getItem('aiChat.apiConfigs');
      if (stored) {
        const parsed = JSON.parse(stored);
        // Migrate old data structure (single model) to new structure (models array)
        apiConfigs = parsed.map((config: any) => {
          if (config.models) {
            return config; // Already new format
          }
          // Migrate old format
          return {
            ...config,
            models: config.model ? [{ id: config.model, name: config.model }] : []
          };
        });
      }
      const activeId = localStorage.getItem('aiChat.activeConfigId');
      const activeModel = localStorage.getItem('aiChat.activeModelId');
      if (activeId) {
        activeConfigId = activeId;
        activeModelId = activeModel || '';
        const config = apiConfigs.find(c => c.id === activeId);
        if (config && config.models.length > 0) {
          settings = {
            ...settings,
            provider: config.provider,
            apiBaseUrl: config.apiBaseUrl,
            apiKey: config.apiKey,
            model: activeModelId && config.models.find(m => m.id === activeModelId) 
              ? activeModelId 
              : config.models[0].id
          };
        }
      }
    } catch (e) {
      console.error('Failed to load API configs:', e);
    }
  }

  // Save API configs to storage
  function saveApiConfigs() {
    if (!browser) return;
    try {
      localStorage.setItem('aiChat.apiConfigs', JSON.stringify(apiConfigs));
      localStorage.setItem('aiChat.activeConfigId', activeConfigId);
      localStorage.setItem('aiChat.activeModelId', activeModelId);
    } catch (e) {
      console.error('Failed to save API configs:', e);
    }
  }

  // Add new API config
  function addApiConfig() {
    if (!newConfig.provider || !newConfig.apiBaseUrl || !newConfig.apiKey || newConfig.models.length === 0) {
      return;
    }
    const configName = newConfig.name || `${newConfig.provider} - ${newConfig.models.map(m => m.name).join(', ')}`;
    const config: ApiConfig = {
      id: crypto.randomUUID(),
      name: configName,
      provider: newConfig.provider,
      apiBaseUrl: newConfig.apiBaseUrl,
      apiKey: newConfig.apiKey,
      models: newConfig.models
    };
    apiConfigs = [...apiConfigs, config];
    activeConfigId = config.id;
    activeModelId = config.models[0].id;
    settings = {
      ...settings,
      provider: config.provider,
      apiBaseUrl: config.apiBaseUrl,
      apiKey: config.apiKey,
      model: config.models[0].id
    };
    saveApiConfigs();
    showAddConfigModal = false;
    newConfig = { name: '', provider: 'openai', apiBaseUrl: '', apiKey: '', models: [], currentModel: '' };
  }

  // Delete API config
  function deleteApiConfig(id: string) {
    apiConfigs = apiConfigs.filter(c => c.id !== id);
    if (activeConfigId === id && apiConfigs.length > 0) {
      activeConfigId = apiConfigs[0].id;
      activeModelId = apiConfigs[0].models[0]?.id || '';
      const config = apiConfigs[0];
      settings = {
        ...settings,
        provider: config.provider,
        apiBaseUrl: config.apiBaseUrl,
        apiKey: config.apiKey,
        model: config.models[0]?.id || ''
      };
    }
    saveApiConfigs();
  }

  // Edit API config
  function startEditConfig(config: ApiConfig) {
    editingConfigId = config.id;
    newConfig = {
      name: config.name,
      provider: config.provider,
      apiBaseUrl: config.apiBaseUrl,
      apiKey: config.apiKey,
      models: [...config.models],
      currentModel: ''
    };
    showManageConfigsModal = false;
    showEditConfigModal = true;
  }

  function saveEditConfig() {
    if (!editingConfigId || !newConfig.provider || !newConfig.apiBaseUrl || !newConfig.apiKey || newConfig.models.length === 0) {
      return;
    }
    const configIndex = apiConfigs.findIndex(c => c.id === editingConfigId);
    if (configIndex === -1) return;

    const configName = newConfig.name || `${newConfig.provider} - ${newConfig.models.map(m => m.name).join(', ')}`;
    const updatedConfig: ApiConfig = {
      id: editingConfigId,
      name: configName,
      provider: newConfig.provider,
      apiBaseUrl: newConfig.apiBaseUrl,
      apiKey: newConfig.apiKey,
      models: newConfig.models
    };

    apiConfigs = apiConfigs.map(c => c.id === editingConfigId ? updatedConfig : c);

    // If this was the active config, update settings
    if (activeConfigId === editingConfigId) {
      const modelId = activeModelId && updatedConfig.models.find(m => m.id === activeModelId)
        ? activeModelId
        : updatedConfig.models[0].id;
      settings = {
        ...settings,
        provider: updatedConfig.provider,
        apiBaseUrl: updatedConfig.apiBaseUrl,
        apiKey: updatedConfig.apiKey,
        model: modelId
      };
    }

    saveApiConfigs();
    showEditConfigModal = false;
    editingConfigId = '';
    newConfig = { name: '', provider: 'openai', apiBaseUrl: '', apiKey: '', models: [], currentModel: '' };
  }

  // Switch to config
  function switchToConfig(config: ApiConfig) {
    activeConfigId = config.id;
    activeModelId = config.models[0]?.id || '';
    settings = {
      ...settings,
      provider: config.provider,
      apiBaseUrl: config.apiBaseUrl,
      apiKey: config.apiKey,
      model: config.models[0]?.id || ''
    };
    saveApiConfigs();
    showManageConfigsModal = false;
  }

  // Get configured providers (those with complete API config)
  let configuredProviders = $derived(() => {
    const providers = new Set<string>();
    for (const config of apiConfigs) {
      if (config.apiKey && config.apiBaseUrl) {
        providers.add(config.provider);
      }
    }
    return providers;
  });

  // Get available models from configured API configs
  let availableModelGroups = $derived(() => {
    const query = modelSearchQuery.toLowerCase();
    const filtered: Record<string, Array<{ id: string; name: string; tag?: string; configId: string }>> = {};
    
    for (const config of apiConfigs) {
      if (!config.apiKey || !config.apiBaseUrl || config.models.length === 0) continue;
      
      const matchingModels = query 
        ? config.models.filter(m => 
            m.name.toLowerCase().includes(query) || 
            m.id.toLowerCase().includes(query)
          )
        : config.models;
        
      if (matchingModels.length > 0) {
        // For custom provider, use the config name (service provider name) as group name
        // For preset providers, use the provider ID
        const groupName = config.provider === 'custom' 
          ? (config.name || 'Custom')
          : (config.name || config.provider);
        filtered[groupName] = matchingModels.map(m => ({ ...m, configId: config.id }));
      }
    }
    return filtered;
  });

  // Model definitions with groups
  const MODEL_GROUPS: Record<string, Array<{ id: string; name: string; tag?: string }>> = {
    'OpenAI': [
      { id: 'gpt-5.4', name: 'GPT-5.4' },
      { id: 'gpt-5.4-mini', name: 'GPT-5.4 Mini' },
      { id: 'gpt-5.4-nano', name: 'GPT-5.4 Nano' },
      { id: 'gpt-4o', name: 'GPT-4o' },
      { id: 'gpt-4o-mini', name: 'GPT-4o Mini' }
    ],
    'DeepSeek': [
      { id: 'deepseek-chat', name: 'DeepSeek Chat' },
      { id: 'deepseek-reasoner', name: 'DeepSeek Reasoner' }
    ],
    'Anthropic': [
      { id: 'claude-3-5-sonnet-20241022', name: 'Claude 3.5 Sonnet' },
      { id: 'claude-3-5-haiku-20241022', name: 'Claude 3.5 Haiku' },
      { id: 'claude-3-opus-20240229', name: 'Claude 3 Opus' }
    ],
    'OpenRouter': [
      { id: 'openai/gpt-4o-mini', name: 'GPT-4o Mini', tag: 'Free' },
      { id: 'anthropic/claude-3.5-sonnet', name: 'Claude 3.5 Sonnet' },
      { id: 'google/gemini-flash-1.5', name: 'Gemini Flash 1.5', tag: 'Free' },
      { id: 'meta-llama/llama-3.3-70b-instruct', name: 'Llama 3.3 70B', tag: 'Free' }
    ],
    'Groq': [
      { id: 'llama-3.3-70b-versatile', name: 'Llama 3.3 70B', tag: 'Free' },
      { id: 'llama-3.1-8b-instant', name: 'Llama 3.1 8B', tag: 'Free' },
      { id: 'mixtral-8x7b-32768', name: 'Mixtral 8x7B', tag: 'Free' }
    ],
    'Mistral': [
      { id: 'mistral-large-2512', name: 'Mistral Large' },
      { id: 'mistral-medium', name: 'Mistral Medium' },
      { id: 'mistral-small', name: 'Mistral Small' }
    ],
    '智谱 AI': [
      { id: 'glm-5', name: 'GLM-5' },
      { id: 'glm-4-plus', name: 'GLM-4 Plus' },
      { id: 'glm-4-air', name: 'GLM-4 Air' }
    ],
    'MiniMax': [
      { id: 'MiniMax-M2.5', name: 'MiniMax M2.5' }
    ],
    'Kimi': [
      { id: 'kimi-k2.5', name: 'Kimi K2.5' },
      { id: 'kimi-k2', name: 'Kimi K2' }
    ],
    'DashScope': [
      { id: 'qwen3.5-plus', name: 'Qwen 3.5 Plus' },
      { id: 'qwen3.5-turbo', name: 'Qwen 3.5 Turbo' },
      { id: 'qwen-long', name: 'Qwen Long' }
    ]
  };

  // Provider to model group mapping
  const PROVIDER_MODEL_MAP: Record<string, string[]> = {
    'openai': ['OpenAI'],
    'deepseek': ['DeepSeek'],
    'anthropic': ['Anthropic'],
    'openrouter': ['OpenRouter'],
    'groq': ['Groq'],
    'mistral': ['Mistral'],
    'zhipu': ['智谱 AI'],
    'minimax': ['MiniMax'],
    'kimi': ['Kimi'],
    'dashscope': ['DashScope'],
    'azure-openai': ['OpenAI'],
    'custom': Object.keys(MODEL_GROUPS) // Custom shows all models
  };

  let filteredModelGroups = $derived(() => {
    const allowedGroups = PROVIDER_MODEL_MAP[settings.provider] || [];
    const query = modelSearchQuery.toLowerCase();
    const filtered: Record<string, Array<{ id: string; name: string; tag?: string }>> = {};
    
    for (const groupName of allowedGroups) {
      const models = MODEL_GROUPS[groupName];
      if (!models) continue;
      
      const matchingModels = query 
        ? models.filter(m => 
            m.name.toLowerCase().includes(query) || 
            m.id.toLowerCase().includes(query)
          )
        : models;
        
      if (matchingModels.length > 0) {
        filtered[groupName] = matchingModels;
      }
    }
    return filtered;
  });

  function selectModel(modelId: string, configId?: string) {
    if (configId) {
      const config = apiConfigs.find(c => c.id === configId);
      if (config) {
        activeConfigId = configId;
        activeModelId = modelId;
        settings = {
          ...settings,
          provider: config.provider,
          apiBaseUrl: config.apiBaseUrl,
          apiKey: config.apiKey,
          model: modelId
        };
        saveApiConfigs();
      }
    } else {
      settings.model = modelId;
    }
    showModelDropdown = false;
    modelSearchQuery = '';
  }

  function getModelDisplayName(modelId: string): string {
    // Search in apiConfigs first
    for (const config of apiConfigs) {
      const model = config.models.find(m => m.id === modelId);
      if (model) return model.name;
    }
    // Fallback to MODEL_GROUPS for backward compatibility
    for (const models of Object.values(MODEL_GROUPS)) {
      const model = models.find(m => m.id === modelId);
      if (model) return model.name;
    }
    return modelId;
  }

  function getModelDisplayWithProvider(modelId: string): string {
    // Search in apiConfigs first
    for (const config of apiConfigs) {
      const model = config.models.find(m => m.id === modelId);
      if (model) {
        const providerName = config.name || config.provider;
        return `${providerName}/${model.name}`;
      }
    }
    // Fallback to MODEL_GROUPS for backward compatibility
    for (const [groupName, models] of Object.entries(MODEL_GROUPS)) {
      const model = models.find(m => m.id === modelId);
      if (model) return `${groupName}/${model.name}`;
    }
    return modelId;
  }

  // Close dropdown when clicking outside
  onMount(() => {
    const handleClickOutside = (e: MouseEvent) => {
      const target = e.target as HTMLElement;
      if (!target.closest('.model-dropdown-container')) {
        showModelDropdown = false;
      }
    };
    document.addEventListener('click', handleClickOutside);
    return () => document.removeEventListener('click', handleClickOutside);
  });

  let settings = $state<ChatSettings>({
    provider: 'openai',
    apiBaseUrl: PROVIDER_PRESETS.openai.apiBaseUrl,
    apiKey: '',
    model: PROVIDER_PRESETS.openai.model,
    systemPrompt: '',
    maxHistory: 10
  });

  function t(key: string): string {
    const keys = key.split('.');
    let value: any = translations;
    for (const k of keys) {
      value = value?.[k];
    }
    return value || key;
  }

  function addMessage(role: ChatMessage['role'], content: string, thinking?: string) {
    messages = [
      ...messages,
      {
        id: crypto.randomUUID(),
        role,
        content,
        createdAt: Date.now(),
        thinking
      }
    ];
  }

  function updateMessage(id: string, patch: Partial<ChatMessage>, immediate = false) {
    if (immediate) {
      messages = messages.map((m) => (m.id === id ? { ...m, ...patch } : m));
      return;
    }

    const current = messageUpdateQueue.get(id) || {};
    messageUpdateQueue.set(id, { ...current, ...patch });

    if (updateTimeout) return;

    updateTimeout = setTimeout(() => {
      messages = messages.map((m) => {
        const patch = messageUpdateQueue.get(m.id);
        return patch ? { ...m, ...patch } : m;
      });
      messageUpdateQueue.clear();
      updateTimeout = null;
    }, 50);
  }

  function flushMessageUpdates() {
    if (updateTimeout) {
      clearTimeout(updateTimeout);
      updateTimeout = null;
    }
    if (messageUpdateQueue.size > 0) {
      messages = messages.map((m) => {
        const patch = messageUpdateQueue.get(m.id);
        return patch ? { ...m, ...patch } : m;
      });
      messageUpdateQueue.clear();
    }
  }

  function stopGeneration() {
    abortController?.abort();
    abortController = null;
    isSending = false;
    flushMessageUpdates();
  }

  function newChat() {
    // Save current session before starting new one
    if (messages.length > 0 && currentSessionId) {
      saveCurrentSession({
        id: currentSessionId,
        title: generateSessionTitle(messages),
        messages: [...messages],
        createdAt: Date.now(),
        updatedAt: Date.now()
      });
    }
    messages = [];
    error = '';
    currentSessionId = crypto.randomUUID();
    chatSessions = loadChatSessions();
  }

  function generateSessionTitle(msgs: ChatMessage[]): string {
    const firstUserMsg = msgs.find(m => m.role === 'user');
    if (firstUserMsg) {
      return firstUserMsg.content.slice(0, 30) + (firstUserMsg.content.length > 30 ? '...' : '');
    }
    return 'New Chat';
  }

  function loadSession(session: ChatSession) {
    // Save current session before switching
    if (messages.length > 0 && currentSessionId) {
      saveCurrentSession({
        id: currentSessionId,
        title: generateSessionTitle(messages),
        messages: [...messages],
        createdAt: Date.now(),
        updatedAt: Date.now()
      });
    }
    currentSessionId = session.id;
    messages = [...session.messages];
    error = '';
    showChatHistoryModal = false;
    chatSessions = loadChatSessions();
  }

  function deleteSession(sessionId: string) {
    deleteChatSession(sessionId);
    chatSessions = loadChatSessions();
    if (currentSessionId === sessionId) {
      currentSessionId = crypto.randomUUID();
      messages = [];
    }
  }

  function openChatHistory() {
    chatSessions = loadChatSessions();
    showChatHistoryModal = true;
  }

  async function sendMessage() {
    if (!input.trim() && attachedFiles.length === 0) return;

    if (!settings.apiBaseUrl.trim() || !settings.apiKey.trim() || !settings.model.trim()) {
      error = t('aiChat.errors.missingConfig');
      return;
    }

    // Build user content
    const textContent = input.trim();
    const filesToSend = [...attachedFiles];
    
    input = '';
    error = '';
    attachedFiles = []; // Clear attached files

    // Build display content for UI
    let displayContent = textContent;
    if (filesToSend.length > 0) {
      const fileNames = filesToSend.map(f => f.type.startsWith('image/') ? `[图片: ${f.name}]` : `[文件: ${f.name}]`).join(' ');
      displayContent = textContent ? `${textContent} ${fileNames}` : fileNames;
    }
    addMessage('user', displayContent);

    isSending = true;
    try {
      const apiBase = settings.apiBaseUrl.replace(/\/+$/, '');

      const history = messages
        .filter((m) => m.role === 'user' || m.role === 'assistant')
        .slice(-settings.maxHistory);

      const apiMessages: { role: 'system' | 'user' | 'assistant'; content: string | Array<{type: string; text?: string; image_url?: {url: string}}> }[] = [];

      if (settings.systemPrompt.trim()) {
        apiMessages.push({ role: 'system', content: settings.systemPrompt.trim() });
      }

      for (const m of history) {
        apiMessages.push({ role: m.role, content: m.content });
      }

      // Build multimodal content for API
      const userMessageContent: Array<{type: string; text?: string; image_url?: {url: string}}> = [];
      
      if (textContent) {
        userMessageContent.push({ type: 'text', text: textContent });
      }
      
      // Add images in multimodal format
      for (const file of filesToSend) {
        if (file.type.startsWith('image/')) {
          userMessageContent.push({
            type: 'image_url',
            image_url: { url: file.content }
          });
        } else {
          // For non-image files, add as text
          userMessageContent.push({
            type: 'text',
            text: `[File: ${file.name}]\n\`\`\`\n${file.content}\n\`\`\``
          });
        }
      }
      
      apiMessages.push({ role: 'user', content: userMessageContent });

      const assistantId = crypto.randomUUID();
      messages = [
        ...messages,
        {
          id: assistantId,
          role: 'assistant',
          content: '',
          createdAt: Date.now()
        }
      ];

      abortController = new AbortController();

      const response = await fetch(`${apiBase}/chat/completions`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          Authorization: `Bearer ${settings.apiKey.trim()}`,
          ...(settings.provider === 'anthropic' ? { 'anthropic-version': '2023-06-01' } : {})
        },
        body: JSON.stringify({
          model: settings.model.trim(),
          messages: apiMessages,
          ...(settings.provider !== 'groq' ? { stream: true } : {})
        }),
        signal: abortController.signal
      });

      if (!response.ok) {
        const text = await response.text();
        throw new Error(`${response.status} ${response.statusText}: ${text}`);
      }

      if (!response.body) {
        throw new Error(t('aiChat.errors.invalidResponse'));
      }

      const reader = response.body.getReader();
      const decoder = new TextDecoder('utf-8');
      const parser = createThinkStreamParser();

      let sseBuffer = '';
      let hasThinking = false;

      while (true) {
        const { value, done } = await reader.read();
        if (done) break;
        if (!value) continue;

        sseBuffer += decoder.decode(value, { stream: true });

        const parts = sseBuffer.split('\n\n');
        sseBuffer = parts.pop() || '';

        for (const part of parts) {
          const lines = part
            .split('\n')
            .map((l) => l.trim())
            .filter(Boolean);

          for (const line of lines) {
            if (!line.startsWith('data:')) continue;
            const dataStr = line.slice(5).trim();
            if (!dataStr) continue;
            if (dataStr === '[DONE]') {
              const final = parser.flush();
              updateMessage(assistantId, {
                content: final.answer,
                thinking: final.thinking
              });
              if (hasThinking) {
                thinkingExpanded = { ...thinkingExpanded, [assistantId]: false };
              }
              return;
            }

            try {
              const payload = JSON.parse(dataStr);
              const delta =
                payload?.choices?.[0]?.delta?.content ??
                payload?.choices?.[0]?.message?.content ??
                '';
              if (!delta) continue;

              const { answerDelta, thinkingDelta } = parser.feed(String(delta));

              if (thinkingDelta) {
                hasThinking = true;
                const curThinking =
                  messages.find((m) => m.id === assistantId)?.thinking || '';
                updateMessage(assistantId, {
                  thinking: curThinking + thinkingDelta
                });
                thinkingExpanded = { ...thinkingExpanded, [assistantId]: true };
              }

              if (answerDelta) {
                const cur = messages.find((m) => m.id === assistantId)?.content || '';
                updateMessage(assistantId, { content: cur + answerDelta });
              }
            } catch {
              // ignore non-JSON lines
            }
          }
        }
      }

      const final = parser.flush();
      updateMessage(assistantId, {
        content: final.answer,
        thinking: final.thinking
      });
      if (hasThinking) {
        thinkingExpanded = { ...thinkingExpanded, [assistantId]: false };
      }
    } catch (e) {
      if (e instanceof Error && e.name === 'AbortError') {
        return;
      }
      const msg =
        e instanceof Error ? e.message : typeof e === 'string' ? e : t('aiChat.errors.requestFailed');
      error = msg;
    } finally {
      isSending = false;
      abortController = null;
      flushMessageUpdates();
    }
  }

  function handleKeyDown(event: KeyboardEvent) {
    if (event.key === 'Enter' && !event.shiftKey) {
      event.preventDefault();
      if (!isSending) {
        sendMessage();
      }
    }
  }

  function adjustTextareaHeight(event: Event) {
    const target = event.target as HTMLTextAreaElement;
    target.style.height = 'auto';
    target.style.height = Math.min(target.scrollHeight, 192) + 'px';
  }

  async function copyMessage(text: string, mode: 'text' | 'markdown' = 'text') {
    if (!text) return;
    try {
      await navigator.clipboard.writeText(text);
      copied = true;
      copyMode = mode;
      setTimeout(() => {
        copied = false;
      }, 1500);
    } catch (e) {
      console.error('Failed to copy:', e);
    }
  }

  function saveSettings() {
    saveToStorage(settings, messages);
  }

  function applyProviderPreset(id: ProviderPresetId) {
    settings = {
      ...settings,
      provider: id
    };

    if (id !== 'custom') {
      const preset = PROVIDER_PRESETS[id];
      settings = {
        ...settings,
        apiBaseUrl: preset.apiBaseUrl,
        model: preset.model,
        systemPrompt: settings.systemPrompt ? settings.systemPrompt : (preset.systemPrompt || '')
      };
    }
  }

  function resetSettings() {
    settings = {
      provider: 'openai',
      apiBaseUrl: PROVIDER_PRESETS.openai.apiBaseUrl,
      apiKey: '',
      model: PROVIDER_PRESETS.openai.model,
      systemPrompt: '',
      maxHistory: 10
    };
    saveToStorage(settings, messages);
  }

  onMount(() => {
    const saved = loadFromStorage();
    if (saved) {
      if (saved.settings) {
        settings = saved.settings;
      }
      if (saved.messages) {
        messages = saved.messages;
      }
    }
    loadApiConfigs();
    // Initialize session ID
    currentSessionId = crypto.randomUUID();
    // Load existing sessions
    chatSessions = loadChatSessions();
  });

  $effect(() => {
    saveToStorage(settings, messages);
  });
</script>

<div class="flex flex-col h-full w-full ml-0 mr-0 p-2 space-y-2">
  <div class="flex flex-col h-full min-h-0">
    <!-- 聊天区域 -->
    <div class="flex-1 flex flex-col min-h-0">
      <div class="card flex-1 flex flex-col min-h-0">
        <div class="flex items-center justify-between mb-3">
          <div class="flex items-center gap-2">
            <MessageSquare class="w-5 h-5 text-primary-500" />
            <h1 class="text-lg font-semibold text-gray-900 dark:text-gray-100">
              {t('aiChat.title')}
            </h1>
          </div>
          <div class="flex items-center gap-2">
            <button
              class="btn-secondary text-xs flex items-center gap-1"
              onclick={openChatHistory}
            >
              <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z" stroke-linecap="round" stroke-linejoin="round"/>
              </svg>
              {t('aiChat.chatHistory')}
            </button>
            <button
              class="btn-secondary text-xs flex items-center gap-1"
              onclick={newChat}
            >
              <Trash2 class="w-3.5 h-3.5" />
              {t('aiChat.newChat')}
            </button>
          </div>
        </div>

        <div class="flex-1 min-h-0 border border-gray-200 dark:border-gray-700 rounded-lg p-3 bg-white dark:bg-gray-900 overflow-y-auto space-y-3">
          {#if messages.length === 0}
            <div class="h-full flex flex-col items-center justify-center text-center text-gray-500 dark:text-gray-400 px-4">
              <MessageSquare class="w-10 h-10 mb-3 text-gray-400 dark:text-gray-500" />
              <p class="text-sm mb-1">{t('aiChat.description')}</p>
              <p class="text-xs text-gray-400 dark:text-gray-500">
                {t('aiChat.settingsHint')}
              </p>
            </div>
          {:else}
            {#each messages as msg (msg.id)}
              <div class="flex gap-3">
                <div class="w-8 flex-shrink-0 flex items-start justify-center">
                  <div class="w-7 h-7 rounded-full flex items-center justify-center text-xs font-semibold
                    {msg.role === 'user'
                      ? 'bg-primary-600 text-white'
                      : msg.role === 'assistant'
                      ? 'bg-emerald-600 text-white'
                      : 'bg-gray-500 text-white'}">
                    {msg.role === 'user'
                      ? t('aiChat.user')
                      : msg.role === 'assistant'
                      ? t('aiChat.assistant')
                      : 'S'}
                  </div>
                </div>
                <div class="flex-1 space-y-1">
                  <div class="flex items-center justify-between gap-2">
                    <span class="text-xs font-medium text-gray-500 dark:text-gray-400">
                      {new Date(msg.createdAt).toLocaleTimeString()}
                    </span>
                  </div>

                  {#if msg.thinking}
                    <div>
                      <button
                        type="button"
                        class="text-[11px] px-1.5 py-0.5 rounded-full border border-gray-200 dark:border-gray-700 text-gray-500 dark:text-gray-400 hover:text-gray-800 dark:hover:text-gray-200 hover:border-gray-400 dark:hover:border-gray-500 bg-gray-50 dark:bg-gray-900/40"
                        onclick={() => {
                          thinkingExpanded = {
                            ...thinkingExpanded,
                            [msg.id]: !thinkingExpanded[msg.id]
                          };
                        }}
                      >
                        {thinkingExpanded[msg.id]
                          ? t('aiChat.hideThinking')
                          : t('aiChat.showThinking')}
                      </button>
                    </div>

                    {#if thinkingExpanded[msg.id]}
                      <div class="mt-1 rounded-md bg-gray-50 dark:bg-gray-900/60 border border-gray-200 dark:border-gray-700 px-2 py-1.5 max-h-40 overflow-auto">
                        <p class="text-[11px] text-gray-400 dark:text-gray-500">
                          {t('aiChat.thinkingDisclaimer')}
                        </p>
                        <pre
                          class="mt-1 text-[11px] text-gray-600 dark:text-gray-300 whitespace-pre-wrap break-words font-mono leading-snug"
                        >{msg.thinking}</pre>
                      </div>
                    {/if}
                  {/if}

                  {#if msg.role === 'user'}
                    <div class="text-sm text-gray-900 dark:text-gray-100 whitespace-pre-wrap break-words">
                      {msg.content}
                    </div>
                  {:else}
                    <div class="prose prose-sm max-w-none break-words 
                      prose-headings:font-semibold prose-headings:mt-2 prose-headings:mb-1
                      prose-p:my-1 prose-p:leading-relaxed
                      prose-ul:my-1 prose-ol:my-1 prose-li:my-0.5
                      prose-code:bg-gray-100 dark:prose-code:bg-gray-800 prose-code:px-1 prose-code:py-0.5 prose-code:rounded prose-code:text-pink-600 dark:prose-code:text-pink-400 prose-code:before:content-none prose-code:after:content-none
                      prose-pre:bg-gray-100 dark:prose-pre:bg-gray-800 prose-pre:border prose-pre:border-gray-200 dark:prose-pre:border-gray-700 prose-pre:rounded-lg prose-pre:p-3
                      prose-blockquote:border-l-4 prose-blockquote:border-primary-500 prose-blockquote:bg-gray-50 dark:prose-blockquote:bg-gray-800/50 prose-blockquote:py-1 prose-blockquote:px-3 prose-blockquote:not-italic
                      prose-a:text-primary-600 dark:prose-a:text-primary-400 prose-a:no-underline hover:prose-a:underline
                      prose-hr:border-gray-200 dark:prose-hr:border-gray-700
                      prose-table:border prose-table:border-gray-200 dark:prose-table:border-gray-700 prose-th:border-b prose-th:border-gray-200 dark:prose-th:border-gray-700 prose-th:bg-gray-50 dark:prose-th:bg-gray-800 prose-td:border-b prose-td:border-gray-100 dark:prose-td:border-gray-800">
                      {@html renderMarkdown(msg.content)}
                    </div>
                  {/if}

                  <div class="flex items-center gap-2 pt-1">
                    <button
                      class="text-xs text-gray-400 hover:text-gray-700 dark:hover:text-gray-200 flex items-center gap-1"
                      onclick={() => copyMessage(msg.content)}
                      title={t('common.copy')}
                    >
                      {#if copied && copyMode === 'text'}
                        <Check class="w-3.5 h-3.5" />
                      {:else}
                        <Copy class="w-3.5 h-3.5" />
                      {/if}
                    </button>
                    {#if msg.role === 'assistant'}
                      <button
                        class="text-xs text-gray-400 hover:text-gray-700 dark:hover:text-gray-200 flex items-center gap-1"
                        onclick={() => copyMessage(msg.content, 'markdown')}
                        title={t('aiChat.copyMarkdown')}
                      >
                        {#if copied && copyMode === 'markdown'}
                          <Check class="w-3.5 h-3.5" />
                        {:else}
                          <FileCode class="w-3.5 h-3.5" />
                        {/if}
                      </button>
                    {/if}
                  </div>
                </div>
              </div>
            {/each}
          {/if}
        </div>

        {#if error}
          <div class="mt-2 px-3 py-2 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg">
            <p class="text-xs text-red-800 dark:text-red-200 break-words">{error}</p>
          </div>
        {/if}

        <!-- 输入区域 -->
        <div class="sticky bottom-0 bg-gradient-to-t from-gray-50 dark:from-gray-900 via-gray-50 dark:via-gray-900 to-transparent pt-4 pb-3 px-2">
          <div class="bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-2xl p-4 shadow-sm">
            <!-- Attached Files Display -->
            {#if attachedFiles.length > 0}
              <div class="flex flex-wrap gap-2 mb-3">
                {#each attachedFiles as file, index}
                  <div class="flex items-center gap-1 px-2 py-1 bg-gray-100 dark:bg-gray-700 rounded-lg text-xs text-gray-700 dark:text-gray-300">
                    {#if file.type.startsWith('image/')}
                      <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                        <rect x="3" y="3" width="18" height="18" rx="2" ry="2"/>
                        <circle cx="8.5" cy="8.5" r="1.5"/>
                        <path d="M21 15l-5-5L5 21"/>
                      </svg>
                    {:else}
                      <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                        <path d="M14 2H6a2 2 0 00-2 2v16a2 2 0 002 2h12a2 2 0 002-2V8z"/>
                        <polyline points="14 2 14 8 20 8"/>
                      </svg>
                    {/if}
                    <span class="truncate max-w-[120px]">{file.name}</span>
                    <button
                      type="button"
                      class="ml-1 text-gray-500 hover:text-red-500"
                      onclick={() => removeAttachedFile(index)}
                      title={t('aiChat.removeFile')}
                    >
                      <svg class="w-3 h-3" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                        <path d="M18 6L6 18M6 6l12 12" stroke-linecap="round" stroke-linejoin="round"/>
                      </svg>
                    </button>
                  </div>
                {/each}
              </div>
            {/if}
            <textarea
              bind:value={input}
              rows={1}
              class="w-full bg-transparent border-none outline-none resize-none text-sm min-h-[24px] max-h-40 py-1 px-0 text-gray-900 dark:text-gray-100 placeholder-gray-400"
              placeholder={t('aiChat.inputPlaceholder')}
              onkeydown={handleKeyDown}
              oninput={adjustTextareaHeight}
            ></textarea>
            <div class="flex items-center justify-between mt-3">
                <div class="flex items-center gap-2">
                  <!-- Model Selector Button -->
                <div class="relative model-dropdown-container">
                  <button
                    type="button"
                    class="flex items-center gap-1.5 px-2 py-1 text-xs text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-gray-200 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg transition-colors"
                    onclick={() => showModelDropdown = !showModelDropdown}
                  >
                    <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                      <rect x="3" y="3" width="18" height="18" rx="2" ry="2"/>
                      <line x1="3" y1="9" x2="21" y2="9"/>
                      <line x1="9" y1="21" x2="9" y2="9"/>
                    </svg>
                    <span class="truncate max-w-[200px]">{apiConfigs.length > 0 ? getModelDisplayWithProvider(settings.model) : t('aiChat.pleaseConfigModel')}</span>
                    <svg class="w-3 h-3" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                      <path d="M6 9l6 6 6-6" stroke-linecap="round" stroke-linejoin="round"/>
                    </svg>
                  </button>
                  
                  {#if showModelDropdown}
                    <div class="absolute bottom-full left-0 mb-2 w-80 bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-xl shadow-lg max-h-96 overflow-y-auto">
                      <!-- Search and Actions -->
                      <div class="p-2 border-b border-gray-100 dark:border-gray-700">
                        <div class="flex items-center gap-2">
                          <div class="relative flex-1">
                            <svg class="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-gray-400" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                              <circle cx="11" cy="11" r="8" stroke-linecap="round" stroke-linejoin="round"/>
                              <path d="M21 21l-4.35-4.35" stroke-linecap="round" stroke-linejoin="round"/>
                            </svg>
                            <input
                              type="text"
                              bind:value={modelSearchQuery}
                              placeholder={t('aiChat.searchModels')}
                              class="w-full pl-9 pr-3 py-1.5 bg-gray-100 dark:bg-gray-700 border-0 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-gray-400 dark:focus:ring-gray-500"
                            />
                          </div>
                          <!-- Add Config Button -->
                          <button
                            type="button"
                            class="p-1.5 text-gray-500 hover:text-gray-700 dark:hover:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg transition-colors"
                            onclick={() => { showModelDropdown = false; showAddConfigModal = true; }}
                            title={t('aiChat.addApiConfigTitle')}
                          >
                            <svg class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                              <path d="M12 5v14M5 12h14" stroke-linecap="round" stroke-linejoin="round"/>
                            </svg>
                          </button>
                          <!-- Manage Configs Button -->
                          <button
                            type="button"
                            class="p-1.5 text-gray-500 hover:text-gray-700 dark:hover:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg transition-colors"
                            onclick={() => { showModelDropdown = false; showManageConfigsModal = true; }}
                            title={t('aiChat.manageApiConfigsTitle')}
                          >
                            <svg class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                              <circle cx="18" cy="6" r="2" fill="currentColor"/>
                              <path d="M4 6h12" stroke-linecap="round" stroke-linejoin="round"/>
                              <circle cx="6" cy="18" r="2" fill="currentColor"/>
                              <path d="M10 18h10" stroke-linecap="round" stroke-linejoin="round"/>
                            </svg>
                          </button>
                        </div>
                      </div>
                      
                      <!-- Model Groups -->
                      {#if Object.keys(availableModelGroups()).length === 0}
                        <div class="p-4 text-center text-sm text-gray-500 dark:text-gray-400">
                          {t('aiChat.noApiConfigsAvailable')}
                          <button
                            type="button"
                            class="text-primary-500 hover:underline ml-1"
                            onclick={() => { showModelDropdown = false; showAddConfigModal = true; }}
                          >
                            {t('aiChat.addOne')}
                          </button>
                        </div>
                      {:else}
                        {#each Object.entries(availableModelGroups()) as [provider, models]}
                          {#if models.length > 0}
                            <div class="py-1">
                              <!-- Provider Level -->
                              <div class="px-3 py-1.5 text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">
                                {provider}
                              </div>
                              <!-- Models Level (indented) -->
                              <div class="pl-2">
                                {#each models as model}
                                  <button
                                    type="button"
                                    class="w-full px-3 py-2 text-left text-sm hover:bg-gray-100 dark:hover:bg-gray-700 flex items-center justify-between {settings.model === model.id ? 'bg-gray-50 dark:bg-gray-700/50' : ''}"
                                    onclick={() => selectModel(model.id, model.configId)}
                                  >
                                    <span class="truncate">{model.name}</span>
                                    {#if model.tag}
                                      <span class="ml-2 px-1.5 py-0.5 text-[10px] bg-gray-200 dark:bg-gray-600 text-gray-600 dark:text-gray-300 rounded flex-shrink-0">
                                        {model.tag}
                                      </span>
                                    {/if}
                                    {#if settings.model === model.id}
                                      <svg class="w-4 h-4 text-gray-900 dark:text-gray-100 ml-2 flex-shrink-0" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                                        <path d="M5 12l5 5L20 7" stroke-linecap="round" stroke-linejoin="round"/>
                                      </svg>
                                    {/if}
                                  </button>
                                {/each}
                              </div>
                            </div>
                          {/if}
                        {/each}
                      {/if}
                    </div>
                  {/if}
                </div>
              </div>
              <div class="flex items-center gap-2">
                <!-- File Upload Button -->
                <input
                  type="file"
                  bind:this={fileInputRef}
                  onchange={handleFileSelect}
                  multiple
                  class="hidden"
                  id="file-upload"
                />
                <label
                  for="file-upload"
                  class="flex items-center justify-center h-8 w-8 text-gray-500 hover:text-gray-700 dark:hover:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg cursor-pointer transition-colors"
                  title={t('aiChat.attachFile')}
                >
                  <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <path d="M21.44 11.05l-9.19 9.19a6 6 0 01-8.49-8.49l9.19-9.19a4 4 0 015.66 5.66l-9.2 9.19a2 2 0 01-2.83-2.83l8.49-8.48"/>
                  </svg>
                </label>
                {#if isSending}
                  <button
                    class="bg-red-500 hover:bg-red-600 dark:bg-red-600 dark:hover:bg-red-500 text-white flex items-center justify-center h-10 w-10 flex-shrink-0 rounded-xl shadow-sm transition-colors"
                    onclick={stopGeneration}
                    title={t('aiChat.stop')}
                  >
                    <svg class="w-5 h-5" fill="currentColor" viewBox="0 0 24 24">
                      <rect x="6" y="6" width="12" height="12" rx="2" />
                    </svg>
                  </button>
                {:else}
                  <button
                    class="bg-gray-900 dark:bg-gray-100 hover:bg-gray-700 dark:hover:bg-gray-300 text-white dark:text-gray-900 flex items-center justify-center h-8 w-8 flex-shrink-0 rounded-lg shadow-sm transition-colors disabled:opacity-40 disabled:cursor-not-allowed"
                    onclick={sendMessage}
                    disabled={!input.trim() && attachedFiles.length === 0}
                    title={t('aiChat.send')}
                  >
                    <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                      <path d="M12 19V5M5 12l7-7 7 7" stroke-linecap="round" stroke-linejoin="round"/>
                    </svg>
                  </button>
                {/if}
              </div>
            </div>
          </div>
          <p class="mt-2 text-xs text-gray-400 dark:text-gray-500 text-center">
            {t('aiChat.settingsHint')}
          </p>
        </div>
      </div>
    </div>
  </div>
</div>

<!-- Add Config Modal -->
{#if showAddConfigModal}
  <div class="fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4">
    <div class="bg-white dark:bg-gray-800 rounded-xl shadow-xl w-full max-w-md max-h-[90vh] overflow-y-auto">
      <div class="p-4 border-b border-gray-200 dark:border-gray-700 flex items-center justify-between">
        <h3 class="text-lg font-semibold text-gray-900 dark:text-gray-100">{t('aiChat.addApiConfig')}</h3>
        <button
          type="button"
          class="text-gray-400 hover:text-gray-600 dark:hover:text-gray-300"
          onclick={() => showAddConfigModal = false}
        >
          <svg class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M18 6L6 18M6 6l12 12" stroke-linecap="round" stroke-linejoin="round"/>
          </svg>
        </button>
      </div>
      <div class="p-4 space-y-4">
        <div>
          <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">{t('aiChat.providerPreset')}</label>
          <select
            bind:value={newConfig.provider}
            class="w-full px-3 py-2 bg-white dark:bg-gray-800 border border-gray-300 dark:border-gray-600 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-primary-500 appearance-none cursor-pointer"
            style="background-image: url('data:image/svg+xml;charset=UTF-8,%3Csvg xmlns=%27http://www.w3.org/2000/svg%27 viewBox=%270 0 24 24%27 fill=%27none%27 stroke=%27currentColor%27 stroke-width=%272%27%3E%3Cpath d=%27M6 9l6 6 6-6%27 stroke-linecap=%27round%27 stroke-linejoin=%27round%27/%3E%3C/svg%3E'); background-repeat: no-repeat; background-position: right 0.5rem center; background-size: 1rem; padding-right: 2rem;"
          >
            <option value="openai">OpenAI</option>
            <option value="deepseek">DeepSeek</option>
            <option value="anthropic">Anthropic</option>
            <option value="openrouter">OpenRouter</option>
            <option value="groq">Groq</option>
            <option value="mistral">Mistral</option>
            <option value="zhipu">Zhipu</option>
            <option value="minimax">MiniMax</option>
            <option value="kimi">Kimi</option>
            <option value="dashscope">DashScope</option>
            <option value="azure-openai">Azure OpenAI</option>
            <option value="custom">{t('aiChat.providerCustom')}</option>
          </select>
        </div>
        {#if newConfig.provider === 'custom'}
          <div>
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">{t('aiChat.configName')} <span class="text-red-500">*</span></label>
            <input
              type="text"
              bind:value={newConfig.name}
              placeholder={t('aiChat.configNamePlaceholder')}
              class="w-full px-3 py-2 bg-white dark:bg-gray-800 border border-gray-300 dark:border-gray-600 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-primary-500"
            />
          </div>
        {/if}
        <div>
          <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">API Base URL</label>
          <input
            type="text"
            bind:value={newConfig.apiBaseUrl}
            placeholder="https://api.openai.com/v1"
            class="w-full px-3 py-2 bg-white dark:bg-gray-800 border border-gray-300 dark:border-gray-600 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-primary-500"
          />
        </div>
        <div>
          <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">{t('aiChat.apiKey')}</label>
          <div class="relative">
            <input
              type={showApiKeyInAddModal ? 'text' : 'password'}
              bind:value={newConfig.apiKey}
              placeholder={t('aiChat.apiKeyPlaceholder')}
              class="w-full px-3 py-2 pr-10 bg-white dark:bg-gray-800 border border-gray-300 dark:border-gray-600 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-primary-500"
            />
            <button
              type="button"
              class="absolute right-2 top-1/2 -translate-y-1/2 text-gray-500 hover:text-gray-700 dark:hover:text-gray-300"
              onclick={() => showApiKeyInAddModal = !showApiKeyInAddModal}
              title={showApiKeyInAddModal ? t('aiChat.hideApiKey') : t('aiChat.showApiKey')}
            >
              {#if showApiKeyInAddModal}
                <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <path d="M17.94 17.94A10.07 10.07 0 0 1 12 20c-7 0-11-8-11-8a18.45 18.45 0 0 1 5.06-5.94M9.9 4.24A9.12 9.12 0 0 1 12 4c7 0 11 8 11 8a18.5 18.5 0 0 1-2.16 3.19m-6.72-1.07a3 3 0 1 1-4.24-4.24" stroke-linecap="round" stroke-linejoin="round"/>
                  <line x1="1" y1="1" x2="23" y2="23" stroke-linecap="round" stroke-linejoin="round"/>
                </svg>
              {:else}
                <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z" stroke-linecap="round" stroke-linejoin="round"/>
                  <circle cx="12" cy="12" r="3" stroke-linecap="round" stroke-linejoin="round"/>
                </svg>
              {/if}
            </button>
          </div>
        </div>
        <div>
          <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">{t('aiChat.model')}</label>
          <div class="flex gap-2">
            <input
              type="text"
              bind:value={newConfig.currentModel}
              placeholder={getModelPlaceholder()}
              class="flex-1 px-3 py-2 bg-white dark:bg-gray-800 border border-gray-300 dark:border-gray-600 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-primary-500"
              onkeydown={(e) => {
                if (e.key === 'Enter' && newConfig.currentModel.trim()) {
                  e.preventDefault();
                  newConfig.models = [...newConfig.models, { id: newConfig.currentModel.trim(), name: newConfig.currentModel.trim() }];
                  newConfig.currentModel = '';
                }
              }}
            />
            <button
              type="button"
              class="px-3 py-2 bg-primary-500 text-white rounded-lg hover:bg-primary-600 transition-colors disabled:opacity-50"
              disabled={!newConfig.currentModel.trim()}
              onclick={() => {
                if (newConfig.currentModel.trim()) {
                  newConfig.models = [...newConfig.models, { id: newConfig.currentModel.trim(), name: newConfig.currentModel.trim() }];
                  newConfig.currentModel = '';
                }
              }}
            >
              {t('aiChat.add')}
            </button>
          </div>
          {#if newConfig.models.length > 0}
            <div class="mt-2 flex flex-wrap gap-2">
              {#each newConfig.models as model, index}
                <span class="inline-flex items-center gap-1 px-2 py-1 bg-primary-100 dark:bg-primary-900/30 text-primary-700 dark:text-primary-300 rounded text-sm">
                  {model.name}
                  <button
                    type="button"
                    class="text-primary-500 hover:text-primary-700"
                    onclick={() => {
                      newConfig.models = newConfig.models.filter((_, i) => i !== index);
                    }}
                  >
                    <svg class="w-3 h-3" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                      <path d="M18 6L6 18M6 6l12 12" stroke-linecap="round" stroke-linejoin="round"/>
                    </svg>
                  </button>
                </span>
              {/each}
            </div>
          {/if}
        </div>
      </div>
      <div class="p-4 border-t border-gray-200 dark:border-gray-700 flex gap-2">
        <button
          type="button"
          class="flex-1 px-4 py-2 bg-gray-100 dark:bg-gray-700 text-gray-700 dark:text-gray-300 rounded-lg hover:bg-gray-200 dark:hover:bg-gray-600 transition-colors"
          onclick={() => showAddConfigModal = false}
        >
          {t('aiChat.cancel')}
        </button>
        <button
          type="button"
          class="flex-1 px-4 py-2 bg-primary-500 text-white rounded-lg hover:bg-primary-600 transition-colors disabled:opacity-50"
          onclick={addApiConfig}
          disabled={!newConfig.provider || !newConfig.apiBaseUrl || !newConfig.apiKey || newConfig.models.length === 0 || (newConfig.provider === 'custom' && !newConfig.name.trim())}
        >
          {t('aiChat.add')}
        </button>
      </div>
    </div>
  </div>
{/if}

<!-- Edit Config Modal -->
{#if showEditConfigModal}
  <div class="fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4">
    <div class="bg-white dark:bg-gray-800 rounded-xl shadow-xl w-full max-w-md max-h-[90vh] overflow-y-auto">
      <div class="p-4 border-b border-gray-200 dark:border-gray-700 flex items-center justify-between">
        <h3 class="text-lg font-semibold text-gray-900 dark:text-gray-100">{t('aiChat.editApiConfig')}</h3>
        <button
          type="button"
          class="text-gray-400 hover:text-gray-600 dark:hover:text-gray-300"
          onclick={() => { showEditConfigModal = false; editingConfigId = ''; newConfig = { name: '', provider: 'openai', apiBaseUrl: '', apiKey: '', models: [], currentModel: '' }; }}
        >
          <svg class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M18 6L6 18M6 6l12 12" stroke-linecap="round" stroke-linejoin="round"/>
          </svg>
        </button>
      </div>
      <div class="p-4 space-y-4">
        <div>
          <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">{t('aiChat.providerPreset')}</label>
          <select
            bind:value={newConfig.provider}
            class="w-full px-3 py-2 bg-white dark:bg-gray-800 border border-gray-300 dark:border-gray-600 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-primary-500 appearance-none cursor-pointer"
            style="background-image: url('data:image/svg+xml;charset=UTF-8,%3Csvg xmlns=%27http://www.w3.org/2000/svg%27 viewBox=%270 0 24 24%27 fill=%27none%27 stroke=%27currentColor%27 stroke-width=%272%27%3E%3Cpath d=%27M6 9l6 6 6-6%27 stroke-linecap=%27round%27 stroke-linejoin=%27round%27/%3E%3C/svg%3E'); background-repeat: no-repeat; background-position: right 0.5rem center; background-size: 1rem; padding-right: 2rem;"
          >
            <option value="openai">OpenAI</option>
            <option value="deepseek">DeepSeek</option>
            <option value="anthropic">Anthropic</option>
            <option value="openrouter">OpenRouter</option>
            <option value="groq">Groq</option>
            <option value="mistral">Mistral</option>
            <option value="zhipu">Zhipu</option>
            <option value="minimax">MiniMax</option>
            <option value="kimi">Kimi</option>
            <option value="dashscope">DashScope</option>
            <option value="azure-openai">Azure OpenAI</option>
            <option value="custom">{t('aiChat.providerCustom')}</option>
          </select>
        </div>
        {#if newConfig.provider === 'custom'}
          <div>
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">{t('aiChat.configName')} <span class="text-red-500">*</span></label>
            <input
              type="text"
              bind:value={newConfig.name}
              placeholder={t('aiChat.configNamePlaceholder')}
              class="w-full px-3 py-2 bg-white dark:bg-gray-800 border border-gray-300 dark:border-gray-600 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-primary-500"
            />
          </div>
        {/if}
        <div>
          <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">API Base URL</label>
          <input
            type="text"
            bind:value={newConfig.apiBaseUrl}
            placeholder="https://api.openai.com/v1"
            class="w-full px-3 py-2 bg-white dark:bg-gray-800 border border-gray-300 dark:border-gray-600 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-primary-500"
          />
        </div>
        <div>
          <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">{t('aiChat.apiKey')}</label>
          <div class="relative">
            <input
              type={showApiKeyInEditModal ? 'text' : 'password'}
              bind:value={newConfig.apiKey}
              placeholder={t('aiChat.apiKeyPlaceholder')}
              class="w-full px-3 py-2 pr-10 bg-white dark:bg-gray-800 border border-gray-300 dark:border-gray-600 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-primary-500"
            />
            <button
              type="button"
              class="absolute right-2 top-1/2 -translate-y-1/2 text-gray-500 hover:text-gray-700 dark:hover:text-gray-300"
              onclick={() => showApiKeyInEditModal = !showApiKeyInEditModal}
              title={showApiKeyInEditModal ? t('aiChat.hideApiKey') : t('aiChat.showApiKey')}
            >
              {#if showApiKeyInEditModal}
                <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <path d="M17.94 17.94A10.07 10.07 0 0 1 12 20c-7 0-11-8-11-8a18.45 18.45 0 0 1 5.06-5.94M9.9 4.24A9.12 9.12 0 0 1 12 4c7 0 11 8 11 8a18.5 18.5 0 0 1-2.16 3.19m-6.72-1.07a3 3 0 1 1-4.24-4.24" stroke-linecap="round" stroke-linejoin="round"/>
                  <line x1="1" y1="1" x2="23" y2="23" stroke-linecap="round" stroke-linejoin="round"/>
                </svg>
              {:else}
                <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z" stroke-linecap="round" stroke-linejoin="round"/>
                  <circle cx="12" cy="12" r="3" stroke-linecap="round" stroke-linejoin="round"/>
                </svg>
              {/if}
            </button>
          </div>
        </div>
        <div>
          <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">{t('aiChat.model')}</label>
          <div class="flex gap-2">
            <input
              type="text"
              bind:value={newConfig.currentModel}
              placeholder={getModelPlaceholder()}
              class="flex-1 px-3 py-2 bg-white dark:bg-gray-800 border border-gray-300 dark:border-gray-600 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-primary-500"
              onkeydown={(e) => {
                if (e.key === 'Enter' && newConfig.currentModel.trim()) {
                  e.preventDefault();
                  newConfig.models = [...newConfig.models, { id: newConfig.currentModel.trim(), name: newConfig.currentModel.trim() }];
                  newConfig.currentModel = '';
                }
              }}
            />
            <button
              type="button"
              class="px-3 py-2 bg-primary-500 text-white rounded-lg hover:bg-primary-600 transition-colors disabled:opacity-50"
              disabled={!newConfig.currentModel.trim()}
              onclick={() => {
                if (newConfig.currentModel.trim()) {
                  newConfig.models = [...newConfig.models, { id: newConfig.currentModel.trim(), name: newConfig.currentModel.trim() }];
                  newConfig.currentModel = '';
                }
              }}
            >
              {t('aiChat.add')}
            </button>
          </div>
          {#if newConfig.models.length > 0}
            <div class="mt-2 flex flex-wrap gap-2">
              {#each newConfig.models as model, index}
                <span class="inline-flex items-center gap-1 px-2 py-1 bg-primary-100 dark:bg-primary-900/30 text-primary-700 dark:text-primary-300 rounded text-sm">
                  {model.name}
                  <button
                    type="button"
                    class="text-primary-500 hover:text-primary-700"
                    onclick={() => {
                      newConfig.models = newConfig.models.filter((_, i) => i !== index);
                    }}
                  >
                    <svg class="w-3 h-3" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                      <path d="M18 6L6 18M6 6l12 12" stroke-linecap="round" stroke-linejoin="round"/>
                    </svg>
                  </button>
                </span>
              {/each}
            </div>
          {/if}
        </div>
      </div>
      <div class="p-4 border-t border-gray-200 dark:border-gray-700 flex gap-2">
        <button
          type="button"
          class="flex-1 px-4 py-2 bg-gray-100 dark:bg-gray-700 text-gray-700 dark:text-gray-300 rounded-lg hover:bg-gray-200 dark:hover:bg-gray-600 transition-colors"
          onclick={() => { showEditConfigModal = false; editingConfigId = ''; newConfig = { name: '', provider: 'openai', apiBaseUrl: '', apiKey: '', models: [], currentModel: '' }; }}
        >
          {t('aiChat.cancel')}
        </button>
        <button
          type="button"
          class="flex-1 px-4 py-2 bg-primary-500 text-white rounded-lg hover:bg-primary-600 transition-colors disabled:opacity-50"
          onclick={saveEditConfig}
          disabled={!newConfig.provider || !newConfig.apiBaseUrl || !newConfig.apiKey || newConfig.models.length === 0 || (newConfig.provider === 'custom' && !newConfig.name.trim())}
        >
          {t('aiChat.save')}
        </button>
      </div>
    </div>
  </div>
{/if}

<!-- Chat History Modal -->
{#if showChatHistoryModal}
  <div class="fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4">
    <div class="bg-white dark:bg-gray-800 rounded-xl shadow-xl w-full max-w-md max-h-[90vh] overflow-y-auto">
      <div class="p-4 border-b border-gray-200 dark:border-gray-700 flex items-center justify-between">
        <h3 class="text-lg font-semibold text-gray-900 dark:text-gray-100">{t('aiChat.chatHistory')}</h3>
        <button
          type="button"
          class="text-gray-400 hover:text-gray-600 dark:hover:text-gray-300"
          onclick={() => showChatHistoryModal = false}
        >
          <svg class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M18 6L6 18M6 6l12 12" stroke-linecap="round" stroke-linejoin="round"/>
          </svg>
        </button>
      </div>
      <div class="p-4">
        {#if chatSessions.length === 0}
          <div class="text-center py-8 text-gray-500 dark:text-gray-400">
            <p>{t('aiChat.noChatHistory')}</p>
          </div>
        {:else}
          <div class="space-y-2">
            {#each chatSessions as session}
              <div class="flex items-center justify-between p-3 bg-gray-50 dark:bg-gray-700/50 rounded-lg">
                <div class="flex-1 min-w-0 cursor-pointer" onclick={() => loadSession(session)}>
                  <div class="font-medium text-sm text-gray-900 dark:text-gray-100 truncate">{session.title}</div>
                  <div class="text-xs text-gray-500 dark:text-gray-400">
                    {new Date(session.updatedAt).toLocaleString()} • {session.messages.length} messages
                  </div>
                </div>
                <div class="flex items-center gap-1 ml-2">
                  {#if currentSessionId === session.id}
                    <span class="px-2 py-1 text-xs bg-primary-100 dark:bg-primary-900/30 text-primary-700 dark:text-primary-300 rounded">{t('aiChat.active')}</span>
                  {:else}
                    <button
                      type="button"
                      class="p-1.5 text-gray-500 hover:text-primary-500 hover:bg-gray-100 dark:hover:bg-gray-700 rounded transition-colors"
                      onclick={() => loadSession(session)}
                      title={t('aiChat.switchSession')}
                    >
                      <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                        <path d="M5 12h14M12 5l7 7-7 7" stroke-linecap="round" stroke-linejoin="round"/>
                      </svg>
                    </button>
                  {/if}
                  <button
                    type="button"
                    class="p-1.5 text-gray-500 hover:text-red-500 hover:bg-gray-100 dark:hover:bg-gray-700 rounded transition-colors"
                    onclick={() => deleteSession(session.id)}
                    title={t('aiChat.deleteSession')}
                  >
                    <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                      <path d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" stroke-linecap="round" stroke-linejoin="round"/>
                    </svg>
                  </button>
                </div>
              </div>
            {/each}
          </div>
        {/if}
      </div>
    </div>
  </div>
{/if}

<!-- Manage Configs Modal -->
{#if showManageConfigsModal}
  <div class="fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4">
    <div class="bg-white dark:bg-gray-800 rounded-xl shadow-xl w-full max-w-md max-h-[90vh] overflow-y-auto">
      <div class="p-4 border-b border-gray-200 dark:border-gray-700 flex items-center justify-between">
        <h3 class="text-lg font-semibold text-gray-900 dark:text-gray-100">{t('aiChat.manageApiConfigs')}</h3>
        <button
          type="button"
          class="text-gray-400 hover:text-gray-600 dark:hover:text-gray-300"
          onclick={() => showManageConfigsModal = false}
        >
          <svg class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M18 6L6 18M6 6l12 12" stroke-linecap="round" stroke-linejoin="round"/>
          </svg>
        </button>
      </div>
      <div class="p-4">
        {#if apiConfigs.length === 0}
          <div class="text-center py-8 text-gray-500 dark:text-gray-400">
            <p>{t('aiChat.noApiConfigs')}</p>
            <button
              type="button"
              class="mt-2 text-primary-500 hover:underline"
              onclick={() => { showManageConfigsModal = false; showAddConfigModal = true; }}
            >
              {t('aiChat.addFirstConfig')}
            </button>
          </div>
        {:else}
          <div class="space-y-2">
            {#each apiConfigs as config}
              <div class="flex items-center justify-between p-3 bg-gray-50 dark:bg-gray-700/50 rounded-lg">
                <div class="flex-1 min-w-0">
                  <div class="font-medium text-sm text-gray-900 dark:text-gray-100 truncate">{config.name}</div>
                  <div class="text-xs text-gray-500 dark:text-gray-400">{config.provider} • {config.models.map(m => m.name).join(', ')}</div>
                </div>
                <div class="flex items-center gap-1 ml-2">
                  {#if activeConfigId === config.id}
                    <span class="px-2 py-1 text-xs bg-primary-100 dark:bg-primary-900/30 text-primary-700 dark:text-primary-300 rounded">{t('aiChat.active')}</span>
                  {:else}
                    <button
                      type="button"
                      class="p-1.5 text-gray-500 hover:text-primary-500 hover:bg-gray-100 dark:hover:bg-gray-700 rounded transition-colors"
                      onclick={() => switchToConfig(config)}
                      title={t('aiChat.switchConfig')}
                    >
                      <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                        <path d="M5 12h14M12 5l7 7-7 7" stroke-linecap="round" stroke-linejoin="round"/>
                      </svg>
                    </button>
                  {/if}
                  <button
                    type="button"
                    class="p-1.5 text-gray-500 hover:text-blue-500 hover:bg-gray-100 dark:hover:bg-gray-700 rounded transition-colors"
                    onclick={() => startEditConfig(config)}
                    title={t('aiChat.editConfig')}
                  >
                    <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                      <path d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" stroke-linecap="round" stroke-linejoin="round"/>
                    </svg>
                  </button>
                  <button
                    type="button"
                    class="p-1.5 text-gray-500 hover:text-red-500 hover:bg-gray-100 dark:hover:bg-gray-700 rounded transition-colors"
                    onclick={() => deleteApiConfig(config.id)}
                    title={t('aiChat.deleteConfig')}
                  >
                    <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                      <path d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" stroke-linecap="round" stroke-linejoin="round"/>
                    </svg>
                  </button>
                </div>
              </div>
            {/each}
          </div>
        {/if}
      </div>
      <div class="p-4 border-t border-gray-200 dark:border-gray-700">
        <button
          type="button"
          class="w-full px-4 py-2 bg-primary-500 text-white rounded-lg hover:bg-primary-600 transition-colors"
          onclick={() => { showManageConfigsModal = false; showAddConfigModal = true; }}
        >
          {t('aiChat.addNewConfig')}
        </button>
      </div>
    </div>
  </div>
{/if}
