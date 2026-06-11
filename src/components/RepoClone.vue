<script setup lang="ts">
import { ref } from "vue";
import { Plus, Loader2 } from "@lucide/vue";

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
</script>

<template>
  <div class="mb-8 p-4 bg-bg1 border border-border rounded-lg shadow-sm">
    <div class="flex flex-col gap-3">
      <input
        v-model="url"
        placeholder="https://github.com/user/repo.git"
        class="w-full px-4 py-3 bg-bg3 text-fg border border-border rounded-md placeholder-fg-dim outline-none focus:border-green transition-all font-mono text-sm"
        @keyup.enter="handleSubmit"
      />

      <button
        @click="handleSubmit"
        :disabled="!url || cloning"
        class="w-full py-3 font-bold rounded-md transition-all flex items-center justify-center gap-2 active:scale-[0.99]"
        :class="
          url && !cloning
            ? 'bg-green text-bg0 active:brightness-90 cursor-pointer shadow-md'
            : 'bg-bg3 text-fg-dim cursor-not-allowed'
        "
      >
        <Loader2 v-if="cloning" :size="20" class="animate-spin" />
        <Plus v-else :size="20" />
        <span>{{ cloning ? "Cloning..." : "Clone Repository" }}</span>
      </button>
    </div>
  </div>
</template>
