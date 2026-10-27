# Contributing to AI Track Master

Thanks for helping. This page has the rules that apply from your first commit.
Everything else (layers and owners, prohibitions, definition of done) is in
[`AGENTS.md`](AGENTS.md); it applies to human and AI contributors alike.

## License

The project is licensed under [Apache-2.0](LICENSE). By contributing you agree that
your contribution is provided under that license.

## Sign your commits (DCO)

Contributions use the [Developer Certificate of Origin](https://developercertificate.org/)
instead of a CLA. Sign off every commit:

```sh
git commit -s -m "feat(dsp): add biquad filter"
```

This appends a line to the commit message:

```
Signed-off-by: Your Name <you@example.com>
```

The line certifies that you wrote the change or have the right to submit it under the
project license, as stated in the DCO text. Use a name and email that identify you.
A commit without `Signed-off-by` will not be merged.

## Commit messages

Use [Conventional Commits](https://www.conventionalcommits.org/) in English:

```
feat(dsp): add true-peak oversampler
fix(server): reject paths that escape the project folder
docs(adr): record working name AI Track Master
test(analysis): add loudness gating cases
```

## Branches and merging

- One branch per task: `task/<M#>-<NN>-<slug>`, for example `task/M0-22-contributing-md`.
- Keep changes small.
- Never force push and never rewrite published history.
- Never merge into `main` yourself. The maintainer merges after review.

## Before you open a pull request

Run the local checks listed in `AGENTS.md` ("Commands") and update the
"Unreleased" section of `CHANGELOG.md`, and meet the "Definition of done" in `AGENTS.md`.
