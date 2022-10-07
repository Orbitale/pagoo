CREATE TABLE logs_webhooks (
    execution_date TEXT NOT NULL,
    webhook_name TEXT NOT NULL,
    executed_command TEXT NOT NULL,
    command_exit_code INTEGER NOT NULL,
    command_stdout TEXT NOT NULL,
    command_stderr TEXT NOT NULL
);
