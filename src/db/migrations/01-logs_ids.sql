ALTER TABLE logs_webhooks ADD COLUMN id VARCHAR(255) NOT NULL DEFAULT (lower(hex(randomblob(16))));

CREATE UNIQUE INDEX id ON logs_webhooks(id);
