<script lang="ts">
  import { translationsStore } from '$lib/stores/i18n';
  import { page } from '$app/stores';
  import UuidGenerator from '../uuid/+page.svelte';
  import OtpGenerator from '../otp/+page.svelte';
  import BasicAuthGenerator from '../basic-auth/+page.svelte';
  import HmacGenerator from '../hmac/+page.svelte';

  type GeneratorTab = 'uuid' | 'otp' | 'basic-auth' | 'hmac';

  let activeTab = $state<GeneratorTab>('uuid');

  // Check URL parameter for tab
  $effect(() => {
    const tabParam = $page.url.searchParams.get('tab');
    if (tabParam === 'uuid' || tabParam === 'otp' || tabParam === 'basic-auth' || tabParam === 'hmac') {
      activeTab = tabParam as GeneratorTab;
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

  function switchTab(tab: GeneratorTab) {
    activeTab = tab;
  }
</script>

<div class="flex flex-col h-full w-full ml-0 mr-0 p-2">
  <div class="card flex-1 flex flex-col">
    <div class="flex-1 flex flex-col">
      <!-- 标签页切换 -->
      <div class="border-b border-gray-200 dark:border-gray-700 mb-4">
        <div class="flex gap-6">
          <button
            onclick={() => switchTab('uuid')}
            class="px-4 py-2 relative transition-colors font-medium {activeTab === 'uuid'
              ? 'text-primary-600 dark:text-primary-400'
              : 'text-gray-500 dark:text-gray-400 hover:text-gray-700 dark:hover:text-gray-300'}"
          >
            {t('nav.uuid')}
            {#if activeTab === 'uuid'}
              <span class="absolute bottom-0 left-0 right-0 h-0.5 bg-primary-600 dark:text-primary-400"></span>
            {/if}
          </button>
          <button
            onclick={() => switchTab('otp')}
            class="px-4 py-2 relative transition-colors font-medium {activeTab === 'otp'
              ? 'text-primary-600 dark:text-primary-400'
              : 'text-gray-500 dark:text-gray-400 hover:text-gray-700 dark:hover:text-gray-300'}"
          >
            {t('nav.otp')}
            {#if activeTab === 'otp'}
              <span class="absolute bottom-0 left-0 right-0 h-0.5 bg-primary-600 dark:text-primary-400"></span>
            {/if}
          </button>
          <button
            onclick={() => switchTab('basic-auth')}
            class="px-4 py-2 relative transition-colors font-medium {activeTab === 'basic-auth'
              ? 'text-primary-600 dark:text-primary-400'
              : 'text-gray-500 dark:text-gray-400 hover:text-gray-700 dark:hover:text-gray-300'}"
          >
            {t('nav.basicAuth')}
            {#if activeTab === 'basic-auth'}
              <span class="absolute bottom-0 left-0 right-0 h-0.5 bg-primary-600 dark:text-primary-400"></span>
            {/if}
          </button>
          <button
            onclick={() => switchTab('hmac')}
            class="px-4 py-2 relative transition-colors font-medium {activeTab === 'hmac'
              ? 'text-primary-600 dark:text-primary-400'
              : 'text-gray-500 dark:text-gray-400 hover:text-gray-700 dark:hover:text-gray-300'}"
          >
            {t('nav.hmac')}
            {#if activeTab === 'hmac'}
              <span class="absolute bottom-0 left-0 right-0 h-0.5 bg-primary-600 dark:text-primary-400"></span>
            {/if}
          </button>
        </div>
      </div>

      <!-- 内容区域 -->
      <div class="flex-1 overflow-y-auto -m-2">
        {#if activeTab === 'uuid'}
          <div class="p-2">
            <UuidGenerator />
          </div>
        {:else if activeTab === 'otp'}
          <div class="p-2">
            <OtpGenerator />
          </div>
        {:else if activeTab === 'basic-auth'}
          <div class="p-2">
            <BasicAuthGenerator />
          </div>
        {:else if activeTab === 'hmac'}
          <div class="p-2">
            <HmacGenerator />
          </div>
        {/if}
      </div>
    </div>
  </div>
</div>

