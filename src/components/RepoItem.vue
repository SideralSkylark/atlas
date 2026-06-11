<script setup lang="ts">
import type { RepoInfo } from "../composables/useRepos";

defineProps<{
  repo: RepoInfo;
  deleting: boolean;
}>();

defineEmits<{
  (e: "open", repo: RepoInfo): void;
  (e: "delete", repo: RepoInfo): void;
}>();
</script>

<template>
  <div
    class="flex justify-between items-center px-4 py-3 mb-3 bg-bg1 border border-border rounded-md transition-colors"
    :class="!deleting ? 'cursor-pointer active:border-green' : 'opacity-50'"
    @click="!deleting && $emit('open', repo)"
  >
    <div>
      <div class="font-bold text-fg">{{ repo.name }}</div>
      <div class="text-xs text-aqua mt-0.5">{{ repo.branch }}</div>
    </div>

    <button
      @click.stop="$emit('delete', repo)"
      :disabled="deleting"
      class="ml-4 flex items-center gap-1.5 px-3 py-1.5 rounded-md border text-xs font-medium transition-all"
      :class="
        deleting
          ? 'border-border text-fg-dim cursor-not-allowed'
          : 'border-red/40 text-red hover:bg-red/10 hover:border-red active:bg-red/20 cursor-pointer'
      "
    >
      <svg
        v-if="!deleting"
        xmlns="http://www.w3.org/2000/svg"
        class="h-3.5 w-3.5"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="2"
        stroke-linecap="round"
        stroke-linejoin="round"
      >
        <polyline points="3 6 5 6 21 6" />
        <path d="M19 6l-1 14a2 2 0 01-2 2H8a2 2 0 01-2-2L5 6" />
        <path d="M10 11v6M14 11v6" />
        <path d="M9 6V4a1 1 0 011-1h4a1 1 0 011 1v2" />
      </svg>
      <svg v-else class="animate-spin h-3.5 w-3.5" viewBox="0 0 24 24" fill="none">
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
      <span>{{ deleting ? "Deleting..." : "Delete" }}</span>
    </button>
  </div>
</template>
