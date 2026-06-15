<script setup lang="ts">
import { PackageOpen, GitCommit, Settings } from "@lucide/vue";

defineProps<{
  currentView: 'repos' | 'activity' | 'settings';
}>();

defineEmits<{
  (e: 'update:currentView', view: 'repos' | 'activity' | 'settings'): void;
}>();
</script>

<template>
  <nav class="fixed bottom-0 left-0 right-0 z-30 bg-bg1 border-t border-border pb-[env(safe-area-inset-bottom)]">
    <div class="flex max-w-2xl mx-auto">
      <button
        @click="$emit('update:currentView', 'repos')"
        class="flex-1 flex flex-col items-center justify-center py-3 gap-1 text-[9px] uppercase tracking-widest font-bold transition-colors relative"
        :class="currentView === 'repos' ? 'text-yellow' : 'text-fg-dim hover:text-fg'"
      >
        <div v-if="currentView === 'repos'" class="w-4 h-0.5 bg-yellow rounded-full absolute top-1"></div>
        <PackageOpen :size="20" />
        <span>Repositories</span>
      </button>

      <button
        @click="$emit('update:currentView', 'activity')"
        class="flex-1 flex flex-col items-center justify-center py-3 gap-1 text-[9px] uppercase tracking-widest font-bold transition-colors relative"
        :class="currentView === 'activity' ? 'text-yellow' : 'text-fg-dim hover:text-fg'"
      >
        <div v-if="currentView === 'activity'" class="w-4 h-0.5 bg-yellow rounded-full absolute top-1"></div>
        <GitCommit :size="20" />
        <span>Activity</span>
      </button>

      <button
        @click="$emit('update:currentView', 'settings')"
        class="flex-1 flex flex-col items-center justify-center py-3 gap-1 text-[9px] uppercase tracking-widest font-bold transition-colors relative"
        :class="currentView === 'settings' ? 'text-yellow' : 'text-fg-dim hover:text-fg'"
      >
        <div v-if="currentView === 'settings'" class="w-4 h-0.5 bg-yellow rounded-full absolute top-1"></div>
        <Settings :size="20" />
        <span>Settings</span>
      </button>
    </div>
  </nav>
</template>
