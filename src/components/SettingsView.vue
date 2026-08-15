<script setup lang="ts">
import { Moon, Sun, Palette } from "@lucide/vue";
import CredentialsManager from "./CredentialsManager.vue";
import SettingsCard from "./ui/SettingsCard.vue";
import { useTheme } from "../composables/useTheme";

const { theme, appearance, setTheme, toggleAppearance } = useTheme();
</script>

<template>
  <div class="space-y-6">
    <div>
      <h2 class="text-2xl font-black text-yellow tracking-tight leading-none font-sans">Settings</h2>
      <p class="text-fg-dim text-[10px] uppercase tracking-widest font-bold mt-1.5 ml-0.5 font-sans">
        Manage credentials and preferences
      </p>
    </div>

    <SettingsCard title="Appearance" description="Theme and visual mode">
      <template #icon>
        <Palette :size="16" class="text-yellow" />
      </template>

      <div class="grid grid-cols-1 gap-3">
        <button
          @click="toggleAppearance"
          class="flex items-center justify-between p-4 bg-bg1 border border-border rounded-xl hover:border-yellow transition-all active:scale-[0.98] shadow-sm"
          style="box-shadow: var(--shadow-sm), var(--shadow-inset)"
        >
          <div class="flex items-center gap-3">
            <div class="w-10 h-10 rounded-lg bg-bg2 flex items-center justify-center text-yellow">
              <Sun v-if="appearance === 'light'" :size="20" />
              <Moon v-else :size="20" />
            </div>
            <div class="text-left">
              <p class="text-sm font-bold capitalize">{{ appearance }} Mode</p>
              <p class="text-[10px] text-fg-dim">Switch between light and dark</p>
            </div>
          </div>
          <div class="w-12 h-6 bg-bg2 rounded-full relative transition-colors" :class="{ 'bg-yellow/20': true }">
            <div
              class="absolute top-1 w-4 h-4 rounded-full transition-all duration-200"
              :class="appearance === 'dark' ? 'right-1 bg-yellow' : 'right-7 bg-fg-dim'"
            ></div>
          </div>
        </button>

        <div class="p-4 bg-bg1 border border-border rounded-xl space-y-3 shadow-sm" style="box-shadow: var(--shadow-sm), var(--shadow-inset)">
          <p class="text-[10px] font-bold uppercase tracking-widest text-fg-dim ml-1">Color Scheme</p>
          <div class="grid grid-cols-2 gap-2">
            <button
              @click="setTheme('everforest')"
              class="flex flex-col gap-2 p-3 rounded-lg border-2 transition-all text-left"
              :class="theme === 'everforest' ? 'border-yellow bg-bg2' : 'border-transparent bg-bg0 hover:border-border'"
            >
              <div class="flex gap-1">
                <div class="w-3 h-3 rounded-full bg-[#a7c080]"></div>
                <div class="w-3 h-3 rounded-full bg-[#dbbc7f]"></div>
                <div class="w-3 h-3 rounded-full bg-[#83c092]"></div>
              </div>
              <span class="text-xs font-bold">Everforest</span>
            </button>
            <button
              @click="setTheme('gruvbox')"
              class="flex flex-col gap-2 p-3 rounded-lg border-2 transition-all text-left"
              :class="theme === 'gruvbox' ? 'border-yellow bg-bg2' : 'border-transparent bg-bg0 hover:border-border'"
            >
              <div class="flex gap-1">
                <div class="w-3 h-3 rounded-full bg-[#b8bb26]"></div>
                <div class="w-3 h-3 rounded-full bg-[#fabd2f]"></div>
                <div class="w-3 h-3 rounded-full bg-[#8ec07c]"></div>
              </div>
              <span class="text-xs font-bold">Gruvbox</span>
            </button>
            <button
              @click="setTheme('rosepine')"
              class="flex flex-col gap-2 p-3 rounded-lg border-2 transition-all text-left"
              :class="theme === 'rosepine' ? 'border-yellow bg-bg2' : 'border-transparent bg-bg0 hover:border-border'"
            >
              <div class="flex gap-1">
                <div class="w-3 h-3 rounded-full bg-[#31748f]"></div>
                <div class="w-3 h-3 rounded-full bg-[#ebbcba]"></div>
                <div class="w-3 h-3 rounded-full bg-[#eb6f92]"></div>
              </div>
              <span class="text-xs font-bold">Rose Pine</span>
            </button>
            <button
              @click="setTheme('kanagawa')"
              class="flex flex-col gap-2 p-3 rounded-lg border-2 transition-all text-left"
              :class="theme === 'kanagawa' ? 'border-yellow bg-bg2' : 'border-transparent bg-bg0 hover:border-border'"
            >
              <div class="flex gap-1">
                <div class="w-3 h-3 rounded-full bg-[#76946a]"></div>
                <div class="w-3 h-3 rounded-full bg-[#c0a36e]"></div>
                <div class="w-3 h-3 rounded-full bg-[#7aa89f]"></div>
              </div>
              <span class="text-xs font-bold">Kanagawa</span>
            </button>
          </div>
        </div>
      </div>
    </SettingsCard>

    <CredentialsManager />
  </div>
</template>
