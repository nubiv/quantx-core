default:
  @just --list

py-test:
  python bindings/python

py-dev:
  maturin develop --uv --manifest-path bindings/Cargo.toml

test:
  @echo 'Testing!'

build:
  @echo 'Building!'