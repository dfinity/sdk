# Motoko Splay Tree Library

## How to develop

- Write your library code in `*.mo` source files in the `src/` directory.
- Run `make check` to make sure your changes compile (or use the
  VSCode extension to get quicker feedback)
- Add tests to the source files in the `test/` directory, and run them
  with `make test`. The project template is set up to include
  motoko-matchers.
- Generate API documentation locally by running `make docs` and then
  open the resulting `docs/index.html` in your browser

## How to publish

- Create a git tag for the commit you'd like to be the published
  version. For example:
  ```bash
  git tag v1.1.0
  git push origin v1.1.0
  ```
- Follow the instructions at
  [`vessel-package-set`](https://github.com/dfinity/vessel-package-set)
  to make it easy for other to install your library


## Checklist

### Check the initial setup works
- [ ] Make sure you've installed [`vessel`](https://github.com/dfinity/vessel)
- [ ] Make sure you've installed [`wasmtime`](https://wasmtime.dev/)
- [ ] Make sure `make all` runs succesfully. If it doesn't please [open an issue](https://github.com/kritzcreek/motoko-library-template)

### Licensing
- [ ] This template comes with a copy of the Apache License Version
      2.0, if you'd like to use a different license, replace the
      LICENSE file.
- [ ] Change the License section in the README to reference your
      libraries name

### Host library documentation on Github Pages

If you'd like to automatically build and host library documentation
whenever you push a git tag, follow these steps. Otherwise delete
`.github/workflows/release.yml`, the `gh-pages` branch, and the API Documentation section in the README.

- [ ] Turn on [Github Pages](https://pages.github.com/) in the Settings for your repo under:
      `Settings -> GitHub Pages -> Source -> Pick the "gh-pages" branch`
- [ ] Change the Url in the `API Documentation` section in your project

### Finishing touches
- [ ] Check out the "How to develop" and "How to publish" sections in the
      README and finally delete the Checklist section from the
      README

## API Documentation

API documentation for this library can be found at (CHANGE ME) https://kritzcreek.github.io/motoko-library-template

## License

motoko-library-template is distributed under the terms of the Apache License (Version 2.0).

See LICENSE for details.
