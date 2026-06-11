<script setup lang="ts">
defineProps<{
  content: string;
}>();
</script>

<template>
  <div class="bg-bg1 border border-border rounded-md p-4 overflow-x-auto">
    <pre class="text-sm text-fg leading-relaxed font-mono whitespace-pre-wrap">{{ content }}</pre>
  </div>
</template>
