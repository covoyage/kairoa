<script lang="ts">
  import { translationsStore } from '$lib/stores/i18n';
  import { ChevronLeft, ChevronRight, Calendar, Clock, ChevronsLeft, ChevronsRight } from 'lucide-svelte';

  // ── Props ──
  interface Props {
    value: string; // ISO datetime string: YYYY-MM-DDTHH:mm
    onChange: (value: string) => void;
    placeholder?: string;
    id?: string;
  }

  let { value, onChange, placeholder = '', id = '' }: Props = $props();

  // ── i18n ──
  let translations = $derived($translationsStore);

  function t(key: string): string {
    const keys = key.split('.');
    let val: any = translations;
    for (const k of keys) {
      val = val?.[k];
    }
    return val || key;
  }

  function getWeekDays(): string[] {
    const v = (translations as any)?.dateTimePicker?.weekDays;
    return v || ['Su', 'Mo', 'Tu', 'We', 'Th', 'Fr', 'Sa'];
  }

  function getMonthName(month: number): string {
    const v = (translations as any)?.dateTimePicker?.monthNames;
    if (Array.isArray(v) && v[month]) return v[month];
    return '';
  }

  // ── View mode ──
  type ViewMode = 'days' | 'months' | 'years';
  let viewMode = $state<ViewMode>('days');

  // ── State ──
  let open = $state(false);
  let containerRef = $state<HTMLDivElement | null>(null);
  let viewYear = $state(new Date().getFullYear());
  let viewMonth = $state(new Date().getMonth()); // 0-11
  let viewYearRangeStart = $state(new Date().getFullYear() - (new Date().getFullYear() % 10));
  let selectedDate = $state<Date | null>(null);
  let selectedHour = $state(0);
  let selectedMinute = $state(0);
  let showTimePicker = $state(false);

  // ── Initialize from value ──
  $effect(() => {
    if (value) {
      const d = new Date(value);
      if (!isNaN(d.getTime())) {
        selectedDate = new Date(d.getFullYear(), d.getMonth(), d.getDate());
        selectedHour = d.getHours();
        selectedMinute = d.getMinutes();
        viewYear = d.getFullYear();
        viewMonth = d.getMonth();
        viewYearRangeStart = d.getFullYear() - (d.getFullYear() % 10);
      }
    }
  });

  // ── Calendar grid (days view) ──
  let calendarDays = $derived.by(() => {
    const firstDay = new Date(viewYear, viewMonth, 1);
    const lastDay = new Date(viewYear, viewMonth + 1, 0);
    const firstDayWeek = firstDay.getDay();
    const daysInMonth = lastDay.getDate();
    const prevMonthLastDay = new Date(viewYear, viewMonth, 0).getDate();

    const days: { day: number; month: 'prev' | 'current' | 'next'; date: Date }[] = [];

    for (let i = firstDayWeek - 1; i >= 0; i--) {
      const day = prevMonthLastDay - i;
      days.push({ day, month: 'prev', date: new Date(viewYear, viewMonth - 1, day) });
    }
    for (let i = 1; i <= daysInMonth; i++) {
      days.push({ day: i, month: 'current', date: new Date(viewYear, viewMonth, i) });
    }
    const remaining = 42 - days.length;
    for (let i = 1; i <= remaining; i++) {
      days.push({ day: i, month: 'next', date: new Date(viewYear, viewMonth + 1, i) });
    }
    return days;
  });

  // ── Year grid (years view) ──
  let yearGrid = $derived.by(() => {
    const years: number[] = [];
    for (let i = 0; i < 12; i++) {
      years.push(viewYearRangeStart + i);
    }
    return years;
  });

  // ── Display value ──
  let displayValue = $derived.by(() => {
    if (!value) return placeholder;
    const d = new Date(value);
    if (isNaN(d.getTime())) return placeholder;
    const yyyy = d.getFullYear();
    const mm = String(d.getMonth() + 1).padStart(2, '0');
    const dd = String(d.getDate()).padStart(2, '0');
    const hh = String(d.getHours()).padStart(2, '0');
    const min = String(d.getMinutes()).padStart(2, '0');
    return `${yyyy}-${mm}-${dd} ${hh}:${min}`;
  });

  // ── Header label ──
  let headerLabel = $derived.by(() => {
    if (viewMode === 'days') {
      return `${getMonthName(viewMonth)} ${viewYear}`;
    }
    if (viewMode === 'months') {
      return `${viewYear}`;
    }
    // years
    return `${viewYearRangeStart} – ${viewYearRangeStart + 11}`;
  });

  // ── Navigation ──
  function prevPeriod() {
    if (viewMode === 'days') {
      if (viewMonth === 0) { viewMonth = 11; viewYear--; } else { viewMonth--; }
    } else if (viewMode === 'months') {
      viewYear--;
    } else {
      viewYearRangeStart -= 12;
    }
  }

  function nextPeriod() {
    if (viewMode === 'days') {
      if (viewMonth === 11) { viewMonth = 0; viewYear++; } else { viewMonth++; }
    } else if (viewMode === 'months') {
      viewYear++;
    } else {
      viewYearRangeStart += 12;
    }
  }

  function switchToMonths() {
    viewMode = 'months';
  }

  function switchToYears() {
    viewYearRangeStart = viewYear - (viewYear % 10);
    viewMode = 'years';
  }

  function switchToDays() {
    viewMode = 'days';
  }

  // ── Selection ──
  function selectMonth(month: number) {
    viewMonth = month;
    viewMode = 'days';
  }

  function selectYear(year: number) {
    viewYear = year;
    viewYearRangeStart = year - (year % 10);
    viewMode = 'months';
  }

  function selectDay(day: { day: number; month: 'prev' | 'current' | 'next'; date: Date }) {
    selectedDate = new Date(day.date);
    if (day.month === 'prev') { prevPeriod(); }
    else if (day.month === 'next') { nextPeriod(); }
    emitChange();
  }

  function selectHour(h: number) { selectedHour = h; emitChange(); }
  function selectMinute(m: number) { selectedMinute = m; emitChange(); }
  function toggleTimePicker() { showTimePicker = !showTimePicker; }

  function emitChange() {
    if (!selectedDate) return;
    const d = new Date(selectedDate.getFullYear(), selectedDate.getMonth(), selectedDate.getDate(), selectedHour, selectedMinute);
    const yyyy = d.getFullYear();
    const mm = String(d.getMonth() + 1).padStart(2, '0');
    const dd = String(d.getDate()).padStart(2, '0');
    const hh = String(d.getHours()).padStart(2, '0');
    const min = String(d.getMinutes()).padStart(2, '0');
    onChange(`${yyyy}-${mm}-${dd}T${hh}:${min}`);
  }

  // ── Helpers ──
  function isToday(date: Date): boolean {
    const today = new Date();
    return date.getDate() === today.getDate() && date.getMonth() === today.getMonth() && date.getFullYear() === today.getFullYear();
  }

  function isSelected(date: Date): boolean {
    if (!selectedDate) return false;
    return date.getDate() === selectedDate.getDate() && date.getMonth() === selectedDate.getMonth() && date.getFullYear() === selectedDate.getFullYear();
  }

  function isSelectedMonth(month: number): boolean {
    return month === viewMonth && viewYear === (selectedDate?.getFullYear() ?? -1);
  }

  function isSelectedYear(year: number): boolean {
    return year === (selectedDate?.getFullYear() ?? -1);
  }

  function isCurrentMonth(month: number): boolean {
    const now = new Date();
    return month === now.getMonth() && viewYear === now.getFullYear();
  }

  function isCurrentYear(year: number): boolean {
    return year === new Date().getFullYear();
  }

  // ── Click outside ──
  $effect(() => {
    if (!open) return;
    const handler = (e: PointerEvent) => {
      if (containerRef && !containerRef.contains(e.target as Node)) {
        open = false; showTimePicker = false; viewMode = 'days';
      }
    };
    setTimeout(() => document.addEventListener('pointerdown', handler), 0);
    return () => document.removeEventListener('pointerdown', handler);
  });

  // ── Quick actions ──
  function setNow() {
    const now = new Date();
    selectedDate = new Date(now.getFullYear(), now.getMonth(), now.getDate());
    selectedHour = now.getHours();
    selectedMinute = now.getMinutes();
    viewYear = now.getFullYear();
    viewMonth = now.getMonth();
    viewYearRangeStart = now.getFullYear() - (now.getFullYear() % 10);
    emitChange();
  }

  function clearTime() { selectedHour = 0; selectedMinute = 0; emitChange(); }
</script>

<div class="relative" bind:this={containerRef}>
  <!-- Trigger button -->
  <button
    {id}
    type="button"
    onclick={() => { open = !open; showTimePicker = false; viewMode = 'days'; }}
    class="input w-full text-left flex items-center justify-between cursor-pointer hover:bg-gray-50 dark:hover:bg-gray-700"
  >
    <span class="flex items-center gap-2">
      <Calendar class="w-4 h-4 text-gray-400 dark:text-gray-500 flex-shrink-0" />
      <span class={value ? 'text-gray-900 dark:text-gray-100' : 'text-gray-400 dark:text-gray-500'}>
        {displayValue}
      </span>
    </span>
    <Clock class="w-4 h-4 text-gray-400 dark:text-gray-500 flex-shrink-0" />
  </button>

  {#if open}
    <div class="absolute z-50 mt-1 bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-lg shadow-xl" style="width: 320px;">

      <!-- ═══ Header ═══ -->
      <div class="flex items-center justify-between px-2 py-2 border-b border-gray-200 dark:border-gray-700">
        <!-- Left: prev or jump-back -->
        <button type="button" onclick={prevPeriod} class="p-1.5 rounded hover:bg-gray-100 dark:hover:bg-gray-700 text-gray-600 dark:text-gray-400">
          <ChevronLeft class="w-4 h-4" />
        </button>

        <!-- Center: clickable label -->
        <div class="flex items-center gap-1">
          {#if viewMode === 'days'}
            <!-- In days mode: month is clickable → months view, year is clickable → years view -->
            <button type="button" onclick={switchToMonths}
              class="px-2 py-0.5 text-sm font-semibold text-gray-900 dark:text-gray-100 rounded hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors">
              {getMonthName(viewMonth)}
            </button>
            <button type="button" onclick={switchToYears}
              class="px-2 py-0.5 text-sm font-semibold text-gray-900 dark:text-gray-100 rounded hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors">
              {viewYear}
            </button>
          {:else if viewMode === 'months'}
            <button type="button" onclick={switchToYears}
              class="px-2 py-0.5 text-sm font-semibold text-gray-900 dark:text-gray-100 rounded hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors">
              {viewYear}
            </button>
          {:else}
            <span class="px-2 py-0.5 text-sm font-semibold text-gray-900 dark:text-gray-100">
              {viewYearRangeStart} – {viewYearRangeStart + 11}
            </span>
          {/if}
        </div>

        <!-- Right: next -->
        <button type="button" onclick={nextPeriod} class="p-1.5 rounded hover:bg-gray-100 dark:hover:bg-gray-700 text-gray-600 dark:text-gray-400">
          <ChevronRight class="w-4 h-4" />
        </button>
      </div>

      <!-- ═══ Days view ═══ -->
      {#if viewMode === 'days'}
        <div class="p-3">
          <!-- Weekday headers -->
          <div class="grid grid-cols-7 gap-1 mb-2">
            {#each getWeekDays() as day}
              <div class="text-center text-xs font-medium text-gray-500 dark:text-gray-400 py-1">{day}</div>
            {/each}
          </div>
          <!-- Days grid -->
          <div class="grid grid-cols-7 gap-1">
            {#each calendarDays as day}
              <button type="button" onclick={() => selectDay(day)}
                class="aspect-square flex items-center justify-center rounded text-sm transition-colors
                  {day.month === 'current' ? '' : 'text-gray-400 dark:text-gray-600'}
                  {isSelected(day.date)
                    ? 'bg-primary-600 text-white hover:bg-primary-700 font-semibold'
                    : day.month === 'current'
                      ? isToday(day.date)
                        ? 'bg-primary-100 dark:bg-primary-900/30 text-primary-600 dark:text-primary-400 hover:bg-primary-200 dark:hover:bg-primary-900/40 font-medium'
                        : 'text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700'
                      : 'hover:bg-gray-100 dark:hover:bg-gray-700'}">
                {day.day}
              </button>
            {/each}
          </div>
        </div>
      {/if}

      <!-- ═══ Months view ═══ -->
      {#if viewMode === 'months'}
        <div class="p-3">
          <div class="grid grid-cols-3 gap-2">
            {#each Array(12).fill(0).map((_, i) => i) as m}
              <button type="button" onclick={() => selectMonth(m)}
                class="py-2.5 rounded-lg text-sm transition-colors
                  {isSelectedMonth(m)
                    ? 'bg-primary-600 text-white font-semibold'
                    : isCurrentMonth(m)
                      ? 'bg-primary-100 dark:bg-primary-900/30 text-primary-600 dark:text-primary-400 hover:bg-primary-200 dark:hover:bg-primary-900/40'
                      : 'text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700'}">
                {getMonthName(m)}
              </button>
            {/each}
          </div>
        </div>
      {/if}

      <!-- ═══ Years view ═══ -->
      {#if viewMode === 'years'}
        <div class="p-3">
          <div class="grid grid-cols-3 gap-2">
            {#each yearGrid as year}
              <button type="button" onclick={() => selectYear(year)}
                class="py-2.5 rounded-lg text-sm transition-colors
                  {isSelectedYear(year)
                    ? 'bg-primary-600 text-white font-semibold'
                    : isCurrentYear(year)
                      ? 'bg-primary-100 dark:bg-primary-900/30 text-primary-600 dark:text-primary-400 hover:bg-primary-200 dark:hover:bg-primary-900/40'
                      : 'text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700'}">
                {year}
              </button>
            {/each}
          </div>
        </div>
      {/if}

      <!-- ═══ Time section ═══ -->
      <div class="border-t border-gray-200 dark:border-gray-700">
        <button type="button" onclick={toggleTimePicker}
          class="w-full flex items-center justify-between px-3 py-2 hover:bg-gray-50 dark:hover:bg-gray-700/50 transition-colors">
          <span class="flex items-center gap-2 text-sm text-gray-600 dark:text-gray-400">
            <Clock class="w-4 h-4" />
            {t('dateTimePicker.time')}
          </span>
          <span class="text-sm font-mono text-gray-900 dark:text-gray-100">
            {String(selectedHour).padStart(2, '0')}:{String(selectedMinute).padStart(2, '0')}
          </span>
        </button>

        {#if showTimePicker}
          <div class="px-3 pb-3 pt-1 space-y-3">
            <!-- Hours -->
            <div>
              <span class="text-xs text-gray-500 dark:text-gray-400 mb-1.5 block">{t('dateTimePicker.hour')}</span>
              <div class="grid grid-cols-6 gap-1 max-h-32 overflow-y-auto">
                {#each Array(24).fill(0).map((_, i) => i) as h}
                  <button type="button" onclick={() => selectHour(h)}
                    class="py-1 rounded text-xs transition-colors
                      {selectedHour === h ? 'bg-primary-600 text-white font-medium' : 'bg-gray-100 dark:bg-gray-700 text-gray-700 dark:text-gray-300 hover:bg-gray-200 dark:hover:bg-gray-600'}">
                    {String(h).padStart(2, '0')}
                  </button>
                {/each}
              </div>
            </div>
            <!-- Minutes -->
            <div>
              <span class="text-xs text-gray-500 dark:text-gray-400 mb-1.5 block">{t('dateTimePicker.minute')}</span>
              <div class="grid grid-cols-6 gap-1 max-h-24 overflow-y-auto">
                {#each Array(60).fill(0).map((_, i) => i) as m}
                  <button type="button" onclick={() => selectMinute(m)}
                    class="py-1 rounded text-xs transition-colors
                      {selectedMinute === m ? 'bg-primary-600 text-white font-medium' : 'bg-gray-100 dark:bg-gray-700 text-gray-700 dark:text-gray-300 hover:bg-gray-200 dark:hover:bg-gray-600'}">
                    {String(m).padStart(2, '0')}
                  </button>
                {/each}
              </div>
            </div>
          </div>
        {/if}
      </div>

      <!-- ═══ Footer ═══ -->
      <div class="flex items-center justify-between px-3 py-2 border-t border-gray-200 dark:border-gray-700">
        <button type="button" onclick={setNow}
          class="text-xs px-3 py-1 rounded bg-gray-100 dark:bg-gray-700 hover:bg-gray-200 dark:hover:bg-gray-600 text-gray-700 dark:text-gray-300 transition-colors">
          {t('dateTimePicker.now')}
        </button>
        <button type="button" onclick={clearTime}
          class="text-xs px-3 py-1 rounded bg-gray-100 dark:bg-gray-700 hover:bg-gray-200 dark:hover:bg-gray-600 text-gray-700 dark:text-gray-300 transition-colors">
          {t('dateTimePicker.clearTime')}
        </button>
        <button type="button" onclick={() => { open = false; showTimePicker = false; viewMode = 'days'; }}
          class="text-xs px-3 py-1 rounded bg-primary-600 hover:bg-primary-700 text-white transition-colors font-medium">
          {t('dateTimePicker.ok')}
        </button>
      </div>
    </div>
  {/if}
</div>
