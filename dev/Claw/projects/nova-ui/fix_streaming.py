with open('C:/Users/Zwmar/Claw/projects/nova-ui/frontend/index.html', 'r', encoding='utf-8') as f:
    content = f.read()

start = content.find("// For streaming, read the stream")
end = content.find("// Final flush", start) + len("// Final flush")

old_block = content[start:end]

new_block = '''    // Stream: read SSE from our backend
    const reader = r.body.getReader();
    const decoder = new TextDecoder();
    let novaDiv = addMessage("nova", "");

    let buffer = "";

    while (true) {
      const { value, done } = await reader.read();
      if (done) break;

      buffer += decoder.decode(value, { stream: true });

      // Process complete lines
      const lines = buffer.split("\\n");
      buffer = lines.pop() || "";

      for (const line of lines) {
        if (!line.startsWith("data:")) continue;
        const dataStr = line.slice(5).trim();
        if (!dataStr) continue;

        try {
          const event = JSON.parse(dataStr);

          if (event.type === "text") {
            novaDiv.querySelector(".message-bubble").textContent += event.content;
            const messages = document.getElementById("chat-messages");
            messages.scrollTop = messages.scrollHeight;
          } else if (event.type === "done" || event.type === "error") {
            if (event.type === "error") {
              novaDiv.querySelector(".message-bubble").textContent += `\\n[Error: ${event.content}]`;
            }
            break;
          }
        } catch {}
      }
    }'''

new_content = content[:start] + new_block + content[end:]

with open('C:/Users/Zwmar/Claw/projects/nova-ui/frontend/index.html', 'w', encoding='utf-8') as f:
    f.write(new_content)

print("Replaced streaming code")
print(f"Old length: {len(content)}, New length: {len(new_content)}")
