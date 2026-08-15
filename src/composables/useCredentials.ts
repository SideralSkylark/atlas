import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";

export function useCredentials() {
  const pats = ref<Record<string, string>>({});
  const error = ref<string | null>(null);

  async function loadPats() {
    try {
      pats.value = await invoke<Record<string, string>>("get_pats");
      return pats.value;
    } catch (e) {
      error.value = String(e);
      return {};
    }
  }

  async function savePat(domain: string, token: string) {
    try {
      await invoke("save_pat", { domain, token });
      await loadPats();
      return true;
    } catch (e) {
      error.value = String(e);
      return false;
    }
  }

  async function deletePat(domain: string) {
    try {
      await invoke("delete_pat", { domain });
      await loadPats();
      return true;
    } catch (e) {
      error.value = String(e);
      return false;
    }
  }

  return {
    pats,
    error,
    loadPats,
    savePat,
    deletePat,
  };
}
