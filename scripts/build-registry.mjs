import { readdirSync, writeFileSync } from "node:fs";

const titleOverrides = {
  otp_field: "OTP Field",
};
const buttonDependents = new Set([
  "alert_dialog",
  "dialog",
  "drawer",
  "popover",
  "toast",
]);
const bundledFiles = {
  autocomplete: ["combobox"],
  context_menu: ["menu"],
  menubar: ["menu"],
};

const names = readdirSync("registry", { withFileTypes: true })
  .filter((entry) => entry.isDirectory())
  .map((entry) => entry.name)
  .sort((a, b) => a.localeCompare(b));
names.splice(names.indexOf("theme"), 1);
names.unshift("theme");

const title = (name) =>
  titleOverrides[name] ??
  name
    .split("_")
    .map((part) => part[0].toUpperCase() + part.slice(1))
    .join(" ");
const file = (name) => ({
  path: `registry/${name}/${name}.rs`,
  type: "registry:file",
  target: `~/src/ui/${name}.rs`,
});

const items = names.map((name) => {
  const item = {
    name: name.replaceAll("_", "-"),
    type: "registry:item",
    title: title(name),
    description:
      name === "theme"
        ? "The app-owned shadcn Neutral light and dark theme for GPUI."
        : `The shadcn default ${title(name)} visual port backed by Base GPUI behavior.`,
  };

  const dependencies =
    name === "theme"
      ? []
      : [
          "theme",
          ...(buttonDependents.has(name) ? ["button"] : []),
          ...(bundledFiles[name] ?? []),
        ];
  item.files = [...new Set([...dependencies, name])].map(file);
  return item;
});

writeFileSync(
  "registry.json",
  `${JSON.stringify(
    {
      $schema: "https://ui.shadcn.com/schema/registry.json",
      name: "gpuicn",
      homepage: "https://devaryakjha.github.io/gpuicn/",
      items,
    },
    null,
    2,
  )}\n`,
);
