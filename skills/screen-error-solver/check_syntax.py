import ast
with open(r'C:\Users\Zwmar\.openclaw\workspace\projects\starfire\skills\screen-error-solver\marble_watcher.py', encoding='utf-8') as f:
    src = f.read()
try:
    ast.parse(src)
    print('Syntax OK')
except SyntaxError as e:
    print(f'SyntaxError: line {e.lineno}: {e.msg}')
