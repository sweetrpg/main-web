/* Theme picker: light / dark / system, persisted to localStorage. Extracted from the
 * "SweetRPG Hub" Claude Design prototype's <script type="text/x-dc"> reactive-state block —
 * that block ran client-side only and had no server dependency, so it translates directly to
 * plain JS. The anti-flash initial theme application lives in hub.html's inline <head> script,
 * since it must run before first paint; this file only owns the toggle menu after load.
 */
(function () {
  const THEME_KEY = 'sweetrpg_theme_v1';

  const icons = {
    light: '<svg width="16" height="16" viewBox="0 0 256 256"><path fill="currentColor" d="M120,40V16a8,8,0,0,1,16,0V40a8,8,0,0,1-16,0Zm128,88a8,8,0,0,1-8,8H216a8,8,0,0,1,0-16h24A8,8,0,0,1,248,128ZM66.34,66.34a8,8,0,0,0,11.32-11.32l-16-16A8,8,0,0,0,50.34,50.34Zm0,123.32-16,16a8,8,0,0,0,11.32,11.32l16-16a8,8,0,0,0-11.32-11.32ZM192,72a8,8,0,0,0,5.66-2.34l16-16a8,8,0,0,0-11.32-11.32l-16,16A8,8,0,0,0,192,72Zm5.66,117.66a8,8,0,0,0-11.32,11.32l16,16a8,8,0,0,0,11.32-11.32ZM48,128a8,8,0,0,0-8-8H16a8,8,0,0,0,0,16H40A8,8,0,0,0,48,128Zm80,64a8,8,0,0,0-8,8v24a8,8,0,0,0,16,0V200A8,8,0,0,0,128,192Zm0-136a72,72,0,1,0,72,72A72.08,72.08,0,0,0,128,56Zm0,128a56,56,0,1,1,56-56A56.06,56.06,0,0,1,128,184Z"></path></svg>',
    dark: '<svg width="16" height="16" viewBox="0 0 256 256"><path fill="currentColor" d="M233.54,142.23a8,8,0,0,0-8-2,88.08,88.08,0,0,1-109.8-109.8,8,8,0,0,0-10-10,104.84,104.84,0,0,0-52.91,37A104,104,0,0,0,136,224a103.09,103.09,0,0,0,62.52-20.88,104.84,104.84,0,0,0,37-52.91A8,8,0,0,0,233.54,142.23ZM188.9,190.34A88,88,0,0,1,65.66,67.11a89,89,0,0,1,31.4-26A106,106,0,0,0,96,56,104.11,104.11,0,0,0,200,160a106,106,0,0,0,14.92-1.06A89,89,0,0,1,188.9,190.34Z"></path></svg>',
    system: '<svg width="16" height="16" viewBox="0 0 256 256"><path fill="currentColor" d="M208,40H48A24,24,0,0,0,24,64V176a24,24,0,0,0,24,24H208a24,24,0,0,0,24-24V64A24,24,0,0,0,208,40Zm8,136a8,8,0,0,1-8,8H48a8,8,0,0,1-8-8V64a8,8,0,0,1,8-8H208a8,8,0,0,1,8,8ZM160,224a8,8,0,0,1-8,8H104a8,8,0,0,1,0-16h48A8,8,0,0,1,160,224Z"></path></svg>',
  };
  const labels = { light: 'Light', dark: 'Dark', system: 'System' };

  function getStoredTheme() {
    try {
      return localStorage.getItem(THEME_KEY) || 'system';
    } catch (e) {
      return 'system';
    }
  }

  function setStoredTheme(theme) {
    try {
      localStorage.setItem(THEME_KEY, theme);
    } catch (e) {
      /* private browsing / storage disabled — theme just won't persist */
    }
  }

  function applyTheme(theme, systemDark) {
    const dark = theme === 'dark' || (theme === 'system' && systemDark);
    document.documentElement.dataset.theme = dark ? 'dark' : 'light';
  }

  document.addEventListener('DOMContentLoaded', function () {
    const button = document.getElementById('hub-theme-button');
    const menu = document.getElementById('hub-theme-menu');
    const options = menu.querySelectorAll('[data-theme-choice]');
    const mq = window.matchMedia('(prefers-color-scheme: dark)');

    let theme = getStoredTheme();

    options.forEach(function (option) {
      const choice = option.dataset.themeChoice;
      option.innerHTML = icons[choice] + '<span style="margin-left:8px">' + labels[choice] + '</span>';
      option.addEventListener('click', function () {
        theme = choice;
        setStoredTheme(theme);
        applyTheme(theme, mq.matches);
        button.innerHTML = icons[theme] || icons.system;
        menu.hidden = true;
      });
    });

    button.innerHTML = icons[theme] || icons.system;
    button.addEventListener('click', function () {
      menu.hidden = !menu.hidden;
    });
    document.addEventListener('click', function (event) {
      if (!menu.hidden && !menu.contains(event.target) && event.target !== button) {
        menu.hidden = true;
      }
    });

    mq.addEventListener('change', function (event) {
      if (theme === 'system') {
        applyTheme(theme, event.matches);
      }
    });
  });
})();
