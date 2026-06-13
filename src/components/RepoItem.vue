<script setup lang="ts">
import { ref, onUnmounted } from "vue";
import { Folder, Trash2, Loader2, GitBranch, Clock, ChevronRight } from "@lucide/vue";
import type { RepoInfo } from "../composables/useRepos";

const props = defineProps<{
  repo: RepoInfo;
  deleting: boolean;
}>();

const emit = defineEmits<{
  (e: "open", repo: RepoInfo): void;
  (e: "delete", repo: RepoInfo): void;
}>();

const showDeleteConfirm = ref(false);
let pressTimer: number | null = null;

function startPress() {
  if (props.deleting) return;
  pressTimer = window.setTimeout(() => {
    showDeleteConfirm.value = true;
    if ('vibrate' in navigator) navigator.vibrate(30);
  }, 400);
}

function cancelPress() {
  if (pressTimer) {
    clearTimeout(pressTimer);
    pressTimer = null;
  }
}

function handleClick() {
  if (showDeleteConfirm.value) {
    showDeleteConfirm.value = false;
    return;
  }
  emit("open", props.repo);
}

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

onUnmounted(() => cancelPress());
</script>

<template>
  <div
    class="relative group flex items-center px-4 py-4 mb-3 bg-bg1 border border-border rounded-xl transition-all active:scale-[0.98] shadow-sm hover:shadow-md overflow-hidden select-none"
    :class="[
      !deleting ? 'cursor-pointer hover:bg-bg1/80' : 'opacity-50',
      showDeleteConfirm ? 'border-l-4 border-l-red' : ''
    ]"
    style="box-shadow: var(--shadow-sm), var(--shadow-inset)"
    @mousedown="startPress"
    @mouseup="cancelPress"
    @mouseleave="cancelPress"
    @touchstart="startPress"
    @touchend="cancelPress"
    @click="handleClick"
  >
    <div class="flex items-center gap-4 flex-1 min-w-0">
      <div class="p-3 bg-bg2 rounded-xl text-yellow shrink-0" style="box-shadow: var(--shadow-inset)">
        <Folder :size="22" />
      </div>
      <div class="flex-1 min-w-0">
        <div class="font-bold text-fg truncate mb-0.5 font-sans font-medium">{{ repo.name }}</div>
        <div class="flex items-center gap-3">
          <div class="flex items-center gap-1.5 text-[9px] text-aqua uppercase tracking-wider font-bold max-w-[120px] font-mono">
            <GitBranch :size="10" />
            <span class="truncate">{{ repo.branch }}</span>
          </div>
          <div v-if="repo.last_modified" class="flex items-center gap-1.5 text-[10px] text-fg-dim font-medium font-sans">
            <Clock :size="12" />
            <span>{{ formatRelativeTime(repo.last_modified) }}</span>
          </div>
        </div>
      </div>
    </div>

    <!-- Hidden Action / Reveal -->
    <Transition name="slide-reveal">
      <div v-if="showDeleteConfirm" class="absolute inset-y-0 right-0 flex items-center pr-4 bg-bg1">
        <button
          @click.stop="$emit('delete', repo)"
          :disabled="deleting"
          class="flex items-center gap-2 px-4 py-2 bg-red/10 text-red border border-red/20 rounded-lg font-bold text-xs hover:bg-red/20 transition-all"
        >
          <Loader2 v-if="deleting" :size="14" class="animate-spin" />
          <Trash2 v-else :size="14" />
          <span>Delete</span>
        </button>
      </div>
      <div v-else class="text-fg-dim/20 px-2 shrink-0">
        <ChevronRight :size="16" />
      </div>
    </Transition>
  </div>
</template>

<style scoped>
.slide-reveal-enter-active, .slide-reveal-leave-active {
  transition: transform 0.3s cubic-bezier(0.16, 1, 0.3, 1), opacity 0.2s ease;
}
.slide-reveal-enter-from, .slide-reveal-leave-to {
  transform: translateX(20px);
  opacity: 0;
}
</style>
