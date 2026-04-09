project_id         = "replace-with-dev-project-id"
region             = "europe-west1"
environment        = "dev"
firestore_location = "europe-west1"

function_name           = "messenger-api-dev"
function_runtime        = "nodejs20"
function_entry_point    = "handler"
function_source_archive = "../functions/dist/function.zip"

function_memory_mb       = 256
function_timeout_seconds = 60
function_min_instances   = 0
function_max_instances   = 5

identity_platform_autodelete_anonymous_users = true

google_oauth_client_id     = ""
google_oauth_client_secret = ""

secret_names = {
  jwt_signing_key   = "jwt-signing-key"
  vapid_private_key = "vapid-private-key"
  vapid_public_key  = "vapid-public-key"
}

# Leave values empty to create secret containers only.
secret_initial_values = {
  jwt_signing_key   = ""
  vapid_private_key = ""
  vapid_public_key  = ""
}
