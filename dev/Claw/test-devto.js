const https = require('https');

const postData = JSON.stringify({
  article: {
    title: "Test Post",
    body: "Testing Dev.to API",
    published: true
  }
});

const options = {
  hostname: 'dev.to',
  port: 443,
  path: '/api/articles',
  method: 'POST',
  headers: {
    'Content-Type': 'application/json',
    'api-key': 'e86R2mvXTBKqXPBrU1RY98MV',
    'Content-Length': postData.length
  }
};

const req = https.request(options, res => {
  let data = '';
  res.on('data', chunk => data += chunk);
  res.on('end', () => {
    console.log('Status:', res.statusCode);
    console.log('Response:', data);
  });
});

req.on('error', e => console.error('Error:', e.message));
req.write(postData);
req.end();
