"""Shared Browser Server - Fixed threading issues."""

from flask import Flask, render_template_string, jsonify, request
from playwright.sync_api import sync_playwright
import threading
import time
import queue

app = Flask(__name__)

# Browser state
playwright = None
browser = None
page = None
command_queue = queue.Queue()
result_queue = queue.Queue()
running = False


HTML_TEMPLATE = """<!DOCTYPE html>
<html>
<head>
    <title>Shared Browser - Together</title>
    <style>
        body { font-family: sans-serif; padding: 20px; background: #1a1a2e; color: #eee; }
        .nav-bar { display: flex; gap: 10px; margin-bottom: 10px; }
        #url-bar { flex: 1; padding: 10px; font-size: 16px; background: #16213e; color: #eee; border: 1px solid #333; }
        .btn { padding: 10px 15px; background: #00d4ff; border: none; cursor: pointer; color: #000; }
        #question { width: 100%; height: 80px; margin: 10px 0; padding: 10px; background: #16213e; color: #eee; border: 1px solid #333; }
        #status { margin: 10px 0; padding: 10px; background: #16213e; }
        iframe { width: 100%; height: 60vh; border: none; background: white; }
        .chat { margin-top: 10px; padding: 10px; background: #0f0f0f; height: 150px; overflow-y: scroll; }
        .chat-ai { color: #00d4ff; }
        .chat-you { color: #00ff88; }
    </style>
</head>
<body>
    <h1>Shared Browser - Ask me anything!</h1>
    
    <div class="nav-bar">
        <input id="url-bar" type="text" placeholder="Enter URL or search...">
        <button class="btn" id="go">Go</button>
    </div>
    
    <div class="nav-bar">
        <button class="btn" onclick="doAction('back')">Back</button>
        <button class="btn" onclick="doAction('forward')">Forward</button>
        <button class="btn" onclick="doAction('scroll_down')">Scroll Down</button>
        <button class="btn" onclick="doAction('scroll_up')">Scroll Up</button>
    </div>
    
    <div id="status">Status: Ready</div>
    
    <h3>Ask me about the page:</h3>
    <textarea id="question" placeholder="What's on this page? Explain X, find Y..."></textarea>
    <button class="btn" onclick="askQuestion()">Ask</button>
    
    <iframe id="browser-frame"></iframe>
    
    <h3>Our Chat</h3>
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
            fetch('/browser/navigate?url=' + encodeURIComponent(url))
                .then(r => r.json())
                .then(d => {
                    log('Went to: ' + url, 'you');
                    refreshView();
                });
        };
        
        document.getElementById('url-bar').addEventListener('keypress', (e) => {
            if (e.key === 'Enter') document.getElementById('go').click();
        });
        
        function askQuestion() {
            const q = document.getElementById('question').value;
            if (!q.trim()) return;
            log('You: ' + q, 'you');
            document.getElementById('question').value = '';
            document.getElementById('status').innerText = 'Status: Thinking...';
            
            fetch('/browser/ask', method='POST', headers={'Content-Type': 'application/json'}, 
                  body=JSON.stringify({question: q}))
                .then(r => r.json())
                .then(d => {
                    log('AI: ' + d.answer, 'ai');
                    document.getElementById('status').innerText = 'Status: Ready';
                });
        }
        
        function doAction(action) {
            fetch('/browser/action?action=' + action)
                .then(r => r.json())
                .then(d => { refreshView(); });
        }
        
        function refreshView() {
            document.getElementById('browser-frame').src = '/browser/view?t=' + Date.now();
        }
        
        refreshView();
        setInterval(refreshView, 3000);
    </script>
</body>
</html>"""


def browser_worker():
    """Run browser in background thread with its own event loop."""
    global playwright, browser, page, running
    
    # Import and run in this thread
    from playwright.sync_api import sync_playwright
    
    pw = sync_playwright().start()
    b = pw.chromium.launch(headless=True)
    p = b.new_page()
    p.goto("https://example.com")
    
    playwright = pw
    browser = b
    page = p
    running = True
    
    print("Browser started")
    
    # Process commands from queue
    while running:
        try:
            cmd = command_queue.get(timeout=0.5)
            action = cmd.get('action')
            
            if action == 'navigate':
                url = cmd.get('url')
                if not url.startswith('http'):
                    url = 'https://' + url
                try:
                    p.goto(url)
                    result_queue.put({'status': 'ok', 'url': url})
                except Exception as e:
                    result_queue.put({'status': 'error', 'message': str(e)})
            
            elif action == 'action':
                act = cmd.get('type')
                try:
                    if act == 'scroll_down':
                        p.evaluate('window.scrollBy(0, 500)')
                    elif act == 'scroll_up':
                        p.evaluate('window.scrollBy(0, -500)')
                    elif act == 'back':
                        p.go_back()
                    elif act == 'forward':
                        p.go_forward()
                    result_queue.put({'status': 'ok'})
                except Exception as e:
                    result_queue.put({'status': 'error', 'message': str(e)})
            
            elif action == 'get_content':
                try:
                    content = p.content()[:20000]
                    title = p.title()
                    result_queue.put({'content': content, 'title': title})
                except Exception as e:
                    result_queue.put({'error': str(e)})
            
            elif action == 'quit':
                running = False
                result_queue.put({'status': 'ok'})
                
        except queue.Empty:
            continue
    
    b.close()
    pw.stop()
    print("Browser stopped")


@app.route('/')
def index():
    return render_template_string(HTML_TEMPLATE)


@app.route('/browser/navigate')
def navigate():
    url = request.args.get('url', 'https://example.com')
    command_queue.put({'action': 'navigate', 'url': url})
    
    try:
        result = result_queue.get(timeout=10)
        return jsonify(result)
    except queue.Empty:
        return jsonify({'status': 'error', 'message': 'timeout'})


@app.route('/browser/action')
def action():
    act = request.args.get('action', '')
    command_queue.put({'action': 'action', 'type': act})
    
    try:
        result = result_queue.get(timeout=5)
        return jsonify(result)
    except queue.Empty:
        return jsonify({'status': 'error', 'message': 'timeout'})


@app.route('/browser/ask', methods=['POST'])
def ask_question():
    try:
        data = request.get_json(force=True)
        if data is None:
            data = {}
    except:
        data = {}
    
    question = data.get('question', '') if data else ''
    
    # Get page content
    command_queue.put({'action': 'get_content'})
    
    try:
        result = result_queue.get(timeout=10)
        if 'error' in result:
            return jsonify({'answer': f'Error: {result["error"]}'})
        
        content = result.get('content', '')[:15000]
        title = result.get('title', '')
        
        # For now, give a simple response
        answer = f"I can see you're on: {title}. To answer your question about this page, I'd need an LLM configured. Question: '{question}'"
        
        return jsonify({'answer': answer, 'title': title})
    except queue.Empty:
        return jsonify({'answer': 'Browser not responding'})


@app.route('/browser/view')
def browser_view():
    command_queue.put({'action': 'get_content'})
    
    try:
        result = result_queue.get(timeout=10)
        if 'error' in result:
            return "<html><body>Error loading page</body></html>"
        return result.get('content', '<html><body>Loading...</body></html>')
    except:
        return "<html><body>Browser not ready</body></html>"


if __name__ == '__main__':
    # Start browser in background thread
    bt = threading.Thread(target=browser_worker, daemon=True)
    bt.start()
    
    # Give browser time to start
    time.sleep(3)
    
    print("Shared Browser starting at http://localhost:5000")
    app.run(port=5000, debug=False)
