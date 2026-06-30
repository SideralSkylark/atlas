<script setup lang="ts">
import { 
  ChevronLeft, 
  Download, 
  Upload, 
  Folder, 
  FileText, 
  Loader2,
  Home,
  Search,
  Edit2,
  RefreshCcw,
  FilePlus,
  FolderPlus,
  Trash2,
  X,
  Plus
} from "@lucide/vue";
import { invoke } from "@tauri-apps/api/core";
import { onMounted, watch, computed, ref, onUnmounted } from "vue";
import { useFileSystem } from "../composables/useFileSystem";
import { useRepos } from "../composables/useRepos";
import type { RepoInfo } from "../composables/useRepos";
import FileContent from "./FileContent.vue";
import GitWorkflow from "./GitWorkflow.vue";
import Editor from "./Editor.vue";

interface StatusEntry {
  path: string;
  status: string;
  staged: boolean;
}

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
  error,
  loadFiles: fsLoadFiles,
  searchFiles,
  renderFile,
  openPath,
  enterDirectory,
  goBack,
  createFile,
  createDirectory,
  deleteItem,
} = useFileSystem();

const { pullRepo, pushRepo, syncing } = useRepos();

const view = ref<"files" | "git">("files");
const showSearch = ref(false);
const searchQuery = ref("");
const editingPath = ref<string | null>(null);
const editorRef = ref<any>(null);
const gitWorkflowRef = ref<any>(null);

const fileStatuses = ref<StatusEntry[]>([]);

type FileStatus = 'staged' | 'Modified' | 'New' | 'Deleted' | 'Renamed' | null

function entryFullPath(name: string): string {
  return currentRelativePath.value 
    ? `${currentRelativePath.value}/${name}` 
    : name;
}

function getFileStatus(fullPath: string): FileStatus {
  const entry = fileStatuses.value.find(s => s.path === fullPath);
  if (!entry) return null;
  if (entry.staged) return 'staged';
  return entry.status as FileStatus;
}

const dirStatuses = computed(() => {
  const map = new Map<string, FileStatus>();
  
  for (const entry of fileStatuses.value) {
    const parts = entry.path.split('/');
    // Walk up every directory segment of this file's path
    for (let i = 1; i < parts.length; i++) {
      const dirPath = parts.slice(0, i).join('/');
      const existing = map.get(dirPath);
      // Priority: staged > Modified > New > Deleted > Renamed
      const incoming: FileStatus = entry.staged ? 'staged' : entry.status as FileStatus;
      if (!existing) {
        map.set(dirPath, incoming);
      } else {
        // Only upgrade priority, never downgrade
        const priority = ['Renamed', 'Deleted', 'New', 'Modified', 'staged'];
        const incomingIndex = incoming ? priority.indexOf(incoming) : -1;
        const existingIndex = existing ? priority.indexOf(existing) : -1;
        if (incomingIndex > existingIndex) {
          map.set(dirPath, incoming);
        }
      }
    }
  }
  
  return map;
})

function getEntryStatus(name: string, isDir: boolean): FileStatus {
  const fullPath = entryFullPath(name);
  if (isDir) return dirStatuses.value.get(fullPath) ?? null;
  return getFileStatus(fullPath);
}

async function loadFiles(repoId: string) {
  await fsLoadFiles(repoId);
  try {
    fileStatuses.value = await invoke<StatusEntry[]>("get_status", { repoId });
  } catch (e) {
    console.error("Failed to load git status:", e);
  }
}

defineExpose({
  editingPath,
  renderedFile,
  currentRelativePath,
  handleBack,
  editorRef
});

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

  // Pull to refresh detection (allowed in both files and git views when scrolled to top)
  const isScrollAtTop = containerRef.value?.scrollTop === 0;
  const canPull = isScrollAtTop && !renderedFile.value && !showSearch.value;
  if (deltaY > 20 && Math.abs(deltaX) < 30 && canPull) {
    isPulling.value = true;
    pullDelta.value = deltaY;
    e.preventDefault(); // Prevent native scroll
  }
}

async function onTouchEnd() {
  if (isSwiping.value && swipeDelta.value > 100) {
    handleBack();
  }
  
  if (isPulling.value && pullDelta.value > 80 && syncing.value !== props.repo.id) {
    if ('vibrate' in navigator) navigator.vibrate(20);
    await onPull();
    if (view.value === 'git' && gitWorkflowRef.value) {
      await gitWorkflowRef.value.reloadAll();
    }
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

onMounted(async () => {
  await loadFiles(props.repo.id);
});

onUnmounted(() => {
});

watch(() => props.repo.id, () => loadFiles(props.repo.id));

function handleEdit() {
  editingPath.value = currentFilePath.value;
}

async function handleSave() {
  if (editingPath.value) {
    const fileName = editingPath.value.split('/').pop()!;
    await renderFile(props.repo.id, fileName);
    try {
      fileStatuses.value = await invoke<StatusEntry[]>("get_status", { repoId: props.repo.id });
    } catch (e) {
      console.error("Failed to load git status:", e);
    }
  }
}

// File and Folder Creation/Deletion State & Logic
const showCreateModal = ref(false);
const createType = ref<"file" | "directory">("file");
const newName = ref("");
const createError = ref<string | null>(null);

const showDeleteModal = ref(false);
const itemToDelete = ref<{ name: string; is_dir: boolean } | null>(null);
const deleteError = ref<string | null>(null);

const showSpeedDial = ref(false);

function openCreateModal(type: "file" | "directory") {
  createType.value = type;
  newName.value = "";
  createError.value = null;
  showCreateModal.value = true;
}

function triggerCreate(type: "file" | "directory") {
  showSpeedDial.value = false;
  openCreateModal(type);
}

async function handleCreate() {
  if (!newName.value.trim()) {
    createError.value = "Name cannot be empty";
    return;
  }
  
  const path = currentRelativePath.value
    ? `${currentRelativePath.value}/${newName.value.trim()}`
    : newName.value.trim();
    
  let success = false;
  if (createType.value === "file") {
    success = await createFile(props.repo.id, path);
  } else {
    success = await createDirectory(props.repo.id, path);
  }
  
  if (success) {
    showCreateModal.value = false;
    await loadFiles(props.repo.id);
    emit("notify", { 
      type: "success", 
      text: `${createType.value === 'file' ? 'File' : 'Folder'} "${newName.value}" created successfully` 
    });
  } else {
    createError.value = error.value || `Failed to create ${createType.value}`;
  }
}

function confirmDelete(entry: { name: string; is_dir: boolean }) {
  itemToDelete.value = entry;
  deleteError.value = null;
  showDeleteModal.value = true;
}

async function handleDelete() {
  if (!itemToDelete.value) return;
  
  const path = currentRelativePath.value
    ? `${currentRelativePath.value}/${itemToDelete.value.name}`
    : itemToDelete.value.name;
    
  const success = await deleteItem(props.repo.id, path);
  
  if (success) {
    showDeleteModal.value = false;
    itemToDelete.value = null;
    await loadFiles(props.repo.id);
    emit("notify", { 
      type: "success", 
      text: `Deleted successfully` 
    });
  } else {
    deleteError.value = error.value || "Failed to delete item";
  }
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
      ref="editorRef"
      :repo="repo" 
      :relative-path="editingPath" 
      @close="editingPath = null" 
      @save="handleSave"
      @notify="(msg) => emit('notify', msg)"
    />

    <!-- Header -->
    <div 
      class="sticky top-0 z-20 bg-bg0 pb-4 pt-1 space-y-3 shadow-md -mx-6 px-6" 
      style="box-shadow: var(--shadow-md), var(--shadow-inset)"
    >
      <!-- Pull to refresh indicator -->
      <div 
        v-if="isPulling || syncing === repo.id" 
        class="absolute top-0 left-0 right-0 flex justify-center pt-2 pointer-events-none transition-transform"
        :style="{ transform: `translateY(${isPulling ? Math.min(pullDelta / 2, 40) : 40}px)` }"
      >
        <div class="bg-bg1 border border-border p-2 rounded-full shadow-lg" style="box-shadow: var(--shadow-sm), var(--shadow-inset)">
          <RefreshCcw :size="20" class="text-green animate-spin" :style="{ animationDuration: '2s', transform: syncing === repo.id ? 'none' : `rotate(${pullDelta * 2}deg)` }" />
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
          <div v-if="!renderedFile" class="flex bg-bg1 border border-border rounded-lg p-0.5 shadow-sm font-sans">
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
      <div v-if="view === 'files'" class="flex items-center -mx-6 px-6">
        <div v-if="renderedFile" class="flex items-center gap-1.5 px-4 py-2 bg-bg1 border border-border rounded-full text-[11px] font-mono text-fg-dim w-full shadow-inner overflow-hidden">
          <span class="opacity-40 truncate">{{ currentRelativePath ? currentRelativePath + '/' : '' }}</span>
          <span class="text-fg font-bold truncate">{{ currentFilePath?.split('/').pop() }}</span>
        </div>
        <div v-else class="flex items-center gap-1 px-3 py-1.5 bg-bg1 border border-border rounded-full text-[11px] font-mono text-fg-dim overflow-x-auto w-full shadow-inner font-mono">
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
      <GitWorkflow ref="gitWorkflowRef" :repo="repo" @reload-files="loadFiles(repo.id)" @edit-file="(path) => { editingPath = path }" />
    </div>

    <!-- Files View -->
    <div 
      v-else 
      class="flex-1 overflow-y-auto px-6 -mx-6 transition-transform duration-200"
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
        <FileContent :file="renderedFile" :filename="currentFilePath?.split('/').pop() || undefined" @edit="handleEdit" />
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
          <div class="flex items-center gap-3 ml-auto shrink-0">
            <div
              v-if="getEntryStatus(entry.name, entry.is_dir)"
              class="w-1.5 h-1.5 rounded-full shrink-0"
              :class="{
                'bg-aqua':   getEntryStatus(entry.name, entry.is_dir) === 'staged',
                'bg-yellow': getEntryStatus(entry.name, entry.is_dir) === 'Modified',
                'bg-green':  getEntryStatus(entry.name, entry.is_dir) === 'New',
                'bg-red':    getEntryStatus(entry.name, entry.is_dir) === 'Deleted',
                'bg-orange': getEntryStatus(entry.name, entry.is_dir) === 'Renamed',
              }"
              :title="getEntryStatus(entry.name, entry.is_dir) ?? ''"
            />
            
            <button
              @click.stop="confirmDelete(entry)"
              class="w-8 h-8 flex items-center justify-center text-fg-dim opacity-50 hover:text-red hover:opacity-100 hover:bg-bg2/40 active:scale-95 duration-100 transition-all rounded-md cursor-pointer"
              title="Delete"
            >
              <Trash2 :size="16" />
            </button>
          </div>
        </div>
        
        <div v-if="files.length === 0" class="flex flex-col items-center justify-center py-16 text-fg-dim opacity-30">
          <Home :size="40" class="mb-3 stroke-[1.5]" />
          <p class="text-sm font-sans">Empty directory</p>
        </div>
      </div>
    </div>

  <!-- Create File/Folder Modal -->
  <Transition name="fade">
    <div 
      v-if="showCreateModal" 
      class="fixed inset-0 z-50 flex items-center justify-center bg-bg0/80 backdrop-blur-xs p-4"
      @click.self="showCreateModal = false"
    >
      <div 
        class="bg-bg1 border border-border rounded-xl p-6 max-w-sm w-full space-y-4 relative"
        style="box-shadow: var(--shadow-lg), var(--shadow-inset)"
      >
        <!-- Header -->
        <div class="flex items-center justify-between">
          <h3 class="text-sm font-bold uppercase tracking-wider text-fg font-sans flex items-center gap-2">
            <FilePlus v-if="createType === 'file'" :size="16" class="text-green" />
            <FolderPlus v-else :size="16" class="text-yellow" />
            Create New {{ createType === 'file' ? 'File' : 'Folder' }}
          </h3>
          <button 
            @click="showCreateModal = false"
            class="p-1 rounded-md text-fg-dim hover:text-fg hover:bg-bg2 transition-colors cursor-pointer"
          >
            <X :size="16" />
          </button>
        </div>

        <!-- Input -->
        <div class="space-y-1">
          <label class="text-[10px] font-bold uppercase tracking-wider text-fg-dim font-sans">
            Name
          </label>
          <input
            v-model="newName"
            placeholder="e.g. index.ts or components"
            class="w-full px-3 py-2 bg-bg2 border border-border rounded-lg outline-none focus:border-yellow text-sm font-mono"
            @keydown.enter="handleCreate"
            autofocus
          />
        </div>

        <!-- Error -->
        <div v-if="createError" class="text-xs text-red font-mono bg-red/10 border border-red/20 rounded-lg p-2.5">
          {{ createError }}
        </div>

        <!-- Actions -->
        <div class="flex items-center justify-end gap-2 pt-2">
          <button
            @click="showCreateModal = false"
            class="min-h-[36px] px-4 text-xs font-bold rounded-lg border border-border text-fg-dim hover:text-fg hover:bg-bg2 active:scale-95 duration-100 transition-all cursor-pointer font-sans"
          >
            Cancel
          </button>
          <button
            @click="handleCreate"
            class="min-h-[36px] px-4 text-xs font-bold rounded-lg text-bg0 font-sans active:scale-95 duration-100 transition-all cursor-pointer"
            :class="createType === 'file' ? 'bg-green hover:bg-green/90' : 'bg-yellow hover:bg-yellow/90'"
          >
            Create
          </button>
        </div>
      </div>
    </div>
  </Transition>

  <!-- Delete Confirmation Modal -->
  <Transition name="fade">
    <div 
      v-if="showDeleteModal" 
      class="fixed inset-0 z-50 flex items-center justify-center bg-bg0/80 backdrop-blur-xs p-4"
      @click.self="showDeleteModal = false"
    >
      <div 
        class="bg-bg1 border border-border rounded-xl p-6 max-w-sm w-full space-y-4 relative"
        style="box-shadow: var(--shadow-lg), var(--shadow-inset)"
      >
        <!-- Header -->
        <div class="flex items-center justify-between">
          <h3 class="text-sm font-bold uppercase tracking-wider text-red font-sans flex items-center gap-2">
            <Trash2 :size="16" />
            Delete {{ itemToDelete?.is_dir ? 'Folder' : 'File' }}
          </h3>
          <button 
            @click="showDeleteModal = false"
            class="p-1 rounded-md text-fg-dim hover:text-fg hover:bg-bg2 transition-colors cursor-pointer"
          >
            <X :size="16" />
          </button>
        </div>

        <!-- Description -->
        <div class="space-y-2">
          <p class="text-sm text-fg font-sans">
            Are you sure you want to delete <span class="font-mono bg-bg2 px-1.5 py-0.5 rounded text-xs border border-border font-bold">{{ itemToDelete?.name }}</span>?
          </p>
          <p class="text-[11px] text-fg-dim font-sans leading-relaxed">
            This action cannot be undone. All contents will be permanently deleted.
          </p>
        </div>

        <!-- Error -->
        <div v-if="deleteError" class="text-xs text-red font-mono bg-red/10 border border-red/20 rounded-lg p-2.5">
          {{ deleteError }}
        </div>

        <!-- Actions -->
        <div class="flex items-center justify-end gap-2 pt-2">
          <button
            @click="showDeleteModal = false"
            class="min-h-[36px] px-4 text-xs font-bold rounded-lg border border-border text-fg-dim hover:text-fg hover:bg-bg2 active:scale-95 duration-100 transition-all cursor-pointer font-sans"
          >
            Cancel
          </button>
          <button
            @click="handleDelete"
            class="min-h-[36px] px-4 text-xs font-bold rounded-lg bg-red hover:bg-red/90 text-bg0 font-sans active:scale-95 duration-100 transition-all cursor-pointer"
          >
            Delete
          </button>
        </div>
      </div>
    </div>
  </Transition>

  <!-- Speed Dial Backdrop Click-Away -->
  <div 
    v-if="showSpeedDial" 
    class="fixed inset-0 z-35 bg-transparent pointer-events-auto"
    @click="showSpeedDial = false"
  />

  <!-- FAB (Floating Action Button) with Speed Dial -->
  <div 
    v-if="view === 'files' && !renderedFile" 
    class="fixed bottom-[calc(5.5rem+env(safe-area-inset-bottom))] left-0 right-0 z-40 pointer-events-none"
  >
    <div class="max-w-2xl mx-auto px-6 flex flex-col items-end gap-3 pointer-events-auto">
      <!-- Speed Dial Options -->
      <Transition name="speed-dial">
        <div v-if="showSpeedDial" class="flex flex-col items-end gap-3 mb-2">
          <!-- Create File Button -->
          <button
            @click="triggerCreate('file')"
            class="flex items-center gap-2 px-3 py-2 bg-bg1 border border-border rounded-lg text-fg hover:text-green shadow-md active:scale-95 duration-100 transition-all cursor-pointer font-sans text-xs font-bold"
            style="box-shadow: var(--shadow-md), var(--shadow-inset)"
          >
            <span>New File</span>
            <div class="w-8 h-8 rounded-full bg-green/10 flex items-center justify-center text-green">
              <FilePlus :size="16" />
            </div>
          </button>

          <!-- Create Folder Button -->
          <button
            @click="triggerCreate('directory')"
            class="flex items-center gap-2 px-3 py-2 bg-bg1 border border-border rounded-lg text-fg hover:text-yellow shadow-md active:scale-95 duration-100 transition-all cursor-pointer font-sans text-xs font-bold"
            style="box-shadow: var(--shadow-md), var(--shadow-inset)"
          >
            <span>New Folder</span>
            <div class="w-8 h-8 rounded-full bg-yellow/10 flex items-center justify-center text-yellow">
              <FolderPlus :size="16" />
            </div>
          </button>
        </div>
      </Transition>

      <!-- Main FAB Button -->
      <button
        @click="showSpeedDial = !showSpeedDial"
        class="w-12 h-12 rounded-full bg-yellow text-bg0 flex items-center justify-center shadow-lg active:scale-95 duration-150 transition-all cursor-pointer relative"
        style="box-shadow: var(--shadow-lg), var(--shadow-inset)"
        title="Add New..."
      >
        <Plus 
          :size="24" 
          class="transition-transform duration-200"
          :class="{ 'rotate-45': showSpeedDial }"
        />
      </button>
    </div>
  </div>
</div>
</template>

<style scoped>
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

.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.2s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}

.speed-dial-enter-active,
.speed-dial-leave-active {
  transition: all 0.2s cubic-bezier(0.16, 1, 0.3, 1);
}

.speed-dial-enter-from,
.speed-dial-leave-to {
  opacity: 0;
  transform: translateY(15px) scale(0.9);
}
</style>
