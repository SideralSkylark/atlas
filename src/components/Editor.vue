<script setup lang="ts">
import { ref, onMounted, computed } from "vue";
import { 
  Save, 
  X, 
  Check, 
  GitCommit, 
  Upload, 
  Loader2,
  AlertCircle
} from "@lucide/vue";
import { useFileSystem } from "../composables/useFileSystem";
import { useGit } from "../composables/useGit";
import { useRepos } from "../composables/useRepos";
import type { RepoInfo } from "../composables/useRepos";

const props = defineProps<{
  repo: RepoInfo;
  relativePath: string;
}>();

const emit = defineEmits<{
  (e: "close"): void;
  (e: "notify", msg: { type: "success" | "error"; text: string }): void;
}>();

const { readRawFile, saveFile, loading: fsLoading } = useFileSystem();
const { commitChanges, stageFile, loading: gitLoading } = useGit();
const { pushRepo, syncing: pushLoading } = useRepos();

const content = ref("");
const originalContent = ref("");
const commitMessage = ref("");
const showCommitDialog = ref(false);
const shouldPushAfterCommit = ref(false);
const error = ref<string | null>(null);

const stats = computed(() => {
  const lines = content.value ? content.value.split("\n").length : 0;
  const words = content.value ? content.value.trim().split(/\s+/).filter(w => w.length > 0).length : 0;
  return `${lines} lines · ${words} words`;
});

async function loadContent() {
  const data = await readRawFile(props.repo.id, props.relativePath);
  if (data !== null) {
    content.value = data;
    originalContent.value = data;
  }
}

async function onSave() {
  const success = await saveFile(props.repo.id, props.relativePath, content.value);
  if (success) {
    originalContent.value = content.value;
    emit("notify", { type: "success", text: "File saved." });
  } else {
    emit("notify", { type: "error", text: "Failed to save file." });
  }
}

async function onCommit() {
  if (!commitMessage.value) return;
  
  // 1. Save file first
  const saved = await saveFile(props.repo.id, props.relativePath, content.value);
  if (!saved) return;

  // 2. Stage and commit
  await stageFile(props.repo.id, props.relativePath);
  
  const name = localStorage.getItem("atlas_author_name") || "Atlas User";
  const email = localStorage.getItem("atlas_author_email") || "user@atlas.app";

  try {
    await commitChanges(props.repo.id, commitMessage.value, name, email);
    emit("notify", { type: "success", text: "Changes committed." });
    showCommitDialog.value = false;
    commitMessage.value = "";
    originalContent.value = content.value;

    if (shouldPushAfterCommit.value) {
      const res = await pushRepo(props.repo.id);
      emit("notify", { type: res.success ? "success" : "error", text: res.message });
    }
  } catch (e) {
    emit("notify", { type: "error", text: String(e) });
  }
}

const hasChanges = computed(() => content.value !== originalContent.value);

onMounted(loadContent);
</script>

<template>
  <div class="fixed inset-0 z-50 bg-bg0 flex flex-col p-4 md:p-6">
    <!-- Header -->
    <div class="flex items-center justify-between mb-4 gap-4">
      <div class="flex items-center gap-3 min-w-0">
        <button
          @click="emit('close')"
          class="min-w-[44px] min-h-[44px] flex items-center justify-center border border-border rounded-lg text-fg-dim hover:text-fg active:scale-95 duration-100 transition-all cursor-pointer"
        >
          <X :size="20" />
        </button>
        <div class="min-w-0">
          <h2 class="text-sm font-bold truncate font-sans font-medium">{{ relativePath.split('/').pop() }}</h2>
          <p class="text-[10px] text-fg-dim truncate font-mono">{{ relativePath }}</p>
        </div>
      </div>

      <div class="flex items-center gap-2 shrink-0">
        <button
          @click="onSave"
          :disabled="!hasChanges || fsLoading"
          class="min-w-[44px] min-h-[44px] flex items-center justify-center border border-border rounded-lg text-fg-dim hover:text-green hover:border-green active:scale-95 duration-100 transition-all disabled:opacity-30 cursor-pointer"
          title="Save"
        >
          <Loader2 v-if="fsLoading" :size="20" class="animate-spin" />
          <Save v-else :size="20" />
        </button>
        <button
          @click="showCommitDialog = true"
          :disabled="fsLoading || gitLoading"
          class="min-w-[44px] min-h-[44px] flex items-center justify-center border border-border rounded-lg text-fg-dim hover:text-yellow hover:border-yellow active:scale-95 duration-100 transition-all disabled:opacity-30 cursor-pointer"
          title="Commit Changes"
        >
          <GitCommit :size="20" />
        </button>
      </div>
    </div>

    <!-- Editor -->
    <div class="flex-1 relative bg-bg1 border border-border rounded-xl overflow-hidden">
      <textarea
        v-model="content"
        class="w-full h-full p-4 bg-transparent outline-none text-sm font-mono resize-none leading-relaxed"
        spellcheck="false"
      ></textarea>
    </div>

    <!-- Commit Dialog -->
    <Transition name="fade">
      <div v-if="showCommitDialog" class="fixed inset-0 z-[60] bg-bg0/80 backdrop-blur-sm flex items-center justify-center p-6">
        <div class="w-full max-w-sm bg-bg1 border border-border rounded-2xl p-6 shadow-lg space-y-4" style="box-shadow: var(--shadow-lg)">
          <div class="flex items-center justify-between">
            <h3 class="text-lg font-bold font-sans">Commit Changes</h3>
            <button @click="showCommitDialog = false" class="min-w-[44px] min-h-[44px] flex items-center justify-center text-fg-dim hover:text-fg font-sans active:scale-95 duration-100 transition-all"><X :size="20" /></button>
          </div>
          
          <div class="space-y-3">
            <textarea
              v-model="commitMessage"
              placeholder="Commit message..."
              class="w-full h-24 p-3 bg-bg0 border border-border rounded-lg outline-none focus:border-yellow text-sm resize-none font-sans"
              autofocus
            ></textarea>
            
            <label class="flex items-center gap-2 cursor-pointer group font-sans">
              <div class="relative flex items-center">
                <input type="checkbox" v-model="shouldPushAfterCommit" class="peer hidden" />
                <div class="w-5 h-5 border-2 border-border rounded peer-checked:bg-yellow peer-checked:border-yellow transition-all"></div>
                <Check class="absolute text-bg0 scale-0 peer-checked:scale-100 transition-all" :size="16" />
              </div>
              <span class="text-sm text-fg-dim group-hover:text-fg transition-colors font-sans">Push after commit</span>
            </label>
          </div>

          <button
            @click="onCommit"
            :disabled="!commitMessage || gitLoading"
            class="w-full py-4 bg-yellow text-bg0 rounded-xl font-bold flex items-center justify-center gap-2 active:scale-95 duration-100 transition-all disabled:opacity-30 font-sans"
          >
            <Loader2 v-if="gitLoading || pushLoading === repo.id" :size="20" class="animate-spin" />
            <template v-else>
              <GitCommit :size="20" />
              <span class="font-sans">Commit & {{ shouldPushAfterCommit ? 'Push' : 'Save' }}</span>
            </template>
          </button>
        </div>
      </div>
    </Transition>

    <!-- Unsaved Changes Warning -->
    <div class="mt-4 flex items-center justify-between font-sans">
      <div v-if="hasChanges" class="flex items-center gap-2 text-[10px] text-orange animate-pulse font-bold uppercase tracking-wider font-sans">
        <AlertCircle :size="12" />
        <span class="font-sans">Unsaved changes</span>
      </div>
      <div v-else></div>
      <div class="text-[10px] text-fg-dim font-mono bg-bg1 px-2 py-0.5 rounded border border-border/50 font-mono">
        {{ stats }}
      </div>
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
