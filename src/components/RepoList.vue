<script setup lang="ts">
import { ref, onMounted } from "vue";
import { useRepos } from "../composables/useRepos";
import type { RepoInfo } from "../composables/useRepos";
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
    <div class="flex justify-between items-center mb-6">
      <h1 class="text-xl font-bold text-yellow">Atlas</h1>
      <button
        @click="showCredentials = !showCredentials"
        class="text-xs px-2 py-1 rounded border border-border text-fg-dim hover:text-fg transition-colors cursor-pointer"
      >
        {{ showCredentials ? "Close Settings" : "Settings" }}
      </button>
    </div>

    <div v-if="showCredentials">
      <CredentialsManager />
    </div>

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
