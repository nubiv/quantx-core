default:
  @just --list

activate:
  source ./venv/bin/activate

py-test:
  python pyo3/test.py

test:
  @echo 'Testing!'

build:
  @echo 'Building!'