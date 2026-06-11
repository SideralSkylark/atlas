<script setup lang="ts">
import { ref, watch } from "vue";
import { CheckCircle, AlertCircle } from "@lucide/vue";

const props = defineProps<{
  message: { type: "success" | "error"; text: string } | null;
}>();

const visible = ref(false);
let timeout: number | null = null;

watch(() => props.message, (newVal) => {
  if (newVal) {
    visible.value = true;
    if (timeout) clearTimeout(timeout);
    timeout = window.setTimeout(() => {
      visible.value = false;
    }, 3000);
  }
});
</script>

<template>
  <Transition
    enter-active-class="transition duration-300 ease-out"
    enter-from-class="transform translate-y-4 opacity-0 scale-95"
    enter-to-class="transform translate-y-0 opacity-100 scale-100"
    leave-active-class="transition duration-200 ease-in"
    leave-from-class="opacity-100 scale-100"
    leave-to-class="opacity-0 scale-95"
  >
    <div
      v-if="visible && message"
      class="fixed bottom-8 left-6 right-6 p-4 rounded-xl border shadow-2xl z-50 flex items-center gap-3 backdrop-blur-md"
      :class="
        message.type === 'success'
          ? 'bg-bg1/90 border-green text-green'
          : 'bg-bg1/90 border-red text-red'
      "
    >
      <CheckCircle v-if="message.type === 'success'" :size="20" />
      <AlertCircle v-else :size="20" />
      <span class="text-sm font-bold tracking-tight">{{ message.text }}</span>
    </div>
  </Transition>
</template>
