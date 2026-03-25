"""Generate SRMoE_Training.ipynb - Part 1 of 2"""
import json

nb = {
    "nbformat": 4,
    "nbformat_minor": 5,
    "metadata": {"kernelspec": {"display_name": "Python 3", "language": "python", "name": "python3"}},
    "cells": []
}

def md(src):
    return {"cell_type": "markdown", "metadata": {}, "source": src}

def cc(lines):
    return {"cell_type": "code", "execution_count": None, "metadata": {}, "outputs": [], "source": "\n".join(lines)}

# Cell 0: Title
nb["cells"].append(md(
    "# SR-MoE Training\n\n"
    "**Sparse Routing Mixture-of-Experts**: popcode VQ codebook + ring attractor + diversity regularizer.\n\n"
    "Run all cells. GPU strongly recommended.\n\n"
    "## Architecture\n"
    "1. **Popcode Router** - VQ codebook replaces softmax. Router selects a discrete codeword.\n"
    "2. **Ring Attractor** - 2D continuous attractor for path-dependent routing state.\n"
    "3. **Diversity Regularizer** - prevents code collapse.\n\n"
    "## Runtime (adjust in last cell)\n"
    "- GPU A5000/V100: 2000/5000/2000 ep (~20-30 min)\n"
    "- CPU: 100/200/100 ep (~5 min)"
))

# Cell 1: Imports
nb["cells"].append(cc([
    "import subprocess, sys",
    "subprocess.run([sys.executable, '-m', 'pip', 'install', 'numpy', '--quiet'], check=True)",
    "",
    "import os, math, random, json, time",
    "import torch",
    "import torch.nn as nn",
    "import torch.nn.functional as F",
    "",
    "print('torch:', torch.__version__)",
    "print('GPU:', torch.cuda.is_available())",
    "if torch.cuda.is_available():",
    "    print('GPU:', torch.cuda.get_device_name(0))",
    "",
    "DEVICE = torch.device('cuda' if torch.cuda.is_available() else 'cpu')",
]))

# Cell 2: SR-MoE Core
nb["cells"].append(cc([
    "N_ACTIONS = 3  # SKIP=0, READ=1, WRITE=2",
    "ORACLE_TO_ACTION_IDX = {'SKIP': 0, 'READ': 1, 'WRITE': 2, 'PERCEIVE': 0}",
    "IGNORE_INDEX = -100",
    "",
    "class RingAttractor(nn.Module):",
    "    def __init__(self, hidden_dim, ring_dim=16, n_prefs=32, dt=0.5, decay=0.95):",
    "        super().__init__()",
    "        self.hidden_dim = hidden_dim; self.ring_dim = ring_dim",
    "        self.n_prefs = n_prefs; self.dt = dt; self.decay = decay",
    "        self.register_buffer('_prefs', torch.linspace(0, 2*math.pi, n_prefs, dtype=torch.float32))",
    "        self.input_proj = nn.Sequential(nn.Linear(hidden_dim, ring_dim*2), nn.Tanh())",
    "        self.lateral_w = nn.Parameter(torch.zeros(n_prefs, n_prefs))",
    "        self._init_lateral_weights(); self._ring_state = None",
    "",
    "    def _init_lateral_weights(self):",
    "        with torch.no_grad():",
    "            d = self._prefs.unsqueeze(1) - self._prefs.unsqueeze(0)",
    "            d = torch.atan2(torch.sin(d), torch.cos(d))",
    "            exc = torch.exp(-0.5*(d/2.0)**2)",
    "            inh = 0.3*torch.exp(-0.5*(d/5.0)**2)",
    "            w = (exc - inh) / (exc - inh).abs().mean() * 0.1",
    "            self.lateral_w.copy_(w)",
    "",
    "    def reset(self): self._ring_state = None",
    "",
    "    def forward(self, hidden):",
    "        is_1d = hidden.dim() == 1",
    "        if is_1d: hidden = hidden.unsqueeze(0)",
    "        batch_size = hidden.shape[0]",
    "        ri = self.input_proj(hidden)",
    "        theta = torch.atan2(ri[..., 1], ri[..., 0])",
    "        prev = self._ring_state",
    "        if prev is None: prev = torch.zeros(self.n_prefs, device=hidden.device)",
    "        prev_2d = prev.unsqueeze(0)",
    "        drive = F.relu(torch.cos(self._prefs.unsqueeze(0) - theta.unsqueeze(1)))",
    "        lateral = (self.lateral_w @ prev).unsqueeze(0).expand(batch_size, -1)",
    "        new_state = prev_2d + self.dt * (drive + 0.3*lateral - prev_2d - self.decay*prev_2d)",
    "        new_state = F.relu(new_state)",
    "        new_state = new_state / (new_state.sum(dim=-1, keepdim=True) + 1e-8)",
    "        self._ring_state = new_state.squeeze(0).detach()",
    "        return self._ring_state",
    "",
    "    def get_state(self):",
    "        return self._ring_state if self._ring_state is not None else \\",
    "               torch.zeros(self.n_prefs, device=next(self.parameters()).device)",
]))

# Save part 1
with open('C:/dev/nb_part1.json', 'w') as f:
    json.dump(nb, f)
print("Part 1 saved, cells:", len(nb["cells"]))
