<script setup lang="ts">
import { ref, onMounted } from "vue";
import { 
  GitBranch, 
  History as HistoryIcon, 
  Plus, 
  Check, 
  X, 
  ArrowLeftRight,
  Loader2,
  FileCode,
  GitCommit
} from "@lucide/vue";
import { useGit } from "../composables/useGit";
import type { RepoInfo } from "../composables/useRepos";

const props = defineProps<{
  repo: RepoInfo;
}>();

const {
  branches,
  history,
  status,
  diff,
  loading,
  error,
  loadBranches,
  createBranch,
  switchBranch,
  loadHistory,
  loadStatus,
  stageFile,
  unstageFile,
  commitChanges,
  loadDiff,
} = useGit();

const activeTab = ref<"branches" | "history" | "changes">("branches");
const newBranchName = ref("");
const showCreateBranch = ref(false);
const commitMessage = ref("");
const selectedFileForDiff = ref<string | null>(null);

async function onCreateBranch() {
  if (!newBranchName.value) return;
  await createBranch(props.repo.id, newBranchName.value);
  newBranchName.value = "";
  showCreateBranch.value = false;
}

async function onSwitchBranch(name: string) {
  await switchBranch(props.repo.id, name);
}

async function onCommit() {
  if (!commitMessage.value) return;
  await commitChanges(props.repo.id, commitMessage.value, "Atlas User", "user@atlas.app");
  commitMessage.value = "";
}

async function viewDiff(path: string, staged: boolean) {
  selectedFileForDiff.value = path;
  await loadDiff(props.repo.id, path, staged);
}

onMounted(() => {
  loadBranches(props.repo.id);
  loadHistory(props.repo.id);
  loadStatus(props.repo.id);
});
</script>

<template>
  <div class="space-y-6">
    <!-- Tabs -->
    <div class="flex border-b border-border">
      <button
        v-for="tab in ['branches', 'history', 'changes'] as const"
        :key="tab"
        @click="activeTab = tab"
        class="px-4 py-2 text-sm font-medium transition-colors border-b-2 capitalize cursor-pointer"
        :class="activeTab === tab ? 'border-yellow text-yellow' : 'border-transparent text-fg-dim hover:text-fg'"
      >
        {{ tab }}
      </button>
    </div>

    <!-- Error Message -->
    <div v-if="error" class="p-3 bg-red/10 border border-red text-red rounded-lg text-sm flex items-center justify-between">
      <span>{{ error }}</span>
      <button @click="error = null"><X :size="16" /></button>
    </div>

    <!-- Branches Tab -->
    <div v-if="activeTab === 'branches'" class="space-y-4">
      <div class="flex items-center justify-between">
        <h3 class="text-lg font-bold">Branches</h3>
        <button
          @click="showCreateBranch = !showCreateBranch"
          class="p-2 border border-border rounded-lg text-fg-dim hover:text-yellow hover:border-yellow active:scale-95 transition-all cursor-pointer"
        >
          <Plus :size="20" />
        </button>
      </div>

      <div v-if="showCreateBranch" class="flex gap-2">
        <input
          v-model="newBranchName"
          placeholder="New branch name..."
          class="flex-1 px-3 py-2 bg-bg1 border border-border rounded-lg outline-none focus:border-yellow text-sm"
          @keyup.enter="onCreateBranch"
          autofocus
        />
        <button
          @click="onCreateBranch"
          class="px-4 py-2 bg-yellow text-bg0 rounded-lg text-sm font-bold active:scale-95 transition-all cursor-pointer"
        >
          Create
        </button>
      </div>

      <div class="space-y-2">
        <div
          v-for="branch in branches"
          :key="branch.name"
          class="flex items-center justify-between p-3 bg-bg1 border border-border rounded-lg"
          :class="{ 'border-yellow': branch.is_current }"
        >
          <div class="flex items-center gap-3">
            <GitBranch :size="18" :class="branch.is_current ? 'text-yellow' : 'text-fg-dim'" />
            <span :class="{ 'font-bold text-yellow': branch.is_current }" class="text-sm">{{ branch.name }}</span>
            <span v-if="branch.is_remote" class="text-[9px] px-1.5 py-0.5 bg-bg2 rounded text-fg-dim uppercase tracking-wider">Remote</span>
          </div>
          <button
            v-if="!branch.is_current"
            @click="onSwitchBranch(branch.name)"
            class="p-1.5 text-fg-dim hover:text-fg active:scale-95 transition-all cursor-pointer"
            title="Switch to branch"
          >
            <ArrowLeftRight :size="16" />
          </button>
          <Check v-else :size="16" class="text-yellow" />
        </div>
      </div>
    </div>

    <!-- History Tab -->
    <div v-if="activeTab === 'history'" class="space-y-4">
      <div class="space-y-3">
        <div
          v-for="commit in history"
          :key="commit.hash"
          class="p-4 bg-bg1 border border-border rounded-lg space-y-2"
        >
          <div class="flex items-start justify-between gap-4">
            <p class="font-bold text-sm leading-tight">{{ commit.message }}</p>
            <span class="text-[10px] font-mono text-fg-dim bg-bg2 px-1.5 py-0.5 rounded">{{ commit.hash.substring(0, 7) }}</span>
          </div>
          <div class="flex items-center justify-between text-[11px] text-fg-dim">
            <span>{{ commit.author }}</span>
            <span>{{ new Date(commit.date * 1000).toLocaleDateString() }}</span>
          </div>
        </div>
      </div>
    </div>

    <!-- Changes Tab -->
    <div v-if="activeTab === 'changes'" class="space-y-6">
      <!-- Commit Form -->
      <div class="space-y-3 p-4 bg-bg1 border border-border rounded-lg">
        <textarea
          v-model="commitMessage"
          placeholder="Commit message..."
          class="w-full h-20 p-3 bg-bg0 border border-border rounded-lg outline-none focus:border-green text-sm resize-none"
        ></textarea>
        <button
          @click="onCommit"
          :disabled="!commitMessage || status.filter(s => s.staged).length === 0"
          class="w-full py-2.5 bg-green text-bg0 rounded-lg text-sm font-bold active:scale-[0.98] transition-all disabled:opacity-30 disabled:scale-100 cursor-pointer"
        >
          Commit Changes ({{ status.filter(s => s.staged).length }})
        </button>
      </div>

      <!-- Status List -->
      <div class="space-y-4">
        <div v-if="status.length === 0" class="flex flex-col items-center justify-center py-10 text-fg-dim opacity-30">
          <Check :size="40" class="mb-2" />
          <p class="text-sm">No changes to commit</p>
        </div>

        <div v-else class="space-y-2">
          <div
            v-for="entry in status"
            :key="entry.path"
            class="flex items-center justify-between p-3 bg-bg1 border border-border rounded-lg"
            :class="{ 'border-green/50': entry.staged }"
          >
            <div class="flex items-center gap-3 min-w-0 cursor-pointer" @click="viewDiff(entry.path, entry.staged)">
              <div :class="entry.staged ? 'text-green' : 'text-orange'">
                <FileCode :size="18" />
              </div>
              <div class="flex flex-col min-w-0">
                <span class="text-sm font-medium truncate">{{ entry.path }}</span>
                <span class="text-[10px] uppercase opacity-60">{{ entry.status }}</span>
              </div>
            </div>
            
            <div class="flex items-center gap-1">
              <button
                @click="entry.staged ? unstageFile(props.repo.id, entry.path) : stageFile(props.repo.id, entry.path)"
                class="p-2 rounded-lg transition-colors cursor-pointer"
                :class="entry.staged ? 'text-orange hover:bg-orange/10' : 'text-green hover:bg-green/10'"
                :title="entry.staged ? 'Unstage' : 'Stage'"
              >
                <X v-if="entry.staged" :size="18" />
                <Plus v-else :size="18" />
              </button>
            </div>
          </div>
        </div>
      </div>

      <!-- Diff Viewer (Modal) -->
      <Transition name="fade">
        <div v-if="selectedFileForDiff" class="fixed inset-0 z-50 bg-bg0 p-6 flex flex-col">
          <div class="flex items-center justify-between mb-4">
            <h4 class="font-bold text-sm truncate pr-4">{{ selectedFileForDiff }}</h4>
            <button @click="selectedFileForDiff = null" class="p-2 border border-border rounded-lg cursor-pointer">
              <X :size="20" />
            </button>
          </div>
          <div class="flex-1 overflow-auto bg-bg1 rounded-lg border border-border">
            <pre class="p-4 text-[11px] font-mono whitespace-pre text-fg-dim">
              <template v-for="(line, i) in diff.split('\n')" :key="i">
                <div :class="{ 'bg-green/10 text-green': line.startsWith('+'), 'bg-red/10 text-red': line.startsWith('-') }">{{ line }}</div>
              </template>
            </pre>
          </div>
        </div>
      </Transition>
    </div>

    <!-- Loading Overlay -->
    <div v-if="loading && activeTab !== 'changes'" class="fixed inset-0 bg-bg0/50 backdrop-blur-sm flex items-center justify-center z-40">
      <Loader2 :size="32" class="animate-spin text-yellow" />
    </div>
  </div>
</template>

<style scoped>
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.2s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
</style>
