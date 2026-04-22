#!/usr/bin/env node
const fs = require('fs');
const path = require('path');

const MEMORY_DIR = path.join(__dirname, '..', '..', '..', 'memory');

function getDate() {
  return new Date().toISOString().split('T')[0];
}

function loadIndex() {
  const idxPath = path.join(MEMORY_DIR, 'index.json');
  if (!fs.existsSync(idxPath)) return { topics: {}, lastUpdated: null };
  return JSON.parse(fs.readFileSync(idxPath, 'utf8'));
}

function saveIndex(idx) {
  const idxPath = path.join(MEMORY_DIR, 'index.json');
  idx.lastUpdated = new Date().toISOString();
  fs.writeFileSync(idxPath, JSON.stringify(idx, null, 2));
}

function searchSemantic(query, limit = 10) {
  const results = [];
  const queryLower = query.toLowerCase();

  ['facts.jsonl', 'preferences.jsonl', 'remember.jsonl'].forEach(file => {
    const filePath = path.join(MEMORY_DIR, 'semantic', file);
    if (!fs.existsSync(filePath)) return;
    fs.readFileSync(filePath, 'utf8').split('\n').filter(Boolean).forEach(line => {
      const entry = JSON.parse(line);
      if (entry.text.toLowerCase().includes(queryLower) ||
          (entry.tags || []).some(t => t.toLowerCase().includes(queryLower))) {
        results.push({ ...entry, _file: file });
      }
    });
  });

  return results.slice(0, limit);
}

function searchEpisodic(query, limit = 10) {
  const results = [];
  const queryLower = query.toLowerCase();
  const files = fs.readdirSync(path.join(MEMORY_DIR, 'episodic'))
    .filter(f => f.endsWith('.jsonl'))
    .sort()
    .reverse();

  for (const file of files.slice(0, 30)) {
    const filePath = path.join(MEMORY_DIR, 'episodic', file);
    fs.readFileSync(filePath, 'utf8').split('\n').filter(Boolean).forEach(line => {
      const entry = JSON.parse(line);
      if (entry.text.toLowerCase().includes(queryLower) ||
          (entry.tags || []).some(t => t.toLowerCase().includes(queryLower))) {
        results.push({ ...entry, _file: file });
      }
    });
    if (results.length >= limit) break;
  }

  return results.slice(0, limit);
}

function searchProcedural(query, limit = 10) {
  const results = [];
  const queryLower = query.toLowerCase();

  ['workflows.jsonl', 'lessons.jsonl', 'patterns.jsonl'].forEach(file => {
    const filePath = path.join(MEMORY_DIR, 'procedural', file);
    if (!fs.existsSync(filePath)) return;
    fs.readFileSync(filePath, 'utf8').split('\n').filter(Boolean).forEach(line => {
      const entry = JSON.parse(line);
      if (entry.text.toLowerCase().includes(queryLower) ||
          (entry.tags || []).some(t => t.toLowerCase().includes(queryLower))) {
        results.push({ ...entry, _file: file });
      }
    });
  });

  return results.slice(0, limit);
}

function updateIndex(entry) {
  const idx = loadIndex();
  const topics = entry.tags || [];
  topics.forEach(t => {
    if (!idx.topics[t]) idx.topics[t] = { last: null, count: 0 };
    idx.topics[t].last = new Date().toISOString().split('T')[0];
    idx.topics[t].count++;
  });
  saveIndex(idx);
}

const args = process.argv.slice(2);

if (args[0] === 'search' || !args[0]) {
  const query = args[1] || '';
  const limit = parseInt(args[2]) || 10;

  if (!query) {
    const idx = loadIndex();
    console.log(JSON.stringify({ index: idx, message: 'Usage: recall search <query> [limit]' }, null, 2));
    process.exit(0);
  }

  const semantic = searchSemantic(query, limit);
  const episodic = searchEpisodic(query, limit);
  const procedural = searchProcedural(query, limit);

  console.log(JSON.stringify({ query, semantic, episodic, procedural }, null, 2));
} else {
  console.error('Unknown command. Usage: recall [search <query> [limit]]');
  process.exit(1);
}