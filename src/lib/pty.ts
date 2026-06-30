import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';

export class PtyBridge {
  private sessionId: string;
  private unlistenData: UnlistenFn | null = null;
  private unlistenExit: UnlistenFn | null = null;

  // Write batching: accumulate rapid keystrokes into a single IPC call
  private writeBuffer: string = '';
  private writeTimer: ReturnType<typeof setTimeout> | null = null;
  private static readonly WRITE_BATCH_MS = 2;

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

  write(data: string): void {
    this.writeBuffer += data;
    if (this.writeTimer !== null) {
      clearTimeout(this.writeTimer);
    }
    this.writeTimer = setTimeout(() => {
      this.flushWrite();
    }, PtyBridge.WRITE_BATCH_MS);
  }

  private flushWrite(): void {
    if (this.writeBuffer.length === 0) return;
    const data = this.writeBuffer;
    this.writeBuffer = '';
    this.writeTimer = null;
    invoke('pty_write', {
      id: this.sessionId,
      data,
    }).catch(() => {});
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
    this.writeBuffer = '';
    if (this.writeTimer !== null) {
      clearTimeout(this.writeTimer);
      this.writeTimer = null;
    }
    if (this.unlistenData) this.unlistenData();
    if (this.unlistenExit) this.unlistenExit();
  }
}
