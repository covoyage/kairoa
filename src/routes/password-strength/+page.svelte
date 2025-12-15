<script lang="ts">
  import { translationsStore } from '$lib/stores/i18n';
  import { CheckCircle2, XCircle, AlertCircle, Shield, ShieldCheck, ShieldAlert, ShieldOff } from 'lucide-svelte';

  let translations = $derived($translationsStore);

  function t(key: string): string {
    const keys = key.split('.');
    let value: any = translations;
    for (const k of keys) {
      value = value?.[k];
    }
    return value || key;
  }

  type StrengthLevel = 'very-weak' | 'weak' | 'fair' | 'good' | 'strong' | 'very-strong';

  interface CheckResult {
    name: string;
    passed: boolean;
    message: string;
    score: number;
  }

  let password = $state('');
  let strengthLevel = $state<StrengthLevel>('very-weak');
  let strengthScore = $state(0);
  let checks = $state<CheckResult[]>([]);
  let crackTime = $state('');
  let suggestions = $state<string[]>([]);

  // Common weak passwords
  const commonPasswords = [
    'password', '123456', '12345678', '123456789', '1234567890',
    'qwerty', 'abc123', 'password1', 'admin', 'letmein',
    'welcome', 'monkey', '1234567', 'sunshine', 'princess',
    'dragon', 'passw0rd', 'master', 'hello', 'freedom',
    'whatever', 'qazwsx', 'trustno1', 'jordan23', 'harley',
    'shadow', 'superman', 'michael', 'football', 'iloveyou'
  ];

  function calculateStrength() {
    if (!password) {
      strengthLevel = 'very-weak';
      strengthScore = 0;
      checks = [];
      crackTime = '';
      suggestions = [];
      return;
    }

    const results: CheckResult[] = [];
    let totalScore = 0;

    // Length check
    const lengthCheck: CheckResult = {
      name: 'length',
      passed: password.length >= 8,
      message: password.length >= 12 
        ? t('passwordStrength.checks.length.strong')
        : password.length >= 8
        ? t('passwordStrength.checks.length.good')
        : t('passwordStrength.checks.length.weak'),
      score: Math.min(password.length * 2, 20)
    };
    results.push(lengthCheck);
    totalScore += lengthCheck.score;

    // Uppercase check
    const hasUppercase = /[A-Z]/.test(password);
    const uppercaseCheck: CheckResult = {
      name: 'uppercase',
      passed: hasUppercase,
      message: hasUppercase ? t('passwordStrength.checks.uppercase.passed') : t('passwordStrength.checks.uppercase.failed'),
      score: hasUppercase ? 10 : 0
    };
    results.push(uppercaseCheck);
    totalScore += uppercaseCheck.score;

    // Lowercase check
    const hasLowercase = /[a-z]/.test(password);
    const lowercaseCheck: CheckResult = {
      name: 'lowercase',
      passed: hasLowercase,
      message: hasLowercase ? t('passwordStrength.checks.lowercase.passed') : t('passwordStrength.checks.lowercase.failed'),
      score: hasLowercase ? 10 : 0
    };
    results.push(lowercaseCheck);
    totalScore += lowercaseCheck.score;

    // Numbers check
    const hasNumbers = /\d/.test(password);
    const numbersCheck: CheckResult = {
      name: 'numbers',
      passed: hasNumbers,
      message: hasNumbers ? t('passwordStrength.checks.numbers.passed') : t('passwordStrength.checks.numbers.failed'),
      score: hasNumbers ? 10 : 0
    };
    results.push(numbersCheck);
    totalScore += numbersCheck.score;

    // Special characters check
    const hasSpecial = /[!@#$%^&*()_+\-=\[\]{};':"\\|,.<>\/?]/.test(password);
    const specialCheck: CheckResult = {
      name: 'special',
      passed: hasSpecial,
      message: hasSpecial ? t('passwordStrength.checks.special.passed') : t('passwordStrength.checks.special.failed'),
      score: hasSpecial ? 15 : 0
    };
    results.push(specialCheck);
    totalScore += specialCheck.score;

    // Character variety check
    const uniqueChars = new Set(password).size;
    const varietyCheck: CheckResult = {
      name: 'variety',
      passed: uniqueChars >= password.length * 0.5,
      message: uniqueChars >= password.length * 0.7
        ? t('passwordStrength.checks.variety.strong')
        : uniqueChars >= password.length * 0.5
        ? t('passwordStrength.checks.variety.good')
        : t('passwordStrength.checks.variety.weak'),
      score: Math.min(uniqueChars * 2, 15)
    };
    results.push(varietyCheck);
    totalScore += varietyCheck.score;

    // Common password check
    const isCommon = commonPasswords.some(common => password.toLowerCase().includes(common));
    const commonCheck: CheckResult = {
      name: 'common',
      passed: !isCommon,
      message: isCommon ? t('passwordStrength.checks.common.failed') : t('passwordStrength.checks.common.passed'),
      score: isCommon ? -20 : 10
    };
    results.push(commonCheck);
    totalScore += commonCheck.score;

    // Sequential characters check
    const hasSequential = /(012|123|234|345|456|567|678|789|890|abc|bcd|cde|def|efg|fgh|ghi|hij|ijk|jkl|klm|lmn|mno|nop|opq|pqr|qrs|rst|stu|tuv|uvw|vwx|wxy|xyz)/i.test(password);
    const sequentialCheck: CheckResult = {
      name: 'sequential',
      passed: !hasSequential,
      message: hasSequential ? t('passwordStrength.checks.sequential.failed') : t('passwordStrength.checks.sequential.passed'),
      score: hasSequential ? -10 : 10
    };
    results.push(sequentialCheck);
    totalScore += sequentialCheck.score;

    // Repeated characters check
    const hasRepeated = /(.)\1{2,}/.test(password);
    const repeatedCheck: CheckResult = {
      name: 'repeated',
      passed: !hasRepeated,
      message: hasRepeated ? t('passwordStrength.checks.repeated.failed') : t('passwordStrength.checks.repeated.passed'),
      score: hasRepeated ? -10 : 10
    };
    results.push(repeatedCheck);
    totalScore += repeatedCheck.score;

    checks = results;
    strengthScore = Math.max(0, Math.min(100, totalScore));

    // Determine strength level
    if (strengthScore >= 80) {
      strengthLevel = 'very-strong';
    } else if (strengthScore >= 60) {
      strengthLevel = 'strong';
    } else if (strengthScore >= 40) {
      strengthLevel = 'good';
    } else if (strengthScore >= 20) {
      strengthLevel = 'fair';
    } else if (strengthScore >= 10) {
      strengthLevel = 'weak';
    } else {
      strengthLevel = 'very-weak';
    }

    // Calculate crack time estimate
    crackTime = estimateCrackTime(password, strengthScore);

    // Generate suggestions
    suggestions = generateSuggestions(results, password);
  }

  function estimateCrackTime(pwd: string, score: number): string {
    if (!pwd) return '';
    
    // Rough estimation based on password complexity
    const charsetSize = calculateCharsetSize(pwd);
    const combinations = Math.pow(charsetSize, pwd.length);
    
    // Assume 1 billion guesses per second (typical for modern GPUs)
    const guessesPerSecond = 1e9;
    const seconds = combinations / guessesPerSecond;
    
    if (seconds < 1) return t('passwordStrength.crackTime.instant');
    if (seconds < 60) return t('passwordStrength.crackTime.seconds');
    if (seconds < 3600) return t('passwordStrength.crackTime.minutes');
    if (seconds < 86400) return t('passwordStrength.crackTime.hours');
    if (seconds < 31536000) return t('passwordStrength.crackTime.days');
    if (seconds < 31536000000) return t('passwordStrength.crackTime.years');
    return t('passwordStrength.crackTime.centuries');
  }

  function calculateCharsetSize(pwd: string): number {
    let size = 0;
    if (/[a-z]/.test(pwd)) size += 26;
    if (/[A-Z]/.test(pwd)) size += 26;
    if (/\d/.test(pwd)) size += 10;
    if (/[^a-zA-Z0-9]/.test(pwd)) size += 32; // Common special chars
    return size || 1;
  }

  function generateSuggestions(results: CheckResult[], pwd: string): string[] {
    const suggestions: string[] = [];
    
    if (!results.find(r => r.name === 'length')?.passed) {
      suggestions.push(t('passwordStrength.suggestions.length'));
    }
    if (!results.find(r => r.name === 'uppercase')?.passed) {
      suggestions.push(t('passwordStrength.suggestions.uppercase'));
    }
    if (!results.find(r => r.name === 'lowercase')?.passed) {
      suggestions.push(t('passwordStrength.suggestions.lowercase'));
    }
    if (!results.find(r => r.name === 'numbers')?.passed) {
      suggestions.push(t('passwordStrength.suggestions.numbers'));
    }
    if (!results.find(r => r.name === 'special')?.passed) {
      suggestions.push(t('passwordStrength.suggestions.special'));
    }
    if (!results.find(r => r.name === 'common')?.passed) {
      suggestions.push(t('passwordStrength.suggestions.common'));
    }
    if (!results.find(r => r.name === 'sequential')?.passed) {
      suggestions.push(t('passwordStrength.suggestions.sequential'));
    }
    if (!results.find(r => r.name === 'repeated')?.passed) {
      suggestions.push(t('passwordStrength.suggestions.repeated'));
    }
    
    return suggestions;
  }

  function getStrengthColor(): string {
    switch (strengthLevel) {
      case 'very-strong':
        return 'bg-green-500';
      case 'strong':
        return 'bg-green-400';
      case 'good':
        return 'bg-yellow-400';
      case 'fair':
        return 'bg-yellow-500';
      case 'weak':
        return 'bg-orange-500';
      case 'very-weak':
        return 'bg-red-500';
      default:
        return 'bg-gray-300';
    }
  }

  function getStrengthIcon() {
    switch (strengthLevel) {
      case 'very-strong':
      case 'strong':
        return ShieldCheck;
      case 'good':
      case 'fair':
        return Shield;
      case 'weak':
        return ShieldAlert;
      case 'very-weak':
        return ShieldOff;
      default:
        return ShieldOff;
    }
  }

  function clear() {
    password = '';
    calculateStrength();
  }

  $effect(() => {
    calculateStrength();
  });
</script>

<div class="flex flex-col h-full w-full p-2">
  <div class="card flex-1 flex flex-col space-y-6">
    <div class="flex items-center justify-between">
      <div>
        <h1 class="text-xl font-semibold text-gray-800 dark:text-gray-100">{t('passwordStrength.title')}</h1>
        <p class="text-sm text-gray-500 dark:text-gray-400 mt-1">{t('passwordStrength.description')}</p>
      </div>
      {#if password}
        <button type="button" class="btn-secondary" onclick={clear}>
          {t('passwordStrength.clear')}
        </button>
      {/if}
    </div>

    <div class="space-y-4">
      <div>
        <label for="password-input" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
          {t('passwordStrength.password')}
        </label>
        <input
          id="password-input"
          type="password"
          bind:value={password}
          placeholder={t('passwordStrength.placeholder')}
          class="input w-full font-mono"
          autocomplete="off"
        />
      </div>

      {#if password}
        <!-- Strength Indicator -->
        <div class="space-y-3">
          <div class="flex items-center justify-between">
            <span class="text-sm font-medium text-gray-700 dark:text-gray-300">
              {t('passwordStrength.strength')}
            </span>
            <div class="flex items-center gap-2">
              {#if password}
                {@const Icon = getStrengthIcon()}
                <Icon class="w-5 h-5 {strengthLevel === 'very-strong' || strengthLevel === 'strong' ? 'text-green-500' : strengthLevel === 'good' || strengthLevel === 'fair' ? 'text-yellow-500' : 'text-red-500'}" />
              {/if}
              <span class="text-sm font-semibold {strengthLevel === 'very-strong' || strengthLevel === 'strong' ? 'text-green-600 dark:text-green-400' : strengthLevel === 'good' || strengthLevel === 'fair' ? 'text-yellow-600 dark:text-yellow-400' : 'text-red-600 dark:text-red-400'}">
                {t(`passwordStrength.levels.${strengthLevel}`)}
              </span>
            </div>
          </div>
          
          <!-- Progress Bar -->
          <div class="w-full bg-gray-200 dark:bg-gray-700 rounded-full h-3 overflow-hidden">
            <div
              class="h-full transition-all duration-300 {getStrengthColor()}"
              style="width: {strengthScore}%"
            ></div>
          </div>
          
          <div class="text-xs text-gray-500 dark:text-gray-400 text-right">
            {strengthScore} / 100
          </div>
        </div>

        <!-- Crack Time Estimate -->
        {#if crackTime}
          <div class="p-3 bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800 rounded-lg">
            <div class="flex items-start">
              <AlertCircle class="w-5 h-5 text-blue-600 dark:text-blue-400 mr-2 flex-shrink-0 mt-0.5" />
              <div>
                <p class="text-sm font-medium text-blue-800 dark:text-blue-200">
                  {t('passwordStrength.crackTime.title')}
                </p>
                <p class="text-sm text-blue-700 dark:text-blue-300 mt-1">
                  {crackTime}
                </p>
              </div>
            </div>
          </div>
        {/if}

        <!-- Check Results -->
        <div class="space-y-2">
          <h3 class="text-sm font-semibold text-gray-700 dark:text-gray-300">
            {t('passwordStrength.checks.title')}
          </h3>
          <div class="grid grid-cols-1 md:grid-cols-3 gap-2">
            {#each checks as check}
              <div class="flex items-start gap-2 p-2 bg-gray-50 dark:bg-gray-800/50 rounded">
                {#if check.passed}
                  <CheckCircle2 class="w-5 h-5 text-green-500 dark:text-green-400 flex-shrink-0 mt-0.5" />
                {:else}
                  <XCircle class="w-5 h-5 text-red-600 dark:text-red-400 flex-shrink-0 mt-0.5" />
                {/if}
                <div class="flex-1 min-w-0">
                  <p class="text-sm text-gray-900 dark:text-gray-100">
                    {t(`passwordStrength.checks.${check.name}.label`)}
                  </p>
                  <p class="text-xs text-gray-600 dark:text-gray-400 mt-0.5">
                    {check.message}
                  </p>
                </div>
              </div>
            {/each}
          </div>
        </div>

        <!-- Suggestions -->
        {#if suggestions.length > 0}
          <div class="space-y-2">
            <h3 class="text-sm font-semibold text-gray-700 dark:text-gray-300">
              {t('passwordStrength.suggestions.title')}
            </h3>
            <ul class="space-y-1">
              {#each suggestions as suggestion}
                <li class="flex items-start gap-2 text-sm text-gray-700 dark:text-gray-300">
                  <span class="text-primary-500 mt-1">•</span>
                  <span>{suggestion}</span>
                </li>
              {/each}
            </ul>
          </div>
        {/if}
      {:else}
        <div class="text-center py-8 text-gray-400 dark:text-gray-500">
          <Shield class="w-12 h-12 mx-auto mb-3 opacity-50" />
          <p>{t('passwordStrength.empty')}</p>
        </div>
      {/if}
    </div>
  </div>
</div>
