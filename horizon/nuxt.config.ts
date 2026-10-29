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
    "nitro-cloudflare-dev"
  ],

  $development: {
    runtimeConfig: {
      public: {
        apiBase: "http://127.0.0.1:8080",
      },
    },
  },

  $production: {
    // swr: serve cached render, refresh in background after N seconds
    // prerender: rendered once at build time and served as a static file
    routeRules: {
      "/": { swr: 600 },
      "/categories": { swr: 600 },
      "/paintings/**": { swr: 600 },
      "/about": { prerender: true },
      "/contact": { swr: 3600 },
    },
  },

  ssr: true,

  devtools: {
    enabled: true,
  },

  app: {
    pageTransition: { name: "page", mode: "out-in" },
  },

  css: ["~/assets/css/main.css"],

  runtimeConfig: {
    public: {
      apiBase: "https://api.dansansouci.com",
    },
  },

  experimental: {
    // inline the payload on first load so html and data come from the same
    // render; separately cached _payload.json files caused hydration mismatches
    payloadExtraction: "client",
  },

  compatibilityDate: "2026-06-30",

  nitro: {
    preset: "cloudflare_module",

    cloudflare: {
      deployConfig: true,
      nodeCompat: true
    }
  },

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
      { name: "Cinzel", provider: "google", weights: [400] },
      { name: "Inter", provider: "google", weights: [300, 400, 600] },
    ],
  },
});