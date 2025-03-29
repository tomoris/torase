# my-pre-commit
.PHONY: my-pre-commit
my-pre-commit:
	uv run ruff format .
	uv run ruff check . --fix
	uv run mypy .
	uv run python -m unittest discover

