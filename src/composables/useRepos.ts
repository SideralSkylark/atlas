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
  const syncing = ref<string | null>(null); // repoId being synced
  const deletingRepo = ref<string | null>(null);
  const error = ref<string | null>(null);
  const successMessage = ref<string | null>(null);
  const pats = ref<Record<string, string>>({});

  async function loadRepos() {
    try {
      repos.value = await invoke<RepoInfo[]>("list_repos");
    } catch (e) {
      error.value = String(e);
    }
  }

  async function loadPats() {
    try {
      pats.value = await invoke<Record<string, string>>("get_pats");
    } catch (e) {
      error.value = String(e);
    }
  }

  async function savePat(domain: string, token: string) {
    try {
      await invoke("save_pat", { domain, token });
      await loadPats();
    } catch (e) {
      error.value = String(e);
    }
  }

  async function deletePat(domain: string) {
    try {
      await invoke("delete_pat", { domain });
      await loadPats();
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

  async function pullRepo(repoId: string) {
    syncing.value = repoId;
    error.value = null;
    successMessage.value = null;

    try {
      const res = await invoke<{ success: boolean; message: string }>(
        "git_pull",
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
      syncing.value = null;
    }
  }

  async function pushRepo(repoId: string) {
    syncing.value = repoId;
    error.value = null;
    successMessage.value = null;

    try {
      const res = await invoke<{ success: boolean; message: string }>(
        "git_push",
        { repoId }
      );
      if (res.success) {
        successMessage.value = res.message;
      } else {
        error.value = res.message;
      }
      return res;
    } catch (e) {
      error.value = String(e);
      return { success: false, message: String(e) };
    } finally {
      syncing.value = null;
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
    syncing,
    deletingRepo,
    error,
    successMessage,
    pats,
    loadRepos,
    loadPats,
    savePat,
    deletePat,
    cloneRepo,
    pullRepo,
    pushRepo,
    deleteRepo,
    clearMessages,
  };
}
