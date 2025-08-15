-- Add migration script here
CREATE TABLE vuln_information (
    id TEXT NOT NULL UNIQUE PRIMARY KEY,
    title TEXT NOT NULL DEFAULT '',
    description TEXT NOT NULL DEFAULT '',
    severity TEXT NOT NULL DEFAULT '',
    cve TEXT NOT NULL DEFAULT '',
    disclosure TEXT NOT NULL DEFAULT '',
    solutions TEXT NOT NULL DEFAULT '',
    reference_links TEXT [] DEFAULT '{}',
    tags TEXT [] DEFAULT '{}',
    github_search TEXT [] DEFAULT '{}',
    source TEXT NOT NULL DEFAULT '',
    pushed BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
)

CREATE TABLE sync_task (
    id TEXT NOT NULL UNIQUE PRIMARY KEY,
    name TEXT NOT NULL DEFAULT '',
    minute INTEGER NOT NULL DEFAULT 15, -- 默认每15分钟执行一次
    status BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
)


CREATE TABLE sync_task_record (
    id TEXT NOT NULL UNIQUE PRIMARY KEY,
    task_id TEXT NOT NULL,
    started_at TIMESTAMP DEFAULT,
    ended_at TIMESTAMP DEFAULT,
    success BOOLEAN DEFAULT FALSE,
    error_message TEXT DEFAULT '',
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
)