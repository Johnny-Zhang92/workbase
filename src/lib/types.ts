export interface Project {
  id: number;
  name: string;
  root_path: string;
  sort_order: number;
  created_at: string;
  updated_at: string;
}

export interface Session {
  id: number;
  project_id: number;
  name: string;
  cwd: string | null;
  sort_order: number;
  created_at: string;
  last_active_at: string;
  launch_command: string;
  launch_type: string;
}

export interface SessionTemplate {
  id: number;
  name: string;
  launch_command: string;
  icon: string;
  sort_order: number;
  created_at: string;
}

export interface PtySpawnConfig {
  sessionId: string;
  shell: string;
  workingDirectory: string;
  cols: number;
  rows: number;
}
