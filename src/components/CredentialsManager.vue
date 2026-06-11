<script setup lang="ts">
import { ref, onMounted } from "vue";
import { useRepos } from "../composables/useRepos";

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
  <div class="p-4 bg-bg1 border border-border rounded-md mb-8">
    <h2 class="text-yellow font-bold mb-4">Credentials (PAT)</h2>
    
    <div class="flex flex-col gap-2 mb-4">
      <input
        v-model="domain"
        placeholder="domain (e.g. github.com)"
        class="px-3 py-2 bg-bg3 text-fg border border-border rounded-md outline-none focus:border-green text-sm"
      />
      <input
        v-model="token"
        type="password"
        placeholder="Personal Access Token"
        class="px-3 py-2 bg-bg3 text-fg border border-border rounded-md outline-none focus:border-green text-sm"
      />
      <button
        @click="onSave"
        class="py-2 bg-green text-bg0 font-bold rounded-md active:brightness-90 transition-all cursor-pointer"
      >
        Save Token
      </button>
    </div>

    <div v-if="Object.keys(pats).length > 0" class="space-y-2">
      <div
        v-for="(_, d) in pats"
        :key="d"
        class="flex justify-between items-center px-3 py-2 bg-bg0 rounded border border-border/50"
      >
        <span class="text-sm font-mono">{{ d }}</span>
        <button
          @click="deletePat(d as string)"
          class="text-red text-xs hover:underline cursor-pointer"
        >
          Remove
        </button>
      </div>
    </div>
    <p v-else class="text-xs text-fg-dim">No tokens saved yet.</p>
  </div>
</template>
