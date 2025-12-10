## E2E TESTING

using uv to manage the virtual environment.

```bash
$ source .venv/bin/activate         # bash shell
# $ source .venv/bin/activate.fish  # fish shell
```

There should have some python scripts in `tests` for e2e testing which catch the
input data from `data/input` directory. And generate the baseline into
`data/output/baseline` directory.

Then the testing output should be put into `data/output` directory. And compare
with the baseline to check the results.
