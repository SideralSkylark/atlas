import { ref, watchEffect } from 'vue';

export type Theme = 'everforest' | 'gruvbox' | 'rosepine' | 'kanagawa';
export type Appearance = 'dark' | 'light';

function getStoredTheme(): Theme {
  if (typeof localStorage === 'undefined') return 'everforest';
  const saved = localStorage.getItem('atlas_theme') as Theme | null;
  return saved && ['everforest', 'gruvbox', 'rosepine', 'kanagawa'].includes(saved) ? saved : 'everforest';
}

function getStoredAppearance(): Appearance {
  if (typeof localStorage === 'undefined') return 'dark';
  const saved = localStorage.getItem('atlas_appearance') as Appearance | null;
  return saved === 'light' || saved === 'dark' ? saved : 'dark';
}

export const theme = ref<Theme>(getStoredTheme());
export const appearance = ref<Appearance>(getStoredAppearance());

function applyThemeState() {
  const root = document.documentElement;
  root.setAttribute('data-theme', theme.value);
  root.setAttribute('data-appearance', appearance.value);

  if (appearance.value === 'dark') {
    root.classList.add('dark');
  } else {
    root.classList.remove('dark');
  }
}

watchEffect(() => {
  applyThemeState();
});

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

  return {
    theme,
    appearance,
    setTheme,
    setAppearance,
    toggleAppearance
  };
}
