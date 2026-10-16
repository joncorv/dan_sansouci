// https://nuxt.com/docs/api/configuration/nuxt-config
export default defineNuxtConfig({
  modules: [
    "@nuxt/eslint",
    "@nuxt/ui",
    "@nuxt/hints",
    "@nuxt/image",
    "nuxt-icons",
    "@nuxt/fonts",
    "@nuxt/icon",
  ],

  // Client-side rendered. Nuxt emits a static index.html shell and the app
  // boots entirely in the browser, so every $fetch goes out cross-origin.
  ssr: false,

  devtools: {
    enabled: true,
  },

  css: ["~/assets/css/main.css"],

  compatibilityDate: "2026-06-30",

  eslint: {
    config: {
      stylistic: {
        semi: true,
        quotes: "double",
        // commaDangle: "never",
        braceStyle: "1tbs",
      },
    },
  },
  fonts: {
    families: [
      { name: "Sancreek", provider: "google", weights: [400] },
      { name: "Inter", provider: "google", weights: [400, 600] },
    ],
  },
});
