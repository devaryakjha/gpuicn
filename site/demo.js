(function () {
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
    window.parent.postMessage({ imajhaUi: "preview-error", message: message.slice(0, 300) }, "*");
  }

  window.addEventListener("pagehide", stopWorkers);
  window.addEventListener("error", function (event) { fail(event.error || event.message); });
  window.addEventListener("unhandledrejection", function (event) { fail(event.reason); });
})();
