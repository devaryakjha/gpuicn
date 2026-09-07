import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { runInNewContext } from "node:vm";

const listeners = {};
const window = {
  Worker: function () {},
  parent: {},
  location: { origin: "https://ui.imajha.com" },
  addEventListener: (type, callback) => { listeners[type] = callback; },
};
runInNewContext(readFileSync(new URL("../site/demo.js", import.meta.url), "utf8"), { window });
const updates = [];
window.gpuicnPreviewUpdates((...values) => updates.push(values));
const event = {
  source: window.parent,
  origin: window.location.origin,
  data: { gpuicn: "preview-update", theme: "light", icon: "air-vent" },
};
listeners.message(event);
listeners.message({ ...event, data: { ...event.data, theme: "dark", icon: "house" } });
for (const invalid of [
  { ...event, source: {} },
  { ...event, origin: "https://other.example" },
  { ...event, data: null },
  { ...event, data: { ...event.data, gpuicn: "other" } },
  { ...event, data: { ...event.data, theme: "unknown" } },
  { ...event, data: { ...event.data, icon: null } },
]) listeners.message(invalid);
assert.deepEqual(updates, [["light", "air-vent"], ["dark", "house"]]);
console.log("Preview updates accept only valid messages from the same-origin parent.");
