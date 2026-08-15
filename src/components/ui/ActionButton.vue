<script setup lang="ts">
withDefaults(
  defineProps<{
    variant?: "primary" | "secondary" | "danger";
    fullWidth?: boolean;
    loading?: boolean;
  }>(),
  {
    variant: "primary",
    fullWidth: true,
    loading: false,
  }
);

const variantClasses = {
  primary: "bg-green text-bg0 hover:brightness-110 border border-transparent",
  secondary: "bg-bg2 text-fg border border-border hover:border-yellow hover:bg-bg1",
  danger: "bg-red/10 text-red border border-red/20 hover:bg-red/15",
};
</script>

<template>
  <button
    :class="[
      'inline-flex items-center justify-center gap-2 rounded-lg px-4 py-3 font-bold text-sm transition-all duration-100 active:scale-[0.98] disabled:cursor-not-allowed disabled:opacity-60 font-sans',
      variantClasses[variant],
      fullWidth ? 'w-full' : 'min-w-fit',
    ]"
    :disabled="loading"
  >
    <slot name="icon" />
    <span v-if="loading" class="animate-pulse">Working...</span>
    <slot v-else />
  </button>
</template>
