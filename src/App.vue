<script setup lang="ts">
import { ref } from "vue";
import RepoList from "./components/RepoList.vue";
import FileBrowser from "./components/FileBrowser.vue";
import Toast from "./components/Toast.vue";
import type { RepoInfo } from "./composables/useRepos";

const selectedRepo = ref<RepoInfo | null>(null);
const toastMessage = ref<{ type: "success" | "error"; text: string } | null>(null);

function openRepo(repo: RepoInfo) {
  selectedRepo.value = repo;
}

function closeRepo() {
  selectedRepo.value = null;
}

function notify(msg: { type: "success" | "error"; text: string }) {
  toastMessage.value = msg;
}
</script>

<template>
  <div class="min-h-screen bg-bg0 text-fg font-mono p-6">
    <div class="max-w-2xl mx-auto">
      <Transition
        name="fade"
        mode="out-in"
      >
        <div v-if="selectedRepo" :key="'browser-' + selectedRepo.id">
          <FileBrowser :repo="selectedRepo" @close="closeRepo" />
        </div>
        <div v-else key="repo-list">
          <RepoList @open="openRepo" @notify="notify" />
        </div>
      </Transition>
    </div>

    <Toast :message="toastMessage" />
  </div>
</template>

<style>
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.2s ease, transform 0.2s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
  transform: translateY(10px);
}
</style>
