<script setup lang="ts">
defineProps<{ mobile?: boolean }>();
</script>

<template>
  <nav
    class="site-navigation"
    :class="{ 'site-navigation--mobile': mobile }"
    aria-label="Main navigation"
  >
    <NuxtLink class="site-navigation__link" to="/categories">
      Paintings
    </NuxtLink>
    <NuxtLink class="site-navigation__link" to="/about"> About </NuxtLink>
    <NuxtLink class="site-navigation__link" to="/contact"> Contact </NuxtLink>
  </nav>
</template>

<style scoped>
.site-navigation {
  display: flex;
  align-items: center;
  gap: 2rem;
}

.site-navigation__link {
  padding-bottom: 2px;
  border-bottom: 1px solid transparent;
  font-family: "Inter", sans-serif;
  font-size: 0.72rem;
  font-weight: 300;
  letter-spacing: 0.1em;
  text-transform: uppercase;
  text-decoration: none;
  color: var(--portfolio-muted);
  transition: color 200ms ease;
}

.site-navigation__link.router-link-active {
  color: var(--portfolio-accent);
  border-bottom-color: currentColor;
}

.site-navigation__link:focus-visible {
  outline: 2px solid var(--portfolio-accent);
  outline-offset: 4px;
}

.site-navigation__link--disabled {
  opacity: 0.5;
  cursor: default;
}

.site-navigation--mobile {
  flex-direction: column;
  align-items: stretch;
  gap: 0;
}

.site-navigation--mobile .site-navigation__link {
  display: block;
  padding: 0.75rem 0;
  border-bottom-color: var(--portfolio-divider);
  font-family: "Cinzel", serif;
  font-size: 1.8rem;
  font-weight: 400;
  letter-spacing: 0.06em;
  text-transform: none;
}

@media (prefers-reduced-motion: reduce) {
  .site-navigation__link {
    transition: none;
  }
}
</style>
