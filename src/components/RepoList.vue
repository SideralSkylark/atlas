<script setup lang="ts">
import { onMounted } from "vue";
import { useRepos } from "../composables/useRepos";
import type { RepoInfo } from "../composables/useRepos";
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
    <h1 class="text-xl font-bold text-yellow mb-6">Atlas</h1>

    <RepoClone :cloning="cloning" @clone="onClone" />

    <p v-if="repos.length === 0" class="text-fg-dim">No repos yet.</p>

    <div v-else>
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
