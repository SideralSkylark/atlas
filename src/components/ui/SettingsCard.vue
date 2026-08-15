<script setup lang="ts">
withDefaults(
  defineProps<{
    title?: string;
    description?: string;
  }>(),
  {
    title: "",
    description: "",
  }
);
</script>

<template>
  <section
    class="p-5 bg-bg1 border border-border rounded-xl shadow-sm overflow-hidden"
    style="box-shadow: var(--shadow-sm), var(--shadow-inset)"
  >
    <div v-if="title || description" class="flex items-center gap-2 mb-5 font-sans">
      <slot name="icon" />
      <div>
        <h2 v-if="title" class="text-yellow font-bold text-lg font-sans leading-none">
          {{ title }}
        </h2>
        <p
          v-if="description"
          class="text-[10px] uppercase tracking-widest font-bold text-fg-dim mt-1.5"
        >
          {{ description }}
        </p>
      </div>
    </div>

    <slot />
  </section>
</template>
