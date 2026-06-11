<script setup lang="ts">
import type { RenderedFile } from "../composables/useFileSystem";

defineProps<{
  file: RenderedFile;
}>();
</script>

<template>
  <div class="bg-bg1 border border-border rounded-xl overflow-hidden shadow-sm">
    <!-- Code / Plain / HTML -->
    <div v-if="file.file_type === 'code' || file.file_type === 'plain' || file.file_type === 'html'" class="p-4 overflow-x-auto">
      <div v-if="file.file_type === 'code'" v-html="file.content" class="text-sm font-mono leading-relaxed whitespace-pre"></div>
      <pre v-else-if="file.file_type === 'plain'" class="text-sm text-fg leading-relaxed font-mono whitespace-pre-wrap">{{ file.content }}</pre>
      <div v-else-if="file.file_type === 'html'" v-html="file.content" class="bg-white text-black p-2 rounded min-h-[200px]"></div>
    </div>

    <!-- Markdown -->
    <div v-else-if="file.file_type === 'markdown'" class="p-6 prose-custom">
      <div v-html="file.content"></div>
    </div>
  </div>
</template>

<style>
@reference "../main.css";

.prose-custom {
  @apply text-fg leading-relaxed text-sm;
}
.prose-custom h1 { @apply text-2xl font-bold text-yellow mb-4 mt-6 border-b border-border pb-2; }
.prose-custom h2 { @apply text-xl font-bold text-yellow mb-3 mt-5; }
.prose-custom h3 { @apply text-lg font-bold text-yellow mb-2 mt-4; }
.prose-custom p { @apply mb-4; }
.prose-custom ul { @apply list-disc list-inside mb-4 ml-2; }
.prose-custom ol { @apply list-decimal list-inside mb-4 ml-2; }
.prose-custom code { @apply px-1.5 py-0.5 bg-bg3 rounded text-aqua font-mono text-xs; }
.prose-custom pre { @apply p-4 bg-bg0 rounded-lg overflow-x-auto mb-4 border border-border; }
.prose-custom pre code { @apply p-0 bg-transparent text-fg; }
.prose-custom blockquote { @apply border-l-4 border-green pl-4 italic text-fg-dim mb-4; }
.prose-custom a { @apply text-aqua hover:underline; }

/* Syntect styles fix */
.syntect-highlight {
  font-family: inherit;
}
</style>
