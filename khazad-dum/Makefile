REPO_URL ?= https://github.com/OWNER/khazad-dum.git
BRANCH ?= main
CLUSTER ?= example
ENV ?= bilbo
KUBECONFIG ?= $(HOME)/.kube/config
CLUSTER_PATH := ./clusters/$(CLUSTER)
CILIUM_VERSION ?= 1.19.6

.PHONY: bootstrap bootstrap-erebor check-tools seed-cilium status validate

check-tools:
	@command -v flux >/dev/null 2>&1 || { echo "flux CLI not found" >&2; exit 1; }
	@command -v kubectl >/dev/null 2>&1 || { echo "kubectl not found" >&2; exit 1; }
	@test -f "clusters/$(CLUSTER)/kustomization.yaml" || { echo "unknown cluster: $(CLUSTER)" >&2; exit 1; }

bootstrap: check-tools
	@test "$(REPO_URL)" != "https://github.com/OWNER/khazad-dum.git" || { echo "set REPO_URL to the real Git URL" >&2; exit 1; }
	@KUBECONFIG="$(KUBECONFIG)" flux check --pre
	@KUBECONFIG="$(KUBECONFIG)" flux install
	@KUBECONFIG="$(KUBECONFIG)" flux create source git khazad-dum \
		--url="$(REPO_URL)" \
		--branch="$(BRANCH)" \
		--interval=1m \
		--export | KUBECONFIG="$(KUBECONFIG)" kubectl apply -f -
	@KUBECONFIG="$(KUBECONFIG)" flux create kustomization khazad-dum \
		--source=khazad-dum \
		--path="$(CLUSTER_PATH)" \
		--prune=true \
		--wait=true \
		--interval=10m \
		--export | KUBECONFIG="$(KUBECONFIG)" kubectl apply -f -

seed-cilium:
	@command -v cilium >/dev/null 2>&1 || { echo "cilium CLI not found" >&2; exit 1; }
	@command -v kubectl >/dev/null 2>&1 || { echo "kubectl not found" >&2; exit 1; }
	@action=install; \
	if KUBECONFIG="$(KUBECONFIG)" kubectl get daemonset cilium \
		--namespace kube-system >/dev/null 2>&1; then action=upgrade; fi; \
	KUBECONFIG="$(KUBECONFIG)" cilium "$$action" \
		--version "$(CILIUM_VERSION)" \
		--namespace kube-system \
		--set ipam.mode=kubernetes \
		--set kubeProxyReplacement=false \
		--wait \
		--wait-duration 10m

bootstrap-erebor: seed-cilium
	@$(MAKE) bootstrap CLUSTER="erebor/$(ENV)"

status: check-tools
	@KUBECONFIG="$(KUBECONFIG)" flux get all --all-namespaces

validate:
	@command -v kustomize >/dev/null 2>&1 || { echo "kustomize not found" >&2; exit 1; }
	@find apps clusters infrastructure -name kustomization.yaml -print | sort | while read file; do \
		path=$${file%/*}; \
		echo "Validating $$path"; \
		kustomize build "$$path" >/dev/null || exit 1; \
	done
