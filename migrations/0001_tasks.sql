CREATE TABLE tasks (
    id INTEGER PRIMARY KEY,
    title TEXT NOT NULL,
    description TEXT,
    status TEXT CHECK(status IN('pending', 'in_progress', 'completed')) DEFAULT 'pending',
    priority INTEGER DEFAULT 1 NOT NULL,
    created_at DATETIME NOT NULL
);
