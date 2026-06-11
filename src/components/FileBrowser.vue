<script setup lang="ts">
import { 
  ChevronLeft, 
  ChevronRight, 
  Download, 
  Upload, 
  Folder, 
  FileText, 
  Loader2,
  Home
} from "@lucide/vue";
import { onMounted, watch, computed } from "vue";
import { useFileSystem } from "../composables/useFileSystem";
import { useRepos } from "../composables/useRepos";
import type { RepoInfo } from "../composables/useRepos";
import FileContent from "./FileContent.vue";

const props = defineProps<{
  repo: RepoInfo;
}>();

const emit = defineEmits<{
  (e: "close"): void;
  (e: "notify", msg: { type: "success" | "error"; text: string }): void;
}>();

const {
  files,
  currentRelativePath,
  selectedFileContent,
  loading,
  loadFiles,
  readFile,
  enterDirectory,
  goBack,
} = useFileSystem();

const { pullRepo, pushRepo, syncing } = useRepos();

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
  if (selectedFileContent.value !== null) {
    selectedFileContent.value = null;
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
  if (!goBack()) {
    emit("close");
  } else {
    if (selectedFileContent.value === null) {
      loadFiles(props.repo.id);
    }
  }
}

async function handleEntry(entry: { name: string; is_dir: boolean }) {
  if (entry.is_dir) {
    enterDirectory(entry.name);
    await loadFiles(props.repo.id);
  } else {
    await readFile(props.repo.id, entry.name);
  }
}

onMounted(() => loadFiles(props.repo.id));

watch(() => props.repo.id, () => loadFiles(props.repo.id));
</script>

<template>
  <div>
    <!-- Header -->
    <div class="flex items-center justify-between mb-6 gap-4">
      <button
        @click="handleBack"
        class="shrink-0 p-2 border border-border rounded-lg text-fg-dim hover:text-fg hover:border-fg-dim active:scale-95 transition-all cursor-pointer"
        aria-label="Back"
      >
        <ChevronLeft :size="20" />
      </button>

      <div class="flex-1 overflow-x-auto no-scrollbar py-1">
        <div class="flex items-center gap-1.5 whitespace-nowrap text-sm font-medium">
          <template v-for="(crumb, i) in breadcrumbs" :key="crumb.path">
            <button
              @click="navigateToBreadcrumb(i)"
              class="text-fg-dim hover:text-yellow transition-colors cursor-pointer"
              :class="{ 'text-fg font-bold': i === breadcrumbs.length - 1 && selectedFileContent === null }"
            >
              {{ crumb.name }}
            </button>
            <ChevronRight v-if="i < breadcrumbs.length - 1" :size="14" class="text-border" />
          </template>
        </div>
      </div>

      <div v-if="!selectedFileContent && !currentRelativePath" class="flex gap-2 shrink-0">
        <button
          @click="onPull"
          :disabled="syncing === repo.id"
          class="p-2 border border-border rounded-lg text-fg-dim hover:text-green hover:border-green active:scale-95 transition-all disabled:opacity-30 cursor-pointer"
          title="Pull Changes"
        >
          <Loader2 v-if="syncing === repo.id" :size="20" class="animate-spin" />
          <Download v-else :size="20" />
        </button>
        <button
          @click="onPush"
          :disabled="syncing === repo.id"
          class="p-2 border border-border rounded-lg text-fg-dim hover:text-aqua hover:border-aqua active:scale-95 transition-all disabled:opacity-30 cursor-pointer"
          title="Push Changes"
        >
          <Loader2 v-if="syncing === repo.id" :size="20" class="animate-spin" />
          <Upload v-else :size="20" />
        </button>
      </div>
    </div>

    <!-- Content -->
    <div v-if="loading" class="flex flex-col items-center justify-center py-20 opacity-50">
      <Loader2 :size="32" class="animate-spin text-green mb-2" />
      <span class="text-xs font-mono">Loading...</span>
    </div>

    <div v-else-if="selectedFileContent !== null">
      <FileContent :content="selectedFileContent" />
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
</template>

<style scoped>
.no-scrollbar::-webkit-scrollbar {
  display: none;
}
.no-scrollbar {
  -ms-overflow-style: none;
  scrollbar-width: none;
}
</style>
