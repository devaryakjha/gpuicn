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
const keys = [];
const input = { dispatchEvent: (event) => { keys.push(event); return false; } };
const document = { body: {}, querySelector: () => input };
class KeyboardEvent {
  constructor(type, event) { this.type = type; this.key = event.key; }
}
runInNewContext(readFileSync(new URL("../site/demo.js", import.meta.url), "utf8"), { window, document, KeyboardEvent });
let prevented = 0;
for (const type of ["keydown", "keyup"]) {
  const event = { type, key: "Enter", target: document.body, preventDefault: () => prevented++ };
  listeners[type](event);
  listeners[type]({ ...event, target: input }); // Never duplicate native editor events.
  listeners[type]({ ...event, target: { tagName: "BUTTON" } });
}
assert.deepEqual(keys.map(({ type, key }) => [type, key]), [["keydown", "Enter"], ["keyup", "Enter"]]);
assert.equal(prevented, 2);
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
console.log("Preview keys forward once; updates accept only valid same-origin parent messages.");
