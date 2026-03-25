export interface Matcher {
  'match-json-body'?: Record<string, unknown>;
  'match-headers'?: Record<string, string>;
}

export interface Webhook {
  name: string;
  'matchers-strategy'?: 'all' | 'one';
  matchers: Matcher[];
  'actions-to-execute': string[];
}

export interface PagooConfig {
  database_file?: string;
  webhooks: Webhook[];
}

export async function fetchConfig(): Promise<PagooConfig> {
  const response = await fetch('/api/config');
  if (!response.ok) {
    throw new Error(`HTTP ${response.status}: Failed to fetch config`);
  }
  return await response.json();
}

export interface Task {
  id: string;
  execution_date: string;
  webhook_name: string;
  executed_command: string;
  command_exit_code: number;
  command_stdout: string;
  command_stderr: string;
}

export interface FetchTasksOptions {
  cursor?: string;
  limit?: number;
}

export async function fetchTasks(options: FetchTasksOptions = {}): Promise<Task[]> {
  try {
    const params = new URLSearchParams();
    if (options.cursor) {
      params.append('cursor', options.cursor);
    }
    if (options.limit) {
      params.append('limit', options.limit.toString());
    }

    const url = params.toString() ? `/api/tasks?${params.toString()}` : '/api/tasks';
    const response = await fetch(url);

    if (!response.ok) {
      return Promise.reject(Error(`HTTP ${response.status}: Failed to fetch tasks`));
    }

    return await response.json();
  } catch (error) {
    console.error('Error fetching tasks:', error);
    throw error;
  }
}
