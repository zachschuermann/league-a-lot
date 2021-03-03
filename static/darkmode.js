// adapted from https://web.dev/prefers-color-scheme/
// and https://github.com/GoogleChromeLabs/dark-mode-toggle/blob/master/src/dark-mode-toggle.mjs

var darkMode;

(() => {
  const LINK_REL_STYLESHEET = 'link[rel=stylesheet]';
  const MEDIA = 'media'
  const PREFERS_COLOR_SCHEME = 'prefers-color-scheme';

  const darkModeSwitch = document.getElementById('dark-mode-switch');
  const darkCSS = document.querySelectorAll(`${LINK_REL_STYLESHEET}[${MEDIA}*=${PREFERS_COLOR_SCHEME}][${MEDIA}*="dark"]`);
  const lightCSS = document.querySelectorAll(`${LINK_REL_STYLESHEET}[${MEDIA}*=${PREFERS_COLOR_SCHEME}][${MEDIA}*="light"]`);
  const hasNativePrefersColorScheme =
      matchMedia('(prefers-color-scheme: dark)').media !== 'not all';

  // Listen to `prefers-color-scheme` changes.
  if (hasNativePrefersColorScheme) {
    matchMedia('(prefers-color-scheme: dark)').addListener(({matches}) => {
      darkMode = matches;
    });
  }

  if (matchMedia('(prefers-color-scheme: dark)').matches) {
    darkMode = true;
  } else {
    darkMode = false;
  }

  darkModeSwitch.addEventListener("click", () => {
    if (!hasNativePrefersColorScheme) {
        console.log('Your browser does not support the `prefers-color-scheme` media query.');
    }

    // purge plots since they don't dynamically change
    document.getElementById('plot').innerHTML = "";

    darkMode = !darkMode;

    if (!darkMode) {
      lightCSS.forEach((link) => {
        link.media = 'all';
        link.disabled = false;
      });
      darkCSS.forEach((link) => {
        link.media = 'not all';
        link.disabled = true;
      });
    } else {
      darkCSS.forEach((link) => {
        link.media = 'all';
        link.disabled = false;
      });
      lightCSS.forEach((link) => {
        link.media = 'not all';
        link.disabled = true;
      });
    }
  });
})();
