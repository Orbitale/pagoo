<script lang="ts">
  import { fetchTasks } from '$lib/api';
  import type { Task } from '$lib/api';

  const ITEMS_PER_PAGE = 30;

  let allTasks = $state<Task[]>([]);
  let loading = $state(true);
  let alertMessage = $state<string | null>(null);
  let messageType = $state<'danger'|'warning'>('danger');
  let currentPage = $state(0);

  let latestCursor = $state<string | null>(null);

  $effect.pre(async () => {
    await syncTasks();
  });

  async function syncTasks() {
    loading = true;
    alertMessage = null;
    try {
      const newTasks = await fetchTasks({ cursor: latestCursor, limit: ITEMS_PER_PAGE });
      if (newTasks.length > 0) {
        allTasks = [...newTasks, ...allTasks];
        latestCursor = newTasks[0]?.execution_date || latestCursor;
        currentPage = 0;
      }
    } catch (err) {
      alertMessage = err instanceof Error ? err.message : 'Failed to sync tasks';
      messageType = 'danger';
    } finally {
      loading = false;
    }
  }

  let paginatedTasks = $derived.by(() => {
    const start = currentPage * ITEMS_PER_PAGE;
    const end = start + ITEMS_PER_PAGE;
    return allTasks.slice(start, end);
  });

  let totalPages = $derived.by(() => {
    return Math.ceil(allTasks.length / ITEMS_PER_PAGE);
  });

  function nextPage() {
    if (currentPage < totalPages - 1) {
      currentPage++;
    }
  }

  function prevPage() {
    if (currentPage > 0) {
      currentPage--;
    }
  }
</script>

<div class="container-fluid mt-4">
  <div class="row">
    <div class="col align-self-start">
      <h1>Pagoo Admin</h1>
    </div>
    <div class="col-sm-3 align-self-end btn-group-vertical btn-group-lg">
      <button class="btn btn-primary btn-block" disabled={loading} onclick={syncTasks}>
        {#if loading}
          <span class="spinner-border spinner-border-sm me-2" role="status" aria-hidden="true"></span>
          Syncing...
        {:else}
          🔄 Sync
        {/if}
      </button>
    </div>
  </div>

  {#if alertMessage}
    <div class="alert alert-{messageType} alert-dismissible fade show" role="alert">
      {alertMessage}
      <button type="button" class="btn-close" onclick={() => (alertMessage = null)}></button>
    </div>
  {/if}

  {#if !loading && allTasks.length === 0 && !alertMessage}
    <div class="alert alert-secondary" role="alert">
      No tasks found. Run some webhooks to see them here.
    </div>
  {/if}

  {#if allTasks.length > 0}
    <div class="mb-3">
      <p class="text-muted">
        Showing {currentPage * ITEMS_PER_PAGE + 1} to {Math.min((currentPage + 1) * ITEMS_PER_PAGE, allTasks.length)} of {allTasks.length} tasks
      </p>
    </div>

    <div class="table-responsive">
      <table class="table table-striped table-hover">
        <thead class="table-dark">
          <tr>
            <th>#</th>
            <th>Execution Date</th>
            <th>Webhook Name</th>
            <th>Command</th>
            <th>Exit Code</th>
            <th>Stdout</th>
            <th>Stderr</th>
          </tr>
        </thead>
        <tbody>
          {#each paginatedTasks as task}
            <tr>
              <td>{task.id}</td>
              <td>{task.execution_date}</td>
              <td>{task.webhook_name}</td>
              <td><code>{task.executed_command}</code></td>
              <td>
                <span class="badge {task.command_exit_code === 0 ? 'bg-success' : 'bg-danger'}">
                  {task.command_exit_code}
                </span>
              </td>
              <td>
                <pre class="output-code p-3 border border-1 rounded-3 text-muted bg-primary-subtle border-primary" title={task.command_stdout}>
                  {task.command_stdout}
                </pre>
              </td>
              <td>
                <pre class="output-code p-3 border border-1 rounded-3 text-muted bg-danger-subtle border-danger" title={task.command_stderr}>
                  {task.command_stderr}
                </pre>
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>

    {#if totalPages > 1}
      <nav aria-label="Page navigation" class="mt-4">
        <ul class="pagination justify-content-center">
          <li class="page-item {currentPage === 0 ? 'disabled' : ''}">
            <button class="page-link" onclick={prevPage} disabled={currentPage === 0}>Previous</button>
          </li>
          {#each Array.from({ length: totalPages }) as _, i (i)}
            <li class="page-item {currentPage === i ? 'active' : ''}">
              <button class="page-link" onclick={() => (currentPage = i)}>
                {i + 1}
              </button>
            </li>
          {/each}
          <li class="page-item {currentPage === totalPages - 1 ? 'disabled' : ''}">
            <button class="page-link" onclick={nextPage} disabled={currentPage === totalPages - 1}>Next</button>
          </li>
        </ul>
      </nav>
    {/if}
  {/if}
</div>

<style>
  .output-code {
    resize: both;
  }
</style>
