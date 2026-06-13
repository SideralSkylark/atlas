<script setup lang="ts">
import { ref } from "vue";
import { Plus, Loader2, Clipboard } from "@lucide/vue";

const props = defineProps<{
  cloning: boolean;
}>();

const emit = defineEmits<{
  (e: "clone", url: string): void;
}>();

const url = ref("");

function handleSubmit() {
  if (url.value && !props.cloning) {
    emit("clone", url.value);
    url.value = "";
  }
}

async function onPaste() {
  try {
    const text = await navigator.clipboard.readText();
    url.value = text;
  } catch (err) {
    console.error("Failed to read clipboard:", err);
  }
}
</script>

<template>
  <div class="sticky top-0 z-20 bg-bg0 pb-4 pt-1">
    <div class="flex flex-col gap-2">
      <label class="text-[10px] uppercase tracking-widest text-fg-dim font-bold ml-1 font-sans">
        Clone a repository
      </label>
      <div class="flex flex-col gap-3 p-4 bg-bg1 border border-border rounded-xl shadow-md" style="box-shadow: var(--shadow-md), var(--shadow-inset)">
        <div class="relative">
          <input
            v-model="url"
            placeholder="https://github.com/user/repo.git"
            class="w-full px-4 py-4 pr-12 bg-bg3 text-fg border border-border rounded-xl placeholder-fg-dim outline-none focus:border-green transition-all font-mono text-sm shadow-inner"
            @keyup.enter="handleSubmit"
          />
          <button 
            @click="onPaste"
            class="absolute right-3 top-1/2 -translate-y-1/2 p-2 text-fg-dim hover:text-fg transition-colors cursor-pointer"
            title="Paste from clipboard"
          >
            <Clipboard :size="18" />
          </button>
        </div>

        <button
          @click="handleSubmit"
          :disabled="!url || cloning"
          class="w-full py-3.5 font-bold rounded-xl transition-all flex items-center justify-center gap-2 active:scale-[0.99] shadow-lg font-sans"
          :class="
            url && !cloning
              ? 'bg-green text-bg0 active:brightness-90 cursor-pointer'
              : 'bg-bg3 text-fg-dim cursor-not-allowed'
          "
        >
          <Loader2 v-if="cloning" :size="20" class="animate-spin" />
          <Plus v-else :size="20" />
          <span class="font-sans">{{ cloning ? "Cloning..." : "Clone Repository" }}</span>
        </button>
      </div>
    </div>
    <!-- Fading Bottom Border -->
    <div class="h-px bg-gradient-to-r from-transparent via-border to-transparent opacity-50 mt-4"></div>
  </div>
</template>
