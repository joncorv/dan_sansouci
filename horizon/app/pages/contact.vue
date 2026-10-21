<script setup lang="ts">
import { z } from "zod";
import type { FormErrorEvent } from "@nuxt/ui";

const title = "Contact | Dan San Souci";
const description = "Get in touch with Dan San Souci about original artwork, illustration commissions, or speaking engagements.";

useSeoMeta({
  title,
  description,
  ogTitle: title,
  ogDescription: description,
});

const schema = z.object({
  name: z.string().trim().min(1, "Please enter your name."),
  email: z.string().trim().pipe(z.email("Please enter a valid email address.")),
  subject: z.string().trim().optional(),
  message: z.string().trim().min(1, "Please enter a message."),
});

const state = reactive<z.input<typeof schema>>({
  name: "",
  email: "",
  subject: "",
  message: "",
});

const fieldUi = {
  label: "contact__label",
  container: "contact__field-content",
  error: "contact__error",
};

const inputUi = {
  root: "contact__control",
  base: "contact__input",
};

async function onError(event: FormErrorEvent) {
  await nextTick();
  const firstErrorId = event.errors[0]?.id;
  if (firstErrorId) {
    document.getElementById(firstErrorId)?.focus();
  }
}

function onSubmit() {
  // Validation only for now. Sending will be implemented when the API is ready.
}
</script>

<template>
  <section class="contact" aria-labelledby="contact-title">
    <header>
      <p class="contact__eyebrow">
        Get in Touch
      </p>
      <h1 id="contact-title" class="contact__title">
        Contact
      </h1>
      <div class="contact__accent-rule" aria-hidden="true" />
      <p class="contact__introduction">
        For inquiries about original artwork, illustration commissions, or speaking
        engagements, please use the form below.
      </p>
    </header>

    <UForm
      :schema="schema"
      :state="state"
      :loading-auto="false"
      class="contact__form"
      novalidate
      @error="onError"
      @submit="onSubmit"
    >
      <div class="contact__identity">
        <UFormField name="name" label="Name" :ui="fieldUi" required>
          <UInput
            v-model="state.name"
            type="text"
            autocomplete="name"
            placeholder="Your name"
            variant="none"
            :ui="inputUi"
            required
          />
        </UFormField>

        <UFormField name="email" label="Email" :ui="fieldUi" required>
          <UInput
            v-model="state.email"
            type="email"
            autocomplete="email"
            placeholder="your@email.com"
            variant="none"
            :ui="inputUi"
            required
          />
        </UFormField>
      </div>

      <UFormField name="subject" label="Subject" :ui="fieldUi">
        <UInput
          v-model="state.subject"
          type="text"
          placeholder="What is this regarding?"
          variant="none"
          :ui="inputUi"
        />
      </UFormField>

      <UFormField name="message" label="Message" :ui="fieldUi" required>
        <UTextarea
          v-model="state.message"
          :rows="7"
          placeholder="Your message..."
          variant="none"
          :ui="inputUi"
          required
        />
      </UFormField>

      <div>
        <UButton type="submit" :ui="{ base: 'contact__submit' }">
          Send Message
        </UButton>
      </div>
    </UForm>
  </section>
</template>

<style scoped>
.contact {
  --contact-introduction: rgb(26 23 20 / 0.6);
  --contact-border: rgb(0 0 0 / 0.15);
  --contact-input-bg: #e8e2d9;
  --contact-placeholder: rgb(26 23 20 / 0.3);

  width: 100%;
  max-width: 680px;
  margin: 0 auto;
  padding: 4rem 2.5rem 6rem;
}

.dark .contact {
  --contact-introduction: rgb(232 227 216 / 0.65);
  --contact-border: rgb(255 255 255 / 0.12);
  --contact-input-bg: #1a1815;
  --contact-placeholder: rgb(232 227 216 / 0.3);
}

.contact__eyebrow {
  margin: 0 0 0.75rem;
  font-family: "Cinzel", serif;
  font-size: 0.72rem;
  letter-spacing: 0.22em;
  text-transform: uppercase;
  color: var(--portfolio-accent);
}

.contact__title {
  margin: 0 0 1.25rem;
  font-family: "Cinzel", serif;
  font-size: clamp(1.6rem, 3vw, 2.4rem);
  font-weight: 400;
  letter-spacing: 0.06em;
}

.contact__accent-rule {
  width: 40px;
  height: 1px;
  margin-bottom: 1.5rem;
  background: var(--portfolio-accent);
}

.contact__introduction {
  margin: 0 0 3rem;
  font-family: "Inter", sans-serif;
  font-size: 0.95rem;
  font-weight: 300;
  line-height: 1.8;
  color: var(--contact-introduction);
}

.contact__form {
  display: flex;
  flex-direction: column;
  gap: 1.5rem;
}

.contact__identity {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 1.5rem;
}

.contact :deep(.contact__label) {
  display: block;
  margin-bottom: 0.4rem;
  font-family: "Inter", sans-serif;
  font-size: 0.68rem;
  font-weight: 300;
  letter-spacing: 0.12em;
  text-transform: uppercase;
  color: var(--portfolio-muted);
}

.contact :deep(.contact__field-content) {
  margin-top: 0;
}

.contact :deep(.contact__control) {
  display: flex;
  width: 100%;
}

.contact :deep(.contact__input) {
  box-sizing: border-box;
  width: 100%;
  padding: 0.85rem 1rem;
  border: 1px solid var(--contact-border);
  border-radius: 0;
  outline: none;
  background: var(--contact-input-bg);
  box-shadow: none;
  font-family: "Inter", sans-serif;
  font-size: 0.9rem;
  font-weight: 300;
  line-height: 1.5;
  color: var(--portfolio-text);
  appearance: none;
  transition: border-color 200ms ease;
}

.contact :deep(.contact__input::placeholder) {
  color: var(--contact-placeholder);
}

.contact :deep(textarea.contact__input) {
  min-height: 160px;
  resize: vertical;
}

.contact :deep(.contact__input:focus) {
  border-color: var(--portfolio-accent);
}

.contact :deep(.contact__input[aria-invalid="true"]) {
  border-color: var(--ui-error);
}

.contact :deep(.contact__error) {
  margin-top: 0.5rem;
  font-size: 0.8rem;
  font-weight: 300;
  line-height: 1.5;
  color: var(--ui-error);
}

.contact :deep(.contact__submit) {
  display: inline-flex;
  justify-content: center;
  padding: 0.9rem 2.5rem;
  border: 0;
  border-radius: 0;
  background: var(--portfolio-accent);
  box-shadow: none;
  font-family: "Cinzel", serif;
  font-size: 0.75rem;
  font-weight: 400;
  line-height: 1.5;
  letter-spacing: 0.18em;
  text-transform: uppercase;
  color: var(--portfolio-bg);
  cursor: pointer;
  transition: opacity 200ms ease;
}

.contact :deep(.contact__submit:hover) {
  opacity: 0.85;
}

.contact :deep(.contact__submit:focus-visible) {
  outline: 2px solid var(--portfolio-accent);
  outline-offset: 4px;
}

@media (max-width: 767px) {
  .contact {
    padding: 2.5rem 1.25rem 5rem;
  }

  .contact__identity {
    grid-template-columns: minmax(0, 1fr);
  }
}

@media (prefers-reduced-motion: reduce) {
  .contact :deep(.contact__input),
  .contact :deep(.contact__submit) {
    transition: none;
  }
}
</style>
