<script lang="ts">
  import { translationsStore } from '$lib/stores/i18n';
  import DateTimePicker from '$lib/components/DateTimePicker.svelte';
  import { ArrowRight, ArrowLeftRight, Calendar, Clock, Copy, Check, CalendarDays } from 'lucide-svelte';

  let translations = $derived($translationsStore);

  function t(key: string): string {
    const keys = key.split('.');
    let value: any = translations;
    for (const k of keys) {
      value = value?.[k];
    }
    return value || key;
  }

  // 日期输入值（datetime-local 格式: YYYY-MM-DDTHH:mm）
  let startDate = $state(getLocalDateTimeStr(new Date()));
  let endDate = $state(getLocalDateTimeStr(new Date(Date.now() + 7 * 24 * 60 * 60 * 1000)));
  let resultCopied = $state(false);

  // 获取本地日期时间字符串（用于 datetime-local input）
  function getLocalDateTimeStr(date: Date): string {
    const offset = date.getTimezoneOffset();
    const localDate = new Date(date.getTime() - offset * 60000);
    return localDate.toISOString().slice(0, 16);
  }

  // 解析 datetime-local 字符串为 Date 对象
  function parseDateTime(dateStr: string): Date | null {
    if (!dateStr) return null;
    const date = new Date(dateStr);
    if (isNaN(date.getTime())) return null;
    return date;
  }

  // 计算结果
  interface DiffResult {
    totalMilliseconds: number;
    totalSeconds: number;
    totalMinutes: number;
    totalHours: number;
    totalDays: number;
    totalWeeks: number;
    calendarYears: number;
    calendarMonths: number;
    calendarDays: number;
    hours: number;
    minutes: number;
    seconds: number;
    businessDays: number;
    weekendDays: number;
    isNegative: boolean;
  }

  let result = $derived.by((): DiffResult | null => {
    const start = parseDateTime(startDate);
    const end = parseDateTime(endDate);
    if (!start || !end) return null;

    const diffMs = end.getTime() - start.getTime();
    const isNegative = diffMs < 0;
    const absMs = Math.abs(diffMs);

    const totalSeconds = Math.floor(absMs / 1000);
    const totalMinutes = Math.floor(totalSeconds / 60);
    const totalHours = Math.floor(totalMinutes / 60);
    const totalDays = Math.floor(totalHours / 24);
    const totalWeeks = Math.floor(totalDays / 7);

    // 日历分解（年、月、日、时、分、秒）
    const earlier = isNegative ? end : start;
    const later = isNegative ? start : end;

    let calendarYears = later.getFullYear() - earlier.getFullYear();
    let calendarMonths = later.getMonth() - earlier.getMonth();
    let calendarDays = later.getDate() - earlier.getDate();
    let hours = later.getHours() - earlier.getHours();
    let minutes = later.getMinutes() - earlier.getMinutes();
    let seconds = later.getSeconds() - earlier.getSeconds();

    if (seconds < 0) {
      seconds += 60;
      minutes--;
    }
    if (minutes < 0) {
      minutes += 60;
      hours--;
    }
    if (hours < 0) {
      hours += 24;
      calendarDays--;
    }
    if (calendarDays < 0) {
      // 从上个月借天数
      const prevMonth = new Date(later.getFullYear(), later.getMonth(), 0);
      calendarDays += prevMonth.getDate();
      calendarMonths--;
    }
    if (calendarMonths < 0) {
      calendarMonths += 12;
      calendarYears--;
    }

    // 计算工作日和周末日
    let businessDays = 0;
    let weekendDays = 0;
    const iterStart = new Date(earlier);
    // 将时间归零到当天开始
    iterStart.setHours(0, 0, 0, 0);
    const iterEnd = new Date(later);
    iterEnd.setHours(0, 0, 0, 0);

    // 如果开始和结束是同一天，不计入天数
    if (iterStart.getTime() !== iterEnd.getTime()) {
      const tempDate = new Date(iterStart);
      tempDate.setDate(tempDate.getDate() + 1); // 从第二天开始算
      while (tempDate <= iterEnd) {
        const dayOfWeek = tempDate.getDay();
        if (dayOfWeek === 0 || dayOfWeek === 6) {
          weekendDays++;
        } else {
          businessDays++;
        }
        tempDate.setDate(tempDate.getDate() + 1);
      }
    }

    return {
      totalMilliseconds: absMs,
      totalSeconds,
      totalMinutes,
      totalHours,
      totalDays,
      totalWeeks,
      calendarYears,
      calendarMonths,
      calendarDays,
      hours,
      minutes,
      seconds,
      businessDays,
      weekendDays,
      isNegative
    };
  });

  // 格式化日历分解结果
  let calendarBreakdown = $derived.by((): string => {
    if (!result) return '';
    const parts: string[] = [];
    if (result.calendarYears > 0) {
      parts.push(`${result.calendarYears} ${t('dateDiff.years')}`);
    }
    if (result.calendarMonths > 0) {
      parts.push(`${result.calendarMonths} ${t('dateDiff.months')}`);
    }
    if (result.calendarDays > 0) {
      parts.push(`${result.calendarDays} ${t('dateDiff.days')}`);
    }
    if (result.hours > 0) {
      parts.push(`${result.hours} ${t('dateDiff.hours')}`);
    }
    if (result.minutes > 0) {
      parts.push(`${result.minutes} ${t('dateDiff.minutes')}`);
    }
    if (result.seconds > 0) {
      parts.push(`${result.seconds} ${t('dateDiff.seconds')}`);
    }
    if (parts.length === 0) {
      return t('dateDiff.sameTime');
    }
    let str = parts.join(', ');
    if (result.isNegative) {
      str = `${t('dateDiff.negativePrefix')} ${str}`;
    }
    return str;
  });

  // 快速预设
  function setPreset(preset: string) {
    const now = new Date();
    switch (preset) {
      case 'today':
        startDate = getLocalDateTimeStr(now);
        break;
      case 'yesterday':
        startDate = getLocalDateTimeStr(new Date(now.getTime() - 24 * 60 * 60 * 1000));
        endDate = getLocalDateTimeStr(now);
        break;
      case 'tomorrow':
        startDate = getLocalDateTimeStr(now);
        endDate = getLocalDateTimeStr(new Date(now.getTime() + 24 * 60 * 60 * 1000));
        break;
      case 'nextWeek':
        startDate = getLocalDateTimeStr(now);
        endDate = getLocalDateTimeStr(new Date(now.getTime() + 7 * 24 * 60 * 60 * 1000));
        break;
      case 'lastWeek':
        startDate = getLocalDateTimeStr(new Date(now.getTime() - 7 * 24 * 60 * 60 * 1000));
        endDate = getLocalDateTimeStr(now);
        break;
      case 'nextMonth':
        startDate = getLocalDateTimeStr(now);
        endDate = getLocalDateTimeStr(new Date(now.getFullYear(), now.getMonth() + 1, now.getDate(), now.getHours(), now.getMinutes()));
        break;
      case 'lastMonth':
        startDate = getLocalDateTimeStr(new Date(now.getFullYear(), now.getMonth() - 1, now.getDate(), now.getHours(), now.getMinutes()));
        endDate = getLocalDateTimeStr(now);
        break;
      case 'nextYear':
        startDate = getLocalDateTimeStr(now);
        endDate = getLocalDateTimeStr(new Date(now.getFullYear() + 1, now.getMonth(), now.getDate(), now.getHours(), now.getMinutes()));
        break;
      case 'lastYear':
        startDate = getLocalDateTimeStr(new Date(now.getFullYear() - 1, now.getMonth(), now.getDate(), now.getHours(), now.getMinutes()));
        endDate = getLocalDateTimeStr(now);
        break;
    }
  }

  // 交换开始和结束日期
  function swapDates() {
    const temp = startDate;
    startDate = endDate;
    endDate = temp;
  }

  // 格式化日期为可读字符串
  function formatDateTime(dateStr: string): string {
    if (!dateStr) return '';
    const d = new Date(dateStr);
    if (isNaN(d.getTime())) return dateStr;
    const yyyy = d.getFullYear();
    const mm = String(d.getMonth() + 1).padStart(2, '0');
    const dd = String(d.getDate()).padStart(2, '0');
    const hh = String(d.getHours()).padStart(2, '0');
    const min = String(d.getMinutes()).padStart(2, '0');
    return `${yyyy}-${mm}-${dd} ${hh}:${min}`;
  }

  // 格式化数字（添加千分位分隔符）
  function formatNumber(num: number): string {
    return num.toLocaleString();
  }

  // 复制结果
  async function copyResult() {
    if (!result) return;
    const lines: string[] = [];
    lines.push(`${t('dateDiff.startDate')}: ${formatDateTime(startDate)}`);
    lines.push(`${t('dateDiff.endDate')}: ${formatDateTime(endDate)}`);
    lines.push('');
    lines.push(`${t('dateDiff.calendarBreakdown')}: ${calendarBreakdown}`);
    lines.push(`${t('dateDiff.totalDays')}: ${formatNumber(result.totalDays)}`);
    lines.push(`${t('dateDiff.totalHours')}: ${formatNumber(result.totalHours)}`);
    lines.push(`${t('dateDiff.totalMinutes')}: ${formatNumber(result.totalMinutes)}`);
    lines.push(`${t('dateDiff.totalSeconds')}: ${formatNumber(result.totalSeconds)}`);
    lines.push(`${t('dateDiff.totalWeeks')}: ${formatNumber(result.totalWeeks)}`);
    lines.push(`${t('dateDiff.businessDays')}: ${formatNumber(result.businessDays)}`);
    lines.push(`${t('dateDiff.weekendDays')}: ${formatNumber(result.weekendDays)}`);

    try {
      await navigator.clipboard.writeText(lines.join('\n'));
      resultCopied = true;
      setTimeout(() => { resultCopied = false; }, 1500);
    } catch (error) {
      console.error('Failed to copy:', error);
    }
  }
</script>

<div class="w-full ml-0 mr-0 p-2 space-y-6">
  <!-- 输入区域卡片 -->
  <div class="card">
    <div class="space-y-4">
      <div class="flex items-center gap-2">
        <CalendarDays class="w-5 h-5 text-primary-600 dark:text-primary-400" />
        <h2 class="text-lg font-bold text-gray-900 dark:text-gray-100">{t('dateDiff.title')}</h2>
      </div>

      <!-- 日期输入 -->
      <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
        <div>
          <label for="start-date" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
            {t('dateDiff.startDate')}
          </label>
          <DateTimePicker
            id="start-date"
            value={startDate}
            onChange={(val) => startDate = val}
          />
        </div>
        <div>
          <label for="end-date" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
            {t('dateDiff.endDate')}
          </label>
          <div class="flex gap-2">
            <div class="flex-1">
              <DateTimePicker
                id="end-date"
                value={endDate}
                onChange={(val) => endDate = val}
              />
            </div>
            <button
              onclick={swapDates}
              class="btn-secondary whitespace-nowrap flex items-center gap-1 self-stretch"
              title={t('dateDiff.swap')}
            >
              <ArrowLeftRight class="w-4 h-4" />
            </button>
          </div>
        </div>
      </div>

      <!-- 快速预设 -->
      <div>
        <span class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
          {t('dateDiff.quickPresets')}
        </span>
        <div class="flex flex-wrap gap-2">
          <button onclick={() => setPreset('yesterday')} class="btn-secondary text-sm py-1.5 px-3">{t('dateDiff.presetYesterday')}</button>
          <button onclick={() => setPreset('today')} class="btn-secondary text-sm py-1.5 px-3">{t('dateDiff.presetToday')}</button>
          <button onclick={() => setPreset('tomorrow')} class="btn-secondary text-sm py-1.5 px-3">{t('dateDiff.presetTomorrow')}</button>
          <button onclick={() => setPreset('lastWeek')} class="btn-secondary text-sm py-1.5 px-3">{t('dateDiff.presetLastWeek')}</button>
          <button onclick={() => setPreset('nextWeek')} class="btn-secondary text-sm py-1.5 px-3">{t('dateDiff.presetNextWeek')}</button>
          <button onclick={() => setPreset('lastMonth')} class="btn-secondary text-sm py-1.5 px-3">{t('dateDiff.presetLastMonth')}</button>
          <button onclick={() => setPreset('nextMonth')} class="btn-secondary text-sm py-1.5 px-3">{t('dateDiff.presetNextMonth')}</button>
          <button onclick={() => setPreset('lastYear')} class="btn-secondary text-sm py-1.5 px-3">{t('dateDiff.presetLastYear')}</button>
          <button onclick={() => setPreset('nextYear')} class="btn-secondary text-sm py-1.5 px-3">{t('dateDiff.presetNextYear')}</button>
        </div>
      </div>
    </div>
  </div>

  <!-- 结果区域 -->
  {#if result}
    <div class="card">
      <div class="space-y-5">
        <div class="flex items-center justify-between">
          <h3 class="text-xl font-bold text-gray-900 dark:text-gray-100">{t('dateDiff.results')}</h3>
          <button
            onclick={copyResult}
            class="btn-secondary whitespace-nowrap {resultCopied ? 'bg-green-500 hover:bg-green-600 text-white' : ''}"
          >
            {#if resultCopied}
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
        </div>

        <!-- 日历分解 -->
        <div class="bg-primary-50 dark:bg-primary-900/20 rounded-lg p-4 border border-primary-200 dark:border-primary-800">
          <div class="flex items-center gap-2 mb-2">
            <Calendar class="w-4 h-4 text-primary-600 dark:text-primary-400" />
            <span class="text-sm font-medium text-primary-700 dark:text-primary-300">{t('dateDiff.calendarBreakdown')}</span>
          </div>
          <p class="text-lg font-semibold text-gray-900 dark:text-gray-100">
            {calendarBreakdown}
          </p>
        </div>

        <!-- 时间线显示 -->
        <div class="flex items-center gap-3 text-sm text-gray-600 dark:text-gray-400">
          <span class="font-mono px-2 py-1 bg-gray-100 dark:bg-gray-700 rounded">{formatDateTime(startDate)}</span>
          <ArrowRight class="w-4 h-4 flex-shrink-0" />
          <span class="font-mono px-2 py-1 bg-gray-100 dark:bg-gray-700 rounded">{formatDateTime(endDate)}</span>
        </div>

        <!-- 总计统计网格 -->
        <div class="grid grid-cols-2 md:grid-cols-3 gap-3">
          <div class="bg-gray-50 dark:bg-gray-800 rounded-lg p-4 border border-gray-200 dark:border-gray-700">
            <div class="flex items-center gap-2 mb-1">
              <CalendarDays class="w-4 h-4 text-gray-400 dark:text-gray-500" />
              <span class="text-xs font-medium text-gray-500 dark:text-gray-400">{t('dateDiff.totalDays')}</span>
            </div>
            <p class="text-2xl font-bold text-gray-900 dark:text-gray-100">{formatNumber(result.totalDays)}</p>
          </div>

          <div class="bg-gray-50 dark:bg-gray-800 rounded-lg p-4 border border-gray-200 dark:border-gray-700">
            <div class="flex items-center gap-2 mb-1">
              <Clock class="w-4 h-4 text-gray-400 dark:text-gray-500" />
              <span class="text-xs font-medium text-gray-500 dark:text-gray-400">{t('dateDiff.totalHours')}</span>
            </div>
            <p class="text-2xl font-bold text-gray-900 dark:text-gray-100">{formatNumber(result.totalHours)}</p>
          </div>

          <div class="bg-gray-50 dark:bg-gray-800 rounded-lg p-4 border border-gray-200 dark:border-gray-700">
            <div class="flex items-center gap-2 mb-1">
              <Clock class="w-4 h-4 text-gray-400 dark:text-gray-500" />
              <span class="text-xs font-medium text-gray-500 dark:text-gray-400">{t('dateDiff.totalMinutes')}</span>
            </div>
            <p class="text-2xl font-bold text-gray-900 dark:text-gray-100">{formatNumber(result.totalMinutes)}</p>
          </div>

          <div class="bg-gray-50 dark:bg-gray-800 rounded-lg p-4 border border-gray-200 dark:border-gray-700">
            <div class="flex items-center gap-2 mb-1">
              <Clock class="w-4 h-4 text-gray-400 dark:text-gray-500" />
              <span class="text-xs font-medium text-gray-500 dark:text-gray-400">{t('dateDiff.totalSeconds')}</span>
            </div>
            <p class="text-2xl font-bold text-gray-900 dark:text-gray-100">{formatNumber(result.totalSeconds)}</p>
          </div>

          <div class="bg-gray-50 dark:bg-gray-800 rounded-lg p-4 border border-gray-200 dark:border-gray-700">
            <div class="flex items-center gap-2 mb-1">
              <CalendarDays class="w-4 h-4 text-gray-400 dark:text-gray-500" />
              <span class="text-xs font-medium text-gray-500 dark:text-gray-400">{t('dateDiff.totalWeeks')}</span>
            </div>
            <p class="text-2xl font-bold text-gray-900 dark:text-gray-100">{formatNumber(result.totalWeeks)}</p>
          </div>

          <div class="bg-gray-50 dark:bg-gray-800 rounded-lg p-4 border border-gray-200 dark:border-gray-700">
            <div class="flex items-center gap-2 mb-1">
              <Calendar class="w-4 h-4 text-gray-400 dark:text-gray-500" />
              <span class="text-xs font-medium text-gray-500 dark:text-gray-400">{t('dateDiff.totalMilliseconds')}</span>
            </div>
            <p class="text-2xl font-bold text-gray-900 dark:text-gray-100">{formatNumber(result.totalMilliseconds)}</p>
          </div>
        </div>

        <!-- 工作日和周末 -->
        <div class="grid grid-cols-2 gap-3">
          <div class="bg-blue-50 dark:bg-blue-900/20 rounded-lg p-4 border border-blue-200 dark:border-blue-800">
            <span class="text-xs font-medium text-blue-600 dark:text-blue-400">{t('dateDiff.businessDays')}</span>
            <p class="text-2xl font-bold text-blue-700 dark:text-blue-300">{formatNumber(result.businessDays)}</p>
          </div>
          <div class="bg-amber-50 dark:bg-amber-900/20 rounded-lg p-4 border border-amber-200 dark:border-amber-800">
            <span class="text-xs font-medium text-amber-600 dark:text-amber-400">{t('dateDiff.weekendDays')}</span>
            <p class="text-2xl font-bold text-amber-700 dark:text-amber-300">{formatNumber(result.weekendDays)}</p>
          </div>
        </div>

        {#if result.isNegative}
          <p class="text-sm text-amber-600 dark:text-amber-400">
            {t('dateDiff.negativeNote')}
          </p>
        {/if}
      </div>
    </div>
  {:else}
    <div class="card">
      <div class="text-center py-8 text-gray-500 dark:text-gray-400">
        <CalendarDays class="w-12 h-12 mx-auto mb-3 opacity-40" />
        <p>{t('dateDiff.selectDates')}</p>
      </div>
    </div>
  {/if}
</div>
