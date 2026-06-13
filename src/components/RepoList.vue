<script setup lang="ts">
import { ref, onMounted } from "vue";
import { useRepos } from "../composables/useRepos";
import type { RepoInfo } from "../composables/useRepos";
import { Settings, X, PackageOpen } from "@lucide/vue";
import RepoClone from "./RepoClone.vue";
import RepoItem from "./RepoItem.vue";
import CredentialsManager from "./CredentialsManager.vue";

const emit = defineEmits<{
  (e: "open", repo: RepoInfo): void;
  (e: "notify", msg: { type: "success" | "error"; text: string }): void;
}>();

const showCredentials = ref(false);

const {
  repos,
  cloning,
  deletingRepo,
  loadRepos,
  cloneRepo,
  deleteRepo,
} = useRepos();

async function onClone(url: string) {
  const res = await cloneRepo(url);
  if (res) {
    emit("notify", {
      type: res.success ? "success" : "error",
      text: res.message,
    });
    if (res.success && 'vibrate' in navigator) {
      navigator.vibrate([10, 50, 10]);
    }
  }
}

async function onDelete(repo: RepoInfo) {
  const res = await deleteRepo(repo.id);
  if (res) {
    emit("notify", {
      type: res.success ? "success" : "error",
      text: res.message,
    });
  }
}

onMounted(loadRepos);
</script>

<template>
  <div>
    <div class="flex justify-between items-start mb-10">
      <div>
        <h1 class="text-3xl font-black text-yellow tracking-tight leading-none font-sans">Atlas</h1>
        <p class="text-fg-dim text-[10px] uppercase tracking-widest font-bold mt-1.5 ml-0.5 font-sans">Git on Android</p>
      </div>
      <button
        @click="showCredentials = true"
        class="min-w-[44px] min-h-[44px] flex items-center justify-center text-fg-dim hover:text-fg transition-all active:scale-95 duration-100 cursor-pointer"
        title="Settings"
      >
        <Settings :size="24" />
      </button>
    </div>

    <RepoClone :cloning="cloning" @clone="onClone" />

    <div class="flex items-center justify-between mt-8 mb-4 px-1">
      <h2 class="text-[10px] font-bold uppercase tracking-widest text-fg-dim font-sans">Repositories</h2>
      <span v-if="repos.length > 0" class="text-[10px] px-2 py-0.5 bg-bg3 text-fg-dim rounded-full font-bold border border-border/50 font-sans">
        {{ repos.length }}
      </span>
    </div>

    <div v-if="repos.length === 0" class="flex flex-col items-center justify-center py-16 bg-bg1/50 border border-dashed border-border rounded-xl text-fg-dim opacity-40">
      <PackageOpen :size="48" class="mb-4 stroke-[1.5]" />
      <p class="text-sm font-medium">No repositories yet.</p>
    </div>

    <div v-else class="space-y-2">
      <RepoItem
        v-for="repo in repos"
        :key="repo.id"
        :repo="repo"
        :deleting="deletingRepo === repo.id"
        @open="$emit('open', $event)"
        @delete="onDelete"
      />
    </div>

    <!-- Bottom Sheet for Credentials -->
    <Teleport to="body">
      <Transition name="fade">
        <div v-if="showCredentials" 
             class="fixed inset-0 bg-bg0/80 backdrop-blur-sm z-40"
             @click="showCredentials = false"></div>
      </Transition>
      <Transition name="sheet">
        <div v-if="showCredentials" 
             class="fixed bottom-0 left-0 right-0 bg-bg1 border-t border-border rounded-t-3xl p-6 z-50 max-h-[90vh] overflow-y-auto shadow-lg"
             style="box-shadow: var(--shadow-lg)">
          <div class="w-12 h-1.5 bg-bg3 rounded-full mx-auto mb-6" @click="showCredentials = false"></div>
          <div class="flex justify-between items-center mb-6">
            <h2 class="text-xl font-bold text-fg">Settings</h2>
            <button @click="showCredentials = false" class="min-w-[44px] min-h-[44px] flex items-center justify-center text-fg-dim hover:text-fg transition-all active:scale-95 duration-100 cursor-pointer">
              <X :size="24" />
            </button>
          </div>
          <CredentialsManager />
        </div>
      </Transition>
    </Teleport>
  </div>
</template>

<style scoped>
.fade-enter-active, .fade-leave-active {
  transition: opacity 0.3s ease;
}
.fade-enter-from, .fade-leave-to {
  opacity: 0;
}

.sheet-enter-active, .sheet-leave-active {
  transition: transform 0.4s cubic-bezier(0.16, 1, 0.3, 1);
}
.sheet-enter-from, .sheet-leave-to {
  transform: translateY(100%);
}
</style>
