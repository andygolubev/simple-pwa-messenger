# Runbooks

## Provisioning

### Prerequisites
- Terraform/OpenTofu ≥ 1.6 installed.
- `gcloud` CLI authenticated with project owner/editor.
- `npm` ≥ 10 and Rust + `wasm-pack` for local builds.

### Terraform init/plan/apply

```bash
cd infra/

# Initialise providers and modules
terraform init

# Review changes for dev environment
terraform plan -var-file=environments/dev.tfvars

# Apply (requires confirmation)
terraform apply -var-file=environments/dev.tfvars

# Apply non-interactively (CI)
terraform apply -auto-approve -var-file=environments/dev.tfvars
```

> **Note:** Do NOT run `terraform apply` against `prod.tfvars` locally.
> Production applies are gated behind CI approval workflows.

### Secret Manager bootstrapping

After first `terraform apply`, populate the secrets:

```bash
# JWT signing key (minimum 32 random bytes)
openssl rand -hex 32 | gcloud secrets versions add jwt-signing-key --data-file=-

# VAPID keys
npx web-push generate-vapid-keys --json > /tmp/vapid.json
cat /tmp/vapid.json | jq -r .publicKey | gcloud secrets versions add vapid-public-key --data-file=-
cat /tmp/vapid.json | jq -r .privateKey | gcloud secrets versions add vapid-private-key --data-file=-
rm /tmp/vapid.json
```

---

## Function deployment

### Build and package

```bash
cd functions/
npm ci
npm run build
npm run build:zip          # produces dist/function.zip
```

### Deploy via Terraform

Once `dist/function.zip` is up-to-date, run Terraform to redeploy the function:

```bash
cd infra/
terraform apply -var-file=environments/dev.tfvars
```

### Deploy directly (for quick iteration)

```bash
gcloud functions deploy messenger-api \
  --gen2 \
  --runtime=nodejs20 \
  --region=europe-west1 \
  --source=functions/dist \
  --entry-point=handler \
  --trigger-http \
  --allow-unauthenticated
```

---

## Local development

### Cloud Functions (functions API)

```bash
cd functions/
npm ci
npm run start              # runs on http://localhost:8080
```

### PWA (React app)

```bash
cd apps/pwa/
npm ci
npm run dev                # runs on http://localhost:3000
```

Set environment variables in `apps/pwa/.env.local`:

```env
VITE_API_BASE_URL=http://localhost:8080
VITE_VAPID_PUBLIC_KEY=<your-vapid-public-key>
```

### WASM build

```bash
# Install wasm-pack if not present
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

# Build WASM module (outputs to apps/pwa/pkg/)
cd crates/messenger-crypto/
./build-wasm.sh
```

---

## Verifying a deployment

```bash
BASE_URL="https://<region>-<project>.cloudfunctions.net"

# Health check
curl "${BASE_URL}/healthz"

# Auth (requires a valid Google ID token)
ID_TOKEN="<from google sign-in>"
RESPONSE=$(curl -s -X POST "${BASE_URL}/auth/google" \
  -H "Content-Type: application/json" \
  -d "{\"idToken\": \"${ID_TOKEN}\"}")
JWT=$(echo $RESPONSE | jq -r .jwt)
echo "JWT: ${JWT}"

# Key bundle check
curl -s -H "Authorization: Bearer ${JWT}" \
  "${BASE_URL}/keys/bundle?uid=<uid>&countOnly=true"
```
