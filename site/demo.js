(function () {
  // GPUI 0.3.4 listens on its IME textarea, then blurs it for non-editable
  // controls. Forward those keys without refocusing it or opening a mobile IME.
  for (var type of ["keydown", "keyup"]) {
    window.addEventListener(type, function (event) {
      if (event.target !== document.body && event.target?.tagName !== "CANVAS") return;
      var input = document.querySelector("body > textarea");
      if (input && !input.dispatchEvent(new KeyboardEvent(event.type, event))) {
        event.preventDefault();
      }
    });
  }

  window.gpuicnPreviewUpdates = function (update) {
    window.addEventListener("message", function (event) {
      var data = event.data;
      if (event.source !== window.parent || event.origin !== window.location.origin ||
          data?.gpuicn !== "preview-update" ||
          (data.theme !== "light" && data.theme !== "dark") ||
          typeof data.icon !== "string") return;
      update(data.theme, data.icon);
    });
  };

  var NativeWorker = window.Worker;
  var workers = [];
  window.Worker = function (url, options) {
    var worker = new NativeWorker(url, options);
    workers.push(worker);
    return worker;
  };
  window.Worker.prototype = NativeWorker.prototype;

  function stopWorkers() {
    workers.forEach(function (worker) {
      try { worker.terminate(); } catch (_) {}
    });
    workers.length = 0;
  }

  function fail(reason) {
    if (window.parent === window) return;
    var message = reason && reason.message ? reason.message : String(reason || "unknown error");
    window.parent.postMessage({ gpuicn: "preview-error", message: message.slice(0, 300) }, "*");
  }

  window.addEventListener("pagehide", stopWorkers);
  window.addEventListener("error", function (event) { fail(event.error || event.message); });
  window.addEventListener("unhandledrejection", function (event) { fail(event.reason); });
})();
