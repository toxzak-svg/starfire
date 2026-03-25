const fs = require('fs');
const https = require('https');

const j = JSON.parse(fs.readFileSync('C:/Users/Zwmar/Claw/devto-post.json', 'utf8'));
const data = JSON.stringify(j);

console.log('Posting:', j.article.title);

const req = https.request({
  hostname: 'dev.to',
  port: 443,
  path: '/api/articles',
  method: 'POST',
  headers: {
    'Content-Type': 'application/json',
    'api-key': 'e86R2mvXTBKqXPBrU1RY98MV',
    'Content-Length': data.length
  }
}, res => {
  let d = '';
  res.on('data', c => d += c);
  res.on('end', () => {
    console.log('Status:', res.statusCode);
    console.log('Response:', d);
  });
});

req.on('error', e => console.error('Error:', e.message));
req.write(data);
req.end();
