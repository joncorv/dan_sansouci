<script setup lang="ts">
useSeoMeta({
  title: "Paintings | Dan San Souci",
  description:
    "Explore original paintings by Dan San Souci, from wildlife and landscapes to portraits, illustrations, and book art.",
});

const { apiBase } = useRuntimeConfig().public;

const { data: categories } = await useFetch<Category[]>("/categories", {
  baseURL: apiBase,
  default: () => [],
});

const hoveredCategory = ref<number | null>(null);
const focusedCategory = ref<number | null>(null);
const activeCategory = computed(
  () => hoveredCategory.value ?? focusedCategory.value,
);
</script>

<template>
  <section
    class="paintings"
    :class="{ 'paintings--previewing': activeCategory !== null }"
    aria-labelledby="paintings-title"
  >
    <div class="paintings__heading">
      <p class="paintings__eyebrow">Original Work</p>
      <h1 id="paintings-title" class="paintings__title">Paintings</h1>
    </div>

    <div class="paintings__backgrounds" aria-hidden="true">
      <div
        v-for="category in categories"
        :key="category.category_id"
        class="paintings__background"
        :class="{
          'paintings__background--active':
            activeCategory === category.category_id,
        }"
      >
        <img
          :src="category.img_lg"
          alt=""
          loading="lazy"
          class="paintings__background-image"
        />
      </div>
    </div>

    <ul class="paintings__categories">
      <li
        v-for="(category, index) in categories"
        :key="category.category_id"
        class="paintings__category"
        :class="{
          'paintings__category--active':
            activeCategory === category.category_id,
        }"
      >
        <NuxtLink
          :to="`/paintings/${category.category_id}`"
          class="paintings__link"
          @mouseenter="hoveredCategory = category.category_id"
          @mouseleave="hoveredCategory = null"
          @focus="focusedCategory = category.category_id"
          @blur="focusedCategory = null"
        >
          <img
            :src="category.img_md"
            alt=""
            :loading="index < 2 ? 'eager' : 'lazy'"
            class="paintings__tile-image"
          />
          <span class="paintings__number" aria-hidden="true">
            {{ String(index + 1).padStart(2, "0") }}
          </span>
          <h2 class="paintings__name">
            {{ category.name }}
          </h2>
          <span class="paintings__count">
            <!-- {{ category.count }} works -->
            10 works
            <span class="paintings__arrow" aria-hidden="true">→</span>
          </span>
        </NuxtLink>
      </li>
    </ul>
  </section>
</template>

<style scoped>
.paintings {
  position: relative;
  isolation: isolate;
  display: flex;
  align-items: center;
  min-height: calc(100vh - 60px);
  min-height: calc(100dvh - 60px);
}

.paintings__heading {
  position: absolute;
  width: 1px;
  height: 1px;
  padding: 0;
  overflow: hidden;
  clip-path: inset(50%);
  white-space: nowrap;
}

.paintings__backgrounds {
  position: fixed;
  z-index: -1;
  inset: 0;
  pointer-events: none;
}

.paintings__background {
  position: absolute;
  inset: 0;
  opacity: 0;
  transition: opacity 650ms ease;
}

.paintings__background--active {
  opacity: 1;
}

.paintings__background::after {
  position: absolute;
  inset: 0;
  content: "";
  background: rgb(0 0 0 / 0.52);
}

.paintings__background-image {
  display: block;
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.paintings__categories {
  width: 100%;
  padding: 4rem 0;
  margin: 0;
  list-style: none;
}

.paintings__category {
  border-top: 1px solid var(--portfolio-divider);
  transition: border-color 400ms ease;
}

.paintings__category:last-child {
  border-bottom: 1px solid var(--portfolio-divider);
}

.paintings__link {
  display: flex;
  align-items: baseline;
  gap: 2rem;
  width: 100%;
  padding: clamp(1.1rem, 2vw, 1.6rem) clamp(2.5rem, 6vw, 6rem);
  color: inherit;
  text-decoration: none;
}

.paintings__link:focus-visible {
  outline: 2px solid var(--portfolio-accent);
  outline-offset: -3px;
}

.paintings__tile-image {
  display: none;
}

.paintings__number,
.paintings__count {
  flex-shrink: 0;
  font-size: 0.65rem;
  font-weight: 300;
  letter-spacing: 0.1em;
  color: var(--portfolio-meta);
  transition: color 400ms ease;
}

.paintings__number {
  width: 2rem;
}

.paintings__name {
  flex: 1;
  margin: 0;
  font-family: "Cinzel", serif;
  font-size: clamp(1.3rem, 2.8vw, 2.4rem);
  font-weight: 400;
  line-height: 1;
  letter-spacing: 0.04em;
  transition: color 350ms ease;
}

.paintings__count {
  display: flex;
  align-items: center;
  gap: 0.6rem;
  font-size: 0.68rem;
  text-transform: uppercase;
}

.paintings__arrow {
  font-size: 0.9rem;
  opacity: 0;
  transform: translateX(-8px);
  transition:
    opacity 300ms ease,
    transform 300ms ease;
}

.paintings--previewing .paintings__category {
  border-color: rgb(255 255 255 / 0.1);
}

.paintings--previewing .paintings__name {
  color: rgb(255 255 255 / 0.45);
}

.paintings--previewing .paintings__number,
.paintings--previewing .paintings__count {
  color: rgb(255 255 255 / 0.25);
}

.paintings .paintings__category--active .paintings__name,
.paintings .paintings__category--active .paintings__count {
  color: #c8a96e;
}

.paintings__category--active .paintings__arrow {
  opacity: 1;
  transform: translateX(0);
}

@media (max-width: 767px) {
  .paintings {
    display: block;
    padding-bottom: 4rem;
  }

  .paintings__heading {
    position: static;
    width: auto;
    height: auto;
    padding: 2rem 1.25rem 1.5rem;
    overflow: visible;
    clip-path: none;
    white-space: normal;
  }

  .paintings__eyebrow {
    margin: 0 0 0.5rem;
    font-family: "Cinzel", serif;
    font-size: 0.65rem;
    letter-spacing: 0.22em;
    text-transform: uppercase;
    color: var(--portfolio-accent);
  }

  .paintings__title {
    margin: 0;
    font-family: "Cinzel", serif;
    font-size: 1.5rem;
    font-weight: 400;
    letter-spacing: 0.05em;
  }

  .paintings__backgrounds,
  .paintings__number,
  .paintings__count {
    display: none;
  }

  .paintings__categories {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 2px;
    padding: 0;
  }

  .paintings__category,
  .paintings__category:last-child {
    position: relative;
    display: block;
    aspect-ratio: 1;
    padding: 0;
    overflow: hidden;
    border: 0;
    background: var(--portfolio-image-bg);
  }

  .paintings__link {
    display: block;
    height: 100%;
    padding: 0;
  }

  .paintings__tile-image {
    display: block;
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .paintings .paintings__category .paintings__name {
    position: absolute;
    right: 0;
    bottom: 0;
    left: 0;
    padding: 1.5rem 0.75rem 0.65rem;
    background: linear-gradient(to top, rgb(0 0 0 / 0.72), transparent);
    font-size: 0.7rem;
    line-height: 1.3;
    letter-spacing: 0.06em;
    color: #fff;
  }
}

@media (prefers-reduced-motion: reduce) {
  .paintings__background,
  .paintings__category,
  .paintings__name,
  .paintings__number,
  .paintings__count,
  .paintings__arrow {
    transition: none;
  }
}
</style>
