import Theme from "vitepress/theme";
import "virtual:group-icons.css";
import DocsUrl from "../components/DocsUrl.vue";

export default {
  ...Theme,
  enhanceApp({ app }) {
    app.component("DocsUrl", DocsUrl);
  },
};
