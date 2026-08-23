(function () {
  const frame = document.querySelector("iframe[data-demo]");
  const status = document.querySelector("[data-status]");
  const theme = document.querySelector("select[data-theme]");
  let timer;

  function setStatus(value, detail) {
    status.dataset.status = value;
    status.textContent = detail || value;
    frame.hidden = value !== "ready";
  }

  function load() {
    if (!("gpu" in navigator)) {
      setStatus("unsupported", "This browser does not expose WebGPU.");
      return;
    }
    setStatus("loading", "Loading interactive GPUI preview…");
    frame.src = `../../demo/index.html?demo=${frame.dataset.demo}&theme=${theme.value}`;
    clearTimeout(timer);
    timer = setTimeout(() => setStatus("failed", "The GPUI preview did not start."), 30000);
  }

  window.addEventListener("message", (event) => {
    if (event.source !== frame.contentWindow || !event.data || !event.data.imajhaUi) return;
    clearTimeout(timer);
    if (event.data.imajhaUi === "preview-ready") setStatus("ready");
    if (event.data.imajhaUi === "preview-error") setStatus("failed", event.data.message);
  });
  window.addEventListener("pagehide", () => {
    clearTimeout(timer);
    frame.src = "about:blank";
  });
  theme.addEventListener("change", load);

  new IntersectionObserver((entries, observer) => {
    if (!entries.some((entry) => entry.isIntersecting)) return;
    observer.disconnect();
    load();
  }).observe(frame.parentElement);
})();
