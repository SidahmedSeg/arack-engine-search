import "clsx";
function _layout($$renderer, $$props) {
  $$renderer.component(($$renderer2) => {
    let { children } = $$props;
    let darkMode = false;
    function updateDarkMode() {
      if (darkMode) {
        document.documentElement.classList.add("dark");
      } else {
        document.documentElement.classList.remove("dark");
      }
      localStorage.setItem("darkMode", darkMode.toString());
    }
    function toggleDarkMode() {
      darkMode = !darkMode;
      updateDarkMode();
    }
    if (typeof window !== "undefined") {
      window.toggleDarkMode = toggleDarkMode;
    }
    children($$renderer2);
    $$renderer2.push(`<!---->`);
  });
}
export {
  _layout as default
};
