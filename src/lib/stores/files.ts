import { writable, derived, get } from 'svelte/store';
import { fsGlob, fsRead } from '../api/fs';

export interface FileEntry {
  path: string;
  name: string;
  isDirectory: boolean;
  children?: FileEntry[];
  content?: string;
}

interface FileTreeNode {
  [key: string]: FileEntry;
}

// Reactive store for workspace files
export const files = writable<FileEntry[]>([]);
export const selectedFile = writable<string | null>(null);
export const fileContent = writable<string>('');

// Load all files in the workspace
export async function loadWorkspaceFiles() {
  try {
    const allFiles = await fsGlob('**/*');
    const entries = buildFileTree(allFiles);
    files.set(entries);
  } catch (error) {
    console.error('Failed to load workspace files:', error);
    files.set([]);
  }
}

// Build a tree structure from flat file list
function buildFileTree(paths: string[]): FileEntry[] {
  const root: FileTreeNode = {};
  
  for (const path of paths) {
    const parts = path.split('/');
    let current: FileTreeNode = root;
    
    for (let i = 0; i < parts.length; i++) {
      const part = parts[i];
      const isLast = i === parts.length - 1;
      const fullPath = parts.slice(0, i + 1).join('/');
      
      if (!current[part]) {
        current[part] = {
          path: fullPath,
          name: part,
          isDirectory: !isLast,
          children: isLast ? undefined : [],
        };
      }
      
      if (!isLast && current[part].children) {
        // Convert children array to object for next iteration
        const childrenObj: FileTreeNode = {};
        for (const child of current[part].children) {
          childrenObj[child.name] = child;
        }
        current = childrenObj;
      }
    }
  }
  
  // Convert to array and sort
  return Object.values(root).sort((a, b) => {
    if (a.isDirectory !== b.isDirectory) {
      return a.isDirectory ? -1 : 1;
    }
    return a.name.localeCompare(b.name);
  });
}

// Read and cache file content
export async function readFileContent(path: string): Promise<string> {
  try {
    const content = await fsRead(path);
    fileContent.set(content);
    selectedFile.set(path);
    return content;
  } catch (error) {
    console.error('Failed to read file:', error);
    throw error;
  }
}

// Clear selected file
export function clearSelectedFile() {
  selectedFile.set(null);
  fileContent.set('');
}

// Derived store for flattened file list (for search)
export const flatFiles = derived(files, ($files) => {
  const result: FileEntry[] = [];
  
  function flatten(entries: FileEntry[]) {
    for (const entry of entries) {
      result.push(entry);
      if (entry.children) {
        flatten(entry.children);
      }
    }
  }
  
  flatten($files);
  return result;
});

// Search files by name
export function searchFiles(query: string): FileEntry[] {
  const allFiles = get(flatFiles);
  const lowerQuery = query.toLowerCase();
  
  return allFiles.filter((file) => 
    file.name.toLowerCase().includes(lowerQuery)
  );
}