<script setup>
useHead({
  meta: [{ name: "viewport", content: "width=device-width, initial-scale=1" }],
  link: [{ rel: "icon", href: "/favicon.ico" }],
  htmlAttrs: {
    lang: "en",
  },
});

const title = "Dan SanSouci";
const description =
  "The online portfolio of Painter and Author Daniel SanSouci";

useSeoMeta({
  title,
  description,
  ogTitle: title,
  ogDescription: description,
  twitterCard: "summary_large_image",
});
const toaster = {
  position: "top-center",
  expand: false,
};
</script>

<template>
  <UApp :toaster="toaster">
    <NuxtLayout>
      <NuxtPage />
    </NuxtLayout>
  </UApp>
</template>

<style>
.page-enter-active,
.page-leave-active {
  transition: opacity 0.1s;
}

/* no filter/transform here: either one re-anchors position: fixed children
   (the categories backgrounds) to the page and makes them jump */
.page-enter-from,
.page-leave-to {
  opacity: 0;
}
</style>
