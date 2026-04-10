export interface ChatMessage {
  id: string;
  role: 'user' | 'assistant' | 'system';
  content: string;
  createdAt: number;
  thinking?: string;
}

export type ApiProtocol = 'openai-chat' | 'openai-responses' | 'anthropic' | 'gemini' | 'gemini-openai';

export type ProviderPresetId =
  | 'custom'
  | 'openai'
  | 'deepseek'
  | 'azure-openai'
  | 'openrouter'
  | 'anthropic'
  | 'gemini'
  | 'groq'
  | 'mistral'
  | 'zhipu'
  | 'minimax'
  | 'kimi'
  | 'dashscope';

export interface ChatSettings {
  provider: ProviderPresetId;
  apiBaseUrl: string;
  apiKey: string;
  model: string;
  systemPrompt: string;
  maxHistory: number;
}

export interface ChatSession {
  id: string;
  title: string;
  messages: ChatMessage[];
  createdAt: number;
  updatedAt: number;
}

export const PROVIDER_PRESETS: Record<
  Exclude<ProviderPresetId, 'custom'>,
  { apiBaseUrl: string; model: string; systemPrompt?: string }
> = {
  openai: {
    apiBaseUrl: 'https://api.openai.com/v1',
    model: 'gpt-5.4',
    systemPrompt:
      'You are a helpful assistant for software developers. Prefer accurate, concise answers using GPT-5.4.'
  },
  deepseek: {
    apiBaseUrl: 'https://api.deepseek.com',
    model: 'deepseek-v3.2',
    systemPrompt: '你是一个擅长中文的开发助手，回答要简洁准确。'
  },
  'azure-openai': {
    apiBaseUrl: 'https://{your-resource-name}.openai.azure.com',
    model: 'gpt-4o-mini',
    systemPrompt:
      'You are an Azure OpenAI assistant. The user is calling you from a desktop developer toolbox.'
  },
  openrouter: {
    apiBaseUrl: 'https://openrouter.ai/api/v1',
    model: 'openai/gpt-4o-mini',
    systemPrompt:
      'You are an assistant accessed via OpenRouter inside a local developer toolbox app.'
  },
  anthropic: {
    apiBaseUrl: 'https://api.anthropic.com',
    model: 'claude-sonnet-4.6',
    systemPrompt:
      'You are Claude integrated into a desktop utility app. Focus on clear, concise answers.'
  },
  gemini: {
    apiBaseUrl: 'https://generativelanguage.googleapis.com',
    model: 'gemini-2.5-flash',
    systemPrompt:
      'You are Gemini integrated into a desktop utility app. Provide helpful, accurate answers.'
  },
  groq: {
    apiBaseUrl: 'https://api.groq.com/openai/v1',
    model: 'llama-3.3-70b-versatile',
    systemPrompt:
      'You are a fast low-latency assistant running via Groq. Prefer brief answers.'
  },
  mistral: {
    apiBaseUrl: 'https://api.mistral.ai',
    model: 'mistral-large-2512',
    systemPrompt:
      'You are a helpful assistant using Mistral Large 3. Answer in a developer-friendly way.'
  },
  zhipu: {
    apiBaseUrl: 'https://open.bigmodel.cn/api/paas/v4',
    model: 'glm-5',
    systemPrompt:
      '你是智谱 GLM-5 接入的助手，回答要兼顾中文体验和代码示例，对国内开发者友好。'
  },
  minimax: {
    apiBaseUrl: 'https://api.minimax.chat/v1',
    model: 'MiniMax-M2.5',
    systemPrompt:
      'You are an assistant using MiniMax M2.5, optimized for concise and practical answers.'
  },
  kimi: {
    apiBaseUrl: 'https://api.moonshot.cn/v1',
    model: 'kimi-k2.5',
    systemPrompt:
      '你是 Kimi（月之暗面）最新 K2.5 模型助手，擅长长文本理解和中文场景，回答清晰、有条理。'
  },
  dashscope: {
    apiBaseUrl: 'https://dashscope.aliyuncs.com/compatible-mode/v1',
    model: 'qwen3.5-plus',
    systemPrompt:
      'You are a Qwen (Tongyi Qianwen) assistant accessed via Alibaba DashScope compatible mode, using the latest Qwen3.5-Plus model.'
  }
};
