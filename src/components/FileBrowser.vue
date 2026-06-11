<script setup lang="ts">
import { onMounted, watch } from "vue";
import { useFileSystem } from "../composables/useFileSystem";
import type { RepoInfo } from "../composables/useRepos";
import FileContent from "./FileContent.vue";

const props = defineProps<{
  repo: RepoInfo;
}>();

const emit = defineEmits<{
  (e: "close"): void;
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
    <div class="flex items-center gap-3 mb-4">
      <button
        @click="handleBack"
        class="px-3 py-1 border border-border rounded-md text-fg hover:border-fg-dim active:border-green transition-colors"
      >
        ← Back
      </button>
      <span class="font-bold text-fg truncate">
        {{
          selectedFileContent !== null
            ? currentRelativePath.split("/").pop()
            : !currentRelativePath
            ? repo.name
            : currentRelativePath.split("/").pop()
        }}
      </span>
    </div>

    <div v-if="loading" class="flex justify-center py-10">
      <svg class="animate-spin h-8 w-8 text-green" viewBox="0 0 24 24" fill="none">
        <circle
          class="opacity-25"
          cx="12"
          cy="12"
          r="10"
          stroke="currentColor"
          stroke-width="4"
        />
        <path
          class="opacity-75"
          fill="currentColor"
          d="M4 12a8 8 0 018-8v4a4 4 0 00-4 4H4z"
        />
      </svg>
    </div>

    <div v-else-if="selectedFileContent !== null">
      <FileContent :content="selectedFileContent" />
    </div>

    <div v-else>
      <div
        v-for="entry in files"
        :key="entry.name"
        @click="handleEntry(entry)"
        class="flex items-center gap-3 px-4 py-3 mb-2 bg-bg1 border border-border rounded-md cursor-pointer active:border-green transition-colors"
      >
        <span class="text-lg">{{ entry.is_dir ? "📁" : "📄" }}</span>
        <span class="truncate">{{ entry.name }}</span>
      </div>
    </div>
  </div>
</template>
