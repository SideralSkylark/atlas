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
  Edit2
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
  <div>
    <!-- Editor Overlay -->
    <Editor 
      v-if="editingPath" 
      :repo="repo" 
      :relative-path="editingPath" 
      @close="editingPath = null" 
      @notify="(msg) => emit('notify', msg)"
    />

    <!-- Header -->
    <div class="sticky top-0 z-20 bg-bg0 pb-4 pt-1">
      <div class="flex items-center justify-between gap-4">
        <div class="flex items-center gap-2">
          <button
            @click="handleBack"
            class="shrink-0 p-2 border border-border rounded-lg text-fg-dim hover:text-fg hover:border-fg-dim active:scale-95 transition-all cursor-pointer bg-bg1"
            aria-label="Back"
          >
            <ChevronLeft :size="20" />
          </button>

          <button
            v-if="view === 'files' && renderedFile"
            @click="handleEdit"
            class="shrink-0 p-2 border border-border rounded-lg text-fg-dim hover:text-yellow hover:border-yellow active:scale-95 transition-all cursor-pointer bg-bg1"
            title="Edit File"
          >
            <Edit2 :size="20" />
          </button>

          <div class="flex bg-bg1 border border-border rounded-lg p-0.5 shadow-sm">
            <button
              @click="view = 'files'"
              class="px-3 py-1 text-[10px] font-bold uppercase tracking-wider rounded-md transition-all cursor-pointer"
              :class="view === 'files' ? 'bg-bg3 text-fg shadow-sm' : 'text-fg-dim hover:text-fg'"
            >
              Files
            </button>
            <button
              @click="view = 'git'"
              class="px-3 py-1 text-[10px] font-bold uppercase tracking-wider rounded-md transition-all cursor-pointer"
              :class="view === 'git' ? 'bg-bg3 text-fg shadow-sm' : 'text-fg-dim hover:text-fg'"
            >
              Git
            </button>
          </div>
        </div>

        <div v-if="view === 'files'" class="flex-1 flex items-center justify-end overflow-hidden">
          <div v-if="renderedFile" class="flex items-center gap-2 overflow-hidden">
            <span class="text-sm font-bold text-fg truncate">{{ renderedFile.name }}</span>
            <span class="px-2 py-0.5 bg-bg3 text-aqua text-[10px] font-bold uppercase tracking-wider rounded border border-border/50 shrink-0">
              {{ renderedFile.file_type }}
            </span>
          </div>

          <div v-else class="flex items-center gap-3 overflow-hidden">
            <div class="flex items-center gap-1 flex-wrap justify-end text-sm font-medium">
              <template v-if="breadcrumbs.length > 2">
                <button
                  @click="navigateToBreadcrumb(0)"
                  class="text-fg-dim hover:text-yellow transition-colors cursor-pointer"
                >
                  ...
                </button>
                <ChevronRight :size="14" class="text-border" />
                <template v-for="(crumb, i) in breadcrumbs.slice(-2)" :key="crumb.path">
                  <button
                    @click="navigateToBreadcrumb(breadcrumbs.length - 2 + i)"
                    class="text-fg-dim hover:text-yellow transition-colors cursor-pointer"
                    :class="{ 'text-fg font-bold': i === 1 }"
                  >
                    {{ crumb.name }}
                  </button>
                  <ChevronRight v-if="i === 0" :size="14" class="text-border" />
                </template>
              </template>
              <template v-else>
                <template v-for="(crumb, i) in breadcrumbs" :key="crumb.path">
                  <button
                    @click="navigateToBreadcrumb(i)"
                    class="text-fg-dim hover:text-yellow transition-colors cursor-pointer"
                    :class="{ 'text-fg font-bold': i === breadcrumbs.length - 1 }"
                  >
                    {{ crumb.name }}
                  </button>
                  <ChevronRight v-if="i < breadcrumbs.length - 1" :size="14" class="text-border" />
                </template>
              </template>
            </div>

            <div class="flex gap-1.5 shrink-0">
              <button
                @click="showSearch = !showSearch"
                class="p-2 border border-border rounded-lg text-fg-dim hover:text-yellow hover:border-yellow active:scale-95 transition-all cursor-pointer bg-bg1"
                :class="{ 'bg-bg3 border-yellow text-yellow': showSearch }"
                title="Search Files"
              >
                <Search :size="18" />
              </button>
              <button
                @click="onPull"
                :disabled="syncing === repo.id"
                class="p-2 border border-border rounded-lg text-fg-dim hover:text-green hover:border-green active:scale-95 transition-all disabled:opacity-30 cursor-pointer bg-bg1"
                title="Pull Changes"
              >
                <Loader2 v-if="syncing === repo.id" :size="18" class="animate-spin" />
                <Download v-else :size="18" />
              </button>
              <button
                @click="onPush"
                :disabled="syncing === repo.id"
                class="p-2 border border-border rounded-lg text-fg-dim hover:text-aqua hover:border-aqua active:scale-95 transition-all disabled:opacity-30 cursor-pointer bg-bg1"
                title="Push Changes"
              >
                <Loader2 v-if="syncing === repo.id" :size="18" class="animate-spin" />
                <Upload v-else :size="18" />
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Git View -->
    <div v-if="view === 'git'">
      <GitWorkflow :repo="repo" />
    </div>

    <!-- Files View -->
    <div v-else>
      <!-- Search Bar -->
      <Transition name="slide">
        <div v-if="showSearch" class="mb-6">
          <div class="relative">
            <Search class="absolute left-3 top-1/2 -translate-y-1/2 text-fg-dim" :size="16" />
            <input
              v-model="searchQuery"
              placeholder="Search filenames..."
              class="w-full pl-10 pr-4 py-2.5 bg-bg1 border border-border rounded-lg outline-none focus:border-yellow transition-all text-sm"
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
        <FileContent :file="renderedFile" @edit="handleEdit" />
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
            <span class="truncate font-bold text-sm">{{ result.name }}</span>
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
          <span class="truncate font-medium text-sm">{{ entry.name }}</span>
        </div>
        
        <div v-if="files.length === 0" class="flex flex-col items-center justify-center py-16 text-fg-dim opacity-30">
          <Home :size="40" class="mb-3 stroke-[1.5]" />
          <p class="text-sm">Empty directory</p>
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
