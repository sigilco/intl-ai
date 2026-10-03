const KEY = "intl-ai-demo:theme";
const THEMES = { light: "illolight", dark: "illodark" } as const;
type Mode = keyof typeof THEMES;

function systemMode(): Mode {
  return window.matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light";
}

function savedMode(): Mode | null {
  try {
    const t = localStorage.getItem(KEY);
    if (t === THEMES.light) return "light";
    if (t === THEMES.dark) return "dark";
  } catch {
    /* storage unavailable */
  }
  return null;
}

/** Effective theme: the saved override, else whatever the OS reports. */
export const theme = $state<{ mode: Mode; override: boolean }>({
  mode: "light",
  override: false,
});

export function initTheme() {
  const saved = savedMode();
  theme.mode = saved ?? systemMode();
  theme.override = saved !== null;
  // Track the OS while there is no explicit choice.
  window.matchMedia("(prefers-color-scheme: dark)").addEventListener("change", (e) => {
    if (!theme.override) theme.mode = e.matches ? "dark" : "light";
  });
}

export function toggleTheme() {
  theme.mode = theme.mode === "dark" ? "light" : "dark";
  theme.override = true;
  document.documentElement.dataset.theme = THEMES[theme.mode];
  try {
    localStorage.setItem(KEY, THEMES[theme.mode]);
  } catch {
    /* storage unavailable */
  }
}
