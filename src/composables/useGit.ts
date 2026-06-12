import { invoke } from "@tauri-apps/api/core";
import { ref } from "vue";

export interface BranchInfo {
  name: string;
  is_current: boolean;
  is_remote: boolean;
}

export interface CommitInfo {
  hash: string;
  author: string;
  message: string;
  date: number;
}

export interface StatusEntry {
  path: string;
  status: string;
  staged: boolean;
}

export function useGit() {
  const branches = ref<BranchInfo[]>([]);
  const history = ref<CommitInfo[]>([]);
  const status = ref<StatusEntry[]>([]);
  const diff = ref<string>("");
  const loading = ref(false);
  const error = ref<string | null>(null);

  async function loadBranches(repoId: string) {
    loading.value = true;
    try {
      branches.value = await invoke<BranchInfo[]>("list_branches", { repoId });
    } catch (e) {
      error.value = String(e);
    } finally {
      loading.value = false;
    }
  }

  async function createBranch(repoId: string, branchName: string) {
    loading.value = true;
    try {
      await invoke("create_branch", { repoId, branchName });
      await loadBranches(repoId);
    } catch (e) {
      error.value = String(e);
    } finally {
      loading.value = false;
    }
  }

  async function switchBranch(repoId: string, branchName: string) {
    loading.value = true;
    try {
      await invoke("switch_branch", { repoId, branchName });
      await loadBranches(repoId);
    } catch (e) {
      error.value = String(e);
    } finally {
      loading.value = false;
    }
  }

  async function loadHistory(repoId: string, limit: number = 50) {
    loading.value = true;
    try {
      history.value = await invoke<CommitInfo[]>("get_commit_history", { repoId, limit });
    } catch (e) {
      error.value = String(e);
    } finally {
      loading.value = false;
    }
  }

  async function loadStatus(repoId: string) {
    loading.value = true;
    try {
      status.value = await invoke<StatusEntry[]>("get_status", { repoId });
    } catch (e) {
      error.value = String(e);
    } finally {
      loading.value = false;
    }
  }

  async function stageFile(repoId: string, filepath: string) {
    try {
      await invoke("stage_file", { repoId, filepath });
      await loadStatus(repoId);
    } catch (e) {
      error.value = String(e);
    }
  }

  async function unstageFile(repoId: string, filepath: string) {
    try {
      await invoke("unstage_file", { repoId, filepath });
      await loadStatus(repoId);
    } catch (e) {
      error.value = String(e);
    }
  }

  async function commitChanges(repoId: string, message: string, authorName: string, authorEmail: string) {
    loading.value = true;
    try {
      await invoke("commit_changes", { repoId, message, authorName, authorEmail });
      await loadStatus(repoId);
      await loadHistory(repoId);
    } catch (e) {
      error.value = String(e);
    } finally {
      loading.value = false;
    }
  }

  async function loadDiff(repoId: string, filepath?: string, staged: boolean = false) {
    try {
      diff.value = await invoke<string>("get_diff", { repoId, filepath, staged });
    } catch (e) {
      error.value = String(e);
    }
  }

  return {
    branches,
    history,
    status,
    diff,
    loading,
    error,
    loadBranches,
    createBranch,
    switchBranch,
    loadHistory,
    loadStatus,
    stageFile,
    unstageFile,
    commitChanges,
    loadDiff,
  };
}
