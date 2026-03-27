"""Shared Browser - UI Only (for testing).

Run this first to test the UI works.
"""

from flask import Flask, render_template_string
import time

app = Flask(__name__)


HTML_TEMPLATE = """
<!DOCTYPE html>
<html>
<head>
    <title>Shared Browser - Together</title>
    <style>
        body { 
            font-family: sans-serif; 
            padding: 20px; 
            background: #1a1a2e;
            color: #eee;
        }
        .nav-bar { 
            display: flex;
            gap: 10px;
            margin-bottom: 10px;
        }
        #url-bar { 
            flex: 1;
            padding: 10px; 
            font-size: 16px;
            background: #16213e;
            color: #eee;
            border: 1px solid #333;
        }
        .btn { 
            padding: 10px 15px; 
            background: #00d4ff;
            border: none;
            cursor: pointer;
            color: #000;
        }
        #question {
            width: 100%;
            height: 80px;
            margin: 10px 0;
            padding: 10px;
            background: #16213e;
            color: #eee;
            border: 1px solid #333;
        }
        #status {
            margin: 10px 0;
            padding: 10px;
            background: #16213e;
        }
        iframe {
            width: 100%;
            height: 70vh;
            border: none;
            background: white;
        }
        .chat {
            margin-top: 10px;
            padding: 10px;
            background: #0f0f0f;
            height: 150px;
            overflow-y: scroll;
        }
        .chat-ai { color: #00d4ff; }
        .chat-you { color: #00ff88; }
        .error { color: #ff4444; }
    </style>
</head>
<body>
    <h1>🖥️ Shared Browser - Test Mode</h1>
    
    <div class="error">
        ⚠️ Browser not connected. Run the full server to enable.
    </div>
    
    <div class="nav-bar">
        <input id="url-bar" type="text" placeholder="Enter URL or search..." value="https://example.com">
        <button class="btn" id="go">Go</button>
    </div>
    
    <div class="nav-bar">
        <button class="btn" onclick="doAction('back')">← Back</button>
        <button class="btn" onclick="doAction('forward')">Forward →</button>
        <button class="btn" onclick="doAction('scroll_down')">↓ Scroll</button>
        <button class="btn" onclick="doAction('scroll_up')">↑ Scroll</button>
    </div>
    
    <div id="status">Status: Test mode - browser not connected</div>
    
    <h3>💬 Ask me about the page:</h3>
    <textarea id="question" placeholder="What's on this page? Explain X, find Y..."></textarea>
    <button class="btn" onclick="askQuestion()">Ask</button>
    
    <h3>🌐 Browser View</h3>
    <iframe id="browser-frame"></iframe>
    
    <h3>📝 Our Chat</h3>
    <div class="chat" id="chat-log"></div>
    
    <script>
        const log = (msg, type='ai') => {
            const div = document.createElement('div');
            div.className = 'chat-' + type;
            div.innerHTML = msg;
            document.getElementById('chat-log').appendChild(div);
            document.getElementById('chat-log').scrollTop = document.getElementById('chat-log').scrollHeight;
        };
        
        document.getElementById('go').onclick = () => {
            const url = document.getElementById('url-bar').value;
            log('📍 Tried to go to: ' + url, 'you');
        };
        
        document.getElementById('url-bar').addEventListener('keypress', (e) => {
            if (e.key === 'Enter') {
                log('📍 Would navigate to: ' + document.getElementById('url-bar').value, 'you');
            }
        });
        
        function askQuestion() {
            const q = document.getElementById('question').value;
            if (!q.trim()) return;
            
            log('You: ' + q, 'you');
            document.getElementById('question').value = '';
            
            log('🤖 Browser not connected - run the full server with Playwright to enable AI!', 'ai');
        }
        
        function doAction(action) {
            log('⚠️ Would do: ' + action + ' (browser not connected)', 'ai');
        }
    </script>
</body>
</html>
"""


@app.route('/')
def index():
    return render_template_string(HTML_TEMPLATE)


if __name__ == '__main__':
    print("Shared Browser UI Test")
    print("Open http://localhost:5000")
    app.run(port=5000, debug=False)
