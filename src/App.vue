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
import { useTheme } from "./composables/useTheme";

const { theme, appearance } = useTheme();
const currentView = ref<'repos' | 'activity' | 'settings'>('repos');
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

// Navigation Swipe Gestures (repos <-> activity <-> settings)
const touchStartX = ref(0);
const touchStartY = ref(0);
const touchCurrentX = ref(0);
const touchCurrentY = ref(0);
const isNavSwiping = ref(false);

function onNavTouchStart(e: TouchEvent) {
  if (selectedRepo.value) return; // Do not swipe tabs if inside repo
  touchStartX.value = e.touches[0].clientX;
  touchStartY.value = e.touches[0].clientY;
  touchCurrentX.value = touchStartX.value;
  touchCurrentY.value = touchStartY.value;
  isNavSwiping.value = false;
}

function onNavTouchMove(e: TouchEvent) {
  if (selectedRepo.value) return;
  touchCurrentX.value = e.touches[0].clientX;
  touchCurrentY.value = e.touches[0].clientY;
  
  const deltaX = touchCurrentX.value - touchStartX.value;
  const deltaY = touchCurrentY.value - touchStartY.value;

  // Swiping horizontally (must be mostly horizontal and exceed threshold to commit)
  if (!isNavSwiping.value && Math.abs(deltaX) > 40 && Math.abs(deltaX) > Math.abs(deltaY) * 2.5) {
    isNavSwiping.value = true;
  }
}

function onNavTouchEnd() {
  if (!isNavSwiping.value) return;
  
  const deltaX = touchCurrentX.value - touchStartX.value;
  const swipeThreshold = 120; // safe threshold to prevent accidental switches

  if (Math.abs(deltaX) > swipeThreshold) {
    const views: ('repos' | 'activity' | 'settings')[] = ['repos', 'activity', 'settings'];
    const currentIndex = views.indexOf(currentView.value);
    
    if (deltaX > 0) {
      // Swipe right (moves to left tab)
      if (currentIndex > 0) {
        currentView.value = views[currentIndex - 1];
        if ('vibrate' in navigator) navigator.vibrate(10);
      }
    } else {
      // Swipe left (moves to right tab)
      if (currentIndex < views.length - 1) {
        currentView.value = views[currentIndex + 1];
        if ('vibrate' in navigator) navigator.vibrate(10);
      }
    }
  }
  isNavSwiping.value = false;
}

onMounted(async () => {
  await listen("backButton", () => {
    // 1. If the Editor overlay is open
    if (fileBrowserRef.value?.editingPath) {
      if (fileBrowserRef.value.editorRef?.isDirty) {
        // Editor will handle showing the unsaved changes dialog via its own listener
        return;
      }
      fileBrowserRef.value.editingPath = null;
      return;
    }

    // 2. Else if a file is being viewed
    if (fileBrowserRef.value?.renderedFile) {
      fileBrowserRef.value.handleBack();
      return;
    }

    // 3. Else if inside a subdirectory
    if (selectedRepo.value && fileBrowserRef.value?.currentRelativePath !== "") {
      fileBrowserRef.value.handleBack();
      return;
    }

    // 4. Else if on a non-repos tab
    if (currentView.value !== 'repos') {
      currentView.value = 'repos';
      return;
    }

    // 5. Else if selectedRepo is set but at root
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
       @touchstart="onNavTouchStart"
       @touchmove="onNavTouchMove"
       @touchend="onNavTouchEnd">
    <div 
      class="max-w-2xl mx-auto h-full overflow-y-auto px-6 pb-16"
      :class="{ 'content-fade-bottom': !(selectedRepo && fileBrowserRef?.renderedFile) && !fileBrowserRef?.editingPath }"
    >
      <Transition
        name="tab-fade"
        mode="out-in"
      >
        <div v-if="currentView === 'repos'" key="repos" class="h-full">
          <Transition
            name="fade"
            mode="out-in"
          >
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
