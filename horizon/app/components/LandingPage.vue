<script setup lang="ts">
const SLIDE_DURATION = 7000;
const FADE_DURATION = 1800;

const slides = [
  {
    src: "/landing/slideshow_001.jpg",
    width: 2000,
    height: 1500,
    alt: "Dan San Souci illustration of a herd of elephants",
  },
  {
    src: "/landing/slideshow_002.jpg",
    width: 2048,
    height: 1529,
    alt: "Dan San Souci portrait of a woman beside a lion",
  },
  {
    src: "/landing/slideshow_003.jpg",
    width: 2048,
    height: 1531,
    alt: "Dan San Souci cityscape with flower-covered arches",
  },
] as const;

const currentSlide = ref(0);
const animationRuns = ref(slides.map(() => 0));
const timingStyles = {
  "--landing-fade-duration": `${FADE_DURATION}ms`,
  "--landing-animation-duration": `${SLIDE_DURATION + FADE_DURATION}ms`,
};

let autoplayTimer: ReturnType<typeof setInterval> | undefined;
let reducedMotionQuery: MediaQueryList | undefined;

function showSlide(index: number) {
  animationRuns.value[index] = (animationRuns.value[index] ?? 0) + 1;
  currentSlide.value = index;
}

function showNextSlide() {
  showSlide((currentSlide.value + 1) % slides.length);
}

function startAutoplay() {
  if (autoplayTimer || reducedMotionQuery?.matches) {
    return;
  }

  autoplayTimer = setInterval(showNextSlide, SLIDE_DURATION);
}

function stopAutoplay() {
  if (!autoplayTimer) {
    return;
  }

  clearInterval(autoplayTimer);
  autoplayTimer = undefined;
}

function handleVisibilityChange() {
  if (document.hidden) {
    stopAutoplay();
  } else {
    startAutoplay();
  }
}

function handleMotionPreference() {
  if (reducedMotionQuery?.matches) {
    stopAutoplay();
  } else if (!document.hidden) {
    startAutoplay();
  }
}

onMounted(() => {
  reducedMotionQuery = window.matchMedia("(prefers-reduced-motion: reduce)");
  reducedMotionQuery.addEventListener("change", handleMotionPreference);
  document.addEventListener("visibilitychange", handleVisibilityChange);
  startAutoplay();
});

onBeforeUnmount(() => {
  stopAutoplay();
  reducedMotionQuery?.removeEventListener("change", handleMotionPreference);
  document.removeEventListener("visibilitychange", handleVisibilityChange);
});
</script>

<template>
  <main
    class="landing"
    :style="timingStyles"
    role="region"
    aria-roledescription="carousel"
    aria-label="Selected artwork by Dan San Souci"
  >
    <div
      v-for="(slide, index) in slides"
      :key="slide.src"
      class="landing__slide"
      :class="{ 'landing__slide--active': index === currentSlide }"
      :aria-hidden="index !== currentSlide"
    >
      <NuxtImg
        :key="`${slide.src}-${animationRuns[index]}`"
        :src="slide.src"
        :alt="slide.alt"
        :width="slide.width"
        :height="slide.height"
        sizes="sm:100vw md:100vw lg:100vw xl:100vw"
        format="webp"
        :preload="index === 0"
        :fetchpriority="index === 0 ? 'high' : 'auto'"
        :loading="index === 0 ? 'eager' : 'lazy'"
        class="landing__image"
        :class="`landing__image--${index + 1}`"
        draggable="false"
      />
    </div>

    <div class="landing__vignette" aria-hidden="true" />
    <div class="landing__top-gradient" aria-hidden="true" />

    <header class="landing__header">
      <h1 class="landing__title">
        Dan San Souci
      </h1>

      <NuxtLink class="landing__enter" to="/paintings">
        Enter
      </NuxtLink>
    </header>

    <nav class="landing__indicators" aria-label="Choose artwork">
      <button
        v-for="(_, index) in slides"
        :key="index"
        type="button"
        class="landing__indicator"
        :aria-label="`Show artwork ${index + 1}`"
        :aria-current="index === currentSlide ? 'true' : undefined"
        @click="showSlide(index)"
      >
        <span class="landing__indicator-mark" aria-hidden="true" />
      </button>
    </nav>
  </main>
</template>

<style scoped>
.landing {
  --landing-fade-duration: 1800ms;
  --landing-animation-duration: 8800ms;

  position: relative;
  width: 100%;
  height: 100vh;
  height: 100dvh;
  min-height: 30rem;
  overflow: hidden;
  color: #fff;
  background: #000;
}

.landing__slide {
  position: absolute;
  z-index: 0;
  inset: 0;
  opacity: 0;
  transition: opacity var(--landing-fade-duration) ease-in-out;
}

.landing__slide--active {
  z-index: 1;
  opacity: 1;
}

.landing__image {
  width: 100%;
  height: 100%;
  object-fit: cover;
  transform-origin: center;
  user-select: none;
  pointer-events: none;
}

.landing__image--1 {
  animation: landing-kenburns-1 var(--landing-animation-duration) ease-out forwards;
}

.landing__image--2 {
  animation: landing-kenburns-2 var(--landing-animation-duration) ease-out forwards;
}

.landing__image--3 {
  animation: landing-kenburns-3 var(--landing-animation-duration) ease-out forwards;
}

.landing__vignette,
.landing__top-gradient {
  position: absolute;
  z-index: 2;
  pointer-events: none;
}

.landing__vignette {
  inset: 0;
  background: radial-gradient(
    ellipse at center,
    transparent 40%,
    rgb(0 0 0 / 0.45) 100%
  );
}

.landing__top-gradient {
  top: 0;
  right: 0;
  left: 0;
  height: 10rem;
  background: linear-gradient(to bottom, rgb(0 0 0 / 0.5), transparent);
}

.landing__header {
  position: absolute;
  z-index: 3;
  top: 0;
  right: 0;
  left: 0;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 2.25rem 2.5rem;
}

.landing__title {
  margin: 0;
  font-family: "Cinzel", serif;
  font-size: clamp(0.85rem, 1.6vw, 1.15rem);
  font-weight: 400;
  line-height: 1.2;
  letter-spacing: 0.18em;
  text-transform: uppercase;
  text-shadow: 0 1px 16px rgb(0 0 0 / 0.5);
  user-select: none;
}

.landing__enter {
  position: relative;
  display: inline-block;
  padding: 0.25rem 0 0.625rem;
  font-family: "Inter", sans-serif;
  font-size: clamp(0.825rem, 1.56vw, 1.125rem);
  font-weight: 300;
  line-height: 1;
  letter-spacing: 0.1em;
  color: rgb(255 255 255 / 0.9);
  text-decoration: none;
  text-transform: uppercase;
  text-shadow: 0 1px 16px rgb(0 0 0 / 0.35);
}

.landing__enter::after {
  position: absolute;
  right: 0;
  bottom: 0;
  left: 0;
  height: 1px;
  content: "";
  background: rgb(255 255 255 / 0.7);
  transform: scaleX(0);
  transform-origin: left;
  transition: transform 400ms cubic-bezier(0.4, 0, 0.2, 1);
}

.landing__enter:hover::after,
.landing__enter:focus-visible::after {
  transform: scaleX(1);
}

.landing__enter:focus-visible,
.landing__indicator:focus-visible {
  outline: 2px solid rgb(255 255 255 / 0.9);
  outline-offset: 3px;
}

.landing__indicators {
  position: absolute;
  z-index: 3;
  bottom: max(2rem, env(safe-area-inset-bottom));
  left: 50%;
  display: flex;
  transform: translateX(-50%);
}

.landing__indicator {
  display: flex;
  align-items: center;
  padding: 0.75rem 0.5rem;
  color: inherit;
  background: transparent;
  border: 0;
  cursor: pointer;
}

.landing__indicator-mark {
  display: block;
  width: 0.5rem;
  height: 2px;
  background: rgb(255 255 255 / 0.35);
  transition:
    width 400ms ease,
    background-color 400ms ease;
}

.landing__indicator[aria-current="true"] .landing__indicator-mark {
  width: 1.75rem;
  background: rgb(255 255 255 / 0.9);
}

@keyframes landing-kenburns-1 {
  from {
    transform: scale(1) translate(0, 0);
  }

  to {
    transform: scale(1.12) translate(-2%, -1.5%);
  }
}

@keyframes landing-kenburns-2 {
  from {
    transform: scale(1.05) translate(1%, 0.5%);
  }

  to {
    transform: scale(1.15) translate(-1.5%, -2%);
  }
}

@keyframes landing-kenburns-3 {
  from {
    transform: scale(1) translate(-1%, 1%);
  }

  to {
    transform: scale(1.1) translate(2%, -1%);
  }
}

@media (max-width: 40rem) {
  .landing__header {
    padding: 1.75rem 1.5rem;
  }
}

@media (prefers-reduced-motion: reduce) {
  .landing__slide,
  .landing__indicator-mark,
  .landing__enter::after {
    transition: none;
  }

  .landing__image {
    animation: none;
  }
}
</style>
