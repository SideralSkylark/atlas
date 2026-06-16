import { ref, watchEffect } from 'vue';

export type Theme = 'everforest' | 'gruvbox' | 'rosepine' | 'kanagawa';
export type Appearance = 'dark' | 'light';

const theme = ref<Theme>((localStorage.getItem('atlas_theme') as Theme) || 'everforest');
const appearance = ref<Appearance>((localStorage.getItem('atlas_appearance') as Appearance) || 'dark');

export function useTheme() {
  function setTheme(newTheme: Theme) {
    theme.value = newTheme;
    localStorage.setItem('atlas_theme', newTheme);
  }

  function setAppearance(newAppearance: Appearance) {
    appearance.value = newAppearance;
    localStorage.setItem('atlas_appearance', newAppearance);
  }

  function toggleAppearance() {
    setAppearance(appearance.value === 'dark' ? 'light' : 'dark');
  }

  watchEffect(() => {
    const root = document.documentElement;
    root.setAttribute('data-theme', theme.value);
    root.setAttribute('data-appearance', appearance.value);
    
    if (appearance.value === 'dark') {
      root.classList.add('dark');
    } else {
      root.classList.remove('dark');
    }
  });

  return {
    theme,
    appearance,
    setTheme,
    setAppearance,
    toggleAppearance
  };
}
