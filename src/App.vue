<script setup lang="ts">
import { ref, onMounted } from "vue";
import { listen } from "@tauri-apps/api/event";
import RepoList from "./components/RepoList.vue";
import FileBrowser from "./components/FileBrowser.vue";
import ActivityView from "./components/ActivityView.vue";
import SettingsView from "./components/SettingsView.vue";
import BottomNav from "./components/BottomNav.vue";
import Toast from "./components/Toast.vue";
import type { RepoInfo } from "./composables/useRepos";
import { useAppNavigation } from "./composables/useAppNavigation";

const { currentView, handleTouchStart, handleTouchMove, handleTouchEnd } = useAppNavigation();
const selectedRepo = ref<RepoInfo | null>(null);
const toastMessage = ref<{ type: "success" | "error"; text: string } | null>(null);
const fileBrowserRef = ref<InstanceType<typeof FileBrowser> | null>(null);

function openRepo(repo: RepoInfo) {
  selectedRepo.value = repo;
}

function closeRepo() {
  selectedRepo.value = null;
}

function notify(msg: { type: "success" | "error"; text: string }) {
  toastMessage.value = msg;
}

onMounted(async () => {
  await listen("backButton", () => {
    if (fileBrowserRef.value?.editingPath) {
      if (fileBrowserRef.value.editorRef?.isDirty) {
        return;
      }
      fileBrowserRef.value.editingPath = null;
      return;
    }

    if (fileBrowserRef.value?.renderedFile) {
      const browser = fileBrowserRef.value;
      if (browser) {
        browser.handleBack();
      }
      return;
    }

    if (selectedRepo.value && fileBrowserRef.value?.currentRelativePath !== "") {
      const browser = fileBrowserRef.value;
      if (browser) {
        browser.handleBack();
      }
      return;
    }

    if (currentView.value !== 'repos') {
      currentView.value = 'repos';
      return;
    }

    if (selectedRepo.value) {
      closeRepo();
      return;
    }
  });
});
</script>

<template>
  <div class="h-screen overflow-hidden bg-bg0 text-fg font-sans antialiased tracking-tight
              pt-[calc(1rem+env(safe-area-inset-top))]
              pb-[env(safe-area-inset-bottom)]
              pl-[env(safe-area-inset-left)]
              pr-[env(safe-area-inset-right)]"
       @touchstart="handleTouchStart($event, !!selectedRepo)"
       @touchmove="handleTouchMove($event, !!selectedRepo)"
       @touchend="handleTouchEnd(!!selectedRepo)">
    <div
      class="max-w-2xl mx-auto h-full overflow-y-auto px-6 pb-16"
      :class="{ 'content-fade-bottom': !(selectedRepo && fileBrowserRef?.renderedFile) && !fileBrowserRef?.editingPath }"
    >
      <Transition name="tab-fade" mode="out-in">
        <div v-if="currentView === 'repos'" key="repos" class="h-full">
          <Transition name="fade" mode="out-in">
            <div v-if="selectedRepo" :key="'browser-' + selectedRepo.id" class="h-full">
              <FileBrowser
                ref="fileBrowserRef"
                :repo="selectedRepo"
                @close="closeRepo"
                @notify="notify"
              />
            </div>
            <div v-else key="repo-list">
              <RepoList @open="openRepo" @notify="notify" />
            </div>
          </Transition>
        </div>
        <div v-else-if="currentView === 'activity'" key="activity">
          <ActivityView />
        </div>
        <div v-else-if="currentView === 'settings'" key="settings">
          <SettingsView />
        </div>
      </Transition>
    </div>

    <BottomNav
      v-show="!(selectedRepo && fileBrowserRef?.renderedFile) && !fileBrowserRef?.editingPath"
      v-model:currentView="currentView"
    />

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

.tab-fade-enter-active,
.tab-fade-leave-active {
  transition: opacity 150ms ease, transform 150ms ease;
}

.tab-fade-enter-from {
  opacity: 0;
  transform: translateY(6px);
}

.tab-fade-leave-to {
  opacity: 0;
  transform: translateY(-6px);
}
</style>
