<script setup lang="ts">
withDefaults(
  defineProps<{
    modelValue: string;
    label: string;
    type?: "text" | "email" | "password";
    placeholder?: string;
    mono?: boolean;
  }>(),
  {
    type: "text",
    placeholder: "",
    mono: false,
  }
);

const emit = defineEmits<{
  (e: "update:modelValue", value: string): void;
}>();

function onInput(event: Event) {
  const target = event.target as HTMLInputElement | null;
  emit("update:modelValue", target?.value ?? "");
}
</script>

<template>
  <div class="space-y-1.5">
    <label class="text-[10px] uppercase tracking-wider font-bold text-fg-dim ml-1 font-sans">
      {{ label }}
    </label>
    <input
      :type="type"
      :value="modelValue"
      :placeholder="placeholder"
      class="w-full px-4 py-2.5 bg-bg3 text-fg border border-border rounded-lg outline-none focus:border-yellow transition-all text-sm"
      :class="mono ? 'font-mono' : 'font-sans'"
      @input="onInput"
    />
  </div>
</template>
