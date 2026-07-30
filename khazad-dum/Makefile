REPO_URL ?= https://github.com/joerivrij/middleearth.git
BRANCH ?= feat/add-experiments
REPO_ROOT ?= khazad-dum
CLUSTER ?= khazad-dum
KUBECONFIG ?= $(HOME)/.kube/config
CLUSTER_PATH := ./$(REPO_ROOT)/clusters/$(CLUSTER)
CILIUM_VERSION ?= 1.19.6
CILIUM_NAMESPACE ?= cilium
CILIUM_CLUSTER_NAME ?= khazad-dum
SINGLE_NODE_KUBECONFIG ?= ../imladris/machine-build/khazad-dum.kubeconfig

.PHONY: bootstrap check-tools install seed-cilium status validate

check-tools:
	@command -v flux >/dev/null 2>&1 || { echo "flux CLI not found" >&2; exit 1; }
	@command -v kubectl >/dev/null 2>&1 || { echo "kubectl not found" >&2; exit 1; }
	@test -f "clusters/$(CLUSTER)/kustomization.yaml" || { echo "unknown cluster: $(CLUSTER)" >&2; exit 1; }

bootstrap: check-tools
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
	@if test "$(CILIUM_NAMESPACE)" != "kube-system" && \
		KUBECONFIG="$(KUBECONFIG)" kubectl get daemonset cilium \
		--namespace kube-system >/dev/null 2>&1; then \
		echo "legacy Cilium installation found in kube-system; recreate or migrate this disposable cluster first" >&2; \
		exit 1; \
	fi
	@KUBECONFIG="$(KUBECONFIG)" kubectl get namespace \
		"$(CILIUM_NAMESPACE)" >/dev/null 2>&1 || \
		KUBECONFIG="$(KUBECONFIG)" kubectl create namespace \
		"$(CILIUM_NAMESPACE)"
	@action=install; \
	if KUBECONFIG="$(KUBECONFIG)" kubectl get daemonset cilium \
		--namespace "$(CILIUM_NAMESPACE)" >/dev/null 2>&1; then action=upgrade; fi; \
	KUBECONFIG="$(KUBECONFIG)" cilium "$$action" \
		--version "$(CILIUM_VERSION)" \
		--namespace "$(CILIUM_NAMESPACE)" \
		--set cluster.name="$(CILIUM_CLUSTER_NAME)" \
		--set ipam.mode=kubernetes \
		--set kubeProxyReplacement=false \
		--wait \
		--wait-duration 10m

install:
	@test -f "$(SINGLE_NODE_KUBECONFIG)" || { \
		echo "single-node kubeconfig not found; run 'make -C ../imladris single-node' first" >&2; \
		exit 1; \
	}
	@$(MAKE) seed-cilium KUBECONFIG="$(SINGLE_NODE_KUBECONFIG)"
	@$(MAKE) bootstrap \
		CLUSTER=khazad-dum \
		KUBECONFIG="$(SINGLE_NODE_KUBECONFIG)"

status: check-tools
	@KUBECONFIG="$(KUBECONFIG)" flux get all --all-namespaces

validate:
	@command -v kustomize >/dev/null 2>&1 || { echo "kustomize not found" >&2; exit 1; }
	@find apps clusters infrastructure -name kustomization.yaml -print | sort | while read file; do \
		path=$${file%/*}; \
		echo "Validating $$path"; \
		kustomize build "$$path" >/dev/null || exit 1; \
	done
	@rendered=$$(mktemp); \
	trap 'rm -f "$$rendered"' EXIT; \
	kustomize build clusters/khazad-dum >"$$rendered"; \
	awk '\
		/^kind: Kustomization$$/ { flux_kustomization=1; namespace=0 } \
		flux_kustomization && /^  namespace: flux-system$$/ { namespace=1 } \
		flux_kustomization && /^---$$/ { \
			if (!namespace) { print "Flux Kustomization missing namespace" > "/dev/stderr"; exit 1 } \
			flux_kustomization=0 \
		} \
		END { \
			if (flux_kustomization && !namespace) { \
				print "Flux Kustomization missing namespace" > "/dev/stderr"; exit 1 \
			} \
		}' "$$rendered"
