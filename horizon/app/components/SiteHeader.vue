<script setup lang="ts">
const colorMode = useColorMode();
const menuOpen = ref(false);
const dark = computed(() => colorMode.value === "dark");

function toggleTheme() {
  colorMode.preference = dark.value ? "light" : "dark";
}

let desktopQuery: MediaQueryList | undefined;

function closeMenuOnDesktop() {
  if (desktopQuery?.matches) {
    menuOpen.value = false;
  }
}

onMounted(() => {
  desktopQuery = window.matchMedia("(min-width: 768px)");
  desktopQuery.addEventListener("change", closeMenuOnDesktop);
});

onBeforeUnmount(() => {
  desktopQuery?.removeEventListener("change", closeMenuOnDesktop);
});
</script>

<template>
  <UHeader
    v-model:open="menuOpen"
    title="Dan San Souci"
    to="/categories"
    class="site-header"
    :ui="{
      container: 'site-header__container',
      left: 'site-header__left',
      center: 'site-header__center',
      right: 'site-header__right',
      header: 'site-header__menu-header',
      body: 'site-header__menu-body',
      content: 'site-header__menu',
      overlay: 'site-header__overlay',
    }"
  >
    <template #title>
      <span class="site-header__brand">Dan San Souci</span>
    </template>

    <template #right>
      <SiteNavigation class="site-header__navigation" />
      <button
        type="button"
        class="site-header__theme"
        :aria-label="dark ? 'Switch to light theme' : 'Switch to dark theme'"
        @click="toggleTheme"
      >
        <UIcon
          :name="dark ? 'i-lucide-sun' : 'i-lucide-moon'"
          class="site-header__theme-icon"
        />
      </button>
    </template>

    <template #toggle="{ open, toggle }">
      <button
        type="button"
        class="site-header__toggle"
        :class="{ 'site-header__toggle--open': open }"
        :aria-label="open ? 'Close menu' : 'Open menu'"
        :aria-expanded="open"
        @click="toggle"
      >
        <span />
        <span />
        <span />
      </button>
    </template>

    <template #body>
      <SiteNavigation mobile />
    </template>
  </UHeader>
</template>

<style>
/* Namespaced global rules also reach Nuxt UI's portalled mobile menu. */
.site-header {
  --ui-header-height: 60px;

  position: sticky;
  z-index: 50;
  top: 0;
  height: 60px;
  background: var(--portfolio-header);
  border-bottom: 1px solid var(--portfolio-header-border);
  backdrop-filter: blur(12px);
  transition: background-color 350ms ease;
}

.site-header__container,
.site-header__menu-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  width: 100%;
  max-width: none;
  height: 60px;
  padding: 0 2.5rem;
}

.site-header__left {
  flex: 0 0 auto;
}

.site-header__center {
  display: none;
}

.site-header__right {
  display: flex;
  flex: 0 0 auto;
  align-items: center;
  gap: 2rem;
}

.site-header__brand {
  font-family: "Cinzel", serif;
  font-size: 0.85rem;
  font-weight: 400;
  letter-spacing: 0.16em;
  color: var(--portfolio-text);
  text-transform: uppercase;
}

.site-header__theme,
.site-header__toggle {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 4px;
  border: 0;
  background: none;
  color: var(--portfolio-muted);
  cursor: pointer;
}

.site-header__theme-icon {
  width: 16px;
  height: 16px;
}

.site-header__theme:focus-visible,
.site-header__toggle:focus-visible {
  outline: 2px solid var(--portfolio-accent);
  outline-offset: 3px;
}

.site-header__toggle,
.site-header__menu,
.site-header__overlay {
  display: none;
}

@media (max-width: 767px) {
  .site-header__container,
  .site-header__menu-header {
    padding: 0 1.25rem;
  }

  .site-header__brand {
    font-size: 0.75rem;
  }

  .site-header__right nav.site-header__navigation {
    display: none;
  }

  .site-header__right {
    gap: 0.25rem;
  }

  .site-header__theme,
  .site-header__toggle {
    padding: 10px;
  }

  .site-header__toggle {
    display: flex;
    flex-direction: column;
    gap: 5px;
    color: var(--portfolio-text);
  }

  .site-header__toggle span {
    display: block;
    width: 22px;
    height: 1px;
    background: currentColor;
    transition:
      transform 300ms ease,
      opacity 300ms ease;
  }

  .site-header__toggle--open span:first-child {
    transform: translateY(6px) rotate(45deg);
  }

  .site-header__toggle--open span:nth-child(2) {
    opacity: 0;
  }

  .site-header__toggle--open span:last-child {
    transform: translateY(-6px) rotate(-45deg);
  }

  .site-header__menu {
    display: flex;
    flex-direction: column;
    background: var(--portfolio-bg);
    color: var(--portfolio-text);
  }

  .site-header__menu-header {
    flex-shrink: 0;
  }

  .site-header__menu-body {
    display: flex;
    flex: 1;
    flex-direction: column;
    justify-content: center;
    padding: 2rem 2.5rem;
    overflow-y: auto;
  }
}

@media (prefers-reduced-motion: reduce) {
  .site-header,
  .site-header__toggle span {
    transition: none;
  }
}
</style>
