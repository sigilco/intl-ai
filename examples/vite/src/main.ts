import { getLocale, t } from "@intl-ai/unplugin";

console.log("Current locale:", getLocale());
console.log("Hello:", t("hello"));
console.log("Welcome:", t("welcome"));
console.log("Goodbye:", t("goodbye"));

document.body.innerHTML = `
  <h1>${t("hello")}</h1>
  <p>${t("welcome")}</p>
`;
