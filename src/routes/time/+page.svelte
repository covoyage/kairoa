<script lang="ts">
  import { translationsStore } from '$lib/stores/i18n';
  import { ChevronDown, Search, X } from 'lucide-svelte';
  
  type ConversionType = 'timestamp' | 'date';
  
  let conversionType = $state<ConversionType>('timestamp');
  let timestampInput = $state('');
  let dateInput = $state('');
  let timezone = $state(Intl.DateTimeFormat().resolvedOptions().timeZone);
  let timestampResult = $state('');
  let dateResult = $state<{ spaceFormat: string; isoFormat: string } | null>(null);
  let timestampCopied = $state(false);
  let dateCopiedSpace = $state(false);
  let dateCopiedIso = $state(false);
  let timezoneSearchTimestamp = $state('');
  let timezoneSearchDate = $state('');
  let timezoneOpenTimestamp = $state(false);
  let timezoneOpenDate = $state(false);
  let timezoneDropdownRefTimestamp = $state<HTMLDivElement | null>(null);
  let timezoneDropdownRefDate = $state<HTMLDivElement | null>(null);

  // 所有标准 IANA 时区列表
  const timezones: string[] = [
    'Africa/Abidjan', 'Africa/Accra', 'Africa/Addis_Ababa', 'Africa/Algiers', 'Africa/Asmara',
    'Africa/Bamako', 'Africa/Bangui', 'Africa/Banjul', 'Africa/Bissau', 'Africa/Blantyre',
    'Africa/Brazzaville', 'Africa/Bujumbura', 'Africa/Cairo', 'Africa/Casablanca', 'Africa/Ceuta',
    'Africa/Conakry', 'Africa/Dakar', 'Africa/Dar_es_Salaam', 'Africa/Djibouti', 'Africa/Douala',
    'Africa/El_Aaiun', 'Africa/Freetown', 'Africa/Gaborone', 'Africa/Harare', 'Africa/Johannesburg',
    'Africa/Juba', 'Africa/Kampala', 'Africa/Khartoum', 'Africa/Kigali', 'Africa/Kinshasa',
    'Africa/Lagos', 'Africa/Libreville', 'Africa/Lome', 'Africa/Luanda', 'Africa/Lubumbashi',
    'Africa/Lusaka', 'Africa/Malabo', 'Africa/Maputo', 'Africa/Maseru', 'Africa/Mbabane',
    'Africa/Mogadishu', 'Africa/Monrovia', 'Africa/Nairobi', 'Africa/Ndjamena', 'Africa/Niamey',
    'Africa/Nouakchott', 'Africa/Ouagadougou', 'Africa/Porto-Novo', 'Africa/Sao_Tome', 'Africa/Tripoli',
    'Africa/Tunis', 'Africa/Windhoek',
    'America/Adak', 'America/Anchorage', 'America/Anguilla', 'America/Antigua', 'America/Araguaina',
    'America/Argentina/Buenos_Aires', 'America/Argentina/Catamarca', 'America/Argentina/Cordoba',
    'America/Argentina/Jujuy', 'America/Argentina/La_Rioja', 'America/Argentina/Mendoza',
    'America/Argentina/Rio_Gallegos', 'America/Argentina/Salta', 'America/Argentina/San_Juan',
    'America/Argentina/San_Luis', 'America/Argentina/Tucuman', 'America/Argentina/Ushuaia',
    'America/Aruba', 'America/Asuncion', 'America/Atikokan', 'America/Bahia', 'America/Bahia_Banderas',
    'America/Barbados', 'America/Belem', 'America/Belize', 'America/Blanc-Sablon', 'America/Boa_Vista',
    'America/Bogota', 'America/Boise', 'America/Cambridge_Bay', 'America/Campo_Grande', 'America/Cancun',
    'America/Caracas', 'America/Cayenne', 'America/Cayman', 'America/Chicago', 'America/Chihuahua',
    'America/Costa_Rica', 'America/Creston', 'America/Cuiaba', 'America/Curacao', 'America/Danmarkshavn',
    'America/Dawson', 'America/Dawson_Creek', 'America/Denver', 'America/Detroit', 'America/Dominica',
    'America/Edmonton', 'America/Eirunepe', 'America/El_Salvador', 'America/Fort_Nelson', 'America/Fortaleza',
    'America/Glace_Bay', 'America/Godthab', 'America/Goose_Bay', 'America/Grand_Turk', 'America/Grenada',
    'America/Guadeloupe', 'America/Guatemala', 'America/Guayaquil', 'America/Guyana', 'America/Halifax',
    'America/Havana', 'America/Hermosillo', 'America/Indiana/Indianapolis', 'America/Indiana/Knox',
    'America/Indiana/Marengo', 'America/Indiana/Petersburg', 'America/Indiana/Tell_City',
    'America/Indiana/Vevay', 'America/Indiana/Vincennes', 'America/Indiana/Winamac', 'America/Inuvik',
    'America/Iqaluit', 'America/Jamaica', 'America/Juneau', 'America/Kentucky/Louisville',
    'America/Kentucky/Monticello', 'America/Kralendijk', 'America/La_Paz', 'America/Lima',
    'America/Los_Angeles', 'America/Lower_Princes', 'America/Maceio', 'America/Managua',
    'America/Manaus', 'America/Marigot', 'America/Martinique', 'America/Matamoros', 'America/Mazatlan',
    'America/Menominee', 'America/Merida', 'America/Metlakatla', 'America/Mexico_City', 'America/Miquelon',
    'America/Moncton', 'America/Monterrey', 'America/Montevideo', 'America/Montserrat', 'America/Nassau',
    'America/New_York', 'America/Nipigon', 'America/Nome', 'America/Noronha', 'America/North_Dakota/Beulah',
    'America/North_Dakota/Center', 'America/North_Dakota/New_Salem', 'America/Ojinaga', 'America/Panama',
    'America/Pangnirtung', 'America/Paramaribo', 'America/Phoenix', 'America/Port-au-Prince',
    'America/Port_of_Spain', 'America/Porto_Velho', 'America/Puerto_Rico', 'America/Punta_Arenas',
    'America/Rainy_River', 'America/Rankin_Inlet', 'America/Recife', 'America/Regina', 'America/Resolute',
    'America/Rio_Branco', 'America/Santarem', 'America/Santiago', 'America/Santo_Domingo', 'America/Sao_Paulo',
    'America/Scoresbysund', 'America/Sitka', 'America/St_Barthelemy', 'America/St_Johns', 'America/St_Kitts',
    'America/St_Lucia', 'America/St_Thomas', 'America/St_Vincent', 'America/Swift_Current', 'America/Tegucigalpa',
    'America/Thule', 'America/Thunder_Bay', 'America/Tijuana', 'America/Toronto', 'America/Tortola',
    'America/Vancouver', 'America/Whitehorse', 'America/Winnipeg', 'America/Yakutat', 'America/Yellowknife',
    'Antarctica/Casey', 'Antarctica/Davis', 'Antarctica/DumontDUrville', 'Antarctica/Macquarie',
    'Antarctica/Mawson', 'Antarctica/McMurdo', 'Antarctica/Palmer', 'Antarctica/Rothera', 'Antarctica/Syowa',
    'Antarctica/Troll', 'Antarctica/Vostok',
    'Arctic/Longyearbyen',
    'Asia/Aden', 'Asia/Almaty', 'Asia/Amman', 'Asia/Anadyr', 'Asia/Aqtau', 'Asia/Aqtobe', 'Asia/Ashgabat',
    'Asia/Atyrau', 'Asia/Baghdad', 'Asia/Bahrain', 'Asia/Baku', 'Asia/Bangkok', 'Asia/Barnaul', 'Asia/Beirut',
    'Asia/Bishkek', 'Asia/Brunei', 'Asia/Chita', 'Asia/Choibalsan', 'Asia/Colombo', 'Asia/Damascus', 'Asia/Dhaka',
    'Asia/Dili', 'Asia/Dubai', 'Asia/Dushanbe', 'Asia/Famagusta', 'Asia/Gaza', 'Asia/Hebron', 'Asia/Ho_Chi_Minh',
    'Asia/Hong_Kong', 'Asia/Hovd', 'Asia/Irkutsk', 'Asia/Jakarta', 'Asia/Jayapura', 'Asia/Jerusalem', 'Asia/Kabul',
    'Asia/Kamchatka', 'Asia/Karachi', 'Asia/Kathmandu', 'Asia/Khandyga', 'Asia/Kolkata', 'Asia/Krasnoyarsk',
    'Asia/Kuala_Lumpur', 'Asia/Kuching', 'Asia/Kuwait', 'Asia/Macau', 'Asia/Magadan', 'Asia/Makassar',
    'Asia/Manila', 'Asia/Muscat', 'Asia/Nicosia', 'Asia/Novokuznetsk', 'Asia/Novosibirsk', 'Asia/Omsk',
    'Asia/Oral', 'Asia/Phnom_Penh', 'Asia/Pontianak', 'Asia/Pyongyang', 'Asia/Qatar', 'Asia/Qostanay',
    'Asia/Qyzylorda', 'Asia/Riyadh', 'Asia/Sakhalin', 'Asia/Samarkand', 'Asia/Seoul', 'Asia/Shanghai',
    'Asia/Singapore', 'Asia/Srednekolymsk', 'Asia/Taipei', 'Asia/Tashkent', 'Asia/Tbilisi', 'Asia/Tehran',
    'Asia/Thimphu', 'Asia/Tokyo', 'Asia/Tomsk', 'Asia/Ulaanbaatar', 'Asia/Urumqi', 'Asia/Ust-Nera',
    'Asia/Vientiane', 'Asia/Vladivostok', 'Asia/Yakutsk', 'Asia/Yangon', 'Asia/Yekaterinburg', 'Asia/Yerevan',
    'Atlantic/Azores', 'Atlantic/Bermuda', 'Atlantic/Canary', 'Atlantic/Cape_Verde', 'Atlantic/Faroe',
    'Atlantic/Madeira', 'Atlantic/Reykjavik', 'Atlantic/South_Georgia', 'Atlantic/St_Helena', 'Atlantic/Stanley',
    'Australia/Adelaide', 'Australia/Brisbane', 'Australia/Broken_Hill', 'Australia/Currie', 'Australia/Darwin',
    'Australia/Eucla', 'Australia/Hobart', 'Australia/Lindeman', 'Australia/Lord_Howe', 'Australia/Melbourne',
    'Australia/Perth', 'Australia/Sydney',
    'Europe/Amsterdam', 'Europe/Andorra', 'Europe/Astrakhan', 'Europe/Athens', 'Europe/Belgrade',
    'Europe/Berlin', 'Europe/Bratislava', 'Europe/Brussels', 'Europe/Bucharest', 'Europe/Budapest',
    'Europe/Busingen', 'Europe/Chisinau', 'Europe/Copenhagen', 'Europe/Dublin', 'Europe/Gibraltar',
    'Europe/Guernsey', 'Europe/Helsinki', 'Europe/Isle_of_Man', 'Europe/Istanbul', 'Europe/Jersey',
    'Europe/Kaliningrad', 'Europe/Kiev', 'Europe/Kirov', 'Europe/Lisbon', 'Europe/Ljubljana', 'Europe/London',
    'Europe/Luxembourg', 'Europe/Madrid', 'Europe/Malta', 'Europe/Mariehamn', 'Europe/Minsk', 'Europe/Monaco',
    'Europe/Moscow', 'Europe/Oslo', 'Europe/Paris', 'Europe/Podgorica', 'Europe/Prague', 'Europe/Riga',
    'Europe/Rome', 'Europe/Samara', 'Europe/San_Marino', 'Europe/Sarajevo', 'Europe/Saratov', 'Europe/Simferopol',
    'Europe/Skopje', 'Europe/Sofia', 'Europe/Stockholm', 'Europe/Tallinn', 'Europe/Tirane', 'Europe/Ulyanovsk',
    'Europe/Uzhgorod', 'Europe/Vaduz', 'Europe/Vatican', 'Europe/Vienna', 'Europe/Vilnius', 'Europe/Volgograd',
    'Europe/Warsaw', 'Europe/Zagreb', 'Europe/Zaporozhye', 'Europe/Zurich',
    'Indian/Antananarivo', 'Indian/Chagos', 'Indian/Christmas', 'Indian/Cocos', 'Indian/Comoro',
    'Indian/Kerguelen', 'Indian/Mahe', 'Indian/Maldives', 'Indian/Mauritius', 'Indian/Mayotte',
    'Indian/Reunion',
    'Pacific/Apia', 'Pacific/Auckland', 'Pacific/Bougainville', 'Pacific/Chatham', 'Pacific/Chuuk',
    'Pacific/Easter', 'Pacific/Efate', 'Pacific/Enderbury', 'Pacific/Fakaofo', 'Pacific/Fiji',
    'Pacific/Funafuti', 'Pacific/Galapagos', 'Pacific/Gambier', 'Pacific/Guadalcanal', 'Pacific/Guam',
    'Pacific/Honolulu', 'Pacific/Kiritimati', 'Pacific/Kosrae', 'Pacific/Kwajalein', 'Pacific/Majuro',
    'Pacific/Marquesas', 'Pacific/Midway', 'Pacific/Nauru', 'Pacific/Niue', 'Pacific/Norfolk',
    'Pacific/Noumea', 'Pacific/Pago_Pago', 'Pacific/Palau', 'Pacific/Pitcairn', 'Pacific/Pohnpei',
    'Pacific/Port_Moresby', 'Pacific/Rarotonga', 'Pacific/Saipan', 'Pacific/Tahiti', 'Pacific/Tarawa',
    'Pacific/Tongatapu', 'Pacific/Wake', 'Pacific/Wallis',
    'UTC'
  ].sort();
  
  // 确保当前时区在列表中（通常在列表中，但为了保险）
  const availableTimezones = $derived.by(() => {
    const tz = timezone;
    if (!timezones.includes(tz)) {
      return [tz, ...timezones];
    }
    return timezones;
  });

  // 过滤时区列表
  const filteredTimezonesTimestamp = $derived.by(() => {
    const search = timezoneSearchTimestamp;
    if (!search || !search.trim()) {
      // 没有搜索时，也滚动到当前选中的时区
      if (timezoneOpenTimestamp) {
        setTimeout(() => scrollToMatchedTimezone('timestamp'), 100);
      }
      return availableTimezones;
    }
    const lowerSearch = search.toLowerCase();
    const filtered = availableTimezones.filter(tz => tz.toLowerCase().includes(lowerSearch));
    // 搜索时自动滚动到匹配项
    setTimeout(() => scrollToMatchedTimezone('timestamp'), 100);
    return filtered;
  });

  const filteredTimezonesDate = $derived.by(() => {
    const search = timezoneSearchDate;
    if (!search || !search.trim()) {
      // 没有搜索时，也滚动到当前选中的时区
      if (timezoneOpenDate) {
        setTimeout(() => scrollToMatchedTimezone('date'), 100);
      }
      return availableTimezones;
    }
    const lowerSearch = search.toLowerCase();
    const filtered = availableTimezones.filter(tz => tz.toLowerCase().includes(lowerSearch));
    // 搜索时自动滚动到匹配项
    setTimeout(() => scrollToMatchedTimezone('date'), 100);
    return filtered;
  });

  function selectTimezone(tz: string, type: 'timestamp' | 'date') {
    timezone = tz;
    if (type === 'timestamp') {
      timezoneSearchTimestamp = '';
      timezoneOpenTimestamp = false;
    } else {
      timezoneSearchDate = '';
      timezoneOpenDate = false;
    }
  }

  // 滚动到匹配的时区选项
  function scrollToMatchedTimezone(type: 'timestamp' | 'date') {
    setTimeout(() => {
      const container = type === 'timestamp' ? timezoneDropdownRefTimestamp : timezoneDropdownRefDate;
      if (!container) return;
      
      const dropdown = container.querySelector('.timezone-dropdown');
      if (!dropdown) return;
      
      const search = type === 'timestamp' ? timezoneSearchTimestamp : timezoneSearchDate;
      const filtered = type === 'timestamp' ? filteredTimezonesTimestamp : filteredTimezonesDate;
      
      if (filtered.length === 0) return;
      
      // 如果有搜索关键词，滚动到第一个匹配项
      // 如果没有搜索关键词，滚动到当前选中的时区
      let targetButton: HTMLElement | null = null;
      
      if (search && search.trim()) {
        // 滚动到第一个匹配项
        targetButton = dropdown.querySelector('button') as HTMLElement;
      } else {
        // 滚动到当前选中的时区
        const buttons = dropdown.querySelectorAll('button');
        for (const btn of buttons) {
          if (btn.textContent?.trim() === timezone) {
            targetButton = btn as HTMLElement;
            break;
          }
        }
      }
      
      if (targetButton) {
        targetButton.scrollIntoView({ block: 'center', behavior: 'auto' });
      }
    }, 100);
  }

  // 点击外部关闭下拉列表
  function handleClickOutside(event: MouseEvent, type: 'timestamp' | 'date') {
    const ref = type === 'timestamp' ? timezoneDropdownRefTimestamp : timezoneDropdownRefDate;
    if (ref && !ref.contains(event.target as Node)) {
      if (type === 'timestamp') {
        timezoneOpenTimestamp = false;
        timezoneSearchTimestamp = '';
      } else {
        timezoneOpenDate = false;
        timezoneSearchDate = '';
      }
    }
  }

  // 初始化点击外部监听
  $effect(() => {
    if (timezoneOpenTimestamp || timezoneOpenDate) {
      const handler = (e: MouseEvent) => {
        handleClickOutside(e, 'timestamp');
        handleClickOutside(e, 'date');
      };
      // 延迟添加监听器，避免立即触发
      setTimeout(() => {
        document.addEventListener('click', handler);
      }, 0);
      return () => {
        setTimeout(() => {
          document.removeEventListener('click', handler);
        }, 0);
      };
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

  function formatDate(date: Date, tz: string): { spaceFormat: string; isoFormat: string } {
    const formatter = new Intl.DateTimeFormat('en-US', {
      timeZone: tz,
      year: 'numeric',
      month: '2-digit',
      day: '2-digit',
      hour: '2-digit',
      minute: '2-digit',
      second: '2-digit',
      hour12: false
    });
    
    // 格式化日期，得到类似 "MM/DD/YYYY, HH:mm:ss" 的格式
    const parts = formatter.formatToParts(date);
    const year = parts.find(p => p.type === 'year')?.value || '';
    const month = parts.find(p => p.type === 'month')?.value.padStart(2, '0') || '';
    const day = parts.find(p => p.type === 'day')?.value.padStart(2, '0') || '';
    const hour = parts.find(p => p.type === 'hour')?.value.padStart(2, '0') || '';
    const minute = parts.find(p => p.type === 'minute')?.value.padStart(2, '0') || '';
    const second = parts.find(p => p.type === 'second')?.value.padStart(2, '0') || '';
    
    const spaceFormat = `${year}-${month}-${day} ${hour}:${minute}:${second}`;
    const isoFormat = `${year}-${month}-${day}T${hour}:${minute}:${second}`;
    
    return { spaceFormat, isoFormat };
  }

  function convertTimestampToDate() {
    if (!timestampInput.trim()) {
      dateResult = null;
      return;
    }

    try {
      const ts = parseInt(timestampInput);
      if (isNaN(ts)) {
        dateResult = { spaceFormat: 'Invalid timestamp', isoFormat: 'Invalid timestamp' };
        return;
      }

      // 如果时间戳是10位数字，认为是秒；如果是13位，认为是毫秒
      const timestamp = ts.toString().length === 10 ? ts * 1000 : ts;
      const date = new Date(timestamp);
      
      if (isNaN(date.getTime())) {
        dateResult = { spaceFormat: 'Invalid timestamp', isoFormat: 'Invalid timestamp' };
        return;
      }

      dateResult = formatDate(date, timezone);
    } catch (error) {
      const errorMsg = `Error: ${error instanceof Error ? error.message : 'Unknown error'}`;
      dateResult = { spaceFormat: errorMsg, isoFormat: errorMsg };
    }
  }

  function convertDateToTimestamp() {
    if (!dateInput.trim()) {
      timestampResult = '';
      return;
    }

    try {
      // 支持多种日期格式输入
      let dateStr = dateInput.trim();
      
      // 如果输入格式是 YYYY-MM-DDTHH:mm:ss 或 YYYY-MM-DDTHH:mm，直接使用
      // 如果输入格式是 YYYY-MM-DD HH:mm:ss 或 YYYY-MM-DD HH:mm，转换为标准格式
      dateStr = dateStr.replace(' ', 'T');
      
      // 如果只有日期部分，添加默认时间 00:00:00
      if (!dateStr.includes('T')) {
        dateStr = dateStr + 'T00:00:00';
      }
      
      // 解析日期字符串
      const [datePart, timePart] = dateStr.split('T');
      if (!datePart) {
        timestampResult = 'Invalid date format';
        return;
      }
      
      const [year, month, day] = datePart.split('-').map(Number);
      const timeParts = (timePart || '00:00:00').split(':');
      const hour = Number(timeParts[0] || 0);
      const minute = Number(timeParts[1] || 0);
      const second = Number(timeParts[2] || 0);
      
      // 构造一个 ISO 格式的日期字符串
      const isoString = `${year}-${String(month).padStart(2, '0')}-${String(day).padStart(2, '0')}T${String(hour).padStart(2, '0')}:${String(minute).padStart(2, '0')}:${String(second).padStart(2, '0')}`;
      
      // 创建一个临时日期对象来获取时区信息
      // 首先创建一个本地时间的 Date 对象
      const localDate = new Date(isoString);
      
      if (isNaN(localDate.getTime())) {
        timestampResult = 'Invalid date';
        return;
      }

      // 获取当前本地时区和指定时区的偏移量差
      const localTimezone = Intl.DateTimeFormat().resolvedOptions().timeZone;
      
      // 获取指定时区的偏移量（相对于 UTC，单位：分钟）
      const targetOffset = getTimezoneOffset(timezone);
      const localOffset = getTimezoneOffset(localTimezone);
      
      // 计算时区差异
      const offsetDiff = (targetOffset - localOffset) * 60000;
      
      // 将本地时间转换为 UTC，然后应用时区差异
      const utcTime = localDate.getTime() - offsetDiff;
      const ts = Math.floor(utcTime / 1000);
      
      timestampResult = ts.toString();
    } catch (error) {
      timestampResult = `Error: ${error instanceof Error ? error.message : 'Unknown error'}`;
    }
  }

  function getTimezoneOffset(tz: string): number {
    // 获取指定时区相对于 UTC 的偏移量（分钟）
    // 使用当前时间来计算，考虑夏令时
    const now = new Date();
    const utcTime = new Date(now.toLocaleString('en-US', { timeZone: 'UTC' }));
    const tzTime = new Date(now.toLocaleString('en-US', { timeZone: tz }));
    return (tzTime.getTime() - utcTime.getTime()) / 60000;
  }

  function setCurrentTime() {
    const now = new Date();
    const ts = Math.floor(now.getTime() / 1000);
    timestampInput = ts.toString();
    convertTimestampToDate();
  }

  function clear() {
    if (conversionType === 'timestamp') {
      timestampInput = '';
      dateResult = null;
    } else {
      dateInput = '';
      timestampResult = '';
    }
  }

  function switchConversionType(type: ConversionType) {
    conversionType = type;
    timestampInput = '';
    dateInput = '';
    timestampResult = '';
    dateResult = null;
  }

  async function copyToClipboard(text: string, type: 'timestamp' | 'date-space' | 'date-iso') {
    if (!text) return;
    
    try {
      await navigator.clipboard.writeText(text);
      if (type === 'timestamp') {
        timestampCopied = true;
        setTimeout(() => { timestampCopied = false; }, 1000);
      } else if (type === 'date-space') {
        dateCopiedSpace = true;
        setTimeout(() => { dateCopiedSpace = false; }, 1000);
      } else if (type === 'date-iso') {
        dateCopiedIso = true;
        setTimeout(() => { dateCopiedIso = false; }, 1000);
      }
    } catch (error) {
      console.error('Failed to copy:', error);
    }
  }

  // 监听输入变化，自动转换
  $effect(() => {
    if (conversionType === 'timestamp' && timestampInput) {
      convertTimestampToDate();
    } else {
      dateResult = null;
    }
  });

  $effect(() => {
    if (conversionType === 'date' && dateInput) {
      convertDateToTimestamp();
    } else {
      timestampResult = '';
    }
  });

  $effect(() => {
    if (timezone) {
      if (conversionType === 'timestamp' && timestampInput) {
        convertTimestampToDate();
      }
      if (conversionType === 'date' && dateInput) {
        convertDateToTimestamp();
      }
    }
  });
</script>

<div class="w-full ml-0 mr-0 p-2 space-y-6">
  <!-- 输入区域卡片 -->
  <div class="card">
    <div class="space-y-4">
      <!-- 转换类型切换 -->
      <div class="border-b border-gray-200 dark:border-gray-700">
        <div class="flex gap-6">
          <button
            onclick={() => switchConversionType('timestamp')}
            class="px-4 py-2 relative transition-colors font-medium {conversionType === 'timestamp'
              ? 'text-primary-600 dark:text-primary-400'
              : 'text-gray-500 dark:text-gray-400 hover:text-gray-700 dark:hover:text-gray-300'}"
          >
            {t('time.timestampToDate')}
            {#if conversionType === 'timestamp'}
              <span class="absolute bottom-0 left-0 right-0 h-0.5 bg-primary-600 dark:bg-primary-400"></span>
            {/if}
          </button>
          <button
            onclick={() => switchConversionType('date')}
            class="px-4 py-2 relative transition-colors font-medium {conversionType === 'date'
              ? 'text-primary-600 dark:text-primary-400'
              : 'text-gray-500 dark:text-gray-400 hover:text-gray-700 dark:hover:text-gray-300'}"
          >
            {t('time.dateToTimestamp')}
            {#if conversionType === 'date'}
              <span class="absolute bottom-0 left-0 right-0 h-0.5 bg-primary-600 dark:bg-primary-400"></span>
            {/if}
          </button>
        </div>
      </div>

      {#if conversionType === 'timestamp'}
        <!-- 时间戳转日期 -->
        <div class="space-y-4">
          <div>
            <input
              id="timestamp-input"
              type="text"
              bind:value={timestampInput}
              placeholder={t('time.timestamp')}
              class="input"
            />
          </div>
          
          <div>
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
              {t('time.timezone')}
            </label>
            <div class="relative" bind:this={timezoneDropdownRefTimestamp}>
              <button
                type="button"
                onclick={() => {
                  timezoneOpenTimestamp = !timezoneOpenTimestamp;
                  if (timezoneOpenTimestamp) {
                    timezoneSearchTimestamp = '';
                    // 打开时滚动到当前选中的时区
                    setTimeout(() => scrollToMatchedTimezone('timestamp'), 150);
                  }
                }}
                class="input w-full pr-10 text-left flex items-center justify-between cursor-pointer hover:bg-gray-50 dark:hover:bg-gray-700"
              >
                <span class="flex-1 truncate">{timezone}</span>
                <ChevronDown class="w-4 h-4 text-gray-400 dark:text-gray-500 flex-shrink-0 transition-transform {timezoneOpenTimestamp ? 'rotate-180' : ''}" />
              </button>
              {#if timezoneOpenTimestamp}
                <div class="absolute z-50 w-full mt-1 bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-lg shadow-lg overflow-hidden flex flex-col" style="max-height: 300px;">
                  <!-- 搜索框 -->
                  <div class="p-2 border-b border-gray-200 dark:border-gray-700 flex-shrink-0">
                    <div class="relative">
                      <div class="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none">
                        <Search class="w-4 h-4 text-gray-400 dark:text-gray-500" />
                      </div>
                      <input
                        type="text"
                        bind:value={timezoneSearchTimestamp}
                        oninput={(e) => {
                          timezoneSearchTimestamp = (e.target as HTMLInputElement).value;
                        }}
                        placeholder={t('time.searchTimezone')}
                        class="input w-full pl-10 pr-8 text-sm"
                        autofocus
                      />
                      {#if timezoneSearchTimestamp}
                        <button
                          type="button"
                          onclick={() => {
                            timezoneSearchTimestamp = '';
                          }}
                          class="absolute inset-y-0 right-0 pr-2 flex items-center text-gray-400 dark:text-gray-500 hover:text-gray-600 dark:hover:text-gray-300"
                        >
                          <X class="w-3 h-3" />
                        </button>
                      {/if}
                    </div>
                  </div>
                  <!-- 时区列表 -->
                  <div class="overflow-y-auto flex-1 timezone-dropdown">
                    {#if filteredTimezonesTimestamp && filteredTimezonesTimestamp.length > 0}
                      {#each filteredTimezonesTimestamp as tz}
                        <button
                          type="button"
                          onclick={() => selectTimezone(tz, 'timestamp')}
                          class="w-full text-left px-4 py-2 hover:bg-gray-100 dark:hover:bg-gray-700 {timezone === tz ? 'bg-primary-50 dark:bg-primary-900/20 text-primary-600 dark:text-primary-400' : 'text-gray-900 dark:text-gray-100'}"
                        >
                          {tz}
                        </button>
                      {/each}
                    {:else}
                      <div class="px-4 py-2 text-gray-500 dark:text-gray-400 text-sm">
                        {t('time.noTimezoneFound')}
                      </div>
                    {/if}
                  </div>
                </div>
              {/if}
            </div>
          </div>
          
          <div class="flex gap-2">
            <button 
              onclick={setCurrentTime}
              class="btn-secondary"
              style="background-color: #818089; color: white;"
            >
              {t('time.now')}
            </button>
            <button onclick={clear} class="btn-secondary">
              {t('time.clear')}
            </button>
          </div>
        </div>
      {:else}
        <!-- 日期转时间戳 -->
        <div class="space-y-4">
          <div>
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
              {t('time.timezone')}
            </label>
            <div class="relative" bind:this={timezoneDropdownRefDate}>
              <button
                type="button"
                onclick={() => {
                  timezoneOpenDate = !timezoneOpenDate;
                  if (timezoneOpenDate) {
                    timezoneSearchDate = '';
                  }
                }}
                class="input w-full pr-10 text-left flex items-center justify-between cursor-pointer hover:bg-gray-50 dark:hover:bg-gray-700"
              >
                <span class="flex-1 truncate">{timezone}</span>
                <ChevronDown class="w-4 h-4 text-gray-400 dark:text-gray-500 flex-shrink-0 transition-transform {timezoneOpenDate ? 'rotate-180' : ''}" />
              </button>
              {#if timezoneOpenDate}
                <div class="absolute z-50 w-full mt-1 bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-lg shadow-lg overflow-hidden flex flex-col" style="max-height: 300px;">
                  <!-- 搜索框 -->
                  <div class="p-2 border-b border-gray-200 dark:border-gray-700 flex-shrink-0">
                    <div class="relative">
                      <div class="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none">
                        <Search class="w-4 h-4 text-gray-400 dark:text-gray-500" />
                      </div>
                      <input
                        type="text"
                        bind:value={timezoneSearchDate}
                        oninput={(e) => {
                          timezoneSearchDate = (e.target as HTMLInputElement).value;
                        }}
                        placeholder={t('time.searchTimezone')}
                        class="input w-full pl-10 pr-8 text-sm"
                        autofocus
                      />
                      {#if timezoneSearchDate}
                        <button
                          type="button"
                          onclick={() => {
                            timezoneSearchDate = '';
                          }}
                          class="absolute inset-y-0 right-0 pr-2 flex items-center text-gray-400 dark:text-gray-500 hover:text-gray-600 dark:hover:text-gray-300"
                        >
                          <X class="w-3 h-3" />
                        </button>
                      {/if}
                    </div>
                  </div>
                  <!-- 时区列表 -->
                  <div class="overflow-y-auto flex-1 timezone-dropdown">
                    {#if filteredTimezonesDate.length > 0}
                      {#each filteredTimezonesDate as tz}
                        <button
                          type="button"
                          onclick={() => selectTimezone(tz, 'date')}
                          class="w-full text-left px-4 py-2 hover:bg-gray-100 dark:hover:bg-gray-700 {timezone === tz ? 'bg-primary-50 dark:bg-primary-900/20 text-primary-600 dark:text-primary-400' : 'text-gray-900 dark:text-gray-100'}"
                        >
                          {tz}
                        </button>
                      {/each}
                    {:else}
                      <div class="px-4 py-2 text-gray-500 dark:text-gray-400 text-sm">
                        {t('time.noTimezoneFound')}
                      </div>
                    {/if}
                  </div>
                </div>
              {/if}
            </div>
          </div>
          
          <div>
            <input
              id="date-input"
              type="text"
              bind:value={dateInput}
              placeholder="YYYY-MM-DD HH:mm:ss 或 YYYY-MM-DDTHH:mm:ss"
              class="input"
            />
            <p class="text-xs text-gray-500 dark:text-gray-400 mt-1">
              支持格式：YYYY-MM-DD HH:mm:ss 或 YYYY-MM-DDTHH:mm:ss
            </p>
          </div>
          
          <div class="flex gap-2">
            <button onclick={clear} class="btn-secondary">
              {t('time.clear')}
            </button>
          </div>
        </div>
      {/if}
    </div>
  </div>

  <!-- 结果显示卡片 -->
  {#if conversionType === 'timestamp' && timestampInput && dateResult}
    <div class="card">
      <div class="space-y-4">
        <div>
          <h3 class="text-xl font-bold text-gray-900 dark:text-gray-100 mb-1">
            {t('time.date')}
          </h3>
        </div>
        
        <!-- YYYY-MM-DD HH:mm:ss 格式 -->
        <div>
          <div class="flex gap-2">
            <input
              type="text"
              readonly
              value={dateResult.spaceFormat}
              class="input font-mono text-sm flex-1"
            />
            <button
              onclick={() => copyToClipboard(dateResult.spaceFormat, 'date-space')}
              class="btn-secondary whitespace-nowrap {dateCopiedSpace ? 'bg-green-500 hover:bg-green-600 text-white' : ''}"
            >
              {#if dateCopiedSpace}
                <span class="flex items-center gap-1">
                  <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7"></path>
                  </svg>
                  {t('common.copied')}
                </span>
              {:else}
                {t('common.copy')}
              {/if}
            </button>
          </div>
        </div>
        
        <!-- YYYY-MM-DDTHH:mm:ss 格式 -->
        <div>
          <div class="flex gap-2">
            <input
              type="text"
              readonly
              value={dateResult.isoFormat}
              class="input font-mono text-sm flex-1"
            />
            <button
              onclick={() => copyToClipboard(dateResult.isoFormat, 'date-iso')}
              class="btn-secondary whitespace-nowrap {dateCopiedIso ? 'bg-green-500 hover:bg-green-600 text-white' : ''}"
            >
              {#if dateCopiedIso}
                <span class="flex items-center gap-1">
                  <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7"></path>
                  </svg>
                  {t('common.copied')}
                </span>
              {:else}
                {t('common.copy')}
              {/if}
            </button>
          </div>
        </div>
      </div>
    </div>
  {:else if conversionType === 'date' && dateInput && timestampResult}
    <div class="card">
      <div class="space-y-4">
        <div>
          <h3 class="text-xl font-bold text-gray-900 dark:text-gray-100 mb-1">
            {t('time.timestamp')}
          </h3>
        </div>
        <div class="flex gap-2">
          <input
            type="text"
            readonly
            value={timestampResult}
            class="input font-mono text-sm flex-1"
          />
          <button
            onclick={() => copyToClipboard(timestampResult, 'timestamp')}
            class="btn-secondary whitespace-nowrap {timestampCopied ? 'bg-green-500 hover:bg-green-600 text-white' : ''}"
          >
            {#if timestampCopied}
              <span class="flex items-center gap-1">
                <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7"></path>
                </svg>
                {t('common.copied')}
              </span>
            {:else}
              {t('common.copy')}
            {/if}
          </button>
        </div>
      </div>
    </div>
  {/if}
</div>