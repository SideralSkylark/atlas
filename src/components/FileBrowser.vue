<script setup lang="ts">
import { 
  ChevronLeft, 
  ChevronRight, 
  Download, 
  Upload, 
  Folder, 
  FileText, 
  Loader2,
  Home,
  Search,
  GitBranch,
  Edit2,
  RefreshCcw
} from "@lucide/vue";
import { onMounted, watch, computed, ref } from "vue";
import { useFileSystem } from "../composables/useFileSystem";
import { useRepos } from "../composables/useRepos";
import type { RepoInfo } from "../composables/useRepos";
import FileContent from "./FileContent.vue";
import GitWorkflow from "./GitWorkflow.vue";
import Editor from "./Editor.vue";

const props = defineProps<{
  repo: RepoInfo;
}>();

const emit = defineEmits<{
  (e: "close"): void;
  (e: "notify", msg: { type: "success" | "error"; text: string }): void;
}>();

const {
  files,
  searchResults,
  currentRelativePath,
  currentFilePath,
  renderedFile,
  loading,
  loadFiles,
  searchFiles,
  renderFile,
  openPath,
  enterDirectory,
  goBack,
} = useFileSystem();

const { pullRepo, pushRepo, syncing } = useRepos();

const view = ref<"files" | "git">("files");
const showSearch = ref(false);
const searchQuery = ref("");
const editingPath = ref<string | null>(null);

// Gestures State
const touchStartX = ref(0);
const touchStartY = ref(0);
const touchCurrentX = ref(0);
const touchCurrentY = ref(0);
const isSwiping = ref(false);
const isPulling = ref(false);
const pullDelta = ref(0);
const swipeDelta = ref(0);
const containerRef = ref<HTMLElement | null>(null);

const breadcrumbs = computed(() => {
  const parts = currentRelativePath.value ? currentRelativePath.value.split("/") : [];
  return [
    { name: props.repo.name, path: "" },
    ...parts.map((p, i) => ({
      name: p,
      path: parts.slice(0, i + 1).join("/")
    }))
  ];
});

async function navigateToBreadcrumb(index: number) {
  if (renderedFile.value !== null) {
    renderedFile.value = null;
  }
  
  if (index === 0) {
    currentRelativePath.value = "";
  } else {
    currentRelativePath.value = breadcrumbs.value[index].path;
  }
  await loadFiles(props.repo.id);
}

async function onPull() {
  const res = await pullRepo(props.repo.id);
  emit("notify", { type: res.success ? "success" : "error", text: res.message });
  if (res.success) {
    await loadFiles(props.repo.id);
  }
}

async function onPush() {
  const res = await pushRepo(props.repo.id);
  emit("notify", { type: res.success ? "success" : "error", text: res.message });
}

function handleBack() {
  if (view.value === "git") {
    view.value = "files";
    return;
  }

  if (showSearch.value) {
    showSearch.value = false;
    searchQuery.value = "";
    return;
  }

  if (!goBack()) {
    emit("close");
  } else {
    if (renderedFile.value === null) {
      loadFiles(props.repo.id);
    }
  }
}

async function handleEntry(entry: { name: string; is_dir: boolean }) {
  if (entry.is_dir) {
    enterDirectory(entry.name);
    await loadFiles(props.repo.id);
  } else {
    await renderFile(props.repo.id, entry.name);
  }
}

async function handleSearchEntry(entry: { relative_path: string; is_dir: boolean }) {
  showSearch.value = false;
  searchQuery.value = "";
  await openPath(props.repo.id, entry.relative_path, entry.is_dir);
}

// Gesture Handlers
function onTouchStart(e: TouchEvent) {
  touchStartX.value = e.touches[0].clientX;
  touchStartY.value = e.touches[0].clientY;
  touchCurrentX.value = touchStartX.value;
  touchCurrentY.value = touchStartY.value;
  isSwiping.value = false;
  isPulling.value = false;
  swipeDelta.value = 0;
  pullDelta.value = 0;
}

function onTouchMove(e: TouchEvent) {
  touchCurrentX.value = e.touches[0].clientX;
  touchCurrentY.value = e.touches[0].clientY;
  
  const deltaX = touchCurrentX.value - touchStartX.value;
  const deltaY = touchCurrentY.value - touchStartY.value;

  // Swipe back detection
  if (deltaX > 20 && Math.abs(deltaY) < 30 && view.value === 'files' && !showSearch.value) {
    isSwiping.value = true;
    swipeDelta.value = deltaX;
  }

  // Pull to refresh detection
  if (deltaY > 20 && Math.abs(deltaX) < 30 && containerRef.value?.scrollTop === 0 && view.value === 'files' && !renderedFile.value && !showSearch.value) {
    isPulling.value = true;
    pullDelta.value = deltaY;
    e.preventDefault(); // Prevent native scroll
  }
}

async function onTouchEnd() {
  if (isSwiping.value && swipeDelta.value > 100) {
    handleBack();
  }
  
  if (isPulling.value && pullDelta.value > 80) {
    await loadFiles(props.repo.id);
    if ('vibrate' in navigator) navigator.vibrate(20);
  }

  isSwiping.value = false;
  isPulling.value = false;
  swipeDelta.value = 0;
  pullDelta.value = 0;
}

let searchTimeout: number | null = null;
watch(searchQuery, (newVal) => {
  if (searchTimeout) clearTimeout(searchTimeout);
  searchTimeout = window.setTimeout(() => {
    searchFiles(props.repo.id, newVal);
  }, 300);
});

onMounted(() => loadFiles(props.repo.id));

watch(() => props.repo.id, () => loadFiles(props.repo.id));

function handleEdit() {
  editingPath.value = currentFilePath.value;
}
</script>

<template>
  <div 
    ref="containerRef"
    class="h-full flex flex-col"
    @touchstart="onTouchStart"
    @touchmove="onTouchMove"
    @touchend="onTouchEnd"
  >
    <!-- Editor Overlay -->
    <Editor 
      v-if="editingPath" 
      :repo="repo" 
      :relative-path="editingPath" 
      @close="editingPath = null" 
      @notify="(msg) => emit('notify', msg)"
    />

    <!-- Header -->
    <div 
      class="sticky top-0 z-20 bg-bg0 pb-4 pt-1 space-y-3 shadow-md -mx-6 px-6" 
      style="box-shadow: var(--shadow-md), var(--shadow-inset)"
    >
      <!-- Pull to refresh indicator -->
      <div 
        v-if="isPulling" 
        class="absolute top-0 left-0 right-0 flex justify-center pt-2 pointer-events-none transition-transform"
        :style="{ transform: `translateY(${Math.min(pullDelta / 2, 40)}px)` }"
      >
        <div class="bg-bg1 border border-border p-2 rounded-full shadow-lg">
          <RefreshCcw :size="20" class="text-green animate-spin" :style="{ animationDuration: '2s', transform: `rotate(${pullDelta * 2}deg)` }" />
        </div>
      </div>

      <!-- Row 1: Actions & Navigation -->
      <div class="flex items-center justify-between gap-3">
        <!-- Left: Back / Contextual Action -->
        <div class="flex items-center gap-2">
          <button
            @click="handleBack"
            class="min-w-[44px] min-h-[44px] flex items-center justify-center border border-border rounded-lg text-fg-dim hover:text-fg hover:border-fg-dim active:scale-95 duration-100 transition-all cursor-pointer bg-bg1 shadow-sm"
            aria-label="Back"
          >
            <ChevronLeft :size="20" />
          </button>
        </div>

        <!-- Center: Contextual Title or Switcher -->
        <div class="flex-1 flex justify-center min-w-0 px-2">
          <div v-if="renderedFile" class="flex justify-center">
            <div class="px-3 py-1 bg-bg3 text-aqua text-[10px] font-bold uppercase tracking-widest rounded-full border border-border/50 font-sans shadow-sm">
              {{ renderedFile.file_type }}
            </div>
          </div>
          <div v-else class="flex bg-bg1 border border-border rounded-lg p-0.5 shadow-sm font-sans">
            <button
              @click="view = 'files'"
              class="min-h-[40px] px-3 py-1 text-[10px] font-bold uppercase tracking-wider rounded-md transition-all active:scale-95 duration-100 cursor-pointer font-sans"
              :class="view === 'files' ? 'bg-bg3 text-fg shadow-sm' : 'text-fg-dim hover:text-fg'"
            >
              Files
            </button>
            <button
              @click="view = 'git'"
              class="min-h-[40px] px-3 py-1 text-[10px] font-bold uppercase tracking-wider rounded-md transition-all active:scale-95 duration-100 cursor-pointer font-sans"
              :class="view === 'git' ? 'bg-bg3 text-fg shadow-sm' : 'text-fg-dim hover:text-fg'"
            >
              Git
            </button>
          </div>
        </div>

        <!-- Right: Actions Group -->
        <div class="flex items-center">
          <button
            v-if="view === 'files' && renderedFile"
            @click="handleEdit"
            class="min-h-[44px] flex items-center gap-2 px-3 border border-border rounded-lg text-fg-dim hover:text-yellow hover:border-yellow active:scale-95 duration-100 transition-all cursor-pointer bg-bg1 shadow-sm font-sans"
            title="Edit File"
          >
            <Edit2 :size="16" />
            <span class="text-xs font-bold">Edit</span>
          </button>
          <div v-else-if="view === 'files'" class="flex bg-bg1 border border-border rounded-xl overflow-hidden divide-x divide-border shadow-sm">
            <button
              @click="showSearch = !showSearch"
              class="min-w-[44px] min-h-[44px] flex items-center justify-center text-fg-dim hover:text-yellow active:bg-bg3 active:scale-95 duration-100 transition-all cursor-pointer"
              :class="{ 'bg-bg3 text-yellow': showSearch }"
              title="Search Files"
            >
              <Search :size="18" />
            </button>
            <button
              @click="onPull"
              :disabled="syncing === repo.id"
              class="min-w-[44px] min-h-[44px] flex items-center justify-center text-fg-dim hover:text-green active:bg-bg3 active:scale-95 duration-100 transition-all disabled:opacity-30 cursor-pointer"
              title="Pull Changes"
            >
              <Loader2 v-if="syncing === repo.id" :size="18" class="animate-spin" />
              <Download v-else :size="18" />
            </button>
            <button
              @click="onPush"
              :disabled="syncing === repo.id"
              class="min-w-[44px] min-h-[44px] flex items-center justify-center text-fg-dim hover:text-aqua active:bg-bg3 active:scale-95 duration-100 transition-all disabled:opacity-30 cursor-pointer"
              title="Push Changes"
            >
              <Loader2 v-if="syncing === repo.id" :size="18" class="animate-spin" />
              <Upload v-else :size="18" />
            </button>
          </div>
        </div>
      </div>

      <!-- Row 2: Breadcrumbs / Meta -->
      <div v-if="view === 'files'" class="flex items-center">
        <div v-if="renderedFile" class="flex items-center gap-1.5 px-4 py-2 bg-bg1 border border-border rounded-full text-[11px] font-mono text-fg-dim w-full shadow-inner overflow-hidden">
          <span class="opacity-40 truncate">{{ currentRelativePath ? currentRelativePath + '/' : '' }}</span>
          <span class="text-fg font-bold truncate">{{ currentFilePath?.split('/').pop() }}</span>
        </div>
        <div v-else class="flex items-center gap-1 px-3 py-1.5 bg-bg1 border border-border rounded-full text-[11px] font-mono text-fg-dim overflow-x-auto no-scrollbar w-full shadow-inner font-mono">
          <template v-if="breadcrumbs.length > 2">
            <button @click="navigateToBreadcrumb(0)" class="hover:text-yellow transition-colors cursor-pointer min-h-[30px] px-1">…</button>
            <span class="text-border">/</span>
            <template v-for="(crumb, i) in breadcrumbs.slice(-2)" :key="crumb.path">
              <button
                @click="navigateToBreadcrumb(breadcrumbs.length - 2 + i)"
                class="hover:text-yellow transition-colors cursor-pointer truncate max-w-[100px] min-h-[30px] px-1"
                :class="{ 'text-fg font-bold': i === 1 }"
              >
                {{ crumb.name }}
              </button>
              <span v-if="i === 0" class="text-border">/</span>
            </template>
          </template>
          <template v-else>
            <template v-for="(crumb, i) in breadcrumbs" :key="crumb.path">
              <button
                @click="navigateToBreadcrumb(i)"
                class="hover:text-yellow transition-colors cursor-pointer truncate max-w-[120px] min-h-[30px] px-1"
                :class="{ 'text-fg font-bold': i === breadcrumbs.length - 1 }"
              >
                {{ crumb.name }}
              </button>
              <span v-if="i < breadcrumbs.length - 1" class="text-border">/</span>
            </template>
          </template>
        </div>
      </div>
    </div>

    <!-- Git View -->
    <div v-if="view === 'git'" class="flex-1 overflow-y-auto px-6 -mx-6">
      <GitWorkflow :repo="repo" />
    </div>

    <!-- Files View -->
    <div 
      v-else 
      class="flex-1 overflow-y-auto transition-transform duration-200"
      :style="{ transform: `translateX(${Math.min(swipeDelta / 2, 80)}px)` }"
    >
      <!-- Search Bar -->
      <Transition name="slide">
        <div v-if="showSearch" class="mb-6">
          <div class="relative">
            <Search class="absolute left-3 top-1/2 -translate-y-1/2 text-fg-dim" :size="16" />
            <input
              v-model="searchQuery"
              placeholder="Search filenames..."
              class="w-full pl-10 pr-4 py-2.5 bg-bg1 border border-border rounded-lg outline-none focus:border-yellow transition-all text-sm font-mono"
              autofocus
            />
          </div>
        </div>
      </Transition>

      <!-- Content -->
      <div v-if="loading" class="flex flex-col items-center justify-center py-20 opacity-50">
        <Loader2 :size="32" class="animate-spin text-green mb-2" />
        <span class="text-xs font-mono">Loading...</span>
      </div>

      <div v-else-if="renderedFile !== null">
        <FileContent :file="renderedFile" :filename="renderedFile.name" @edit="handleEdit" />
      </div>

      <!-- Search Results -->
      <div v-else-if="searchQuery" class="space-y-1">
        <div
          v-for="result in searchResults"
          :key="result.relative_path"
          @click="handleSearchEntry(result)"
          class="flex flex-col px-4 py-3 bg-bg1 border border-border rounded-lg cursor-pointer hover:border-fg-dim active:scale-[0.99] transition-all"
        >
          <div class="flex items-center gap-3">
            <div :class="result.is_dir ? 'text-yellow' : 'text-fg-dim'">
              <Folder v-if="result.is_dir" :size="18" class="fill-yellow/10" />
              <FileText v-else :size="18" />
            </div>
            <span class="truncate font-bold text-sm font-sans font-medium">{{ result.name }}</span>
          </div>
          <span class="text-[10px] text-fg-dim mt-1 truncate font-mono opacity-60 ml-7">{{ result.relative_path }}</span>
        </div>
        
        <div v-if="searchResults.length === 0" class="flex flex-col items-center justify-center py-16 text-fg-dim opacity-30">
          <Search :size="40" class="mb-3 stroke-[1.5]" />
          <p class="text-sm">No results found for "{{ searchQuery }}"</p>
        </div>
      </div>

      <div v-else class="space-y-1">
        <div
          v-for="entry in files"
          :key="entry.name"
          @click="handleEntry(entry)"
          class="flex items-center gap-3 px-4 py-4 bg-bg1 border border-border rounded-lg cursor-pointer hover:border-fg-dim active:scale-[0.99] transition-all"
        >
          <div :class="entry.is_dir ? 'text-yellow' : 'text-fg-dim'">
            <Folder v-if="entry.is_dir" :size="20" class="fill-yellow/10" />
            <FileText v-else :size="20" />
          </div>
          <span class="truncate font-medium text-sm font-sans font-medium">{{ entry.name }}</span>
        </div>
        
        <div v-if="files.length === 0" class="flex flex-col items-center justify-center py-16 text-fg-dim opacity-30">
          <Home :size="40" class="mb-3 stroke-[1.5]" />
          <p class="text-sm font-sans">Empty directory</p>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.no-scrollbar::-webkit-scrollbar {
  display: none;
}
.no-scrollbar {
  -ms-overflow-style: none;
  scrollbar-width: none;
}

.slide-enter-active,
.slide-leave-active {
  transition: all 0.3s ease;
  max-height: 100px;
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
