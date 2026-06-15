<script setup lang="ts">
import CredentialsManager from "./CredentialsManager.vue";
</script>

<template>
  <div class="space-y-6">
    <div>
      <h2 class="text-2xl font-black text-yellow tracking-tight leading-none font-sans">Settings</h2>
      <p class="text-fg-dim text-[10px] uppercase tracking-widest font-bold mt-1.5 ml-0.5 font-sans">
        Manage credentials and preferences
      </p>
    </div>
    
    <div class="bg-bg1 border border-border rounded-xl p-6 shadow-sm">
      <CredentialsManager />
    </div>
  </div>
</template>
