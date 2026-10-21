<script setup lang="ts">
const title = "About | Dan San Souci";
const description = "Meet Dan San Souci, the San Francisco Bay Area painter and illustrator known for his wildlife paintings, landscapes, and celebrated children's books.";

useSeoMeta({
  title,
  description,
  ogTitle: title,
  ogDescription: description,
});

const selectedWorks = [
  { title: "Brave Dog Balto", year: "1989" },
  { title: "The Legend of Scarface", year: "1978" },
  { title: "North Country Night", year: "1990" },
  { title: "Two Bear Cubs", year: "1997" },
  { title: "In the Moonlight Mist", year: "1999" },
  { title: "Clan of the Shape-Shifters", year: "1994" },
] as const;
</script>

<template>
  <article class="about" aria-labelledby="about-title">
    <div class="about__hero">
      <NuxtImg
        src="/dan-about.jpeg"
        alt="Dan San Souci"
        :width="5712"
        :height="3213"
        sizes="sm:100vw md:100vw lg:100vw xl:100vw 1920:100vw"
        format="webp"
        loading="eager"
        fetchpriority="high"
        preload
        class="about__hero-image"
      />
    </div>

    <div class="about__content">
      <header class="about__heading">
        <p class="about__eyebrow">
          About the Artist
        </p>
        <h1 id="about-title" class="about__title">
          Dan San Souci
        </h1>
        <div class="about__accent-rule" aria-hidden="true" />
      </header>

      <div class="about__biography">
        <p>
          Daniel San Souci began his career as a professional illustrator in the 1970s, bringing
          to life the stories his brother Robert D. San Souci wrote and retold. Together they
          produced a body of work spanning Native American legends, fairy tales, and original
          adventures that have been read by millions of children around the world.
        </p>
        <p>
          Trained as a fine artist, Dan has always approached illustration as painting first.
          His oils are built up slowly — layer by layer — to achieve the luminous depth and
          atmospheric naturalism that define his style. He works from life whenever possible,
          making studies of landscapes and animals before composing a final image.
        </p>

        <figure class="about__inline-photo">
          <NuxtImg
            src="/about/dan-day-of-the-dead.jpg"
            alt="Dan San Souci with Day of the Dead artwork"
            :width="3233"
            :height="3149"
            sizes="sm:320px md:260px"
            format="webp"
            loading="lazy"
            class="about__inline-image"
          />
        </figure>

        <p>
          His most celebrated books include <em>Brave Dog Balto</em>, <em>The Legend of Scarface</em>,
          <em>Two Bear Cubs</em>, and <em>North Country Night</em> — the last of which he both wrote and
          illustrated. The book remains a quiet classic of environmental picture-book publishing.
        </p>
        <p>
          Outside of publishing, Dan paints landscapes and wildlife in the tradition of the
          American naturalist painters. His paintings have been exhibited in galleries throughout
          California and the Pacific Northwest.
        </p>
        <p>
          He lives and works in the San Francisco Bay Area.
        </p>
      </div>

      <section class="about__works" aria-labelledby="selected-works-title">
        <h2 id="selected-works-title" class="about__works-title">
          Selected Works
        </h2>
        <ul class="about__works-list">
          <li v-for="work in selectedWorks" :key="work.title" class="about__work">
            <span class="about__work-title">{{ work.title }}</span>
            <time class="about__work-year" :datetime="work.year">{{ work.year }}</time>
          </li>
        </ul>
      </section>
    </div>
  </article>
</template>

<style scoped>
.about {
  --about-body-color: rgb(26 23 20 / 0.8);
  --about-muted-color: rgb(26 23 20 / 0.45);
  --about-photo-brightness: 1;
}

.dark .about {
  --about-body-color: rgb(232 227 216 / 0.82);
  --about-muted-color: rgb(232 227 216 / 0.45);
  --about-photo-brightness: 0.92;
}

.about__hero {
  position: relative;
  height: clamp(300px, 45vw, 520px);
  overflow: hidden;
  background: var(--portfolio-image-bg);
}

.about__hero::after {
  position: absolute;
  inset: 0;
  content: "";
  background: linear-gradient(to bottom, transparent 50%, var(--portfolio-bg) 100%);
  pointer-events: none;
}

.about__hero-image {
  display: block;
  width: 100%;
  height: 100%;
  object-fit: cover;
  object-position: center 20%;
}

.about__content {
  width: 100%;
  max-width: 800px;
  margin: 0 auto;
  padding: 3rem 2.5rem 6rem;
}

.about__eyebrow {
  margin: 0 0 0.75rem;
  font-family: "Cinzel", serif;
  font-size: 0.72rem;
  letter-spacing: 0.22em;
  text-transform: uppercase;
  color: var(--portfolio-accent);
}

.about__title {
  margin: 0 0 1.25rem;
  font-family: "Cinzel", serif;
  font-size: clamp(1.6rem, 3vw, 2.4rem);
  font-weight: 400;
  letter-spacing: 0.06em;
}

.about__accent-rule {
  width: 40px;
  height: 1px;
  margin-bottom: 3rem;
  background: var(--portfolio-accent);
}

.about__biography {
  display: flow-root;
  font-family: "Inter", sans-serif;
  font-size: 1rem;
  font-weight: 300;
  line-height: 1.9;
  color: var(--about-body-color);
}

.about__biography p {
  margin: 0;
}

.about__biography p + p,
.about__inline-photo + p {
  margin-top: 1.5rem;
}

.about__inline-photo {
  float: right;
  clear: right;
  width: clamp(180px, 38%, 260px);
  margin: 2rem 0 0.5rem 2rem;
}

.about__inline-image {
  display: block;
  width: 100%;
  height: auto;
  filter: brightness(var(--about-photo-brightness));
}

.about__works {
  margin-top: 3.5rem;
  padding-top: 3rem;
  border-top: 1px solid var(--portfolio-header-border);
}

.about__works-title {
  margin: 0 0 1.5rem;
  font-family: "Cinzel", serif;
  font-size: 0.7rem;
  font-weight: 400;
  letter-spacing: 0.2em;
  text-transform: uppercase;
  color: var(--about-muted-color);
}

.about__works-list {
  display: flex;
  flex-direction: column;
  gap: 0.6rem;
  margin: 0;
  padding: 0;
  list-style: none;
}

.about__work {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  gap: 1rem;
  padding-bottom: 0.6rem;
  border-bottom: 1px solid var(--portfolio-header-border);
  font-family: "Inter", sans-serif;
  font-weight: 300;
}

.about__work-title {
  font-size: 0.88rem;
}

.about__work-year {
  flex-shrink: 0;
  font-size: 0.72rem;
  color: var(--about-muted-color);
}

@media (max-width: 480px) {
  .about__content {
    padding-right: 1.25rem;
    padding-left: 1.25rem;
  }

  .about__inline-photo {
    float: none;
    width: min(100%, 320px);
    margin: 2rem auto;
  }
}
</style>
