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
    <div class="flex justify-between items-center mb-10">
      <h1 class="text-3xl font-black text-yellow tracking-tight">Atlas</h1>
      <button
        @click="showCredentials = true"
        class="p-2.5 rounded-full border border-border text-fg-dim hover:text-fg hover:border-fg-dim transition-all active:scale-90 cursor-pointer shadow-sm bg-bg1"
        title="Settings"
      >
        <Settings :size="20" />
      </button>
    </div>

    <RepoClone :cloning="cloning" @clone="onClone" />

    <div class="flex items-center justify-between mb-4 px-1">
      <h2 class="text-xs font-bold uppercase tracking-widest text-fg-dim">Your Repositories</h2>
      <span v-if="repos.length > 0" class="text-[10px] px-2 py-0.5 bg-bg3 text-fg-dim rounded-full font-bold border border-border/50">
        {{ repos.length }} {{ repos.length === 1 ? 'repo' : 'repos' }}
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
             class="fixed bottom-0 left-0 right-0 bg-bg1 border-t border-border rounded-t-3xl p-6 z-50 max-h-[90vh] overflow-y-auto shadow-2xl">
          <div class="w-12 h-1.5 bg-bg3 rounded-full mx-auto mb-6" @click="showCredentials = false"></div>
          <div class="flex justify-between items-center mb-6">
            <h2 class="text-xl font-bold text-fg">Settings</h2>
            <button @click="showCredentials = false" class="p-2 hover:bg-bg3 rounded-full transition-colors">
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
