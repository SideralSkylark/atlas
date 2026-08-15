<script setup lang="ts">
import { ref, onMounted } from "vue";
import { 
  GitBranch, 
  Plus, 
  Check, 
  X, 
  ArrowLeftRight,
  Loader2,
  FileCode,
  GitCommit,
  ChevronRight,
  RotateCcw,
  Trash2,
  GitMerge,
  AlertTriangle
} from "@lucide/vue";
import { invoke } from "@tauri-apps/api/core";
import { useGit } from "../composables/useGit";
import { useRepos } from "../composables/useRepos";
import type { RepoInfo } from "../composables/useRepos";
import DiffView from "./DiffView.vue";

const props = defineProps<{
  repo: RepoInfo;
}>();

const emit = defineEmits<{
  (e: "reloadFiles"): void;
  (e: "editFile", filepath: string): void;
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
  mergeBranch,
  getConflicts,
  resolveConflict,
  loadHistory,
  loadStatus,
  stageFile,
  unstageFile,
  commitChanges,
  loadDiff,
} = useGit();
const { pushRepo } = useRepos();

const activeTab = ref<"branches" | "history" | "changes">("branches");
const newBranchName = ref("");
const showCreateBranch = ref(false);
const commitMessage = ref("");
const selectedFileForDiff = ref<string | null>(null);
const showAuthor = ref(false);

const revertingFile = ref<string | null>(null);
const confirmRevert = ref<string | null>(null);
const pushingBranch = ref<string | null>(null);

function triggerRevert(path: string) {
  confirmRevert.value = path;
  if (typeof window !== 'undefined' && 'vibrate' in window.navigator) {
    window.navigator.vibrate(30);
  }
}

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
  await reloadAll();
}

async function onSwitchBranch(name: string) {
  await switchBranch(props.repo.id, name);
  await reloadAll();
}

async function onPushBranch() {
  const currentBranch = branches.value.find((branch) => branch.is_current);
  if (!currentBranch) return;

  pushingBranch.value = currentBranch.name;
  const res = await pushRepo(props.repo.id);
  pushingBranch.value = null;

  if (res.success) {
    error.value = null;
  } else {
    error.value = res.message;
  }

  await reloadAll();
}

const branchToDelete = ref<{ name: string; is_remote: boolean } | null>(null);

async function onDeleteBranch(branch: { name: string; is_remote: boolean }) {
  const success = await deleteBranch(props.repo.id, branch.name, branch.is_remote);
  if (success) {
    branchToDelete.value = null;
  }
}

const conflictedFiles = ref<string[]>([]);
const branchToMerge = ref<{ name: string; is_remote: boolean } | null>(null);

async function checkConflicts() {
  conflictedFiles.value = await getConflicts(props.repo.id);
}

async function onMergeBranch(branchName: string) {
  const res = await mergeBranch(props.repo.id, branchName);
  branchToMerge.value = null;
  
  if (res) {
    if (res.success) {
      error.value = null;
      await reloadAll();
    } else {
      await reloadAll();
    }
  }
}

async function onResolveConflict(filepath: string, choice: string) {
  const success = await resolveConflict(props.repo.id, filepath, choice);
  if (success) {
    await reloadAll();
  }
}

function onEditConflictedFile(filepath: string) {
  emit('editFile', filepath);
}

async function reloadAll() {
  await Promise.all([
    loadBranches(props.repo.id),
    loadStatus(props.repo.id),
    loadHistory(props.repo.id),
    checkConflicts()
  ]);
}

defineExpose({
  reloadAll
});

// Swipe Gestures for Changed Files
const swipeFile = ref<string | null>(null);
const swipeDeltaX = ref(0);
let swipeStartX = 0;
let swipeStartY = 0;
let isHorizontalSwipe = false;

function handleFileTouchStart(e: TouchEvent, path: string) {
  swipeFile.value = path;
  swipeStartX = e.touches[0].clientX;
  swipeStartY = e.touches[0].clientY;
  swipeDeltaX.value = 0;
  isHorizontalSwipe = false;
}

function handleFileTouchMove(e: TouchEvent, path: string) {
  if (swipeFile.value !== path) return;
  
  const currentX = e.touches[0].clientX;
  const currentY = e.touches[0].clientY;
  const deltaX = currentX - swipeStartX;
  const deltaY = currentY - swipeStartY;

  if (!isHorizontalSwipe && Math.abs(deltaX) > 10 && Math.abs(deltaX) > Math.abs(deltaY)) {
    isHorizontalSwipe = true;
  }

  if (isHorizontalSwipe) {
    if (e.cancelable) e.preventDefault();
    
    if (deltaX > 0) {
      swipeDeltaX.value = Math.min(deltaX, 120);
    } else {
      const entry = status.value.find(s => s.path === path);
      if (entry && entry.status === 'New') {
        swipeDeltaX.value = Math.max(deltaX / 3, -30);
      } else {
        swipeDeltaX.value = Math.max(deltaX, -120);
      }
    }
  }
}

async function handleFileTouchEnd(entry: any) {
  if (swipeFile.value !== entry.path) return;

  const finalDelta = swipeDeltaX.value;
  swipeFile.value = null;
  swipeDeltaX.value = 0;

  if (isHorizontalSwipe) {
    if (finalDelta > 80) {
      if (entry.staged) {
        await unstageFile(props.repo.id, entry.path);
      } else {
        await stageFile(props.repo.id, entry.path);
      }
      if ('vibrate' in navigator) navigator.vibrate(20);
    } else if (finalDelta < -80 && entry.status !== 'New') {
      triggerRevert(entry.path);
    }
  }
}


async function onCommit() {
  if (!commitMessage.value) return;
  
  localStorage.setItem("atlas_author_name", authorName.value);
  localStorage.setItem("atlas_author_email", authorEmail.value);

  await commitChanges(props.repo.id, commitMessage.value, authorName.value, authorEmail.value);
  commitMessage.value = "";
  if ('vibrate' in navigator) navigator.vibrate(20);
  await reloadAll();
}

async function onCommitAndPush() {
  if (!commitMessage.value) return;

  localStorage.setItem("atlas_author_name", authorName.value);
  localStorage.setItem("atlas_author_email", authorEmail.value);

  await commitChanges(props.repo.id, commitMessage.value, authorName.value, authorEmail.value);
  commitMessage.value = "";

  const pushResult = await pushRepo(props.repo.id);
  if (!pushResult.success) {
    error.value = pushResult.message;
  }

  if ('vibrate' in navigator) navigator.vibrate(20);
  await reloadAll();
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
  reloadAll();
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
          <template v-else-if="branchToMerge?.name === branch.name">
            <div class="flex flex-col min-w-0 font-sans">
              <span class="text-xs text-green font-bold">Merge branch?</span>
              <span class="text-[10px] text-fg-dim truncate max-w-[200px] font-mono mt-0.5">{{ branch.name }}</span>
            </div>
            <div class="flex items-center gap-1 shrink-0">
              <button
                @click="onMergeBranch(branch.name)"
                class="px-3 py-2 text-green text-xs font-bold hover:bg-green/10 rounded-lg transition-colors cursor-pointer active:scale-95 duration-100 font-sans"
              >
                Yes
              </button>
              <button
                @click="branchToMerge = null"
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
                @click="branchToMerge = branch"
                class="min-w-[44px] min-h-[44px] flex items-center justify-center text-fg-dim hover:text-green active:scale-95 duration-100 transition-all cursor-pointer"
                title="Merge into current branch"
              >
                <GitMerge :size="16" />
              </button>
              <button
                @click="branchToDelete = branch"
                class="min-w-[44px] min-h-[44px] flex items-center justify-center text-fg-dim hover:text-red active:scale-95 duration-100 transition-all cursor-pointer"
                title="Delete branch"
              >
                <Trash2 :size="16" />
              </button>
            </div>
            <div v-else class="flex items-center gap-1 shrink-0">
              <button
                @click="onPushBranch"
                :disabled="pushingBranch !== null"
                class="min-w-[44px] min-h-[44px] flex items-center justify-center text-fg-dim hover:text-green active:scale-95 duration-100 transition-all cursor-pointer disabled:cursor-not-allowed disabled:opacity-60"
                title="Push current branch"
              >
                <Loader2 v-if="pushingBranch === branch.name" :size="16" class="animate-spin" />
                <RotateCcw v-else :size="16" />
              </button>
              <Check :size="16" class="text-yellow shrink-0" />
            </div>
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
      <!-- Conflict Resolution Section -->
      <div v-if="conflictedFiles.length > 0" class="p-4 bg-red/5 border border-red/20 rounded-xl space-y-3 font-sans animate-fade">
        <div class="flex items-center gap-2 text-red">
          <AlertTriangle :size="18" />
          <h4 class="text-sm font-bold uppercase tracking-wider">Unresolved Merge Conflicts</h4>
        </div>
        <p class="text-xs text-fg-dim leading-relaxed">
          The following files have conflicts. Select how you want to resolve each conflict, or edit the files manually.
        </p>

        <div class="space-y-2.5">
          <div 
            v-for="file in conflictedFiles" 
            :key="file"
            class="flex flex-col gap-2.5 p-3 bg-bg1 border border-border rounded-lg"
          >
            <!-- File Info -->
            <div class="flex items-center justify-between min-w-0">
              <span class="text-xs font-mono text-fg font-bold truncate pr-2">{{ file }}</span>
              <span class="text-[9px] font-bold uppercase tracking-wider text-red bg-red/10 px-1.5 py-0.5 rounded shrink-0">Conflicted</span>
            </div>

            <!-- Resolution Actions -->
            <div class="flex items-center gap-2 pt-1.5 border-t border-border/40">
              <button
                @click="onResolveConflict(file, 'ours')"
                class="flex-1 py-1.5 px-2 bg-bg2 hover:bg-bg3 text-[10px] font-bold rounded text-yellow active:scale-95 duration-100 transition-all border border-border cursor-pointer"
                title="Keep ours (current branch)"
              >
                Keep Ours
              </button>
              <button
                @click="onResolveConflict(file, 'theirs')"
                class="flex-1 py-1.5 px-2 bg-bg2 hover:bg-bg3 text-[10px] font-bold rounded text-aqua active:scale-95 duration-100 transition-all border border-border cursor-pointer"
                title="Keep theirs (incoming branch)"
              >
                Keep Theirs
              </button>
              <button
                @click="onEditConflictedFile(file)"
                class="flex-1 py-1.5 px-2 bg-bg2 hover:bg-bg3 text-[10px] font-bold rounded text-fg hover:text-green active:scale-95 duration-100 transition-all border border-border cursor-pointer"
                title="Edit manually"
              >
                Edit
              </button>
              <button
                @click="onResolveConflict(file, 'merged')"
                class="flex-1 py-1.5 px-2 bg-green text-bg0 text-[10px] font-bold rounded active:scale-95 duration-100 transition-all cursor-pointer"
                title="Mark manually resolved as done"
              >
                Resolved
              </button>
            </div>
          </div>
        </div>
      </div>
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
            :disabled="!commitMessage || status.filter(s => s.staged).length === 0 || conflictedFiles.length > 0"
            class="flex-1 py-2.5 bg-green text-bg0 rounded-lg text-sm font-bold active:scale-95 duration-100 transition-all disabled:opacity-30 disabled:scale-100 cursor-pointer shadow-md font-sans"
          >
            Commit ({{ status.filter(s => s.staged).length }})
          </button>
          <button
            @click="onCommitAndPush"
            :disabled="!commitMessage || status.filter(s => s.staged).length === 0 || conflictedFiles.length > 0"
            class="flex-1 py-2.5 bg-yellow text-bg0 rounded-lg text-sm font-bold active:scale-95 duration-100 transition-all disabled:opacity-30 disabled:scale-100 cursor-pointer shadow-md font-sans"
          >
            Commit & Push
          </button>
        </div>
        <p v-if="conflictedFiles.length > 0" class="text-[10px] text-red font-bold font-sans text-center mt-2 animate-fade">
          Resolve conflicts before committing.
        </p>
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
            class="relative overflow-hidden rounded-xl bg-bg0"
            style="touch-action: pan-y;"
          >
            <!-- Background Actions (Visible when swiping) -->
            <div 
              class="absolute inset-0 flex justify-between items-center px-4 rounded-xl pointer-events-none transition-colors"
              :class="{
                'bg-green/10': swipeFile === entry.path && swipeDeltaX > 0,
                'bg-red/10': swipeFile === entry.path && swipeDeltaX < 0 && entry.status !== 'New',
                'opacity-0': swipeFile !== entry.path
              }"
            >
              <!-- Left Swipe Action Indicator (Stage/Unstage) -->
              <div 
                class="flex items-center gap-1.5 text-green text-[10px] font-bold uppercase tracking-wider transition-opacity" 
                :class="{ 'opacity-0': swipeDeltaX <= 20 }"
              >
                <Plus v-if="!entry.staged" :size="14" />
                <X v-else :size="14" />
                <span>{{ entry.staged ? 'Unstage' : 'Stage' }}</span>
              </div>
              
              <!-- Right Swipe Action Indicator (Revert) -->
              <div 
                class="flex items-center gap-1.5 text-red text-[10px] font-bold uppercase tracking-wider transition-opacity" 
                :class="{ 'opacity-0': swipeDeltaX >= -20 || entry.status === 'New' }"
              >
                <span>Revert</span>
                <RotateCcw :size="14" />
              </div>
            </div>

            <!-- Foreground Card -->
            <div
              class="flex items-center justify-between p-3 bg-bg1 border border-border rounded-xl shadow-sm min-h-[64px] relative z-10 select-none"
              :class="{ 
                'border-green/30 bg-bg1/80': entry.staged,
                'transition-transform duration-200 ease-out': swipeFile !== entry.path
              }"
              :style="{ 
                transform: swipeFile === entry.path ? `translateX(${swipeDeltaX}px)` : 'none',
                boxShadow: 'var(--shadow-sm), var(--shadow-inset)'
              }"
              @touchstart="handleFileTouchStart($event, entry.path)"
              @touchmove="handleFileTouchMove($event, entry.path)"
              @touchend="handleFileTouchEnd(entry)"
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
                    @click="triggerRevert(entry.path)"
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
