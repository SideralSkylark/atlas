<script setup lang="ts">
import { ref, onMounted, watch } from "vue";
import { useRepos } from "../composables/useRepos";
import { Key, Save, ShieldAlert, Trash2, User } from "@lucide/vue";

const { pats, loadPats, savePat, deletePat } = useRepos();

const domain = ref("github.com");
const token = ref("");

const authorName = ref("");
const authorEmail = ref("");

async function onSaveToken() {
  if (domain.value && token.value) {
    await savePat(domain.value, token.value);
    token.value = "";
  }
}

function saveAuthorSettings() {
  localStorage.setItem('atlas_author_name', authorName.value);
  localStorage.setItem('atlas_author_email', authorEmail.value);
}

// Watch for changes and save immediately for better UX
watch([authorName, authorEmail], () => {
  saveAuthorSettings();
});

onMounted(() => {
  loadPats();
  authorName.value = localStorage.getItem('atlas_author_name') || "";
  authorEmail.value = localStorage.getItem('atlas_author_email') || "";
});
</script>

<template>
  <div class="space-y-6">
    <!-- Author Identity Card -->
    <div class="p-5 bg-bg1 border border-border rounded-xl shadow-sm overflow-hidden">
      <div class="flex items-center gap-2 mb-5 font-sans">
        <User :size="18" class="text-yellow" />
        <h2 class="text-yellow font-bold text-lg font-sans">Git Identity</h2>
      </div>

      <div class="space-y-4">
        <div class="space-y-1.5">
          <label class="text-[10px] uppercase tracking-wider font-bold text-fg-dim ml-1 font-sans">Author Name</label>
          <input
            v-model="authorName"
            placeholder="e.g. John Doe"
            class="w-full px-4 py-2.5 bg-bg3 text-fg border border-border rounded-lg outline-none focus:border-yellow transition-all text-sm font-sans"
          />
        </div>
        <div class="space-y-1.5">
          <label class="text-[10px] uppercase tracking-wider font-bold text-fg-dim ml-1 font-sans">Author Email</label>
          <input
            v-model="authorEmail"
            type="email"
            placeholder="e.g. john@example.com"
            class="w-full px-4 py-2.5 bg-bg3 text-fg border border-border rounded-lg outline-none focus:border-yellow transition-all text-sm font-sans"
          />
        </div>
        <p class="text-[10px] text-fg-dim italic px-1 leading-relaxed font-sans">
          This identity will be used for all commits you make in Atlas.
        </p>
      </div>
    </div>

    <!-- Git Credentials Card -->
    <div class="p-5 bg-bg1 border border-border rounded-xl shadow-sm overflow-hidden">
      <div class="flex items-center gap-2 mb-5 font-sans">
        <Key :size="18" class="text-yellow" />
        <h2 class="text-yellow font-bold text-lg font-sans">Git Credentials</h2>
      </div>
      
      <div class="space-y-3 mb-6">
        <div class="space-y-1.5">
          <label class="text-[10px] uppercase tracking-wider font-bold text-fg-dim ml-1 font-sans">Domain</label>
          <input
            v-model="domain"
            placeholder="e.g. github.com"
            class="w-full px-4 py-2.5 bg-bg3 text-fg border border-border rounded-lg outline-none focus:border-green transition-all text-sm font-mono"
          />
        </div>
        <div class="space-y-1.5">
          <label class="text-[10px] uppercase tracking-wider font-bold text-fg-dim ml-1 font-sans">Personal Access Token</label>
          <input
            v-model="token"
            type="password"
            placeholder="ghp_xxxxxxxxxxxx"
            class="w-full px-4 py-2.5 bg-bg3 text-fg border border-border rounded-lg outline-none focus:border-green transition-all text-sm font-mono"
          />
        </div>
        <button
          @click="onSaveToken"
          class="w-full py-4 mt-2 bg-green text-bg0 font-bold rounded-lg active:scale-95 duration-100 hover:brightness-110 transition-all flex items-center justify-center gap-2 cursor-pointer font-sans"
        >
          <Save :size="18" />
          <span class="font-sans">Save Token</span>
        </button>
      </div>

      <div v-if="Object.keys(pats).length > 0" class="space-y-2 pt-4 border-t border-border/50">
        <div
          v-for="(_, d) in pats"
          :key="d"
          class="flex justify-between items-center px-4 py-3 bg-bg0/50 rounded-lg border border-border/30 shadow-sm"
          style="box-shadow: var(--shadow-sm), var(--shadow-inset)"
        >
          <div class="flex items-center gap-2" title="Token saved securely">
            <ShieldAlert :size="14" class="text-aqua" />
            <div class="flex flex-col">
              <span class="text-sm font-mono text-fg">{{ d }}</span>
              <span class="font-mono text-[10px] text-fg-dim tracking-widest leading-none">••••••••</span>
            </div>
          </div>
          <button
            @click="deletePat(d as string)"
            class="min-w-[44px] min-h-[44px] flex items-center justify-center text-red/60 hover:text-red hover:bg-red/10 rounded-md transition-all active:scale-95 duration-100 cursor-pointer"
          >
            <Trash2 :size="16" />
          </button>
        </div>
      </div>
      <div v-else class="text-center py-4 opacity-30 font-sans">
        <p class="text-xs italic font-sans">No tokens saved yet.</p>
      </div>
    </div>
  </div>
</template>
