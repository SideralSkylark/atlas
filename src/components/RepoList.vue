<script setup lang="ts">
import { onMounted } from "vue";
import { useRepos } from "../composables/useRepos";
import type { RepoInfo } from "../composables/useRepos";
import { PackageOpen } from "@lucide/vue";
import RepoClone from "./RepoClone.vue";
import RepoItem from "./RepoItem.vue";

const emit = defineEmits<{
  (e: "open", repo: RepoInfo): void;
  (e: "notify", msg: { type: "success" | "error"; text: string }): void;
}>();

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
  </div>
</template>

<style scoped>
</style>
