<template>
  <div class="portfolio-shell">
    <SiteHeader />
    <main>
      <slot />
    </main>
  </div>
</template>

<style>
:root {
  --portfolio-bg: #f2ede6;
  --portfolio-header: rgb(242 237 230 / 0.94);
  --portfolio-text: #1a1714;
  --portfolio-muted: rgb(26 23 20 / 0.5);
  --portfolio-meta: rgb(26 23 20 / 0.32);
  --portfolio-accent: #8a6535;
  --portfolio-divider: rgb(0 0 0 / 0.07);
  --portfolio-header-border: rgb(0 0 0 / 0.08);
  --portfolio-image-bg: #dedad4;
}

:root.dark {
  --portfolio-bg: #0e0d0c;
  --portfolio-header: rgb(14 13 12 / 0.94);
  --portfolio-text: #e8e3d8;
  --portfolio-muted: rgb(232 227 216 / 0.55);
  --portfolio-meta: rgb(232 227 216 / 0.32);
  --portfolio-accent: #c8a96e;
  --portfolio-divider: rgb(255 255 255 / 0.06);
  --portfolio-header-border: rgb(255 255 255 / 0.07);
  --portfolio-image-bg: #161412;
}

.portfolio-shell {
  min-height: 100vh;
  min-height: 100dvh;
  background: var(--portfolio-bg);
  color: var(--portfolio-text);
  font-family: "Inter", sans-serif;
  transition: background-color 350ms ease, color 350ms ease;
}

@media (prefers-reduced-motion: reduce) {
  .portfolio-shell {
    transition: none;
  }
}
</style>
