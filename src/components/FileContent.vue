<script setup lang="ts">
import { ref, computed, watch, onUnmounted, onMounted } from "vue";
import { 
  Clipboard, 
  ClipboardCheck, 
  Edit2, 
  Loader2, 
  Maximize2, 
  Minimize2 
} from "@lucide/vue";
import { listen } from '@tauri-apps/api/event';
import type { RenderedFile } from "../composables/useFileSystem";

const props = defineProps<{
  file: RenderedFile;
  filename?: string;
}>();

const emit = defineEmits<{
  (e: "edit"): void;
}>();

const copied = ref(false);
const htmlViewMode = ref<'code' | 'preview'>('code');
const previewLoading = ref(false);
const isFullscreen = ref(false);

const previewUrl = computed(() => {
  if (props.file.file_type !== 'html' || htmlViewMode.value !== 'preview') 
    return null;
  const blob = new Blob([props.file.content], { type: 'text/html' });
  return URL.createObjectURL(blob);
})

let lastUrl: string | null = null;
watch(previewUrl, (newUrl, oldUrl) => {
  if (oldUrl) URL.revokeObjectURL(oldUrl);
  lastUrl = newUrl;
});

let unlistenBack: (() => void) | null = null;
onMounted(async () => {
  unlistenBack = await listen('backButton', () => {
    if (isFullscreen.value) {
      isFullscreen.value = false;
    }
  });
});

onUnmounted(() => { 
  if (lastUrl) URL.revokeObjectURL(lastUrl); 
  unlistenBack?.();
});

watch(() => props.file.file_type, () => {
  isFullscreen.value = false;
  htmlViewMode.value = 'code';
})

watch(htmlViewMode, (newMode) => {
  if (newMode === 'code') {
    isFullscreen.value = false;
  }
  if (newMode === 'preview') {
    previewLoading.value = true;
    setTimeout(() => {
      previewLoading.value = false;
    }, 300);
  }
});

async function copyContent(content: string) {
  let textToCopy = content;
  // Strip HTML tags if it's code (syntect output)
  if (props.file.file_type === 'code') {
    const tempDiv = document.createElement("div");
    tempDiv.innerHTML = content;
    textToCopy = tempDiv.textContent || tempDiv.innerText || "";
  }
  
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
  <div class="max-w-full overflow-hidden">
    <div class="bg-bg1 border border-border rounded-xl overflow-hidden shadow-sm">
      <!-- Code / Plain / HTML -->
      <div v-if="file.file_type === 'code' || file.file_type === 'plain' || file.file_type === 'html'" class="relative bg-bg0">
        <!-- Toggle and Fullscreen Actions for HTML files -->
        <div 
          v-if="file.file_type === 'html'"
          class="absolute right-4 top-4 z-10 flex items-center gap-2"
        >
          <button
            v-if="htmlViewMode === 'preview'"
            @click="isFullscreen = true"
            class="p-2 bg-bg1 border border-border rounded-lg text-fg-dim hover:text-fg hover:border-fg-dim active:scale-95 transition-all cursor-pointer shadow-md"
            title="Full screen preview"
          >
            <Maximize2 :size="16" />
          </button>

          <div class="flex bg-bg1 border border-border rounded-lg overflow-hidden divide-x divide-border shadow-md">
            <button
              @click="htmlViewMode = 'code'"
              class="px-3 py-1.5 text-[10px] font-bold uppercase tracking-wider transition-all cursor-pointer font-sans"
              :class="htmlViewMode === 'code' ? 'bg-bg3 text-fg' : 'text-fg-dim hover:text-fg'"
            >
              Code
            </button>
            <button
              @click="htmlViewMode = 'preview'"
              class="px-3 py-1.5 text-[10px] font-bold uppercase tracking-wider transition-all cursor-pointer font-sans"
              :class="htmlViewMode === 'preview' ? 'bg-bg3 text-fg' : 'text-fg-dim hover:text-fg'"
            >
              Preview
            </button>
          </div>
        </div>

        <!-- Copy button -->
        <div v-if="file.file_type === 'code' || (file.file_type === 'html' && htmlViewMode === 'code')" 
             class="absolute top-4 z-10"
             :class="file.file_type === 'html' ? 'right-40' : 'right-4'">
          <button
            @click="copyContent(file.content)"
            class="p-2 bg-bg1/80 backdrop-blur-md border border-border rounded-lg text-fg-dim hover:text-fg hover:border-fg-dim transition-all active:scale-95 cursor-pointer shadow-lg"
            :title="copied ? 'Copied!' : 'Copy to clipboard'"
          >
            <ClipboardCheck v-if="copied" :size="16" class="text-green" />
            <Clipboard v-else :size="16" />
          </button>
        </div>

        <div>
          <div v-if="file.file_type === 'code'" v-html="file.content" class="p-6 text-sm font-mono leading-relaxed overflow-x-hidden max-w-full syntect-highlight"></div>
          <div v-else-if="file.file_type === 'plain'" class="max-h-[70vh] overflow-y-auto p-6">
            <pre class="text-sm text-fg leading-relaxed font-mono whitespace-pre-wrap break-words max-w-full overflow-x-hidden">{{ file.content }}</pre>
          </div>
          <div v-else-if="file.file_type === 'html'">
            <div v-if="htmlViewMode === 'code'" class="max-h-[70vh] overflow-y-auto">
              <pre class="text-sm text-fg leading-relaxed font-mono whitespace-pre-wrap break-words max-w-full overflow-x-hidden p-6">{{ file.content }}</pre>
            </div>
            <div v-else class="rounded-b-xl overflow-hidden -mx-0 bg-white relative">
              <div v-if="previewLoading" class="absolute inset-0 flex items-center justify-center bg-white z-20">
                <Loader2 :size="32" class="animate-spin text-bg3" />
              </div>
              <iframe 
                :src="previewUrl ?? ''" 
                sandbox="allow-scripts"
                class="w-full min-h-[60vh] bg-white border-0 rounded-b-xl"
                title="HTML Preview"
              />
            </div>

            <!-- Fullscreen Preview Overlay -->
            <Teleport to="body">
              <div 
                v-if="isFullscreen && file.file_type === 'html' && previewUrl"
                class="fixed inset-0 z-[100] flex flex-col bg-black"
              >
                <!-- Minimal top bar -->
                <div 
                  class="flex items-center justify-between px-4 shrink-0 bg-bg0/90 backdrop-blur-md border-b border-border pt-[env(safe-area-inset-top)]"
                  style="min-height: calc(44px + env(safe-area-inset-top));"
                >
                  <span class="text-xs font-mono text-fg-dim truncate max-w-[70%]">
                    {{ filename ?? 'Preview' }}
                  </span>
                  <div class="flex items-center gap-2">
                    <!-- Minimize back to card -->
                    <button
                      @click="isFullscreen = false"
                      class="p-2 text-fg-dim hover:text-fg active:scale-95 transition-all cursor-pointer"
                      title="Exit full screen"
                    >
                      <Minimize2 :size="18" />
                    </button>
                  </div>
                </div>

                <!-- Full screen iframe -->
                <iframe
                  :src="previewUrl"
                  sandbox="allow-scripts"
                  class="flex-1 w-full border-0 bg-white"
                  title="HTML Full Screen Preview"
                />

                <!-- Bottom safe area spacer -->
                <div 
                  class="shrink-0 bg-bg0/90"
                  style="height: env(safe-area-inset-bottom);"
                />
              </div>
            </Teleport>
          </div>
        </div>
      </div>

      <!-- Markdown -->
      <div v-else-if="file.file_type === 'markdown'" class="p-6 prose-custom bg-bg1 max-w-full overflow-x-hidden">
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
.prose-custom pre { @apply p-6 bg-bg0 rounded-lg overflow-x-auto max-w-full mb-4 border border-border; }
.prose-custom pre code { @apply p-0 bg-transparent text-fg; }
.prose-custom blockquote { @apply border-l-4 border-green pl-4 italic text-fg-dim mb-4; }
.prose-custom a { @apply text-aqua hover:underline; }

/* Syntect styles fix */
.syntect-highlight, .syntect-highlight pre { 
  background: transparent !important; 
  margin: 0 !important; 
  padding: 0 !important;
  white-space: pre-wrap !important;
  word-break: break-all !important;
  overflow-x: hidden !important;
}

.syntect-highlight {
  font-family: inherit;
}
</style>
