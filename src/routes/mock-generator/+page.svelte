<script lang="ts">
  import { translationsStore } from '$lib/stores/i18n';
  import { Sparkles } from 'lucide-svelte';
  import MockOutputDialog from '$lib/components/MockOutputDialog.svelte';

  let translations = $derived($translationsStore);

  function t(key: string): string {
    const keys = key.split('.');
    let value: any = translations;
    for (const k of keys) {
      value = value?.[k];
    }
    return value || key;
  }

  // 状态管理
  let templateMode = $state<'quick' | 'custom'>('quick');
  let quickTemplate = $state('user');
  let customTemplate = $state('');
  let count = $state(10);
  let output = $state('');
  let isGenerating = $state(false);
  let showOutputDialog = $state(false);

  // 快速模板定义
  const quickTemplates: Record<string, Record<string, string>> = {
    user: {
      name: 'fullName',
      email: 'email',
      phone: 'phone',
      address: 'address',
      age: 'integer',
      gender: 'gender'
    },
    employee: {
      firstName: 'firstName',
      lastName: 'lastName',
      email: 'email',
      phone: 'phone',
      company: 'company',
      jobTitle: 'jobTitle',
      department: 'text'
    },
    customer: {
      name: 'fullName',
      email: 'email',
      phone: 'phone',
      address: 'address',
      city: 'city',
      country: 'country',
      zipCode: 'zipCode'
    },
    product: {
      name: 'text',
      price: 'float',
      category: 'text',
      description: 'lorem',
      inStock: 'boolean',
      createdAt: 'datetime'
    },
    order: {
      orderId: 'uuid',
      customerName: 'fullName',
      email: 'email',
      total: 'float',
      status: 'text',
      createdAt: 'datetime'
    }
  };

  // Mock 数据生成器
  const generators: Record<string, (options?: any) => any> = {
    firstName: () => {
      const names = ['James', 'John', 'Robert', 'Michael', 'William', 'David', 'Richard', 'Joseph', 'Thomas', 'Charles'];
      return names[Math.floor(Math.random() * names.length)];
    },
    lastName: () => {
      const names = ['Smith', 'Johnson', 'Williams', 'Brown', 'Jones', 'Garcia', 'Miller', 'Davis', 'Rodriguez', 'Martinez'];
      return names[Math.floor(Math.random() * names.length)];
    },
    fullName: () => {
      return `${generators.firstName()} ${generators.lastName()}`;
    },
    chineseName: () => {
      const surnames = ['王', '李', '张', '刘', '陈', '杨', '赵', '黄', '周', '吴'];
      const givenNames = ['伟', '芳', '娜', '秀英', '敏', '静', '丽', '强', '磊', '军', '洋', '勇', '艳', '杰', '涛', '明', '超', '秀兰', '霞', '平'];
      return surnames[Math.floor(Math.random() * surnames.length)] + 
             givenNames[Math.floor(Math.random() * givenNames.length)] + 
             (Math.random() > 0.5 ? givenNames[Math.floor(Math.random() * givenNames.length)] : '');
    },
    email: () => {
      const domains = ['gmail.com', 'yahoo.com', 'hotmail.com', 'outlook.com', 'example.com'];
      const username = Math.random().toString(36).substring(2, 10);
      return `${username}@${domains[Math.floor(Math.random() * domains.length)]}`;
    },
    phone: () => {
      return `+1-${Math.floor(Math.random() * 900) + 100}-${Math.floor(Math.random() * 900) + 100}-${Math.floor(Math.random() * 9000) + 1000}`;
    },
    chinesePhone: () => {
      const prefixes = ['130', '131', '132', '133', '134', '135', '136', '137', '138', '139', '150', '151', '152', '153', '155', '156', '157', '158', '159', '180', '181', '182', '183', '184', '185', '186', '187', '188', '189'];
      return `${prefixes[Math.floor(Math.random() * prefixes.length)]}${Math.floor(Math.random() * 100000000).toString().padStart(8, '0')}`;
    },
    address: () => {
      const streets = ['Main St', 'Oak Ave', 'Park Blvd', 'Elm St', 'Maple Dr', 'Cedar Ln', 'Pine Rd', 'First St', 'Second Ave', 'Third Blvd'];
      const numbers = Math.floor(Math.random() * 9999) + 1;
      return `${numbers} ${streets[Math.floor(Math.random() * streets.length)]}`;
    },
    chineseAddress: () => {
      const provinces = ['北京市', '上海市', '广东省', '江苏省', '浙江省', '山东省', '河南省', '四川省', '湖北省', '湖南省'];
      const cities = ['市辖区', '市辖区', '市辖区', '市辖区', '市辖区'];
      const districts = ['区', '区', '区', '区', '区'];
      const streets = ['街道', '路', '大道', '街', '巷'];
      return `${provinces[Math.floor(Math.random() * provinces.length)]}${cities[Math.floor(Math.random() * cities.length)]}${districts[Math.floor(Math.random() * districts.length)]}${Math.floor(Math.random() * 999) + 1}号${streets[Math.floor(Math.random() * streets.length)]}`;
    },
    city: () => {
      const cities = ['New York', 'Los Angeles', 'Chicago', 'Houston', 'Phoenix', 'Philadelphia', 'San Antonio', 'San Diego', 'Dallas', 'San Jose'];
      return cities[Math.floor(Math.random() * cities.length)];
    },
    country: () => {
      const countries = ['United States', 'Canada', 'United Kingdom', 'Australia', 'Germany', 'France', 'Japan', 'China', 'India', 'Brazil'];
      return countries[Math.floor(Math.random() * countries.length)];
    },
    zipCode: () => {
      return Math.floor(Math.random() * 90000) + 10000;
    },
    date: (options?: { min?: string; max?: string; format?: string }) => {
      const min = options?.min ? new Date(options.min).getTime() : new Date('2020-01-01').getTime();
      const max = options?.max ? new Date(options.max).getTime() : Date.now();
      const timestamp = Math.floor(Math.random() * (max - min)) + min;
      const date = new Date(timestamp);
      if (options?.format === 'YYYY-MM-DD') {
        return date.toISOString().split('T')[0];
      }
      return date.toLocaleDateString();
    },
    datetime: (options?: { min?: string; max?: string; format?: string }) => {
      const min = options?.min ? new Date(options.min).getTime() : new Date('2020-01-01').getTime();
      const max = options?.max ? new Date(options.max).getTime() : Date.now();
      const timestamp = Math.floor(Math.random() * (max - min)) + min;
      const date = new Date(timestamp);
      if (options?.format === 'ISO') {
        return date.toISOString();
      }
      return date.toLocaleString();
    },
    time: () => {
      const hours = Math.floor(Math.random() * 24).toString().padStart(2, '0');
      const minutes = Math.floor(Math.random() * 60).toString().padStart(2, '0');
      const seconds = Math.floor(Math.random() * 60).toString().padStart(2, '0');
      return `${hours}:${minutes}:${seconds}`;
    },
    number: (options?: { min?: number; max?: number }) => {
      const min = options?.min ?? 0;
      const max = options?.max ?? 100;
      return Math.floor(Math.random() * (max - min + 1)) + min;
    },
    integer: (options?: { min?: number; max?: number }) => {
      const min = options?.min ?? 0;
      const max = options?.max ?? 100;
      return Math.floor(Math.random() * (max - min + 1)) + min;
    },
    float: (options?: { min?: number; max?: number; decimals?: number }) => {
      const min = options?.min ?? 0;
      const max = options?.max ?? 100;
      const decimals = options?.decimals ?? 2;
      const value = Math.random() * (max - min) + min;
      return parseFloat(value.toFixed(decimals));
    },
    boolean: () => {
      return Math.random() > 0.5;
    },
    uuid: () => {
      return 'xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx'.replace(/[xy]/g, (c) => {
        const r = Math.random() * 16 | 0;
        const v = c === 'x' ? r : (r & 0x3 | 0x8);
        return v.toString(16);
      });
    },
    text: (options?: { length?: number; prefix?: string; suffix?: string }) => {
      const length = options?.length ?? 10;
      const chars = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789';
      let result = '';
      for (let i = 0; i < length; i++) {
        result += chars.charAt(Math.floor(Math.random() * chars.length));
      }
      return (options?.prefix || '') + result + (options?.suffix || '');
    },
    lorem: (options?: { words?: number }) => {
      const words = options?.words ?? 10;
      const loremWords = ['lorem', 'ipsum', 'dolor', 'sit', 'amet', 'consectetur', 'adipiscing', 'elit', 'sed', 'do', 'eiusmod', 'tempor', 'incididunt', 'ut', 'labore', 'et', 'dolore', 'magna', 'aliqua'];
      const result: string[] = [];
      for (let i = 0; i < words; i++) {
        result.push(loremWords[Math.floor(Math.random() * loremWords.length)]);
      }
      return result.join(' ');
    },
    color: () => {
      return '#' + Math.floor(Math.random() * 16777215).toString(16).padStart(6, '0');
    },
    ip: () => {
      return `${Math.floor(Math.random() * 255)}.${Math.floor(Math.random() * 255)}.${Math.floor(Math.random() * 255)}.${Math.floor(Math.random() * 255)}`;
    },
    url: () => {
      const domains = ['example.com', 'test.com', 'demo.org', 'sample.net'];
      const paths = ['page', 'article', 'post', 'item', 'product'];
      return `https://${domains[Math.floor(Math.random() * domains.length)]}/${paths[Math.floor(Math.random() * paths.length)]}/${Math.floor(Math.random() * 1000)}`;
    },
    username: () => {
      return 'user' + Math.floor(Math.random() * 10000);
    },
    password: () => {
      const chars = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789!@#$%^&*';
      let result = '';
      for (let i = 0; i < 12; i++) {
        result += chars.charAt(Math.floor(Math.random() * chars.length));
      }
      return result;
    },
    company: () => {
      const companies = ['Acme Corp', 'Tech Solutions', 'Global Industries', 'Digital Services', 'Innovation Labs', 'Future Systems', 'Smart Solutions', 'NextGen Tech'];
      return companies[Math.floor(Math.random() * companies.length)];
    },
    jobTitle: () => {
      const titles = ['Software Engineer', 'Product Manager', 'Designer', 'Marketing Manager', 'Sales Representative', 'Data Analyst', 'Project Manager', 'CEO'];
      return titles[Math.floor(Math.random() * titles.length)];
    },
    age: () => {
      return Math.floor(Math.random() * 50) + 18;
    },
    gender: () => {
      return Math.random() > 0.5 ? 'Male' : 'Female';
    },
    creditCard: () => {
      const prefixes = ['4', '5', '37', '6'];
      let card = prefixes[Math.floor(Math.random() * prefixes.length)];
      for (let i = 0; i < 15; i++) {
        card += Math.floor(Math.random() * 10);
      }
      return card;
    },
    bankAccount: () => {
      return Math.floor(Math.random() * 10000000000).toString().padStart(10, '0');
    }
  };

  // 生成单个数据项
  function generateItem(template: Record<string, any>): Record<string, any> {
    const item: Record<string, any> = {};
    for (const [key, type] of Object.entries(template)) {
      if (typeof type === 'string') {
        item[key] = generators[type]?.() ?? '';
      } else if (typeof type === 'object' && type.type) {
        item[key] = generators[type.type]?.(type.options) ?? '';
      }
    }
    return item;
  }

  // 生成数据
  async function generateData() {
    if (isGenerating) return;
    
    isGenerating = true;
    try {
      let template: Record<string, any>;
      
      if (templateMode === 'quick') {
        template = quickTemplates[quickTemplate] || quickTemplates.user;
      } else {
        try {
          template = JSON.parse(customTemplate);
        } catch (e) {
          alert(t('mockGenerator.errors.invalidTemplate'));
          return;
        }
      }

      if (!count || count < 1 || count > 10000) {
        alert(t('mockGenerator.errors.invalidCount'));
        return;
      }

      const results: Record<string, any>[] = [];
      for (let i = 0; i < count; i++) {
        results.push(generateItem(template));
      }

      output = JSON.stringify(results, null, 2);
      showOutputDialog = true;
    } catch (error) {
      console.error('Generate error:', error);
      alert(t('mockGenerator.errors.generateFailed'));
    } finally {
      isGenerating = false;
    }
  }

  // 清空
  function clearData() {
    customTemplate = '';
  }

  // 加载快速模板示例
  function loadQuickTemplate(name: string) {
    quickTemplate = name;
    templateMode = 'quick';
    customTemplate = JSON.stringify(quickTemplates[name], null, 2);
  }
</script>

<MockOutputDialog
  bind:open={showOutputDialog}
  {output}
  onClose={() => showOutputDialog = false}
  {t}
/>

<div class="ml-0 mr-0 p-2 h-full flex flex-col">
  <div class="max-w-7xl mx-auto w-full h-full flex flex-col">
    <!-- 模板配置区域 -->
    <div class="card flex-1 flex flex-col min-h-0">
      <div class="space-y-4 flex-1 flex flex-col min-h-0">
        <!-- 模板模式选择 -->
        <div class="flex-1 flex flex-col min-h-0">
          <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2 flex-shrink-0">
            {t('mockGenerator.template')}
          </label>
          <div class="flex gap-2 mb-4 flex-shrink-0">
            <button
              onclick={() => templateMode = 'quick'}
              class="px-4 py-2 rounded-lg transition-colors {templateMode === 'quick' ? 'bg-blue-500 text-white' : 'bg-gray-200 dark:bg-gray-700 text-gray-700 dark:text-gray-300'}"
            >
              {t('mockGenerator.quickTemplates')}
            </button>
            <button
              onclick={() => templateMode = 'custom'}
              class="px-4 py-2 rounded-lg transition-colors {templateMode === 'custom' ? 'bg-blue-500 text-white' : 'bg-gray-200 dark:bg-gray-700 text-gray-700 dark:text-gray-300'}"
            >
              {t('mockGenerator.userTemplate')}
            </button>
          </div>

          {#if templateMode === 'quick'}
            <div class="grid grid-cols-2 md:grid-cols-5 gap-2 flex-shrink-0">
              {#each Object.keys(quickTemplates) as templateName}
                <button
                  onclick={() => loadQuickTemplate(templateName)}
                  class="px-3 py-2 text-sm rounded-lg transition-colors {quickTemplate === templateName ? 'bg-blue-500 text-white' : 'bg-gray-100 dark:bg-gray-700 text-gray-700 dark:text-gray-300 hover:bg-gray-200 dark:hover:bg-gray-600'}"
                >
                  {t(`mockGenerator.quickTemplateNames.${templateName}`) || templateName.charAt(0).toUpperCase() + templateName.slice(1)}
                </button>
              {/each}
            </div>
            <textarea
              value={JSON.stringify(quickTemplates[quickTemplate] || {}, null, 2)}
              readonly
              class="input font-mono text-sm resize-none mt-2 flex-1 min-h-0"
            ></textarea>
          {:else}
            <div class="mb-3 p-3 bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800 rounded-lg flex-shrink-0">
              <div class="flex items-start gap-2">
                <div class="text-blue-600 dark:text-blue-400 mt-0.5">
                  <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"></path>
                  </svg>
                </div>
                <div class="flex-1 text-sm text-blue-800 dark:text-blue-200">
                  <div class="font-semibold mb-1">{t('mockGenerator.templateHelp')}</div>
                  <div class="mb-2">{t('mockGenerator.templateHelpDesc')}</div>
                  <div class="space-y-1 font-mono text-xs">
                    <div>{t('mockGenerator.templateExample')}</div>
                    <div class="text-blue-700 dark:text-blue-300">{t('mockGenerator.templateExampleSimple')}</div>
                    <div class="text-blue-700 dark:text-blue-300">{t('mockGenerator.templateExampleAdvanced')}</div>
                    <div class="text-blue-700 dark:text-blue-300">{t('mockGenerator.templateExampleDate')}</div>
                  </div>
                </div>
              </div>
            </div>
            <textarea
              bind:value={customTemplate}
              placeholder={t('mockGenerator.templatePlaceholder')}
              class="input font-mono text-sm resize-none flex-1 min-h-0"
              autocapitalize="off"
              spellcheck={false}
            ></textarea>
          {/if}
        </div>

        <!-- 数量输入和操作按钮 -->
        <div class="flex items-end gap-4 flex-shrink-0">
          <div class="flex-1">
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
              {t('mockGenerator.count')}
            </label>
            <input
              type="number"
              bind:value={count}
              min="1"
              max="10000"
              class="input w-32"
            />
          </div>
          <div class="flex gap-2">
            <button
              onclick={generateData}
              disabled={isGenerating}
              class="btn-primary {isGenerating ? 'opacity-50 cursor-not-allowed' : ''}"
            >
              {#if isGenerating}
                {t('mockGenerator.generate')}...
              {:else}
                {t('mockGenerator.generate')}
              {/if}
            </button>
            {#if templateMode === 'custom' && customTemplate.trim()}
              <button
                onclick={clearData}
                class="btn-secondary"
              >
                {t('mockGenerator.clear')}
              </button>
            {/if}
          </div>
        </div>
      </div>
    </div>
  </div>
</div>

