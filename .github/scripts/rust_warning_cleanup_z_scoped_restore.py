from pathlib import Path

path = Path("lib/latent_roles.rs")
text = path.read_text()
missing = '''        StructuralGraph {
            graph_id,
            edges,
        }
'''
restored = '''        StructuralGraph {
            graph_id,
            nodes,
            edges,
        }
'''
if missing in text:
    path.write_text(text.replace(missing, restored, 1))
