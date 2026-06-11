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
    <div class="flex justify-between items-center mb-8">
      <h1 class="text-2xl font-black text-yellow tracking-tight">Atlas</h1>
      <button
        @click="showCredentials = !showCredentials"
        class="p-2 rounded-full border border-border text-fg-dim hover:text-fg hover:border-fg-dim transition-all active:scale-90 cursor-pointer"
        :class="{ 'bg-bg3 border-fg-dim text-fg': showCredentials }"
        :title="showCredentials ? 'Close Settings' : 'Settings'"
      >
        <X v-if="showCredentials" :size="20" />
        <Settings v-else :size="20" />
      </button>
    </div>

    <Transition name="slide">
      <div v-if="showCredentials" class="mb-8">
        <CredentialsManager />
      </div>
    </Transition>

    <RepoClone :cloning="cloning" @clone="onClone" />

    <div v-if="repos.length === 0" class="flex flex-col items-center justify-center py-16 text-fg-dim opacity-40">
      <PackageOpen :size="48" class="mb-4 stroke-[1.5]" />
      <p class="text-sm font-medium">No repositories yet.</p>
    </div>

    <div v-else class="space-y-1">
      <RepoItem
        v-for="repo in repos"
        :key="repo.id"
        :repo="repo"
        :deleting="deletingRepo === repo.id"
        @open="$emit('open', $event)"
        @delete="onDelete"
      />
    </div>
  </div>
</template>

<style scoped>
.slide-enter-active,
.slide-leave-active {
  transition: all 0.3s ease;
  max-height: 500px;
}

.slide-enter-from,
.slide-leave-to {
  opacity: 0;
  max-height: 0;
  transform: translateY(-10px);
  margin-bottom: 0;
  overflow: hidden;
}
</style>
