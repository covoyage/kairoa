<script lang="ts">
  import { translationsStore } from '$lib/stores/i18n';
  import { Copy, Check } from 'lucide-svelte';

  let translations = $derived($translationsStore);

  function t(key: string): string {
    const keys = key.split('.');
    let value: any = translations;
    for (const k of keys) {
      value = value?.[k];
    }
    return value || key;
  }

  // 权限类型
  type Permission = 'r' | 'w' | 'x';
  type PermissionGroup = 'owner' | 'group' | 'others';

  // 权限状态
  interface Permissions {
    owner: { r: boolean; w: boolean; x: boolean };
    group: { r: boolean; w: boolean; x: boolean };
    others: { r: boolean; w: boolean; x: boolean };
  }

  let permissions = $state<Permissions>({
    owner: { r: false, w: false, x: false },
    group: { r: false, w: false, x: false },
    others: { r: false, w: false, x: false }
  });

  let octalValue = $state('');
  let symbolValue = $state('');
  let copied = $state<{ octal: boolean; symbol: boolean }>({ octal: false, symbol: false });
  let error = $state('');

  // 计算八进制值
  function calculateOctal(): string {
    const owner = (permissions.owner.r ? 4 : 0) + (permissions.owner.w ? 2 : 0) + (permissions.owner.x ? 1 : 0);
    const group = (permissions.group.r ? 4 : 0) + (permissions.group.w ? 2 : 0) + (permissions.group.x ? 1 : 0);
    const others = (permissions.others.r ? 4 : 0) + (permissions.others.w ? 2 : 0) + (permissions.others.x ? 1 : 0);
    return `${owner}${group}${others}`;
  }

  // 计算符号表示
  function calculateSymbol(): string {
    const getSymbol = (group: { r: boolean; w: boolean; x: boolean }) => {
      return `${group.r ? 'r' : '-'}${group.w ? 'w' : '-'}${group.x ? 'x' : '-'}`;
    };
    return `${getSymbol(permissions.owner)}${getSymbol(permissions.group)}${getSymbol(permissions.others)}`;
  }

  // 从八进制值解析权限
  function parseOctal(value: string): Permissions | null {
    if (!/^[0-7]{3}$/.test(value)) {
      return null;
    }

    const parseDigit = (digit: string) => {
      const num = parseInt(digit, 10);
      return {
        r: (num & 4) !== 0,
        w: (num & 2) !== 0,
        x: (num & 1) !== 0
      };
    };

    return {
      owner: parseDigit(value[0]),
      group: parseDigit(value[1]),
      others: parseDigit(value[2])
    };
  }

  // 从符号表示解析权限
  function parseSymbol(value: string): Permissions | null {
    // 移除所有空格和连字符（用于分隔符）
    const clean = value.replace(/[\s-]/g, '');
    
    // 应该是9个字符：rwxrwxrwx
    if (!/^[rwx-]{9}$/i.test(clean)) {
      return null;
    }

    const parseGroup = (str: string) => {
      return {
        r: str[0].toLowerCase() === 'r',
        w: str[1].toLowerCase() === 'w',
        x: str[2].toLowerCase() === 'x'
      };
    };

    return {
      owner: parseGroup(clean.substring(0, 3)),
      group: parseGroup(clean.substring(3, 6)),
      others: parseGroup(clean.substring(6, 9))
    };
  }

  // 更新权限时自动计算八进制和符号
  $effect(() => {
    octalValue = calculateOctal();
    symbolValue = calculateSymbol();
    error = '';
  });

  // 处理八进制输入
  function handleOctalInput(value: string) {
    // 移除非数字字符
    const clean = value.replace(/[^0-7]/g, '').substring(0, 3);
    octalValue = clean;

    if (clean.length === 3) {
      const parsed = parseOctal(clean);
      if (parsed) {
        permissions = parsed;
        error = '';
      } else {
        error = t('chmod.invalidOctal');
      }
    } else if (clean.length > 0) {
      error = t('chmod.incompleteOctal');
    } else {
      error = '';
    }
  }

  // 处理符号输入
  function handleSymbolInput(value: string) {
    symbolValue = value;
    const parsed = parseSymbol(value);
    if (parsed) {
      permissions = parsed;
      error = '';
    } else if (value.length > 0) {
      error = t('chmod.invalidSymbol');
    } else {
      error = '';
    }
  }

  // 切换权限
  function togglePermission(group: PermissionGroup, perm: Permission) {
    permissions[group][perm] = !permissions[group][perm];
  }

  // 复制到剪贴板
  async function copyToClipboard(text: string, type: 'octal' | 'symbol') {
    try {
      await navigator.clipboard.writeText(text);
      copied = { ...copied, [type]: true };
      setTimeout(() => {
        copied = { ...copied, [type]: false };
      }, 2000);
    } catch (error) {
      console.error('Failed to copy:', error);
    }
  }

  // 获取权限组标签
  function getGroupLabel(group: PermissionGroup): string {
    const labels: Record<PermissionGroup, string> = {
      owner: t('chmod.owner'),
      group: t('chmod.group'),
      others: t('chmod.others')
    };
    return labels[group];
  }

  // 获取权限标签
  function getPermissionLabel(perm: Permission): string {
    const labels: Record<Permission, string> = {
      r: t('chmod.read'),
      w: t('chmod.write'),
      x: t('chmod.execute')
    };
    return labels[perm];
  }
</script>

<div class="flex flex-col h-full w-full ml-0 mr-0 p-2">
  <div class="flex-1 grid grid-cols-1 lg:grid-cols-2 gap-4 min-h-0">
    <!-- 权限选择器 -->
    <div class="card">
      <h2 class="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-4">
        {t('chmod.permissions')}
      </h2>
      
      <div class="space-y-4">
        {#each (['owner', 'group', 'others'] as PermissionGroup[]).map(g => g) as group}
          <div class="border border-gray-200 dark:border-gray-700 rounded-lg p-4">
            <div class="font-medium text-gray-700 dark:text-gray-300 mb-3">
              {getGroupLabel(group)}
            </div>
            <div class="flex gap-4">
              {#each (['r', 'w', 'x'] as Permission[]).map(p => p) as perm}
                <label class="flex items-center gap-2 cursor-pointer">
                  <input
                    type="checkbox"
                    checked={permissions[group][perm]}
                    onchange={() => togglePermission(group, perm)}
                    class="w-4 h-4 text-primary-600 border-gray-300 rounded focus:ring-primary-500 dark:focus:ring-primary-400 dark:ring-offset-gray-800 focus:ring-2 dark:bg-gray-700 dark:border-gray-600"
                  />
                  <span class="text-sm text-gray-700 dark:text-gray-300 font-mono">
                    {perm.toUpperCase()}
                  </span>
                  <span class="text-xs text-gray-500 dark:text-gray-400">
                    ({getPermissionLabel(perm)})
                  </span>
                </label>
              {/each}
            </div>
          </div>
        {/each}
      </div>
    </div>

    <!-- 结果显示 -->
    <div class="card space-y-6">
      <h2 class="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-4">
        {t('chmod.result')}
      </h2>

      <!-- 八进制值 -->
      <div>
        <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
          {t('chmod.octal')}
        </label>
        <div class="flex gap-2">
          <input
            type="text"
            bind:value={octalValue}
            oninput={(e) => handleOctalInput((e.target as HTMLInputElement).value)}
            placeholder="000"
            maxlength="3"
            class="input flex-1 font-mono text-lg"
          />
          <button
            onclick={() => copyToClipboard(octalValue, 'octal')}
            class="btn-secondary px-4"
            disabled={!octalValue}
          >
            {#if copied.octal}
              <Check class="w-4 h-4" />
            {:else}
              <Copy class="w-4 h-4" />
            {/if}
          </button>
        </div>
        <p class="text-xs text-gray-500 dark:text-gray-400 mt-1">
          {t('chmod.octalHint')}
        </p>
      </div>

      <!-- 符号表示 -->
      <div>
        <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
          {t('chmod.symbol')}
        </label>
        <div class="flex gap-2">
          <input
            type="text"
            bind:value={symbolValue}
            oninput={(e) => handleSymbolInput((e.target as HTMLInputElement).value)}
            placeholder="rwxrwxrwx"
            maxlength="9"
            class="input flex-1 font-mono text-lg"
          />
          <button
            onclick={() => copyToClipboard(symbolValue, 'symbol')}
            class="btn-secondary px-4"
            disabled={!symbolValue}
          >
            {#if copied.symbol}
              <Check class="w-4 h-4" />
            {:else}
              <Copy class="w-4 h-4" />
            {/if}
          </button>
        </div>
        <p class="text-xs text-gray-500 dark:text-gray-400 mt-1">
          {t('chmod.symbolHint')}
        </p>
      </div>

      <!-- 错误提示 -->
      {#if error}
        <div class="p-3 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg">
          <p class="text-sm text-red-800 dark:text-red-200">
            {error}
          </p>
        </div>
      {/if}

      <!-- 使用示例 -->
      <div class="border-t border-gray-200 dark:border-gray-700 pt-4">
        <h3 class="text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
          {t('chmod.example')}
        </h3>
        <div class="bg-gray-50 dark:bg-gray-900 rounded-lg p-3 font-mono text-sm text-gray-800 dark:text-gray-200">
          <div class="mb-1">chmod {octalValue || '755'} filename</div>
          <div class="text-gray-500 dark:text-gray-400 text-xs">
            {t('chmod.exampleDesc')}
          </div>
        </div>
      </div>
    </div>
  </div>
</div>

