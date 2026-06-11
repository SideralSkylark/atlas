import { invoke } from "@tauri-apps/api/core";
import { ref } from "vue";

export interface FileEntry {
  name: string;
  is_dir: boolean;
}

export function useFileSystem() {
  const files = ref<FileEntry[]>([]);
  const currentRelativePath = ref("");
  const selectedFileContent = ref<string | null>(null);
  const loading = ref(false);
  const error = ref<string | null>(null);

  async function loadFiles(repoId: string) {
    loading.value = true;
    error.value = null;
    try {
      files.value = await invoke<FileEntry[]>("list_files", {
        repoId,
        relativePath: currentRelativePath.value,
      });
    } catch (e) {
      error.value = String(e);
    } finally {
      loading.value = false;
    }
  }

  async function readFile(repoId: string, fileName: string) {
    loading.value = true;
    error.value = null;
    const path = currentRelativePath.value
      ? `${currentRelativePath.value}/${fileName}`
      : fileName;
    try {
      selectedFileContent.value = await invoke<string>("read_file", {
        repoId,
        relativePath: path,
      });
    } catch (e) {
      error.value = String(e);
      selectedFileContent.value = null;
    } finally {
      loading.value = false;
    }
  }

  function enterDirectory(dirName: string) {
    currentRelativePath.value = currentRelativePath.value
      ? `${currentRelativePath.value}/${dirName}`
      : dirName;
    selectedFileContent.value = null;
  }

  function goBack() {
    if (selectedFileContent.value !== null) {
      selectedFileContent.value = null;
      return true; // Stayed in the same directory
    }

    if (!currentRelativePath.value) {
      return false; // Already at root
    }

    const parts = currentRelativePath.value.split("/");
    parts.pop();
    currentRelativePath.value = parts.join("/");
    return true;
  }

  function reset() {
    currentRelativePath.value = "";
    selectedFileContent.value = null;
    files.value = [];
  }

  return {
    files,
    currentRelativePath,
    selectedFileContent,
    loading,
    error,
    loadFiles,
    readFile,
    enterDirectory,
    goBack,
    reset,
  };
}
