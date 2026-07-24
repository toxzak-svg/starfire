# Kaggle publication

The materializer writes Kaggle-ready metadata with dataset identifier:

```text
zacharymaronek/starfire-sgp-cnn-rnn-training-corpora
```

After generating `tools/stlm_l1d/data/sgp_v1`, publish the first version with:

```bash
python -m pip install --upgrade kaggle
kaggle datasets create \
  -p tools/stlm_l1d/data/sgp_v1 \
  --dir-mode zip
```

Publish later revisions with:

```bash
kaggle datasets version \
  -p tools/stlm_l1d/data/sgp_v1 \
  -m "Refresh Starfire SGP CNN/RNN corpora" \
  --dir-mode zip
```

Authentication must be supplied through Kaggle's supported credentials, normally `KAGGLE_USERNAME` and `KAGGLE_KEY` or `~/.kaggle/kaggle.json`. Credentials are intentionally not stored in the repository.
