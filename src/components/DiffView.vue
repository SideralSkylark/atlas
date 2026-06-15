<script setup lang="ts">
import { computed } from 'vue';

const props = withDefaults(defineProps<{
  diff: string;
  maxHeight?: string;
}>(), {
  maxHeight: '40vh'
});

const lines = computed(() => {
  if (!props.diff) return [];
  const rawLines = props.diff.split('\n');
  let currentLineNum = 0;
  
  return rawLines.map(line => {
    if (line.startsWith('@@')) {
      const match = line.match(/@@ -\d+(?:,\d+)? \+(\d+)(?:,\d+)? @@/);
      if (match) {
        currentLineNum = parseInt(match[1]);
      }
      return { content: line, type: 'header' as const, lineNum: null };
    }
    
    if (line.startsWith('+')) {
      return { content: line, type: 'added' as const, lineNum: currentLineNum++ };
    } else if (line.startsWith('-')) {
      return { content: line, type: 'removed' as const, lineNum: null };
    } else {
      return { content: line, type: 'context' as const, lineNum: currentLineNum++ };
    }
  });
});

function getLineStyle(type: 'added' | 'removed' | 'header' | 'context') {
  switch (type) {
    case 'added':
      return {
        background: 'rgba(var(--color-green, 167, 192, 128), 0.12)',
        class: 'text-green border-l-2 border-green'
      };
    case 'removed':
      return {
        background: 'rgba(var(--color-red, 230, 126, 128), 0.12)',
        class: 'text-red border-l-2 border-red'
      };
    case 'header':
      return {
        class: 'bg-bg2 text-aqua text-[10px] border-y border-border py-1 px-3 font-bold tracking-wide w-full'
      };
    default:
      return {
        class: 'text-fg-dim border-l-2 border-transparent'
      };
  }
}
</script>

<template>
  <!-- NOTE: Ensure no parent container has overflow-x-hidden for optimal horizontal scrolling -->
  <div 
    class="font-mono text-[11px] leading-relaxed overflow-x-auto overflow-y-auto bg-bg0 rounded-b-xl"
    :style="{ maxHeight: props.maxHeight }"
  >
    <div 
      v-for="(line, index) in lines" 
      :key="index"
      :class="getLineStyle(line.type).class"
      :style="{ background: getLineStyle(line.type).background }"
      class="flex min-w-fit"
    >
      <span 
        v-if="line.type !== 'header'"
        class="select-none w-10 shrink-0 text-right pr-3 text-fg-dim/30 text-[10px] font-mono border-r border-border/30 mr-2 py-0.5"
      >
        {{ line.lineNum }}
      </span>
      <span 
        class="flex-1 whitespace-pre py-0.5"
        :class="{ 'pl-10': line.type === 'header' }"
      >
        {{ line.content }}
      </span>
    </div>
  </div>
</template>
