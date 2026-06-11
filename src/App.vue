<script setup lang="ts">
import { invoke } from "@tauri-apps/api/core";
import { ref, onMounted } from "vue";

interface RepoInfo {
  id: string;
  name: string;
  branch: string;
}

interface FileEntry {
  name: string;
  is_dir: boolean;
}

const repos = ref<RepoInfo[]>([]);
const url = ref("");
const cloning = ref(false);
const message = ref("");
const messageOk = ref(true);

const selectedRepo = ref<RepoInfo | null>(null);
const files = ref<FileEntry[]>([]);

const currentRelativePath = ref("");
const deletingRepo = ref<string | null>(null);

async function loadRepos() {
  repos.value = await invoke<RepoInfo[]>("list_repos");
}

async function cloneRepo() {
  if (!url.value || cloning.value) return;

  cloning.value = true;
  message.value = "Cloning...";
  messageOk.value = true;

  try {
    const res = await invoke<{ success: boolean; message: string }>(
      "clone_repo",
      {
        url: url.value,
      },
    );

    messageOk.value = res.success;
    message.value = res.message;

    if (res.success) {
      url.value = "";
      await loadRepos();
    }
  } finally {
    cloning.value = false;
  }
}

async function deleteRepo(repo: RepoInfo) {
  deletingRepo.value = repo.id;

  try {
    const res = await invoke<{ success: boolean; message: string }>(
      "delete_repo",
      {
        repoId: repo.id,
      },
    );

    messageOk.value = res.success;
    message.value = res.message;

    if (res.success) {
      await loadRepos();
    }
  } finally {
    deletingRepo.value = null;
  }
}

async function loadFiles() {
  if (!selectedRepo.value) return;

  files.value = await invoke<FileEntry[]>("list_files", {
    repoId: selectedRepo.value.id,
    relativePath: currentRelativePath.value,
  });
}

async function openRepo(repo: RepoInfo) {
  selectedRepo.value = repo;
  currentRelativePath.value = "";
  await loadFiles();
}

async function openEntry(entry: FileEntry) {
  if (!entry.is_dir) return;

  currentRelativePath.value = currentRelativePath.value
    ? `${currentRelativePath.value}/${entry.name}`
    : entry.name;

  await loadFiles();
}

async function goBack() {
  if (!currentRelativePath.value) {
    selectedRepo.value = null;
    return;
  }

  const parts = currentRelativePath.value.split("/");
  parts.pop();

  currentRelativePath.value = parts.join("/");

  await loadFiles();
}

onMounted(loadRepos);
</script>

<template>
  <div class="min-h-screen bg-[#272e33] text-[#d3c6aa] font-mono p-6">
    <!-- File browser -->
    <div v-if="selectedRepo">
      <div class="flex items-center gap-3 mb-4">
        <button
          @click="goBack"
          class="px-3 py-1 border border-[#4a555b] rounded-md text-[#d3c6aa] hover:border-[#859289] active:border-[#a7c080] transition-colors"
        >
          ← Back
        </button>
        <span class="font-bold text-[#d3c6aa]">
          {{
            !currentRelativePath
              ? selectedRepo.name
              : currentRelativePath.split("/").pop()
          }}
        </span>
      </div>

      <div
        v-for="entry in files"
        :key="entry.name"
        @click="openEntry(entry)"
        class="flex items-center gap-3 px-4 py-3 mb-2 bg-[#2e383c] border border-[#4a555b] rounded-md cursor-pointer active:border-[#a7c080] transition-colors"
      >
        <span>{{ entry.is_dir ? "📁" : "📄" }}</span>
        <span>{{ entry.name }}</span>
      </div>
    </div>

    <!-- Repo list -->
    <div v-else>
      <h1 class="text-xl font-bold text-[#dbbc7f] mb-6">Atlas</h1>

      <!-- Clone section -->
      <div class="mb-8">
        <input
          v-model="url"
          placeholder="https://github.com/user/repo.git"
          class="w-full px-3 py-2 mb-2 bg-[#3c4841] text-[#d3c6aa] border border-[#4a555b] rounded-md placeholder-[#859289] outline-none focus:border-[#a7c080] transition-colors font-mono text-sm"
        />

        <button
          @click="cloneRepo"
          :disabled="!url || cloning"
          class="w-full py-2 font-semibold rounded-md transition-all"
          :class="
            url && !cloning
              ? 'bg-[#a7c080] text-[#272e33] active:brightness-90 cursor-pointer'
              : 'bg-[#3c4841] text-[#859289] cursor-not-allowed'
          "
        >
          <span v-if="cloning" class="flex items-center justify-center gap-2">
            <svg class="animate-spin h-4 w-4" viewBox="0 0 24 24" fill="none">
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
            Cloning...
          </span>
          <span v-else>Clone</span>
        </button>

        <p
          v-if="message && !selectedRepo"
          class="text-sm mt-2"
          :class="messageOk ? 'text-[#a7c080]' : 'text-[#e67e80]'"
        >
          {{ message }}
        </p>
      </div>

      <!-- Repo list -->
      <p v-if="repos.length === 0" class="text-[#859289]">No repos yet.</p>

      <div
        v-for="repo in repos"
        :key="repo.id"
        class="flex justify-between items-center px-4 py-3 mb-3 bg-[#2e383c] border border-[#4a555b] rounded-md transition-colors"
        :class="
          deletingRepo !== repo.id
            ? 'cursor-pointer active:border-[#a7c080]'
            : 'opacity-50'
        "
        @click="deletingRepo !== repo.id && openRepo(repo)"
      >
        <div>
          <div class="font-bold text-[#d3c6aa]">{{ repo.name }}</div>
          <div class="text-xs text-[#83c092] mt-0.5">{{ repo.branch }}</div>
        </div>

        <!-- Delete button -->
        <button
          @click.stop="deleteRepo(repo)"
          :disabled="deletingRepo === repo.id"
          class="ml-4 flex items-center gap-1.5 px-3 py-1.5 rounded-md border text-xs font-medium transition-all"
          :class="
            deletingRepo === repo.id
              ? 'border-[#4a555b] text-[#859289] cursor-not-allowed'
              : 'border-[#e67e80]/40 text-[#e67e80] hover:bg-[#e67e80]/10 hover:border-[#e67e80] active:bg-[#e67e80]/20 cursor-pointer'
          "
        >
          <svg
            v-if="deletingRepo !== repo.id"
            xmlns="http://www.w3.org/2000/svg"
            class="h-3.5 w-3.5"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
          >
            <polyline points="3 6 5 6 21 6" />
            <path d="M19 6l-1 14a2 2 0 01-2 2H8a2 2 0 01-2-2L5 6" />
            <path d="M10 11v6M14 11v6" />
            <path d="M9 6V4a1 1 0 011-1h4a1 1 0 011 1v2" />
          </svg>
          <svg
            v-else
            class="animate-spin h-3.5 w-3.5"
            viewBox="0 0 24 24"
            fill="none"
          >
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
          <span>{{ deletingRepo === repo.id ? "Deleting..." : "Delete" }}</span>
        </button>
      </div>
    </div>
  </div>
</template>
