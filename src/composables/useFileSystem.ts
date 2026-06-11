import { invoke } from "@tauri-apps/api/core";
import { ref } from "vue";

export interface FileEntry {
  name: string;
  is_dir: boolean;
}

export interface RenderedFile {
  content: string;
  file_type: "markdown" | "code" | "html" | "plain";
}

export interface SearchResult {
  name: string;
  relative_path: string;
  is_dir: boolean;
}

export function useFileSystem() {
  const files = ref<FileEntry[]>([]);
  const searchResults = ref<SearchResult[]>([]);
  const currentRelativePath = ref("");
  const renderedFile = ref<RenderedFile | null>(null);
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
      searchResults.value = [];
    } catch (e) {
      error.value = String(e);
    } finally {
      loading.value = false;
    }
  }

  async function searchFiles(repoId: string, query: string) {
    if (!query) {
      searchResults.value = [];
      return;
    }
    loading.value = true;
    error.value = null;
    try {
      searchResults.value = await invoke<SearchResult[]>("search_files", {
        repoId,
        query,
      });
    } catch (e) {
      error.value = String(e);
    } finally {
      loading.value = false;
    }
  }

  async function renderFile(repoId: string, fileName: string) {
    loading.value = true;
    error.value = null;
    const path = currentRelativePath.value
      ? `${currentRelativePath.value}/${fileName}`
      : fileName;
    try {
      renderedFile.value = await invoke<RenderedFile>("render_file", {
        repoId,
        relativePath: path,
      });
    } catch (e) {
      error.value = String(e);
      renderedFile.value = null;
    } finally {
      loading.value = false;
    }
  }

  async function openPath(repoId: string, path: string, isDir: boolean) {
    if (isDir) {
      currentRelativePath.value = path;
      searchResults.value = [];
      await loadFiles(repoId);
    } else {
      loading.value = true;
      error.value = null;
      try {
        renderedFile.value = await invoke<RenderedFile>("render_file", {
          repoId,
          relativePath: path,
        });
        // Update currentRelativePath to the parent of this file
        const parts = path.split("/");
        parts.pop();
        currentRelativePath.value = parts.join("/");
        searchResults.value = [];
      } catch (e) {
        error.value = String(e);
        renderedFile.value = null;
      } finally {
        loading.value = false;
      }
    }
  }

  function enterDirectory(dirName: string) {
    currentRelativePath.value = currentRelativePath.value
      ? `${currentRelativePath.value}/${dirName}`
      : dirName;
    renderedFile.value = null;
  }

  function goBack() {
    if (renderedFile.value !== null) {
      renderedFile.value = null;
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
    renderedFile.value = null;
    files.value = [];
    searchResults.value = [];
  }

  return {
    files,
    searchResults,
    currentRelativePath,
    renderedFile,
    loading,
    error,
    loadFiles,
    searchFiles,
    renderFile,
    openPath,
    enterDirectory,
    goBack,
    reset,
  };
}
