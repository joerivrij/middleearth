.PHONY: validate

validate:
	@command -v kustomize >/dev/null 2>&1 || { echo "kustomize not found" >&2; exit 1; }
	@find clusters infrastructure stacks -name kustomization.yaml -print | sort | while read file; do \
		path=$${file%/*}; \
		echo "Validating $$path"; \
		kustomize build "$$path" >/dev/null || exit 1; \
	done

