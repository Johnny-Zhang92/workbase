import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';

export class PtyBridge {
  private sessionId: string;
  private unlistenData: UnlistenFn | null = null;
  private unlistenExit: UnlistenFn | null = null;

  constructor(sessionId: string) {
    this.sessionId = sessionId;
  }

  async spawn(shell: string, workingDirectory: string, cols: number, rows: number): Promise<void> {
    await invoke('pty_spawn', {
      sessionId: this.sessionId,
      shell,
      workingDirectory,
      cols,
      rows,
    });
  }

  async write(data: string): Promise<void> {
    await invoke('pty_write', {
      id: this.sessionId,
      data,
    });
  }

  async resize(cols: number, rows: number): Promise<void> {
    await invoke('pty_resize', {
      id: this.sessionId,
      cols,
      rows,
    });
  }

  async kill(): Promise<void> {
    await invoke('pty_kill', { id: this.sessionId });
  }

  async onData(callback: (data: string) => void): Promise<void> {
    this.unlistenData = await listen<string>(`pty:data:${this.sessionId}`, (event) => {
      callback(event.payload);
    });
  }

  async onExit(callback: (code: number) => void): Promise<void> {
    this.unlistenExit = await listen<number>(`pty:exit:${this.sessionId}`, (event) => {
      callback(event.payload);
    });
  }

  async destroy(): Promise<void> {
    if (this.unlistenData) this.unlistenData();
    if (this.unlistenExit) this.unlistenExit();
  }
}
