<script setup lang="ts">
import { ref, onMounted, watch } from "vue";
import { useCredentials } from "../composables/useCredentials";
import { Key, Save, ShieldAlert, Trash2, User } from "@lucide/vue";
import ActionButton from "./ui/ActionButton.vue";
import SettingsCard from "./ui/SettingsCard.vue";
import TextInput from "./ui/TextInput.vue";

const { pats, loadPats, savePat, deletePat } = useCredentials();

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
  localStorage.setItem("atlas_author_name", authorName.value);
  localStorage.setItem("atlas_author_email", authorEmail.value);
}

watch([authorName, authorEmail], () => {
  saveAuthorSettings();
});

onMounted(() => {
  loadPats();
  authorName.value = localStorage.getItem("atlas_author_name") || "";
  authorEmail.value = localStorage.getItem("atlas_author_email") || "";
});
</script>

<template>
  <div class="space-y-6">
    <SettingsCard title="Git Identity" description="Commit author">
      <template #icon>
        <User :size="18" class="text-yellow" />
      </template>

      <div class="space-y-4">
        <TextInput v-model="authorName" label="Author Name" placeholder="e.g. John Doe" />
        <TextInput v-model="authorEmail" label="Author Email" type="email" placeholder="e.g. john@example.com" />
        <p class="text-[10px] text-fg-dim italic px-1 leading-relaxed font-sans">
          This identity will be used for all commits you make in Atlas.
        </p>
      </div>
    </SettingsCard>

    <SettingsCard title="Git Credentials" description="Secure PAT storage">
      <template #icon>
        <Key :size="18" class="text-yellow" />
      </template>

      <div class="space-y-3 mb-6">
        <TextInput v-model="domain" label="Domain" placeholder="e.g. github.com" mono />
        <TextInput v-model="token" label="Personal Access Token" type="password" placeholder="ghp_xxxxxxxxxxxx" mono />

        <ActionButton @click="onSaveToken" variant="primary">
          <template #icon>
            <Save :size="18" />
          </template>
          Save Token
        </ActionButton>
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
          <ActionButton variant="danger" :full-width="false" @click="deletePat(d as string)">
            <template #icon>
              <Trash2 :size="16" />
            </template>
            Remove
          </ActionButton>
        </div>
      </div>
      <div v-else class="text-center py-5 px-3 rounded-lg border border-dashed border-border bg-bg0/30 text-fg-dim">
        <p class="text-xs italic font-sans">No tokens saved yet.</p>
      </div>
    </SettingsCard>
  </div>
</template>
