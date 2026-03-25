<script lang="ts">
  import { fetchConfig } from '$lib/api';
  import type { PagooConfig, Webhook, Matcher } from '$lib/api';

  let config = $state<PagooConfig | null>(null);
  let loading = $state(true);
  let error = $state<string | null>(null);

  $effect.pre(async () => {
    try {
      config = await fetchConfig();
    } catch (err) {
      error = err instanceof Error ? err.message : 'Failed to fetch config';
    } finally {
      loading = false;
    }
  });
</script>

<div class="container-fluid mt-4">
  <h1>Configuration</h1>

  {#if loading}
    <div class="d-flex justify-content-center my-5">
      <div class="spinner-border text-primary" role="status">
        <span class="visually-hidden">Loading...</span>
      </div>
    </div>
  {/if}

  {#if error}
    <div class="alert alert-danger alert-dismissible fade show" role="alert">
      {error}
      <button type="button" class="btn-close" aria-label="Dismiss" onclick={() => (error = null)}></button>
    </div>
  {/if}

  {#if !loading && config && config.webhooks.length === 0}
    <div class="alert alert-secondary" role="alert">
      No webhooks configured.
    </div>
  {/if}

  {#if config && config.webhooks.length > 0}
    <div class="row g-4">
      {#each config.webhooks as webhook (webhook.name)}
        <div class="col-12 col-lg-6">
          <div class="card">
            <div class="card-header">
              <h5 class="mb-0">{webhook.name}</h5>
            </div>
            <div class="card-body">
              <div class="mb-3">
                <strong>Matchers Strategy:</strong>
                <span class="badge bg-info ms-2">
                  {webhook['matchers-strategy'] ?? 'all'}
                </span>
              </div>

              {#if webhook.matchers.length > 0}
                <div class="mb-3">
                  <strong>Matchers:</strong>
                  {#each webhook.matchers as matcher, i (i)}
                    <div class="ms-2 mt-2">
                      {#if matcher['match-json-body']}
                        <div>
                          <span class="badge bg-secondary">JSON Body</span>
                          <table class="table table-sm table-bordered mt-1 mb-0">
                            <thead class="table-light">
                              <tr><th>Key</th><th>Value</th></tr>
                            </thead>
                            <tbody>
                              {#each Object.entries(matcher['match-json-body']) as [key, value] (key)}
                                <tr>
                                  <td><code>{key}</code></td>
                                  <td>{typeof value === 'object' ? JSON.stringify(value) : String(value)}</td>
                                </tr>
                              {/each}
                            </tbody>
                          </table>
                        </div>
                      {/if}
                      {#if matcher['match-headers']}
                        <div>
                          <span class="badge bg-secondary">Headers</span>
                          <table class="table table-sm table-bordered mt-1 mb-0">
                            <thead class="table-light">
                              <tr><th>Header</th><th>Value</th></tr>
                            </thead>
                            <tbody>
                              {#each Object.entries(matcher['match-headers']) as [key, value] (key)}
                                <tr>
                                  <td><code>{key}</code></td>
                                  <td>{value}</td>
                                </tr>
                              {/each}
                            </tbody>
                          </table>
                        </div>
                      {/if}
                    </div>
                  {/each}
                </div>
              {/if}

              <div>
                <strong>Actions to Execute:</strong>
                <pre>
                  {webhook['actions-to-execute'].join(" ")}
                </pre>
              </div>
            </div>
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>
