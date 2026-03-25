# WeRender Product Plan

## What It Already Does Well
- Zero-config (mDNS auto-discovery)
- No shared storage needed
- Fault-tolerant (workers can crash, frames reassign)
- Works on mixed hardware
- Web dashboard

## The Big Idea
**"Your own render farm in 30 seconds"**

---

## Quick Wins (v0.3)

### 1. Better Dashboard
- Per-frame progress visualization
- Real-time render preview thumbnails
- Worker performance metrics (frames/min, CPU/GPU usage)
- Estimated time remaining

### 2. Performance Metrics
- Benchmark mode: "This machine renders X frames/min"
- Historical data: "Your farm averaged 12 frames/min this week"
- Per-scene complexity scoring

### 3. GPU Selection
- Let workers choose which GPU to use
- Multi-GPU support on single machine

### 4. Progress Tracking
- Job queue with priorities
- Resume interrupted renders
- Render history

---

## Differentiation (Why It's Special)

| Competitor | WeRender |
|------------|----------|
| AWS/GCP Render Farm | Free (use your own hardware) |
| RenderPal | Requires $500+ license + config |
| Blender Network Render | Needs shared storage |
| SheepIt | Upload your own PC (not local) |

**The moat:** Zero-config is insanely hard. Nobody else has solved "no shared storage + no IP config."

---

## Go-To-Market

### Target Audiences
1. **Indie studios** (3-5 PCs) — biggest market
2. **Freelancers** — use client's hardware
3. **Classrooms** — teacher coordinates, students contribute
4. **VFX shops** — overnight renders

### Channels
- **YouTube** — "Turn your laptops into a render farm"
- **Blender Community** — r/blender, Blender Artists forum
- **Gumroad/Patreon** — for premium/cloud features
- **Product Hunt** — launch v1.0

### Demo Videos Needed
1. "3 PCs → 3x faster render in 60 seconds"
2. "Mixed hardware (gaming PC + old laptop) = render farm"
3. "Fault tolerance demo: kill a worker, watch it recover"

---

## Monetization

### Free Tier
- Local network rendering
- All current features

### Cloud Workers (v0.4)
- Add AWS/GCP/Azure workers on-demand
- Pay per frame rendered
- Integration with existing free version

### Premium (v0.5)
- Analytics dashboard
- Priority support
- Render history + archiving
- Multi-user teams

---

## Roadmap

### v0.3 (This Month)
- [ ] Per-frame progress
- [ ] Performance metrics
- [ ] GPU selection

### v0.4 (Next Month)
- [ ] Cloud worker integration
- [ ] Job queuing

### v1.0 (3 months)
- [ ] Polish everything
- [ ] Marketing push
- [ ] Product Hunt launch

---

## Action Items

1. **Fix one thing**: Focus on dashboard improvements first
2. **Demo video**: Record a 60-second "it just works" video
3. **Blender subreddit**: Post about it, help people
4. **GitHub stars**: Get more visibility

This is a real product. The zero-config angle is genuinely unique.
