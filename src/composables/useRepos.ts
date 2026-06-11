import { invoke } from "@tauri-apps/api/core";
import { ref } from "vue";

export interface RepoInfo {
  id: string;
  name: string;
  branch: string;
}

export function useRepos() {
  const repos = ref<RepoInfo[]>([]);
  const cloning = ref(false);
  const deletingRepo = ref<string | null>(null);
  const error = ref<string | null>(null);
  const successMessage = ref<string | null>(null);

  async function loadRepos() {
    try {
      repos.value = await invoke<RepoInfo[]>("list_repos");
    } catch (e) {
      error.value = String(e);
    }
  }

  async function cloneRepo(url: string) {
    if (!url || cloning.value) return;

    cloning.value = true;
    error.value = null;
    successMessage.value = null;

    try {
      const res = await invoke<{ success: boolean; message: string }>(
        "clone_repo",
        { url }
      );

      if (res.success) {
        successMessage.value = res.message;
        await loadRepos();
      } else {
        error.value = res.message;
      }
      return res;
    } catch (e) {
      error.value = String(e);
      return { success: false, message: String(e) };
    } finally {
      cloning.value = false;
    }
  }

  async function deleteRepo(repoId: string) {
    deletingRepo.value = repoId;
    error.value = null;
    successMessage.value = null;

    try {
      const res = await invoke<{ success: boolean; message: string }>(
        "delete_repo",
        { repoId }
      );

      if (res.success) {
        successMessage.value = res.message;
        await loadRepos();
      } else {
        error.value = res.message;
      }
      return res;
    } catch (e) {
      error.value = String(e);
      return { success: false, message: String(e) };
    } finally {
      deletingRepo.value = null;
    }
  }

  function clearMessages() {
    error.value = null;
    successMessage.value = null;
  }

  return {
    repos,
    cloning,
    deletingRepo,
    error,
    successMessage,
    loadRepos,
    cloneRepo,
    deleteRepo,
    clearMessages,
  };
}
