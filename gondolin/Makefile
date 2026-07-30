KUBECONFIG ?= $(HOME)/.kube/config

.PHONY: decrypt-example encrypt-example export-private-key init-key install-flux-key validate

init-key:
	@./scripts/init-key.sh

encrypt-example:
	@./scripts/create-example-secret.sh

decrypt-example:
	@GNUPGHOME="$(CURDIR)/.keys/gnupg" \
		sops decrypt secrets/example/example-secret.enc.yaml

export-private-key:
	@test -f .keys/fingerprint || { echo "run make init-key first" >&2; exit 1; }
	@GNUPGHOME="$(CURDIR)/.keys/gnupg" gpg --batch --armor \
		--export-secret-keys "$$(cat .keys/fingerprint)"

install-flux-key:
	@test -f .keys/fingerprint || { echo "run make init-key or import the backup first" >&2; exit 1; }
	@KUBECONFIG="$(KUBECONFIG)" kubectl get namespace flux-system >/dev/null
	@GNUPGHOME="$(CURDIR)/.keys/gnupg" gpg --batch --armor \
		--export-secret-keys "$$(cat .keys/fingerprint)" | \
		KUBECONFIG="$(KUBECONFIG)" kubectl --namespace flux-system \
		create secret generic gondolin-sops-gpg \
		--from-file=sops.asc=/dev/stdin \
		--dry-run=client --output=yaml | \
		KUBECONFIG="$(KUBECONFIG)" kubectl apply --filename=-

validate:
	@command -v kustomize >/dev/null 2>&1 || { echo "kustomize not found" >&2; exit 1; }
	@kustomize build secrets/example >/dev/null
	@kustomize build clusters/default >/dev/null
	@test -f keys/gondolin-sops.asc
	@! rg -q 'REPLACE_WITH_GONDOLIN_FINGERPRINT' .sops.yaml
	@rg -q 'ENC\[AES256_GCM' secrets/example/example-secret.enc.yaml
	@rg -q "$$(awk '/pgp:/ { print $$2; exit }' .sops.yaml)" \
		secrets/example/example-secret.enc.yaml
