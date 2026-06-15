<script setup lang="ts">
import { computed } from 'vue';

const props = withDefaults(defineProps<{
  diff: string;
  maxHeight?: string;
}>(), {
  maxHeight: '40vh'
});

const diffLines = computed(() => {
  if (!props.diff) return [];
  return props.diff.split('\n');
});

function getLineClass(line: string) {
  if (line.startsWith('+')) return 'bg-green/10 text-green';
  if (line.startsWith('-')) return 'bg-red/10 text-red';
  if (line.startsWith('@@')) return 'bg-aqua/10 text-aqua/70 text-[10px]';
  return 'text-fg-dim';
}
</script>

<template>
  <div 
    class="overflow-x-auto overflow-y-auto whitespace-pre font-mono text-[11px] leading-relaxed"
    :style="{ maxHeight: props.maxHeight }"
  >
    <div 
      v-for="(line, index) in diffLines" 
      :key="index"
      :class="getLineClass(line)"
      class="whitespace-pre px-2 min-w-fit"
    >
      {{ line }}
    </div>
  </div>
</template>
