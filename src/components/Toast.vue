<script setup lang="ts">
import { ref, watch } from "vue";

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
    enter-from-class="transform translate-y-4 opacity-0"
    enter-to-class="transform translate-y-0 opacity-100"
    leave-active-class="transition duration-200 ease-in"
    leave-from-class="opacity-100"
    leave-to-class="opacity-0"
  >
    <div
      v-if="visible && message"
      class="fixed bottom-6 left-6 right-6 p-4 rounded-md border shadow-lg z-50 flex items-center gap-3"
      :class="
        message.type === 'success'
          ? 'bg-bg1 border-green text-green'
          : 'bg-bg1 border-red text-red'
      "
    >
      <svg
        v-if="message.type === 'success'"
        class="h-5 w-5"
        viewBox="0 0 20 20"
        fill="currentColor"
      >
        <path
          fill-rule="evenodd"
          d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z"
          clip-rule="evenodd"
        />
      </svg>
      <svg v-else class="h-5 w-5" viewBox="0 0 20 20" fill="currentColor">
        <path
          fill-rule="evenodd"
          d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z"
          clip-rule="evenodd"
        />
      </svg>
      <span class="text-sm font-medium">{{ message.text }}</span>
    </div>
  </Transition>
</template>
