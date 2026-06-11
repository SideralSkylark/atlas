<script setup lang="ts">
import { ref } from "vue";

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
  <div class="mb-8">
    <input
      v-model="url"
      placeholder="https://github.com/user/repo.git"
      class="w-full px-3 py-2 mb-2 bg-bg3 text-fg border border-border rounded-md placeholder-fg-dim outline-none focus:border-green transition-colors font-mono text-sm"
      @keyup.enter="handleSubmit"
    />

    <button
      @click="handleSubmit"
      :disabled="!url || cloning"
      class="w-full py-2 font-semibold rounded-md transition-all flex items-center justify-center gap-2"
      :class="
        url && !cloning
          ? 'bg-green text-bg0 active:brightness-90 cursor-pointer'
          : 'bg-bg3 text-fg-dim cursor-not-allowed'
      "
    >
      <svg
        v-if="cloning"
        class="animate-spin h-4 w-4"
        viewBox="0 0 24 24"
        fill="none"
      >
        <circle
          class="opacity-25"
          cx="12"
          cy="12"
          r="10"
          stroke="currentColor"
          stroke-width="4"
        />
        <path
          class="opacity-75"
          fill="currentColor"
          d="M4 12a8 8 0 018-8v4a4 4 0 00-4 4H4z"
        />
      </svg>
      <span>{{ cloning ? "Cloning..." : "Clone" }}</span>
    </button>
  </div>
</template>
