#!/usr/bin/env node
const fs = require('fs');
const path = require('path');

const MEMORY_DIR = path.join(__dirname, '..', '..', '..', 'memory');
const SEMANTIC_DIR = path.join(MEMORY_DIR, 'semantic');
const EPISODIC_DIR = path.join(MEMORY_DIR, 'episodic');
const PROCEDURAL_DIR = path.join(MEMORY_DIR, 'procedural');

function getDate() {
  return new Date().toISOString().split('T')[0];
}

function getTimestamp() {
  return Date.now();
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

function appendEntry(dir, file, entry) {
  const filePath = path.join(dir, file);
  fs.appendFileSync(filePath, JSON.stringify(entry) + '\n');
}

function updateIndex(entry) {
  const idx = loadIndex();
  const topics = entry.tags || [];
  topics.forEach(t => {
    if (!idx.topics[t]) idx.topics[t] = { last: null, count: 0 };
    idx.topics[t].last = getDate();
    idx.topics[t].count++;
  });
  saveIndex(idx);
}

const args = process.argv.slice(2);
const type = args[0];
const text = args[1];
const importance = parseInt(args[2]) || 3;
const tags = args.slice(3);

if (!type || !text) {
  console.error('Usage: remember <type> <text> [importance 1-5] [tags...]');
  console.error('  types: episodic, semantic, procedural');
  console.error('  importance: 1-5 (default 3)');
  process.exit(1);
}

if (!['episodic', 'semantic', 'procedural'].includes(type)) {
  console.error('Type must be: episodic, semantic, or procedural');
  process.exit(1);
}

const entry = {
  ts: getTimestamp(),
  date: getDate(),
  text,
  importance,
  tags
};

if (type === 'episodic') {
  const file = `${getDate()}.jsonl`;
  appendEntry(EPISODIC_DIR, file, entry);
} else if (type === 'semantic') {
  appendEntry(SEMANTIC_DIR, 'remember.jsonl', entry);
} else {
  appendEntry(PROCEDURAL_DIR, 'lessons.jsonl', entry);
}

updateIndex(entry);

console.log(JSON.stringify({ success: true, entry }, null, 2));