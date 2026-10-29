<script setup lang="ts">
definePageMeta({
  validate: (route) => /^\d+$/.test(String(route.params.category_id)),
});

const route = useRoute();
const categoryId = computed(() => Number(route.params.category_id));
const { apiBase } = useRuntimeConfig().public;

const [{ data: categories }, { data: paintings }] = await Promise.all([
  useFetch<Category[]>("/categories", {
    baseURL: apiBase,
    default: () => [],
  }),
  useFetch<Painting[]>(() => `/paintings/${categoryId.value}`, {
    baseURL: apiBase,
    default: () => [],
  }),
]);

const category = computed(() =>
  categories.value.find((c) => c.category_id === categoryId.value),
);

if (!category.value) {
  throw createError({
    statusCode: 404,
    statusMessage: "Category not found",
    fatal: true,
  });
}

// TODO: replace with a description column on the categories table
const description =
  "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.";

useSeoMeta({
  title: () => `${category.value?.name} | Dan San Souci`,
  description,
});

const selectedIndex = ref<number | null>(null);
const selected = computed(() =>
  selectedIndex.value === null
    ? undefined
    : paintings.value[selectedIndex.value],
);

// img_lg is nullable; fall back to img_md so the lightbox always has a source
function fullImage(painting: Painting) {
  if (painting.img_lg && painting.img_lg_width && painting.img_lg_height) {
    return {
      src: painting.img_lg,
      width: painting.img_lg_width,
      height: painting.img_lg_height,
    };
  }

  return {
    src: painting.img_md,
    width: painting.img_md_width,
    height: painting.img_md_height,
  };
}

const selectedImage = computed(
  () => selected.value && fullImage(selected.value),
);

const isOpen = computed({
  get: () => selectedIndex.value !== null,
  set: (open) => {
    if (!open) {
      selectedIndex.value = null;
    }
  },
});

function step(offset: number) {
  if (selectedIndex.value === null) {
    return;
  }

  const total = paintings.value.length;
  selectedIndex.value = (selectedIndex.value + offset + total) % total;
}

defineShortcuts({
  arrowleft: () => step(-1),
  arrowright: () => step(1),
});

// warm the cache with the neighbouring full-size images so stepping is instant
watch(selectedIndex, (index) => {
  if (index === null) {
    return;
  }

  const total = paintings.value.length;
  for (const offset of [-1, 1]) {
    const neighbour = paintings.value[(index + offset + total) % total];
    if (neighbour) {
      new Image().src = fullImage(neighbour).src;
    }
  }
});
</script>

<template>
  <section class="gallery" aria-labelledby="gallery-title">
    <header class="gallery__header">
      <NuxtLink to="/categories" class="gallery__back">← Paintings</NuxtLink>
      <h1 id="gallery-title" class="gallery__title">
        {{ category?.name }}
      </h1>
      <p class="gallery__description">
        {{ description }}
      </p>
    </header>

    <ul class="gallery__grid">
      <li
        v-for="(painting, index) in paintings"
        :key="painting.id"
        class="gallery__item"
      >
        <figure class="gallery__figure">
          <button
            type="button"
            class="gallery__open"
            :aria-label="`View ${painting.name}`"
            @click="selectedIndex = index"
          >
            <img
              :src="painting.img_md"
              :width="painting.img_md_width"
              :height="painting.img_md_height"
              :alt="painting.name"
              loading="lazy"
              class="gallery__image"
            />
          </button>
          <figcaption>
            <p class="gallery__name">
              {{ painting.name }}
            </p>
            <p class="gallery__medium">
              {{ painting.medium }}
              <template v-if="painting.dimensions">
                · {{ painting.dimensions }}
              </template>
            </p>
          </figcaption>
        </figure>
      </li>
    </ul>

    <UModal
      v-model:open="isOpen"
      fullscreen
      :title="selected?.name"
      :description="selected?.medium"
      :ui="{ content: 'painting-lightbox' }"
    >
      <template #content="{ close }">
        <div
          v-if="selected && selectedImage"
          class="lightbox"
          @click.self="close"
        >
          <figure class="lightbox__figure">
            <img
              :key="selected.id"
              :src="selectedImage.src"
              :width="selectedImage.width"
              :height="selectedImage.height"
              :alt="selected.name"
              :style="{
                '--width': selectedImage.width,
                '--height': selectedImage.height,
                'background-image': `url(${selected.img_md})`,
              }"
              decoding="async"
              class="lightbox__image"
            />
            <figcaption class="lightbox__caption">
              <p class="lightbox__name">
                {{ selected.name }}
              </p>
              <p class="lightbox__medium">
                {{ selected.medium }}
                <template v-if="selected.dimensions">
                  · {{ selected.dimensions }}
                </template>
              </p>
            </figcaption>
          </figure>

          <button
            type="button"
            class="lightbox__button lightbox__button--prev"
            aria-label="Previous painting"
            @click="step(-1)"
          >
            <UIcon name="i-lucide-chevron-left" />
          </button>
          <button
            type="button"
            class="lightbox__button lightbox__button--next"
            aria-label="Next painting"
            @click="step(1)"
          >
            <UIcon name="i-lucide-chevron-right" />
          </button>
          <button
            type="button"
            class="lightbox__button lightbox__button--close"
            aria-label="Close"
            @click="close"
          >
            <UIcon name="i-lucide-x" />
          </button>

          <p class="lightbox__counter">
            {{ (selectedIndex ?? 0) + 1 }} / {{ paintings.length }}
          </p>
        </div>
      </template>
    </UModal>
  </section>
</template>

<style scoped>
.gallery {
  max-width: 1400px;
  padding: 4rem 3rem 8rem;
  margin: 0 auto;
}

.gallery__header {
  margin-bottom: 4.5rem;
}

.gallery__back {
  display: inline-flex;
  align-items: center;
  gap: 0.4rem;
  margin-bottom: 2.5rem;
  font-size: 0.7rem;
  font-weight: 300;
  letter-spacing: 0.1em;
  text-transform: uppercase;
  text-decoration: none;
  color: var(--portfolio-muted);
  transition: color 200ms ease;
}

.gallery__back:hover,
.gallery__back:focus-visible {
  color: var(--portfolio-accent);
}

.gallery__title {
  margin: 0 0 0.75rem;
  font-family: "Cinzel", serif;
  font-size: clamp(1.6rem, 3vw, 2.6rem);
  font-weight: 400;
  letter-spacing: 0.05em;
}

.gallery__description {
  max-width: 520px;
  margin: 0;
  font-size: 0.9rem;
  font-weight: 300;
  line-height: 1.8;
  color: var(--portfolio-muted);
}

.gallery__grid {
  padding: 0;
  margin: 0;
  list-style: none;
  columns: 2;
  column-gap: 2.5rem;
}

.gallery__item {
  margin-bottom: 2.5rem;
  break-inside: avoid;
}

.gallery__figure {
  margin: 0;
}

.gallery__open {
  display: block;
  width: 100%;
  padding: 0;
  margin-bottom: 0.85rem;
  overflow: hidden;
  border: 0;
  background: var(--portfolio-image-bg);
  cursor: pointer;
}

.gallery__open:focus-visible {
  outline: 2px solid var(--portfolio-accent);
  outline-offset: 3px;
}

.gallery__image {
  display: block;
  width: 100%;
  height: auto;
  max-height: 70vh;
  object-fit: cover;
  filter: brightness(0.92);
  transition:
    transform 700ms cubic-bezier(0.25, 0.46, 0.45, 0.94),
    filter 400ms ease;
}

.gallery__open:hover .gallery__image,
.gallery__open:focus-visible .gallery__image {
  filter: brightness(1);
  transform: scale(1.02);
}

.gallery__name {
  margin: 0 0 0.3rem;
  font-family: "Cinzel", serif;
  font-size: clamp(0.78rem, 1.1vw, 0.95rem);
  letter-spacing: 0.04em;
  color: var(--portfolio-text);
}

.gallery__medium {
  margin: 0;
  font-size: 0.72rem;
  font-weight: 300;
  letter-spacing: 0.04em;
  color: var(--portfolio-muted);
}

.lightbox {
  position: relative;
  display: flex;
  align-items: center;
  justify-content: center;
  width: 100%;
  height: 100%;
}

.lightbox__figure {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 1.5rem;
  max-width: 90vw;
  margin: 0;
}

/* sized up front from the API dimensions so the thumbnail placeholder fills
   the final box while the full image loads */
.lightbox__image {
  display: block;
  width: min(90vw, 78vh * var(--width) / var(--height), var(--width) * 1px);
  height: auto;
  background: center / cover no-repeat;
  object-fit: contain;
}

.lightbox__caption {
  text-align: center;
}

.lightbox__name {
  margin: 0 0 0.35rem;
  font-family: "Cinzel", serif;
  font-size: 0.88rem;
  letter-spacing: 0.06em;
  color: rgb(255 255 255 / 0.9);
}

.lightbox__medium {
  margin: 0;
  font-size: 0.72rem;
  font-weight: 300;
  letter-spacing: 0.04em;
  color: rgb(255 255 255 / 0.38);
}

.lightbox__button {
  position: absolute;
  display: flex;
  align-items: center;
  justify-content: center;
  min-width: 44px;
  min-height: 44px;
  padding: 1rem 1.5rem;
  border: 0;
  background: none;
  font-size: 2rem;
  color: rgb(255 255 255 / 0.5);
  cursor: pointer;
  transition: color 200ms ease;
}

.lightbox__button:hover,
.lightbox__button:focus-visible {
  color: rgb(255 255 255 / 0.9);
}

.lightbox__button--prev,
.lightbox__button--next {
  top: 50%;
  transform: translateY(-50%);
}

.lightbox__button--prev {
  left: 0;
}

.lightbox__button--next {
  right: 0;
}

.lightbox__button--close {
  top: 1rem;
  right: 1rem;
  padding: 0.5rem;
  font-size: 1.4rem;
}

.lightbox__counter {
  position: absolute;
  bottom: 2rem;
  left: 50%;
  margin: 0;
  font-size: 0.68rem;
  font-weight: 300;
  letter-spacing: 0.12em;
  color: rgb(255 255 255 / 0.25);
  transform: translateX(-50%);
}

@media (max-width: 767px) {
  .gallery {
    padding: 2rem 1.25rem 5rem;
  }

  .gallery__grid {
    columns: 1;
  }

  .lightbox__button--prev,
  .lightbox__button--next {
    padding: 1.5rem 1.25rem;
  }
}

@media (prefers-reduced-motion: reduce) {
  .gallery__back,
  .gallery__image,
  .lightbox__button {
    transition: none;
  }
}
</style>

<style>
.painting-lightbox {
  background: rgb(0 0 0 / 0.96);
  font-family: "Inter", sans-serif;
}
</style>
