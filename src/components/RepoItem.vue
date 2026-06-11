<script setup lang="ts">
import { Folder, Trash2, Loader2, GitBranch } from "@lucide/vue";
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
    class="group flex justify-between items-center px-4 py-4 mb-3 bg-bg1 border border-border rounded-lg transition-all active:scale-[0.98]"
    :class="!deleting ? 'cursor-pointer hover:border-fg-dim active:border-green' : 'opacity-50'"
    @click="!deleting && $emit('open', repo)"
  >
    <div class="flex items-center gap-4 overflow-hidden">
      <div class="p-2 bg-bg3 rounded-md text-yellow">
        <Folder :size="20" />
      </div>
      <div class="overflow-hidden">
        <div class="font-bold text-fg truncate">{{ repo.name }}</div>
        <div class="flex items-center gap-1.5 text-xs text-aqua mt-0.5">
          <GitBranch :size="12" />
          <span>{{ repo.branch }}</span>
        </div>
      </div>
    </div>

    <button
      @click.stop="$emit('delete', repo)"
      :disabled="deleting"
      class="ml-4 p-2.5 rounded-md border transition-all shrink-0"
      :class="
        deleting
          ? 'border-border text-fg-dim cursor-not-allowed'
          : 'border-red/20 text-red/60 hover:text-red hover:bg-red/10 hover:border-red active:bg-red/20 cursor-pointer'
      "
      aria-label="Delete repository"
    >
      <Loader2 v-if="deleting" :size="18" class="animate-spin" />
      <Trash2 v-else :size="18" />
    </button>
  </div>
</template>
