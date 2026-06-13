<script setup lang="ts">
import { ref } from "vue";
import { Clipboard, ClipboardCheck } from "@lucide/vue";
import type { RenderedFile } from "../composables/useFileSystem";

defineProps<{
  file: RenderedFile;
}>();

const copied = ref(false);

async function copyContent(content: string) {
  // Strip HTML tags if it's code (syntect output)
  const tempDiv = document.createElement("div");
  tempDiv.innerHTML = content;
  const textToCopy = tempDiv.textContent || tempDiv.innerText || "";
  
  try {
    await navigator.clipboard.writeText(textToCopy);
    copied.value = true;
    setTimeout(() => {
      copied.value = false;
    }, 1500);
  } catch (err) {
    console.error("Failed to copy: ", err);
  }
}
</script>

<template>
  <div class="space-y-4">
    <div class="bg-bg1 border border-border rounded-xl overflow-hidden shadow-sm">
      <!-- Code / Plain / HTML -->
      <div v-if="file.file_type === 'code' || file.file_type === 'plain' || file.file_type === 'html'" class="relative">
        <div v-if="file.file_type === 'code'" class="absolute right-4 top-4 z-10">
          <button
            @click="copyContent(file.content)"
            class="p-2 bg-bg0/50 backdrop-blur-md border border-border rounded-lg text-fg-dim hover:text-fg hover:border-fg-dim transition-all active:scale-95 cursor-pointer shadow-lg"
            :title="copied ? 'Copied!' : 'Copy to clipboard'"
          >
            <ClipboardCheck v-if="copied" :size="16" class="text-green" />
            <Clipboard v-else :size="16" />
          </button>
        </div>

        <div class="p-4">
          <div v-if="file.file_type === 'code'" v-html="file.content" class="text-sm font-mono leading-relaxed whitespace-pre-wrap break-all"></div>
          <div v-else-if="file.file_type === 'plain'" class="max-h-[70vh] overflow-y-auto custom-scrollbar">
            <pre class="text-sm text-fg leading-relaxed font-mono whitespace-pre-wrap break-words max-w-full overflow-x-hidden">{{ file.content }}</pre>
          </div>
          <div v-else-if="file.file_type === 'html'" v-html="file.content" class="bg-white text-black p-2 rounded min-h-[200px] max-w-full overflow-x-hidden"></div>
        </div>
      </div>

      <!-- Markdown -->
      <div v-else-if="file.file_type === 'markdown'" class="p-6 prose-custom">
        <div v-html="file.content"></div>
      </div>
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

.custom-scrollbar::-webkit-scrollbar {
  width: 6px;
}
.custom-scrollbar::-webkit-scrollbar-track {
  background: transparent;
}
.custom-scrollbar::-webkit-scrollbar-thumb {
  background: var(--color-bg3);
  border-radius: 10px;
}
.custom-scrollbar::-webkit-scrollbar-thumb:hover {
  background: var(--color-border);
}
</style>
