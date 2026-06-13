<script setup lang="ts">
import { Folder, Trash2, Loader2, GitBranch, Clock } from "@lucide/vue";
import type { RepoInfo } from "../composables/useRepos";

const props = defineProps<{
  repo: RepoInfo;
  deleting: boolean;
}>();

defineEmits<{
  (e: "open", repo: RepoInfo): void;
  (e: "delete", repo: RepoInfo): void;
}>();

function formatRelativeTime(seconds?: number) {
  if (!seconds) return "";
  const now = Math.floor(Date.now() / 1000);
  const diff = now - seconds;

  if (diff < 60) return "just now";
  if (diff < 3600) return `${Math.floor(diff / 60)}m ago`;
  if (diff < 86400) return `${Math.floor(diff / 3600)}h ago`;
  if (diff < 604800) return `${Math.floor(diff / 86400)}d ago`;
  return new Date(seconds * 1000).toLocaleDateString();
}
</script>

<template>
  <div
    class="group flex justify-between items-center px-4 py-4 mb-3 bg-bg1 border border-border rounded-xl transition-all active:scale-[0.98] shadow-sm"
    :class="!deleting ? 'cursor-pointer hover:border-fg-dim hover:bg-bg1/80 active:border-green' : 'opacity-50'"
    @click="!deleting && $emit('open', repo)"
  >
    <div class="flex items-center gap-4 overflow-hidden">
      <div class="p-3 bg-bg3 rounded-xl text-yellow shadow-inner">
        <Folder :size="22" />
      </div>
      <div class="overflow-hidden">
        <div class="font-bold text-fg truncate mb-0.5">{{ repo.name }}</div>
        <div class="flex items-center gap-3">
          <div class="flex items-center gap-1.5 text-[10px] text-aqua uppercase tracking-wider font-bold">
            <GitBranch :size="12" />
            <span>{{ repo.branch }}</span>
          </div>
          <div v-if="repo.last_modified" class="flex items-center gap-1.5 text-[10px] text-fg-dim font-medium">
            <Clock :size="12" />
            <span>{{ formatRelativeTime(repo.last_modified) }}</span>
          </div>
        </div>
      </div>
    </div>

    <button
      @click.stop="$emit('delete', repo)"
      :disabled="deleting"
      class="ml-4 p-3 rounded-xl border transition-all shrink-0"
      :class="
        deleting
          ? 'border-border text-fg-dim cursor-not-allowed'
          : 'border-red/10 text-red/40 hover:text-red hover:bg-red/10 hover:border-red active:bg-red/20 cursor-pointer'
      "
      aria-label="Delete repository"
    >
      <Loader2 v-if="deleting" :size="18" class="animate-spin" />
      <Trash2 v-else :size="18" />
    </button>
  </div>
</template>
