<script setup lang="ts">
import { ref, onMounted } from "vue";
import { useRepos } from "../composables/useRepos";
import { Key, Save, ShieldAlert, Trash2 } from "@lucide/vue";

const { pats, loadPats, savePat, deletePat } = useRepos();

const domain = ref("github.com");
const token = ref("");

async function onSave() {
  if (domain.value && token.value) {
    await savePat(domain.value, token.value);
    token.value = "";
  }
}

onMounted(loadPats);
</script>

<template>
  <div class="p-5 bg-bg1 border border-border rounded-xl shadow-sm overflow-hidden">
    <div class="flex items-center gap-2 mb-5">
      <Key :size="18" class="text-yellow" />
      <h2 class="text-yellow font-bold text-lg">Git Credentials</h2>
    </div>
    
    <div class="space-y-3 mb-6">
      <div class="space-y-1.5">
        <label class="text-[10px] uppercase tracking-wider font-bold text-fg-dim ml-1">Domain</label>
        <input
          v-model="domain"
          placeholder="e.g. github.com"
          class="w-full px-4 py-2.5 bg-bg3 text-fg border border-border rounded-lg outline-none focus:border-green transition-all text-sm font-mono"
        />
      </div>
      <div class="space-y-1.5">
        <label class="text-[10px] uppercase tracking-wider font-bold text-fg-dim ml-1">Personal Access Token</label>
        <input
          v-model="token"
          type="password"
          placeholder="ghp_xxxxxxxxxxxx"
          class="w-full px-4 py-2.5 bg-bg3 text-fg border border-border rounded-lg outline-none focus:border-green transition-all text-sm font-mono"
        />
      </div>
      <button
        @click="onSave"
        class="w-full py-3 mt-2 bg-green text-bg0 font-bold rounded-lg active:scale-[0.98] transition-all flex items-center justify-center gap-2 cursor-pointer"
      >
        <Save :size="18" />
        <span>Save Token</span>
      </button>
    </div>

    <div v-if="Object.keys(pats).length > 0" class="space-y-2 pt-4 border-t border-border/50">
      <div
        v-for="(_, d) in pats"
        :key="d"
        class="flex justify-between items-center px-4 py-3 bg-bg0/50 rounded-lg border border-border/30"
      >
        <div class="flex items-center gap-2" title="Token saved securely">
          <ShieldAlert :size="14" class="text-aqua" />
          <span class="text-sm font-mono text-fg">{{ d }}</span>
        </div>
        <button
          @click="deletePat(d as string)"
          class="p-1.5 text-red/60 hover:text-red hover:bg-red/10 rounded-md transition-all cursor-pointer"
        >
          <Trash2 :size="16" />
        </button>
      </div>
    </div>
    <div v-else class="text-center py-4 opacity-30">
      <p class="text-xs italic">No tokens saved yet.</p>
    </div>
  </div>
</template>
