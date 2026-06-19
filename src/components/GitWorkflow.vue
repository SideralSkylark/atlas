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
  GitCommit,
  ChevronRight,
  ChevronDown,
  RotateCcw,
  Trash2
} from "@lucide/vue";
import { invoke } from "@tauri-apps/api/core";
import { useGit } from "../composables/useGit";
import type { RepoInfo } from "../composables/useRepos";
import DiffView from "./DiffView.vue";

const props = defineProps<{
  repo: RepoInfo;
}>();

const emit = defineEmits<{
  (e: "reloadFiles"): void;
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
  deleteBranch,
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
const showAuthor = ref(false);

const revertingFile = ref<string | null>(null);
const confirmRevert = ref<string | null>(null);

const authorName = ref(localStorage.getItem("atlas_author_name") || "Atlas User");
const authorEmail = ref(localStorage.getItem("atlas_author_email") || "user@atlas.app");

async function onRevert(filepath: string) {
  revertingFile.value = filepath;
  try {
    await invoke('revert_file', { 
      repoId: props.repo.id, 
      filepath 
    });
    await loadStatus(props.repo.id);
    emit('reloadFiles');
  } catch (e) {
    error.value = String(e);
  } finally {
    revertingFile.value = null;
    confirmRevert.value = null;
  }
}

async function onCreateBranch() {
  if (!newBranchName.value) return;
  await createBranch(props.repo.id, newBranchName.value);
  newBranchName.value = "";
  showCreateBranch.value = false;
  await loadBranches(props.repo.id);
}

async function onSwitchBranch(name: string) {
  await switchBranch(props.repo.id, name);
  await Promise.all([
    loadBranches(props.repo.id),
    loadStatus(props.repo.id),
    loadHistory(props.repo.id)
  ]);
}

const branchToDelete = ref<{ name: string; is_remote: boolean } | null>(null);

async function onDeleteBranch(branch: { name: string; is_remote: boolean }) {
  const success = await deleteBranch(props.repo.id, branch.name, branch.is_remote);
  if (success) {
    branchToDelete.value = null;
  }
}

async function onCommit() {
  if (!commitMessage.value) return;
  
  localStorage.setItem("atlas_author_name", authorName.value);
  localStorage.setItem("atlas_author_email", authorEmail.value);

  await commitChanges(props.repo.id, commitMessage.value, authorName.value, authorEmail.value);
  commitMessage.value = "";
  if ('vibrate' in navigator) navigator.vibrate(20);
  await Promise.all([
    loadStatus(props.repo.id),
    loadHistory(props.repo.id)
  ]);
}

async function onStageAll() {
  const unstaged = status.value.filter(s => !s.staged);
  for (const entry of unstaged) {
    await stageFile(props.repo.id, entry.path);
  }
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
    <div class="flex border-b border-border font-sans">
      <button
        v-for="tab in ['branches', 'history', 'changes'] as const"
        :key="tab"
        @click="activeTab = tab"
        class="px-4 py-2 text-sm font-medium transition-colors border-b-2 capitalize cursor-pointer font-sans"
        :class="activeTab === tab ? 'border-yellow text-yellow' : 'border-transparent text-fg-dim hover:text-fg'"
      >
        {{ tab }}
      </button>
    </div>

    <!-- Error Message -->
    <div v-if="error" class="p-3 bg-red/10 border border-red text-red rounded-lg text-sm flex items-center justify-between font-sans">
      <span>{{ error }}</span>
      <button @click="error = null"><X :size="16" /></button>
    </div>

    <!-- Branches Tab -->
    <div v-if="activeTab === 'branches'" class="space-y-4 font-sans">
      <div class="flex items-center justify-between">
        <h3 class="text-lg font-bold font-sans">Branches</h3>
        <button
          @click="showCreateBranch = !showCreateBranch"
          class="min-w-[44px] min-h-[44px] flex items-center justify-center border border-border rounded-lg text-fg-dim hover:text-yellow hover:border-yellow active:scale-95 duration-100 transition-all cursor-pointer"
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
          class="flex items-center justify-between p-3 bg-bg1 border border-border rounded-lg shadow-sm min-h-[64px]"
          :class="{ 'border-yellow': branch.is_current }"
          style="box-shadow: var(--shadow-sm), var(--shadow-inset)"
        >
          <template v-if="branchToDelete?.name === branch.name">
            <div class="flex flex-col min-w-0 font-sans">
              <span class="text-xs text-red font-bold">Delete {{ branch.is_remote ? 'remote' : 'local' }} branch?</span>
              <span class="text-[10px] text-fg-dim truncate max-w-[200px] font-mono mt-0.5">{{ branch.name }}</span>
            </div>
            <div class="flex items-center gap-1 shrink-0">
              <button
                @click="onDeleteBranch(branch)"
                class="px-3 py-2 text-red text-xs font-bold hover:bg-red/10 rounded-lg transition-colors cursor-pointer active:scale-95 duration-100 font-sans"
              >
                Yes
              </button>
              <button
                @click="branchToDelete = null"
                class="px-3 py-2 text-fg-dim text-xs hover:bg-bg3 rounded-lg transition-colors cursor-pointer active:scale-95 duration-100 font-sans"
              >
                Cancel
              </button>
            </div>
          </template>
          <template v-else>
            <div class="flex items-center gap-3 min-w-0">
              <GitBranch :size="18" :class="branch.is_current ? 'text-yellow' : 'text-fg-dim'" />
              <span :class="{ 'font-bold text-yellow': branch.is_current }" class="text-sm font-mono truncate">{{ branch.name }}</span>
              <span v-if="branch.is_remote" class="text-[9px] px-1.5 py-0.5 bg-bg2 rounded text-fg-dim uppercase tracking-wider font-sans shrink-0">Remote</span>
            </div>
            <div v-if="!branch.is_current" class="flex items-center gap-1 shrink-0">
              <button
                @click="onSwitchBranch(branch.name)"
                class="min-w-[44px] min-h-[44px] flex items-center justify-center text-fg-dim hover:text-fg active:scale-95 duration-100 transition-all cursor-pointer"
                title="Switch to branch"
              >
                <ArrowLeftRight :size="16" />
              </button>
              <button
                @click="branchToDelete = branch"
                class="min-w-[44px] min-h-[44px] flex items-center justify-center text-fg-dim hover:text-red active:scale-95 duration-100 transition-all cursor-pointer"
                title="Delete branch"
              >
                <Trash2 :size="16" />
              </button>
            </div>
            <Check v-else :size="16" class="text-yellow shrink-0" />
          </template>
        </div>
      </div>
    </div>

    <!-- History Tab -->
    <div v-if="activeTab === 'history'" class="space-y-4 font-sans">
      <div v-if="history.length === 0" class="flex flex-col items-center justify-center py-20 text-fg-dim opacity-30 font-sans">
        <GitCommit :size="40" class="mb-3" />
        <p class="text-sm font-medium">No commits yet</p>
      </div>
      <div v-else class="space-y-3">
        <div
          v-for="commit in history"
          :key="commit.hash"
          class="p-4 bg-bg1 border border-border rounded-lg space-y-2 shadow-sm"
          style="box-shadow: var(--shadow-sm), var(--shadow-inset)"
        >
          <div class="flex items-start justify-between gap-4">
            <p class="font-bold text-sm leading-tight font-sans">{{ commit.message }}</p>
            <span class="text-[10px] font-mono text-fg-dim bg-bg2 px-1.5 py-0.5 rounded font-mono">{{ commit.hash.substring(0, 7) }}</span>
          </div>
          <div class="flex items-center justify-between text-[11px] text-fg-dim font-sans">
            <span>{{ commit.author }}</span>
            <span>{{ new Date(commit.date * 1000).toLocaleDateString() }}</span>
          </div>
        </div>
      </div>
    </div>

    <!-- Changes Tab -->
    <div v-if="activeTab === 'changes'" class="space-y-6 font-sans">
      <!-- Commit Form -->
      <div class="space-y-3 p-4 bg-bg1 border border-border rounded-lg shadow-md" style="box-shadow: var(--shadow-md), var(--shadow-inset)">
        <div class="space-y-2">
          <button 
            @click="showAuthor = !showAuthor"
            class="min-h-[44px] flex items-center gap-1.5 text-[10px] font-bold uppercase tracking-widest text-fg-dim hover:text-fg transition-colors cursor-pointer font-sans"
          >
            <ChevronRight :size="12" class="transition-transform" :class="{ 'rotate-90': showAuthor }" />
            Author Info
          </button>
          
          <Transition name="slide">
            <div v-if="showAuthor" class="grid grid-cols-2 gap-2 pb-2">
              <input 
                v-model="authorName" 
                placeholder="Name" 
                class="px-2.5 py-1.5 bg-bg0 border border-border rounded text-[11px] outline-none focus:border-yellow font-sans"
              />
              <input 
                v-model="authorEmail" 
                placeholder="Email" 
                class="px-2.5 py-1.5 bg-bg0 border border-border rounded text-[11px] outline-none focus:border-yellow font-sans"
              />
            </div>
          </Transition>
        </div>

        <textarea
          v-model="commitMessage"
          placeholder="Commit message..."
          class="w-full h-20 p-3 bg-bg0 border border-border rounded-lg outline-none focus:border-green text-sm resize-none shadow-inner font-sans"
        ></textarea>
        
        <div class="flex gap-2">
          <button
            v-if="status.filter(s => !s.staged).length > 0"
            @click="onStageAll"
            class="px-4 py-2.5 bg-bg3 text-fg border border-border rounded-lg text-xs font-bold active:scale-95 duration-100 transition-all cursor-pointer whitespace-nowrap font-sans"
          >
            Stage All ({{ status.filter(s => !s.staged).length }})
          </button>
          <button
            @click="onCommit"
            :disabled="!commitMessage || status.filter(s => s.staged).length === 0"
            class="flex-1 py-2.5 bg-green text-bg0 rounded-lg text-sm font-bold active:scale-95 duration-100 transition-all disabled:opacity-30 disabled:scale-100 cursor-pointer shadow-md font-sans"
          >
            Commit ({{ status.filter(s => s.staged).length }})
          </button>
        </div>
      </div>

      <!-- Status List -->
      <div class="space-y-4">
        <div v-if="status.length === 0" class="flex flex-col items-center justify-center py-10 text-fg-dim opacity-30 font-sans">
          <Check :size="40" class="mb-2" />
          <p class="text-sm font-medium">No changes to commit</p>
        </div>

        <div v-else class="space-y-2">
          <div
            v-for="entry in status"
            :key="entry.path"
            class="flex items-center justify-between p-3 bg-bg1 border border-border rounded-xl shadow-sm min-h-[64px]"
            :class="{ 'border-green/30 bg-bg1/80': entry.staged }"
            style="box-shadow: var(--shadow-sm), var(--shadow-inset)"
          >
            <div v-if="revertingFile === entry.path" class="flex items-center justify-center w-full">
              <Loader2 :size="20" class="animate-spin text-red" />
            </div>
            <template v-else-if="confirmRevert === entry.path">
              <span class="text-xs text-red font-bold font-sans">Revert to last commit?</span>
              <div class="flex items-center gap-1">
                <button
                  @click="onRevert(entry.path)"
                  class="px-3 py-2 text-red text-xs font-bold hover:bg-red/10 rounded-lg transition-colors font-sans"
                >
                  Yes
                </button>
                <button
                  @click="confirmRevert = null"
                  class="px-3 py-2 text-fg-dim text-xs hover:bg-bg3 rounded-lg transition-colors font-sans"
                >
                  Cancel
                </button>
              </div>
            </template>
            <template v-else>
              <div class="flex items-center gap-3 min-w-0 cursor-pointer" @click="viewDiff(entry.path, entry.staged)">
                <div :class="entry.staged ? 'text-green' : 'text-fg-dim'">
                  <FileCode :size="18" />
                </div>
                <div class="flex flex-col min-w-0">
                  <span class="text-sm font-bold truncate text-fg font-mono">{{ entry.path }}</span>
                  <span class="text-[9px] font-bold uppercase tracking-wider transition-colors font-sans"
                        :class="{
                          'text-yellow': entry.status === 'Modified',
                          'text-green': entry.status === 'New',
                          'text-red': entry.status === 'Deleted',
                          'text-aqua': entry.status === 'Renamed',
                          'opacity-60': !['Modified', 'New', 'Deleted', 'Renamed'].includes(entry.status)
                        }">{{ entry.status }}</span>
                </div>
              </div>
              
              <div class="flex items-center gap-1">
                <button
                  @click="entry.staged ? unstageFile(props.repo.id, entry.path) : stageFile(props.repo.id, entry.path)"
                  class="min-w-[44px] min-h-[44px] flex items-center justify-center rounded-lg transition-colors cursor-pointer active:scale-95 duration-100"
                  :class="entry.staged ? 'text-fg-dim hover:bg-bg3' : 'text-green hover:bg-green/10'"
                  :title="entry.staged ? 'Unstage' : 'Stage'"
                >
                  <X v-if="entry.staged" :size="18" />
                  <Plus v-else :size="18" />
                </button>
                <button
                  v-if="entry.status !== 'New'"
                  @click="() => { confirmRevert = entry.path; if ('vibrate' in navigator) navigator.vibrate(30); }"
                  class="min-w-[44px] min-h-[44px] flex items-center justify-center rounded-lg text-fg-dim hover:text-red hover:bg-red/5 transition-colors cursor-pointer active:scale-95 duration-100"
                  title="Revert"
                >
                  <RotateCcw :size="16" />
                </button>
              </div>
            </template>
          </div>
        </div>
      </div>

      <!-- Diff Bottom Sheet -->
      <Teleport to="body">
        <Transition name="fade">
          <div v-if="selectedFileForDiff" 
               class="fixed inset-0 bg-bg0/80 backdrop-blur-sm z-40"
               @click="selectedFileForDiff = null"></div>
        </Transition>
        <Transition name="slide-up">
          <div v-if="selectedFileForDiff" 
               class="fixed bottom-0 left-0 right-0 h-[85vh] bg-bg1 border-t border-border rounded-t-3xl flex flex-col z-50 shadow-lg"
               style="box-shadow: var(--shadow-lg)">
            <!-- Parent has no overflow-x-hidden to allow DiffView scrolling -->
            <div class="w-12 h-1.5 bg-bg3 rounded-full mx-auto my-4 shrink-0" @click="selectedFileForDiff = null"></div>
            
            <div class="px-6 pb-4 flex items-center justify-between gap-4 border-b border-border/50">
              <div class="flex flex-col min-w-0">
                <h4 class="font-bold text-fg truncate text-sm font-mono">{{ selectedFileForDiff }}</h4>
                <span class="text-[10px] text-fg-dim uppercase tracking-widest font-bold font-sans">File Difference</span>
              </div>
              <button @click="selectedFileForDiff = null" class="min-w-[44px] min-h-[44px] flex items-center justify-center text-fg-dim hover:text-fg hover:bg-bg3 rounded-full transition-all active:scale-95 duration-100">
                <X :size="24" />
              </button>
            </div>
            
            <div class="flex-1 overflow-hidden flex flex-col">
              <DiffView :diff="diff" maxHeight="100%" />
            </div>
          </div>
        </Transition>
      </Teleport>
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
  transition: opacity 0.3s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}

.slide-up-enter-active,
.slide-up-leave-active {
  transition: transform 0.4s cubic-bezier(0.16, 1, 0.3, 1);
}

.slide-up-enter-from,
.slide-up-leave-to {
  transform: translateY(100%);
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
