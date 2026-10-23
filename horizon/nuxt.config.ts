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
  ssr: true,

  devtools: {
    enabled: true,
  },

  app: {
    pageTransition: { name: "page", mode: "out-in" },
  },

  css: ["~/assets/css/main.css"],

  compatibilityDate: "2026-06-30",

  nitro: {
    preset: "cloudflare_module",

    prerender: {
      routes: ["/"],
      crawlLinks: true,
    },

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