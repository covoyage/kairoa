import { writable } from 'svelte/store';
import { browser } from '$app/environment';

export type Theme = 'kairoa-light' | 'kairoa-dark' | 'solarized-light' | 'solarized-dark';

const allThemes: Theme[] = ['kairoa-light', 'kairoa-dark', 'solarized-light', 'solarized-dark'];

function createThemeStore() {
  const { subscribe, set, update } = writable<Theme>('kairoa-light');

  return {
    subscribe,
    set: (theme: Theme) => {
      if (browser) {
        localStorage.setItem('theme', theme);
        // Remove all theme classes
        document.documentElement.classList.remove(...allThemes);
        // Add the new theme class
        document.documentElement.classList.add(theme);
        
        // Also add 'dark' class for themes that are dark variants (for Tailwind compatibility)
        if (theme === 'kairoa-dark' || theme === 'solarized-dark') {
          document.documentElement.classList.add('dark');
        } else {
          document.documentElement.classList.remove('dark');
        }
      }
      set(theme);
    },
    init: () => {
      if (browser) {
        const stored = localStorage.getItem('theme') as Theme | null;
        const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
        const theme = stored || (prefersDark ? 'kairoa-dark' : 'kairoa-light');
        
        // Remove all theme classes
        document.documentElement.classList.remove(...allThemes);
        // Add the theme class
        document.documentElement.classList.add(theme);
        
        // Also add 'dark' class for dark themes
        if (theme === 'kairoa-dark' || theme === 'solarized-dark') {
          document.documentElement.classList.add('dark');
        } else {
          document.documentElement.classList.remove('dark');
        }
        
        set(theme);
      }
    },
    toggle: () => {
      update((current) => {
        // Cycle through themes: kairoa-light -> kairoa-dark -> github-light -> github-dark -> kairoa-light
        const currentIndex = allThemes.indexOf(current);
        const newTheme = allThemes[(currentIndex + 1) % allThemes.length];
        
        if (browser) {
          localStorage.setItem('theme', newTheme);
          document.documentElement.classList.remove(...allThemes);
          document.documentElement.classList.add(newTheme);
          
          if (newTheme === 'kairoa-dark' || newTheme === 'solarized-dark') {
            document.documentElement.classList.add('dark');
          } else {
            document.documentElement.classList.remove('dark');
          }
        }
        return newTheme;
      });
    }
  };
}

export const theme = createThemeStore();

